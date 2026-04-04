use std::{
    net::TcpListener as StdTcpListener,
    process::{Command, Output},
    sync::{Arc, Mutex},
};

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use socketioxide::{
    SocketIo,
    extract::{AckSender, SocketRef, TryData},
};
use tempfile::TempDir;
use tokio::{sync::oneshot, task::JoinHandle};
use vibe_agent::{
    config::Config,
    credentials::write_credentials,
    encryption::{EncryptionVariant, encode_base64, encrypt_json},
};

struct MockRpcServer {
    server_url: String,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

impl MockRpcServer {
    async fn start(
        secret: [u8; 32],
        session_id: Option<&str>,
        machine_metadata: Value,
        rpc_result: Value,
        expected_method: Arc<Mutex<Option<String>>>,
        expected_params: Arc<Mutex<Option<Value>>>,
    ) -> Self {
        let machine_metadata = encode_base64(
            &encrypt_json(&secret, EncryptionVariant::Legacy, &machine_metadata).unwrap(),
        );
        let session_payload = session_id.map(|session_id| {
            let metadata = encode_base64(
                &encrypt_json(
                    &secret,
                    EncryptionVariant::Legacy,
                    &json!({
                        "machineId": "machine-1",
                        "path": "/tmp/remote-home",
                    }),
                )
                .unwrap(),
            );

            json!({
                "id": session_id,
                "seq": 1,
                "createdAt": 1,
                "updatedAt": 1,
                "active": true,
                "activeAt": 1,
                "metadata": metadata,
                "metadataVersion": 1,
                "agentState": Value::Null,
                "agentStateVersion": 0,
                "dataEncryptionKey": Value::Null,
            })
        });
        let machine_payload = json!([{
            "id": "machine-1",
            "seq": 1,
            "createdAt": 1,
            "updatedAt": 1,
            "active": true,
            "activeAt": 1,
            "metadata": machine_metadata,
            "metadataVersion": 1,
            "daemonState": Value::Null,
            "daemonStateVersion": 0,
            "dataEncryptionKey": Value::Null,
        }]);

        let (layer, io) = SocketIo::builder().req_path("/v1/updates").build_layer();
        io.ns("/", move |socket: SocketRef, _auth: TryData<Value>| {
            let expected_method = expected_method.clone();
            let expected_params = expected_params.clone();
            let rpc_result = rpc_result.clone();
            async move {
                socket.on(
                    "rpc-call",
                    move |ack: AckSender, TryData(payload): TryData<Value>| {
                        let expected_method = expected_method.clone();
                        let expected_params = expected_params.clone();
                        let rpc_result = rpc_result.clone();
                        async move {
                            let Ok(payload) = payload else {
                                return;
                            };

                            *expected_method.lock().unwrap() = payload
                                .get("method")
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned);

                            let decrypted_params = payload
                                .get("params")
                                .and_then(Value::as_str)
                                .and_then(|value| vibe_agent::encryption::decode_base64(value).ok())
                                .and_then(|ciphertext| {
                                    vibe_agent::encryption::decrypt_json(
                                        &secret,
                                        EncryptionVariant::Legacy,
                                        &ciphertext,
                                    )
                                });
                            *expected_params.lock().unwrap() = decrypted_params;

                            let encrypted_result =
                                encrypt_json(&secret, EncryptionVariant::Legacy, &rpc_result)
                                    .unwrap();
                            let _ = ack.send(&json!({
                                "ok": true,
                                "result": encode_base64(&encrypted_result),
                            }));
                        }
                    },
                );
            }
        });

        let app = Router::new()
            .route(
                "/v1/machines",
                get({
                    let machine_payload = machine_payload.clone();
                    move || {
                        let machine_payload = machine_payload.clone();
                        async move { Json(machine_payload) }
                    }
                }),
            )
            .route(
                "/v1/sessions",
                get(move || {
                    let session_payload = session_payload.clone();
                    async move {
                        Json(json!({
                            "sessions": session_payload.into_iter().collect::<Vec<_>>(),
                        }))
                    }
                }),
            )
            .layer(layer);

        let std_listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
        let addr = std_listener.local_addr().unwrap();
        std_listener.set_nonblocking(true).unwrap();
        let listener = tokio::net::TcpListener::from_std(std_listener).unwrap();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
        });

        Self {
            server_url: format!("http://{addr}"),
            shutdown: Some(shutdown_tx),
            task: Some(task),
        }
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

impl Drop for MockRpcServer {
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

fn write_test_credentials(home: &TempDir, server_url: &str, secret: [u8; 32]) {
    let config = Config::from_sources(
        Some(server_url.to_string()),
        Some(home.path().as_os_str().to_owned()),
    )
    .unwrap();
    write_credentials(&config, "token-1", secret).unwrap();
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn spawn_command_round_trips_through_machine_rpc() {
    let secret = [7u8; 32];
    let seen_method = Arc::new(Mutex::new(None));
    let seen_params = Arc::new(Mutex::new(None));
    let server = MockRpcServer::start(
        secret,
        None,
        json!({ "homeDir": "/tmp/remote-home" }),
        json!({ "type": "success", "sessionId": "spawned-session" }),
        seen_method.clone(),
        seen_params.clone(),
    )
    .await;
    let home = TempDir::new().unwrap();
    write_test_credentials(&home, &server.server_url, secret);

    let output = run_cli(
        &home,
        &server.server_url,
        &["spawn", "--machine", "machine-1", "--json"],
    );
    assert!(output.status.success(), "{}", stdout(&output));

    let parsed: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(parsed["type"], "success");
    assert_eq!(parsed["machineId"], "machine-1");
    assert_eq!(parsed["sessionId"], "spawned-session");
    assert_eq!(parsed["directory"], "/tmp/remote-home");
    assert_eq!(
        seen_method.lock().unwrap().as_deref(),
        Some("machine-1:spawn-happy-session")
    );
    assert_eq!(
        seen_params.lock().unwrap().clone(),
        Some(json!({
            "type": "spawn-in-directory",
            "directory": "/tmp/remote-home",
            "approvedNewDirectoryCreation": false,
            "token": Value::Null,
            "agent": Value::Null,
        }))
    );

    server.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn resume_command_round_trips_through_machine_rpc() {
    let secret = [8u8; 32];
    let seen_method = Arc::new(Mutex::new(None));
    let seen_params = Arc::new(Mutex::new(None));
    let server = MockRpcServer::start(
        secret,
        Some("session-1"),
        json!({
            "homeDir": "/tmp/remote-home",
            "resumeSupport": {
                "rpcAvailable": true
            }
        }),
        json!({ "type": "success", "sessionId": "resumed-session" }),
        seen_method.clone(),
        seen_params.clone(),
    )
    .await;
    let home = TempDir::new().unwrap();
    write_test_credentials(&home, &server.server_url, secret);

    let output = run_cli(
        &home,
        &server.server_url,
        &["resume", "session-1", "--json"],
    );
    assert!(output.status.success(), "{}", stdout(&output));

    let parsed: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(parsed["type"], "success");
    assert_eq!(parsed["machineId"], "machine-1");
    assert_eq!(parsed["sourceSessionId"], "session-1");
    assert_eq!(parsed["sessionId"], "resumed-session");
    assert_eq!(
        seen_method.lock().unwrap().as_deref(),
        Some("machine-1:resume-happy-session")
    );
    assert_eq!(
        seen_params.lock().unwrap().clone(),
        Some(json!({
            "sessionId": "session-1",
        }))
    );

    server.shutdown().await;
}
