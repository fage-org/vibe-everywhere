use std::{sync::Arc, time::Duration};

use clap::ValueEnum;
use futures_util::FutureExt;
use rust_socketio::{
    Event, Payload, TransportType,
    asynchronous::{Client as SocketClient, ClientBuilder},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::{Mutex, Notify, oneshot};

use crate::{
    api::DecryptedMachine,
    config::Config,
    encryption::{EncryptionError, decode_base64, decrypt_json, encode_base64, encrypt_json},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum SupportedAgent {
    #[serde(rename = "claude")]
    Claude,
    #[serde(rename = "codex")]
    Codex,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "openclaw")]
    Openclaw,
}

#[derive(Debug, Clone)]
pub struct SpawnMachineSessionOptions {
    pub directory: String,
    pub approved_new_directory_creation: bool,
    pub agent: Option<SupportedAgent>,
    pub provider_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SpawnMachineSessionResult {
    #[serde(rename = "success")]
    Success {
        #[serde(rename = "sessionId")]
        session_id: String,
    },
    #[serde(rename = "requestToApproveDirectoryCreation")]
    RequestToApproveDirectoryCreation { directory: String },
    #[serde(rename = "error")]
    Error {
        #[serde(rename = "errorMessage")]
        error_message: String,
    },
}

#[derive(Debug, Deserialize)]
struct RpcAck {
    ok: bool,
    result: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Default)]
struct ConnectionState {
    connected: bool,
    last_error: Option<String>,
}

