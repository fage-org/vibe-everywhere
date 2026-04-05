use std::net::TcpListener as StdTcpListener;
use std::time::Duration;

use reqwest::Client as HttpClient;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle};
use vibe_agent::{
    api::{ApiClient, DecryptedSession},
    auth::{
        AuthRequestResponse, PendingAccountLink, auth_status, complete_account_link,
        poll_until_authorized, request_account_link,
    },
    config::Config as AgentConfig,
    credentials::{Credentials, read_credentials},
    encryption::{
        EncryptionVariant, encode_base64, encrypt_json, libsodium_encrypt_for_public_key,
        random_bytes, wrap_data_encryption_key,
    },
    session::{SessionClient, SessionClientOptions, SessionEvent},
};
use vibe_server::{
    api::build_router, config::Config as ServerConfig, context::AppContext,
    sessions::SessionsService,
};
use vibe_wire::SessionMessageContent;

struct TestServer {
    ctx: AppContext,
    server_url: String,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

impl TestServer {
    async fn start() -> Self {
        let std_listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = std_listener.local_addr().unwrap();
        std_listener.set_nonblocking(true).unwrap();
        let listener = tokio::net::TcpListener::from_std(std_listener).unwrap();

        let config = ServerConfig {
            host: addr.ip(),
            port: addr.port(),
            master_secret: "wave3-secret".into(),
            ios_up_to_date: ">=1.4.1".into(),
            android_up_to_date: ">=1.4.1".into(),
            ios_store_url: "ios-store".into(),
            android_store_url: "android-store".into(),
            webapp_url: "https://app.vibe.engineering".into(),
        };
        let ctx = AppContext::new(config);
        let app = build_router(ctx.clone());
        let server_url = format!("http://{}", listener.local_addr().unwrap());
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let shutdown_ctx = ctx.clone();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
            shutdown_ctx.shutdown().await;
        });

        Self {
            ctx,
            server_url,
            shutdown: Some(shutdown_tx),
            task: Some(task),
        }
    }

    fn agent_config(&self, home: &TempDir) -> AgentConfig {
        AgentConfig::from_sources(
            Some(self.server_url.clone()),
            Some(home.path().as_os_str().to_owned()),
        )
        .unwrap()
    }

    async fn authenticate(&self) -> (TempDir, AgentConfig, Credentials, String) {
        let home = TempDir::new().unwrap();
        let config = self.agent_config(&home);
        let client = HttpClient::new();
        let pending = PendingAccountLink::new();

        let initial = request_account_link(&client, &config, &pending)
            .await
            .unwrap();
        assert!(matches!(initial, AuthRequestResponse::Requested));

        let account = self
            .ctx
            .db()
            .upsert_account_by_public_key("wave3-account-public-key");
        let approval_token = self.ctx.auth().create_token(&account.id, None);
        let secret = random_bytes::<32>();
        let public_key = pending.public_key_base64();
        let response = encode_base64(&libsodium_encrypt_for_public_key(
            &secret,
            &pending.public_key,
        ));
        let authorize_client = client.clone();
        let authorize_url = format!("{}/v1/auth/account/response", self.server_url);

        let (authorized, ()) = tokio::join!(
            poll_until_authorized(&client, &config, &pending),
            async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                authorize_client
                    .post(authorize_url)
                    .bearer_auth(approval_token)
                    .json(&json!({
                        "publicKey": public_key,
                        "response": response,
                    }))
                    .send()
                    .await
                    .unwrap()
                    .error_for_status()
                    .unwrap();
            }
        );

        let credentials = complete_account_link(&config, &pending, authorized.unwrap()).unwrap();
        let stored = read_credentials(&config).unwrap();
        let status = auth_status(&config).unwrap();

        assert_eq!(credentials.secret, secret);
        assert_eq!(stored.secret, secret);
        assert_eq!(status.secret, secret);
        assert_eq!(
            self.ctx
                .auth()
                .verify_token(&credentials.token)
                .unwrap()
                .user_id,
            account.id
        );

        (home, config, credentials, account.id)
    }

    async fn shutdown(mut self) {
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

fn session_client_options(
    config: &AgentConfig,
    credentials: &Credentials,
    session: &DecryptedSession,
) -> SessionClientOptions {
    SessionClientOptions {
        session_id: session.id.clone(),
        encryption_key: session.encryption.key,
        encryption_variant: session.encryption.variant,
        token: credentials.token.clone(),
        server_url: config.socket_url(),
        initial_metadata: Some(session.metadata.clone()),
        initial_metadata_version: session.metadata_version,
        initial_agent_state: session.agent_state.clone(),
        initial_agent_state_version: session.agent_state_version,
    }
}

fn is_user_text_message(content: &Value, expected_text: &str) -> bool {
    content.get("role").and_then(Value::as_str) == Some("user")
        && content
            .get("content")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str)
            == Some("text")
        && content
            .get("content")
            .and_then(|value| value.get("text"))
            .and_then(Value::as_str)
            == Some(expected_text)
}

