use std::{
    net::TcpListener as StdTcpListener,
    process::{Command, Output},
    time::Duration,
};

use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle};
use vibe_agent::{
    api::ApiClient,
    config::Config as AgentConfig,
    credentials::{Credentials, write_credentials},
    encryption::{
        EncryptionVariant, encode_base64, encrypt_json, random_bytes, wrap_data_encryption_key,
    },
};
use vibe_server::{
    api::build_router, config::Config as ServerConfig, context::AppContext,
    sessions::SessionsService,
};

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
            master_secret: "wave3-cli-secret".into(),
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

    fn provision_credentials(&self) -> (TempDir, AgentConfig, Credentials, String) {
        let home = TempDir::new().unwrap();
        let config = self.agent_config(&home);
        let account = self
            .ctx
            .db()
            .upsert_account_by_public_key("wave3-cli-account");
        let token = self.ctx.auth().create_token(&account.id, None);
        let secret = random_bytes::<32>();

        write_credentials(&config, token.clone(), secret).unwrap();

        let credentials = vibe_agent::credentials::read_credentials(&config).unwrap();
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

fn run_cli(home: &TempDir, server_url: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vibe-agent"))
        .args(args)
        .env("VIBE_HOME_DIR", home.path())
        .env("VIBE_SERVER_URL", server_url)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
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
async fn cli_create_list_status_send_and_history_work_against_real_vibe_server() {
    let server = TestServer::start().await;
    let (home, config, credentials, _user_id) = server.provision_credentials();
    let api = ApiClient::new(config, credentials);

    let create = run_cli(
        &home,
        &server.server_url,
        &[
            "create",
            "--tag",
            "wave3-cli",
            "--path",
            "/tmp/cli-project",
            "--json",
        ],
    );
    assert!(create.status.success(), "{}", stderr(&create));
    let created: Value = serde_json::from_str(&stdout(&create)).unwrap();
    let session_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["metadata"]["tag"], "wave3-cli");
    assert_eq!(created["metadata"]["path"], "/tmp/cli-project");

    let list = run_cli(&home, &server.server_url, &["list", "--json"]);
    assert!(list.status.success(), "{}", stderr(&list));
    let listed: Value = serde_json::from_str(&stdout(&list)).unwrap();
    assert!(
        listed
            .as_array()
            .unwrap()
            .iter()
            .any(|session| session["id"] == session_id)
    );

    let status = run_cli(
        &home,
        &server.server_url,
        &["status", &session_id, "--json"],
    );
    assert!(status.status.success(), "{}", stderr(&status));
    let status_json: Value = serde_json::from_str(&stdout(&status)).unwrap();
    assert_eq!(status_json["id"], session_id);
    assert_eq!(status_json["metadata"]["tag"], "wave3-cli");

    let send = run_cli(
        &home,
        &server.server_url,
        &["send", &session_id, "hello from cli", "--json"],
    );
    assert!(send.status.success(), "{}", stderr(&send));
    let send_json: Value = serde_json::from_str(&stdout(&send)).unwrap();
    assert_eq!(send_json["sessionId"], session_id);
    assert_eq!(send_json["message"], "hello from cli");
    assert_eq!(send_json["sent"], true);

    let history = run_cli(
        &home,
        &server.server_url,
        &["history", &session_id, "--json"],
    );
    assert!(history.status.success(), "{}", stderr(&history));
    let history_json: Value = serde_json::from_str(&stdout(&history)).unwrap();
    assert!(history_json.as_array().unwrap().iter().any(|message| {
        message["content"]["role"] == "user"
            && message["content"]["content"]["type"] == "text"
            && message["content"]["content"]["text"] == "hello from cli"
    }));

    let active = run_cli(&home, &server.server_url, &["list", "--active", "--json"]);
    assert!(active.status.success(), "{}", stderr(&active));
    let active_json: Value = serde_json::from_str(&stdout(&active)).unwrap();
    assert!(
        active_json
            .as_array()
            .unwrap()
            .iter()
            .any(|session| session["id"] == session_id && session["active"] == true)
    );

    let sessions = api.list_sessions().await.unwrap();
    assert!(sessions.iter().any(|session| session.id == session_id));

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn cli_machines_wait_and_stop_work_against_real_vibe_server() {
    let server = TestServer::start().await;
    let (home, config, credentials, user_id) = server.provision_credentials();
    let api = ApiClient::new(config.clone(), credentials.clone());

    let machine_key = random_bytes::<32>();
    let machine_metadata = encode_base64(
        &encrypt_json(
            &machine_key,
            EncryptionVariant::DataKey,
            &json!({
                "host": "builder-host",
                "platform": "linux",
                "homeDir": "/tmp/cli-machine",
            }),
        )
        .unwrap(),
    );
    let machine_state = encode_base64(
        &encrypt_json(
            &machine_key,
            EncryptionVariant::DataKey,
            &json!({
                "status": "ready",
            }),
        )
        .unwrap(),
    );
    let wrapped_machine_key =
        wrap_data_encryption_key(&machine_key, &credentials.content_key_pair.public_key);
    let _ = server.ctx.db().create_or_load_machine(
        &user_id,
        "machine-cli",
        &machine_metadata,
        Some(machine_state),
        Some(wrapped_machine_key),
    );

    let machines = run_cli(&home, &server.server_url, &["machines", "--json"]);
    assert!(machines.status.success(), "{}", stderr(&machines));
    let machines_json: Value = serde_json::from_str(&stdout(&machines)).unwrap();
    assert!(machines_json.as_array().unwrap().iter().any(|machine| {
        machine["id"] == "machine-cli" && machine["metadata"]["homeDir"] == "/tmp/cli-machine"
    }));

    let session = api
        .create_session(
            "wave3-cli-wait",
            &json!({
                "tag": "wave3-cli-wait",
                "path": "/tmp/wait-project",
                "host": "localhost",
            }),
        )
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

    let wait_cmd = {
        let home_path = home.path().to_path_buf();
        let server_url = server.server_url.clone();
        let session_id = session_id.clone();
        tokio::task::spawn_blocking(move || {
            Command::new(env!("CARGO_BIN_EXE_vibe-agent"))
                .args(["wait", &session_id, "--timeout", "5", "--json"])
                .env("VIBE_HOME_DIR", home_path)
                .env("VIBE_SERVER_URL", server_url)
                .output()
                .unwrap()
        })
    };

    tokio::time::sleep(Duration::from_millis(150)).await;
    SessionsService::new(ctx)
        .update_agent_state(&user_id, &session_id, 0, Some(idle_state))
        .unwrap();

    let wait = wait_cmd.await.unwrap();
    assert!(wait.status.success(), "{}", stderr(&wait));
    let wait_json: Value = serde_json::from_str(&stdout(&wait)).unwrap();
    assert_eq!(wait_json["sessionId"], session_id);
    assert_eq!(wait_json["idle"], true);
    assert_eq!(wait_json["timeoutSeconds"], 5);

    let stop = run_cli(&home, &server.server_url, &["stop", &session_id, "--json"]);
    assert!(stop.status.success(), "{}", stderr(&stop));
    let stop_json: Value = serde_json::from_str(&stdout(&stop)).unwrap();
    assert_eq!(stop_json["sessionId"], session_id);
    assert_eq!(stop_json["stopped"], true);

    assert!(wait_for_session_to_become_inactive(&api, &session_id).await);

    let active = api.list_active_sessions().await.unwrap();
    assert!(!active.iter().any(|candidate| candidate.id == session_id));

    server.shutdown().await;
}