#[derive(Debug, Error)]
pub enum MachineRpcError {
    #[error("Timeout waiting for socket connection")]
    ConnectionTimeout,
    #[error("{0}")]
    Connection(String),
    #[error("RPC call timed out")]
    Timeout,
    #[error("RPC call failed")]
    RpcCallFailed,
    #[error("RPC call returned no result")]
    MissingResult,
    #[error("RPC call returned invalid data")]
    InvalidResult,
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Socket(#[from] rust_socketio::Error),
    #[error(transparent)]
    Encryption(#[from] EncryptionError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub async fn spawn_session_on_machine(
    config: &Config,
    machine: &DecryptedMachine,
    token: &str,
    options: SpawnMachineSessionOptions,
) -> Result<SpawnMachineSessionResult, MachineRpcError> {
    call_machine_rpc(
        config,
        machine,
        token,
        "spawn-happy-session",
        serde_json::json!({
            "type": "spawn-in-directory",
            "directory": options.directory,
            "approvedNewDirectoryCreation": options.approved_new_directory_creation,
            "token": options.provider_token,
            "agent": options.agent,
        }),
    )
    .await
}

pub async fn resume_session_on_machine(
    config: &Config,
    machine: &DecryptedMachine,
    token: &str,
    session_id: &str,
) -> Result<SpawnMachineSessionResult, MachineRpcError> {
    call_machine_rpc(
        config,
        machine,
        token,
        "resume-happy-session",
        serde_json::json!({
            "sessionId": session_id,
        }),
    )
    .await
}

async fn call_machine_rpc(
    config: &Config,
    machine: &DecryptedMachine,
    token: &str,
    method_suffix: &str,
    params: Value,
) -> Result<SpawnMachineSessionResult, MachineRpcError> {
    call_machine_rpc_with_timeout(
        config,
        machine,
        token,
        method_suffix,
        params,
        Duration::from_secs(30),
    )
    .await
}

async fn call_machine_rpc_with_timeout(
    config: &Config,
    machine: &DecryptedMachine,
    token: &str,
    method_suffix: &str,
    params: Value,
    timeout: Duration,
) -> Result<SpawnMachineSessionResult, MachineRpcError> {
    let socket = connect_rpc_socket(config, token).await?;
    let outcome: Result<SpawnMachineSessionResult, MachineRpcError> = async {
        let encrypted = encrypt_json(&machine.encryption.key, machine.encryption.variant, &params)?;
        let ack_value = emit_with_ack_value(
            &socket,
            "rpc-call",
            serde_json::json!({
                "method": format!("{}:{method_suffix}", machine.id),
                "params": encode_base64(&encrypted),
            }),
            timeout,
        )
        .await?;
        let ack: RpcAck = serde_json::from_value(unwrap_singleton_array(ack_value))?;

        if !ack.ok {
            return Err(MachineRpcError::Message(normalize_rpc_error(
                ack.error.as_deref(),
                &machine.id,
            )));
        }
        let Some(result) = ack.result else {
            return Err(MachineRpcError::MissingResult);
        };
        let decrypted = decrypt_json(
            &machine.encryption.key,
            machine.encryption.variant,
            &decode_base64(&result)?,
        )
        .ok_or(MachineRpcError::InvalidResult)?;

        parse_spawn_machine_session_result(decrypted)
    }
    .await;

    let _ = socket.disconnect().await;
    outcome
}

async fn connect_rpc_socket(config: &Config, token: &str) -> Result<SocketClient, MachineRpcError> {
    let state = Arc::new(Mutex::new(ConnectionState::default()));
    let ready = Arc::new(Notify::new());

    let connect_state = state.clone();
    let connect_ready = ready.clone();

    let error_state = state.clone();
    let error_ready = ready.clone();

    let close_state = state.clone();
    let close_ready = ready.clone();

    let builder = ClientBuilder::new(config.socket_url())
        .transport_type(TransportType::Websocket)
        .reconnect(false)
        .auth(serde_json::json!({ "token": token }))
        .on(Event::Connect, move |_, _| {
            let state = connect_state.clone();
            let ready = connect_ready.clone();
            async move {
                let mut guard = state.lock().await;
                guard.connected = true;
                guard.last_error = None;
                drop(guard);
                ready.notify_waiters();
            }
            .boxed()
        })
        .on(Event::Error, move |payload, _| {
            let state = error_state.clone();
            let ready = error_ready.clone();
            let message = payload_to_string(&payload);
            async move {
                let mut guard = state.lock().await;
                guard.last_error = Some(message);
                drop(guard);
                ready.notify_waiters();
            }
            .boxed()
        })
        .on(Event::Close, move |_, _| {
            let state = close_state.clone();
            let ready = close_ready.clone();
            async move {
                let mut guard = state.lock().await;
                guard.last_error = Some("Socket disconnected".into());
                drop(guard);
                ready.notify_waiters();
            }
            .boxed()
        });

    let socket = tokio::time::timeout(Duration::from_secs(10), builder.connect())
        .await
        .map_err(|_| MachineRpcError::ConnectionTimeout)??;

    let wait = async {
        loop {
            let notified = ready.notified();
            {
                let guard = state.lock().await;
                if guard.connected {
                    return Ok(());
                }
                if let Some(error) = guard.last_error.clone() {
                    return Err(MachineRpcError::Connection(error));
                }
            }
            notified.await;
        }
    };

    tokio::time::timeout(Duration::from_secs(10), wait)
        .await
        .map_err(|_| MachineRpcError::ConnectionTimeout)??;

    Ok(socket)
}

async fn emit_with_ack_value(
    socket: &SocketClient,
    event: &str,
    payload: Value,
    timeout: Duration,
) -> Result<Value, MachineRpcError> {
    let (sender, receiver) = oneshot::channel();
    let sender = Arc::new(Mutex::new(Some(sender)));

    socket
        .emit_with_ack(event, payload, timeout, {
            let sender = sender.clone();
            move |payload: Payload, _| {
                let sender = sender.clone();
                async move {
                    if let Some(value) = first_payload_value(&payload)
                        && let Some(sender) = sender.lock().await.take()
                    {
                        let _ = sender.send(value);
                    }
                }
                .boxed()
            }
        })
        .await?;

    tokio::time::timeout(timeout + Duration::from_secs(1), receiver)
        .await
        .map_err(|_| MachineRpcError::Timeout)?
        .map_err(|_| MachineRpcError::Timeout)
}

fn normalize_rpc_error(error: Option<&str>, machine_id: &str) -> String {
    match error {
        Some("RPC method not available") => {
            format!("Machine {machine_id} is offline or its daemon is not connected.")
        }
        Some(error) if !error.is_empty() => error.to_string(),
        _ => "RPC call failed".into(),
    }
}

fn parse_spawn_machine_session_result(
    value: Value,
) -> Result<SpawnMachineSessionResult, MachineRpcError> {
    let Value::Object(object) = value else {
        return Err(MachineRpcError::InvalidResult);
    };

    if let Some(error) = object.get("error").and_then(Value::as_str) {
        return Err(MachineRpcError::Message(error.to_string()));
    }

    match object.get("type").and_then(Value::as_str) {
        Some("success" | "requestToApproveDirectoryCreation" | "error") => {
            serde_json::from_value(Value::Object(object)).map_err(MachineRpcError::from)
        }
        _ => Err(MachineRpcError::Message(
            "RPC call returned unexpected data".into(),
        )),
    }
}

fn unwrap_singleton_array(value: Value) -> Value {
    match value {
        Value::Array(mut values) if values.len() == 1 => values.pop().unwrap_or(Value::Null),
        other => other,
    }
}

fn first_payload_value(payload: &Payload) -> Option<Value> {
    match payload {
        Payload::Text(values) => values.first().cloned(),
        Payload::Binary(_) => None,
        #[allow(deprecated)]
        Payload::String(value) => serde_json::from_str(value)
            .ok()
            .or_else(|| Some(Value::String(value.clone()))),
    }
}

fn payload_to_string(payload: &Payload) -> String {
    match payload {
        Payload::Text(values) => values
            .first()
            .map(|value| match value {
                Value::String(value) => value.clone(),
                other => other.to_string(),
            })
            .unwrap_or_else(|| "unknown socket error".into()),
        Payload::Binary(_) => "binary socket error".into(),
        #[allow(deprecated)]
        Payload::String(value) => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::TcpListener as StdTcpListener,
        sync::{Arc, Mutex as StdMutex},
        time::Duration,
    };

    use axum::Router;
    use serde_json::Value;
    use socketioxide::{
        SocketIo,
        extract::{AckSender, SocketRef, TryData},
    };
    use tempfile::TempDir;
    use tokio::{sync::oneshot, task::JoinHandle};

    use crate::{
        api::{DecryptedMachine, RecordEncryption},
        config::Config,
        encryption::{EncryptionVariant, decode_base64, decrypt_json, encode_base64, encrypt_json},
    };

    use serde_json::json;

    use super::{
        MachineRpcError, SpawnMachineSessionOptions, SpawnMachineSessionResult, SupportedAgent,
        call_machine_rpc_with_timeout, connect_rpc_socket, emit_with_ack_value,
        normalize_rpc_error, parse_spawn_machine_session_result, resume_session_on_machine,
        spawn_session_on_machine,
    };

    enum AckBehavior {
        Success(Value),
        Error(String),
        Ignore,
    }

    struct MockRpcState {
        encryption: RecordEncryption,
        expected_method: String,
        expected_params: Option<Value>,
        seen_method: StdMutex<Option<String>>,
        seen_params: StdMutex<Option<Value>>,
        ack_behavior: AckBehavior,
    }

    struct MockRpcServer {
        server_url: String,
        shutdown: Option<oneshot::Sender<()>>,
        task: Option<JoinHandle<()>>,
    }

    impl MockRpcServer {
        async fn start(state: Arc<MockRpcState>) -> Self {
            let (layer, io) = SocketIo::builder().req_path("/v1/updates").build_layer();
            io.ns("/", move |socket: SocketRef, _auth: TryData<Value>| {
                let state = state.clone();
                async move {
                    socket.on(
                        "rpc-call",
                        move |ack: AckSender, TryData(payload): TryData<Value>| {
                            let state = state.clone();
                            async move {
                                let Ok(payload) = payload else {
                                    return;
                                };
                                *state.seen_method.lock().unwrap() = payload
                                    .get("method")
                                    .and_then(Value::as_str)
                                    .map(ToOwned::to_owned);

                                if let Some(expected_params) = state.expected_params.as_ref() {
                                    let decrypted_params = payload
                                        .get("params")
                                        .and_then(Value::as_str)
                                        .and_then(|value| decode_base64(value).ok())
                                        .and_then(|ciphertext| {
                                            decrypt_json(
                                                &state.encryption.key,
                                                state.encryption.variant,
                                                &ciphertext,
                                            )
                                        });
                                    *state.seen_params.lock().unwrap() = decrypted_params.clone();
                                    assert_eq!(decrypted_params.as_ref(), Some(expected_params));
                                }

                                match &state.ack_behavior {
                                    AckBehavior::Success(result) => {
                                        let encrypted_result = encrypt_json(
                                            &state.encryption.key,
                                            state.encryption.variant,
                                            result,
                                        )
                                        .unwrap();
                                        let _ = ack.send(&json!({
                                            "ok": true,
                                            "result": encode_base64(&encrypted_result),
                                        }));
                                    }
                                    AckBehavior::Error(message) => {
                                        let _ = ack.send(&json!({
                                            "ok": false,
                                            "error": message,
                                        }));
                                    }
                                    AckBehavior::Ignore => {}
                                }
                            }
                        },
                    );
                }
            });

            let std_listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
            let addr = std_listener.local_addr().unwrap();
            std_listener.set_nonblocking(true).unwrap();
            let listener = tokio::net::TcpListener::from_std(std_listener).unwrap();
            let (shutdown_tx, shutdown_rx) = oneshot::channel();
            let app = Router::new().layer(layer);
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

        fn config(&self) -> (TempDir, Config) {
            let temp_dir = TempDir::new().unwrap();
            let config = Config::from_sources(
                Some(self.server_url.clone()),
                Some(temp_dir.path().as_os_str().to_owned()),
            )
            .unwrap();
            (temp_dir, config)
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

    fn test_machine() -> DecryptedMachine {
        DecryptedMachine {
            id: "machine-1".into(),
            seq: 1,
            created_at: 1,
            updated_at: 1,
            active: true,
            active_at: 1,
            metadata: json!({}),
            metadata_version: 1,
            daemon_state: None,
            daemon_state_version: 0,
            data_encryption_key: None,
            encryption: RecordEncryption {
                key: [7u8; 32],
                variant: EncryptionVariant::Legacy,
            },
        }
    }

    #[test]
    fn normalize_rpc_error_maps_unavailable_machine_message() {
        assert_eq!(
            normalize_rpc_error(Some("RPC method not available"), "machine-1"),
            "Machine machine-1 is offline or its daemon is not connected."
        );
    }

    #[test]
    fn normalize_rpc_error_preserves_specific_messages() {
        assert_eq!(
            normalize_rpc_error(Some("permission denied"), "machine-1"),
            "permission denied"
        );
        assert_eq!(normalize_rpc_error(None, "machine-1"), "RPC call failed");
    }

    #[test]
    fn spawn_result_shapes_round_trip_through_serde() {
        let success: SpawnMachineSessionResult =
            serde_json::from_value(json!({"type": "success", "sessionId": "ses-1"})).unwrap();
        assert!(matches!(
            success,
            SpawnMachineSessionResult::Success { session_id } if session_id == "ses-1"
        ));

        let create_dir: SpawnMachineSessionResult = serde_json::from_value(json!({
            "type": "requestToApproveDirectoryCreation",
            "directory": "/tmp/project"
        }))
        .unwrap();
        assert!(matches!(
            create_dir,
            SpawnMachineSessionResult::RequestToApproveDirectoryCreation { directory }
                if directory == "/tmp/project"
        ));

        let error: SpawnMachineSessionResult = serde_json::from_value(json!({
            "type": "error",
            "errorMessage": "daemon failed"
        }))
        .unwrap();
        assert!(matches!(
            error,
            SpawnMachineSessionResult::Error { error_message } if error_message == "daemon failed"
        ));
    }

    #[test]
    fn parse_spawn_machine_session_result_prefers_error_field() {
        let error = parse_spawn_machine_session_result(json!({
            "error": "daemon failed",
            "type": "success",
            "sessionId": "ignored",
        }))
        .unwrap_err();

        assert!(matches!(
            error,
            MachineRpcError::Message(message) if message == "daemon failed"
        ));
    }

    #[test]
    fn parse_spawn_machine_session_result_rejects_unexpected_shapes() {
        let error = parse_spawn_machine_session_result(json!({
            "sessionId": "missing-type",
        }))
        .unwrap_err();
        assert!(matches!(
            error,
            MachineRpcError::Message(message) if message == "RPC call returned unexpected data"
        ));

        let error = parse_spawn_machine_session_result(json!("not-an-object")).unwrap_err();
        assert!(matches!(error, MachineRpcError::InvalidResult));
    }

    #[test]
    fn supported_agent_values_match_happy_contract() {
        assert_eq!(
            serde_json::to_value(SupportedAgent::Claude).unwrap(),
            json!("claude")
        );
        assert_eq!(
            serde_json::to_value(SupportedAgent::Codex).unwrap(),
            json!("codex")
        );
        assert_eq!(
            serde_json::to_value(SupportedAgent::Gemini).unwrap(),
            json!("gemini")
        );
        assert_eq!(
            serde_json::to_value(SupportedAgent::Openclaw).unwrap(),
            json!("openclaw")
        );
    }

    #[tokio::test]
    async fn spawn_session_on_machine_round_trips_request_and_response() {
        let machine = test_machine();
        let state = Arc::new(MockRpcState {
            encryption: machine.encryption.clone(),
            expected_method: "machine-1:spawn-happy-session".into(),
            expected_params: Some(json!({
                "type": "spawn-in-directory",
                "directory": "/tmp/project",
                "approvedNewDirectoryCreation": true,
                "token": "provider-token",
                "agent": "codex",
            })),
            seen_method: StdMutex::new(None),
            seen_params: StdMutex::new(None),
            ack_behavior: AckBehavior::Success(json!({
                "type": "success",
                "sessionId": "session-123",
            })),
        });
        let server = MockRpcServer::start(state.clone()).await;
        let (_temp_dir, config) = server.config();

        let result = spawn_session_on_machine(
            &config,
            &machine,
            "token",
            SpawnMachineSessionOptions {
                directory: "/tmp/project".into(),
                approved_new_directory_creation: true,
                agent: Some(SupportedAgent::Codex),
                provider_token: Some("provider-token".into()),
            },
        )
        .await
        .unwrap();

        assert_eq!(
            state.seen_method.lock().unwrap().as_deref(),
            Some(state.expected_method.as_str())
        );
        assert!(matches!(
            result,
            SpawnMachineSessionResult::Success { session_id } if session_id == "session-123"
        ));

        server.shutdown().await;
    }

    #[tokio::test]
    async fn resume_session_on_machine_round_trips_request_and_response() {
        let machine = test_machine();
        let state = Arc::new(MockRpcState {
            encryption: machine.encryption.clone(),
            expected_method: "machine-1:resume-happy-session".into(),
            expected_params: Some(json!({
                "sessionId": "source-session",
            })),
            seen_method: StdMutex::new(None),
            seen_params: StdMutex::new(None),
            ack_behavior: AckBehavior::Success(json!({
                "type": "success",
                "sessionId": "resumed-session",
            })),
        });
        let server = MockRpcServer::start(state.clone()).await;
        let (_temp_dir, config) = server.config();

        let result = resume_session_on_machine(&config, &machine, "token", "source-session")
            .await
            .unwrap();

        assert_eq!(
            state.seen_method.lock().unwrap().as_deref(),
            Some(state.expected_method.as_str())
        );
        assert!(matches!(
            result,
            SpawnMachineSessionResult::Success { session_id } if session_id == "resumed-session"
        ));

        server.shutdown().await;
    }

    #[tokio::test]
    async fn machine_rpc_maps_offline_machine_errors() {
        let machine = test_machine();
        let state = Arc::new(MockRpcState {
            encryption: machine.encryption.clone(),
            expected_method: "machine-1:spawn-happy-session".into(),
            expected_params: Some(json!({
                "type": "spawn-in-directory",
                "directory": "/tmp/project",
                "approvedNewDirectoryCreation": false,
                "token": null,
                "agent": null,
            })),
            seen_method: StdMutex::new(None),
            seen_params: StdMutex::new(None),
            ack_behavior: AckBehavior::Error("RPC method not available".into()),
        });
        let server = MockRpcServer::start(state).await;
        let (_temp_dir, config) = server.config();

        let error = spawn_session_on_machine(
            &config,
            &machine,
            "token",
            SpawnMachineSessionOptions {
                directory: "/tmp/project".into(),
                approved_new_directory_creation: false,
                agent: None,
                provider_token: None,
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            MachineRpcError::Message(message)
                if message == "Machine machine-1 is offline or its daemon is not connected."
        ));

        server.shutdown().await;
    }

    #[tokio::test]
    async fn machine_rpc_surfaces_decrypted_error_objects() {
        let machine = test_machine();
        let state = Arc::new(MockRpcState {
            encryption: machine.encryption.clone(),
            expected_method: "machine-1:spawn-happy-session".into(),
            expected_params: Some(json!({
                "type": "spawn-in-directory",
                "directory": "/tmp/project",
                "approvedNewDirectoryCreation": false,
                "token": null,
                "agent": null,
            })),
            seen_method: StdMutex::new(None),
            seen_params: StdMutex::new(None),
            ack_behavior: AckBehavior::Success(json!({
                "error": "daemon failed",
            })),
        });
        let server = MockRpcServer::start(state).await;
        let (_temp_dir, config) = server.config();

        let error = spawn_session_on_machine(
            &config,
            &machine,
            "token",
            SpawnMachineSessionOptions {
                directory: "/tmp/project".into(),
                approved_new_directory_creation: false,
                agent: None,
                provider_token: None,
            },
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            MachineRpcError::Message(message) if message == "daemon failed"
        ));

        server.shutdown().await;
    }

    #[tokio::test]
    async fn machine_rpc_times_out_when_ack_is_missing() {
        let machine = test_machine();
        let state = Arc::new(MockRpcState {
            encryption: machine.encryption.clone(),
            expected_method: "machine-1:spawn-happy-session".into(),
            expected_params: None,
            seen_method: StdMutex::new(None),
            seen_params: StdMutex::new(None),
            ack_behavior: AckBehavior::Ignore,
        });
        let server = MockRpcServer::start(state).await;
        let (_temp_dir, config) = server.config();

        let error = call_machine_rpc_with_timeout(
            &config,
            &machine,
            "token",
            "spawn-happy-session",
            json!({
                "type": "spawn-in-directory",
                "directory": "/tmp/project",
                "approvedNewDirectoryCreation": false,
                "token": null,
                "agent": null,
            }),
            Duration::from_millis(100),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, MachineRpcError::Timeout));
        server.shutdown().await;
    }

    #[tokio::test]
    async fn emit_with_ack_value_times_out_without_server_ack() {
        let machine = test_machine();
        let state = Arc::new(MockRpcState {
            encryption: machine.encryption.clone(),
            expected_method: "machine-1:spawn-happy-session".into(),
            expected_params: None,
            seen_method: StdMutex::new(None),
            seen_params: StdMutex::new(None),
            ack_behavior: AckBehavior::Ignore,
        });
        let server = MockRpcServer::start(state).await;
        let (_temp_dir, config) = server.config();
        let socket = connect_rpc_socket(&config, "token").await.unwrap();

        let error = emit_with_ack_value(
            &socket,
            "rpc-call",
            json!({
                "method": "machine-1:spawn-happy-session",
                "params": "ignored",
            }),
            Duration::from_millis(50),
        )
        .await
        .unwrap_err();
        assert!(matches!(error, MachineRpcError::Timeout));

        socket.disconnect().await.unwrap();
        server.shutdown().await;
    }
}