async fn wait_for_session_to_become_inactive(api: &ApiClient, session_id: &str) -> bool {
    for _ in 0..50 {
        let sessions = api.list_sessions().await.unwrap();
        if sessions
            .iter()
            .find(|session| session.id == session_id)
            .is_some_and(|session| !session.active)
        {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    false
}

#[tokio::test(flavor = "multi_thread")]
async fn auth_flow_persists_credentials_against_real_vibe_server() {
    let server = TestServer::start().await;

    let (_home, config, credentials, _user_id) = server.authenticate().await;

    let stored = read_credentials(&config).unwrap();
    assert_eq!(stored.secret, credentials.secret);
    assert_eq!(stored.token, credentials.token);

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn api_and_session_socket_flow_handle_lossy_records_against_real_vibe_server() {
    let server = TestServer::start().await;
    let (_home, config, credentials, user_id) = server.authenticate().await;
    let api = ApiClient::new(config.clone(), credentials.clone());

    let data_key = random_bytes::<32>();
    let wrapped_machine_key =
        wrap_data_encryption_key(&data_key, &credentials.content_key_pair.public_key);
    let machine_metadata = encode_base64(
        &encrypt_json(
            &data_key,
            EncryptionVariant::DataKey,
            &json!({
                "name": "builder",
                "homeDir": "/tmp/wave3",
            }),
        )
        .unwrap(),
    );
    let machine_state = encode_base64(
        &encrypt_json(
            &data_key,
            EncryptionVariant::DataKey,
            &json!({
                "pid": 42,
            }),
        )
        .unwrap(),
    );
    let _ = server.ctx.db().create_or_load_machine(
        &user_id,
        "machine-good",
        &machine_metadata,
        Some(machine_state),
        Some(wrapped_machine_key),
    );
    let _ = server.ctx.db().create_or_load_machine(
        &user_id,
        "machine-bad",
        "not-base64",
        Some("not-base64".into()),
        None,
    );
    let _ = server
        .ctx
        .db()
        .create_or_load_session(&user_id, "broken-session", "not-base64", None);

    let session = api
        .create_session(
            "wave3-good-session",
            &json!({
                "tag": "wave3-good-session",
                "path": "/tmp/project",
                "host": "localhost",
            }),
        )
        .await
        .unwrap();

    let sessions = api.list_sessions().await.unwrap();
    assert!(sessions.iter().any(|item| item.id == session.id));
    assert!(
        sessions
            .iter()
            .any(|item| item.id != session.id && item.metadata.is_null())
    );

    let active_sessions = api.list_active_sessions().await.unwrap();
    assert!(active_sessions.iter().any(|item| item.id == session.id));
    assert!(active_sessions.iter().any(|item| item.metadata.is_null()));

    let machines = api.list_machines().await.unwrap();
    assert!(machines.iter().any(|machine| {
        machine.id == "machine-good" && machine.metadata["homeDir"] == "/tmp/wave3"
    }));
    assert!(
        machines
            .iter()
            .any(|machine| machine.id == "machine-bad" && machine.metadata.is_null())
    );

    let machine = api.get_machine("machine-good").await.unwrap();
    assert_eq!(machine.metadata["name"], "builder");
    assert_eq!(machine.metadata["homeDir"], "/tmp/wave3");
    assert_eq!(machine.daemon_state.as_ref().unwrap()["pid"], 42);

    let sender = SessionClient::connect(session_client_options(&config, &credentials, &session))
        .await
        .unwrap();
    let receiver = SessionClient::connect(session_client_options(&config, &credentials, &session))
        .await
        .unwrap();
    let mut receiver_events = receiver.subscribe();

    let socket_outcome: Result<(), ()> = async {
        sender
            .wait_for_connect(Duration::from_secs(10))
            .await
            .unwrap();
        receiver
            .wait_for_connect(Duration::from_secs(10))
            .await
            .unwrap();

        sender.send_message("hello from wave3", None).await.unwrap();

        let message = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                match receiver_events.recv().await {
                    Ok(SessionEvent::Message(message)) => return message,
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(error) => panic!("receiver closed unexpectedly: {error}"),
                }
            }
        })
        .await
        .unwrap();

        assert!(is_user_text_message(&message.content, "hello from wave3"));
        Ok(())
    }
    .await;

    sender.close().await;
    receiver.close().await;
    socket_outcome.unwrap();

    server
        .ctx
        .db()
        .append_message(
            &user_id,
            &session.id,
            SessionMessageContent::new("not-base64"),
            Some("bad-local".into()),
        )
        .unwrap();

    let history = api
        .get_session_messages(&session.id, &session.encryption)
        .await
        .unwrap();
    assert!(
        history
            .iter()
            .any(|message| is_user_text_message(&message.content, "hello from wave3"))
    );
    assert!(history.iter().any(
        |message| message.local_id.as_deref() == Some("bad-local") && message.content.is_null()
    ));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn stop_and_delete_session_against_real_vibe_server() {
    let server = TestServer::start().await;
    let (_home, config, credentials, _user_id) = server.authenticate().await;
    let api = ApiClient::new(config.clone(), credentials.clone());

    let session = api
        .create_session(
            "wave3-stop-session",
            &json!({
                "tag": "wave3-stop-session",
                "path": "/tmp/stop-project",
                "host": "localhost",
            }),
        )
        .await
        .unwrap();

    let active_sessions = api.list_active_sessions().await.unwrap();
    assert!(active_sessions.iter().any(|item| item.id == session.id));

    let client = SessionClient::connect(session_client_options(&config, &credentials, &session))
        .await
        .unwrap();
    client
        .wait_for_connect(Duration::from_secs(10))
        .await
        .unwrap();
    client.send_stop().await.unwrap();
    client.close().await;

    assert!(wait_for_session_to_become_inactive(&api, &session.id).await);

    let active_sessions = api.list_active_sessions().await.unwrap();
    assert!(!active_sessions.iter().any(|item| item.id == session.id));

    api.delete_session(&session.id).await.unwrap();
    let sessions = api.list_sessions().await.unwrap();
    assert!(!sessions.iter().any(|item| item.id == session.id));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn wait_for_idle_reacts_to_real_server_state_updates() {
    let server = TestServer::start().await;
    let (_home, config, credentials, user_id) = server.authenticate().await;
    let api = ApiClient::new(config.clone(), credentials.clone());

    let session = api
        .create_session(
            "wave3-wait-session",
            &json!({
                "tag": "wave3-wait-session",
                "path": "/tmp/wait-project",
                "host": "localhost",
            }),
        )
        .await
        .unwrap();

    let client = SessionClient::connect(session_client_options(&config, &credentials, &session))
        .await
        .unwrap();
    client
        .wait_for_connect(Duration::from_secs(10))
        .await
        .unwrap();

    let ctx = server.ctx.clone();
    let session_id = session.id.clone();
    let idle_state = encode_base64(
        &encrypt_json(
            &session.encryption.key,
            session.encryption.variant,
            &json!({
                "controlledByUser": false,
                "requests": {},
            }),
        )
        .unwrap(),
    );

    let update_task = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        SessionsService::new(ctx)
            .update_agent_state(&user_id, &session_id, 0, Some(idle_state))
            .unwrap();
    });

    client.wait_for_idle(Duration::from_secs(5)).await.unwrap();
    update_task.await.unwrap();
    client.close().await;

    server.shutdown().await;
}
