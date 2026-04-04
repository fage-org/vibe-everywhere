use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use vibe_wire::SessionMessage;

use crate::{
    config::Config,
    credentials::Credentials,
    encryption::{
        EncryptionError, EncryptionVariant, decode_base64, decrypt_json, encrypt_json,
        random_bytes, unwrap_data_encryption_key, wrap_data_encryption_key,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEncryption {
    pub key: [u8; 32],
    pub variant: EncryptionVariant,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSession {
    pub id: String,
    pub seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
    pub active_at: u64,
    pub metadata: String,
    pub metadata_version: u64,
    pub agent_state: Option<String>,
    pub agent_state_version: u64,
    pub data_encryption_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawMachine {
    pub id: String,
    pub seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
    pub active_at: u64,
    pub metadata: String,
    pub metadata_version: u64,
    pub daemon_state: Option<String>,
    pub daemon_state_version: u64,
    pub data_encryption_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptedSession {
    pub id: String,
    pub seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
    pub active_at: u64,
    pub metadata: Value,
    #[serde(skip_serializing)]
    pub metadata_version: u64,
    pub agent_state: Option<Value>,
    #[serde(skip_serializing)]
    pub agent_state_version: u64,
    #[serde(skip_serializing)]
    pub data_encryption_key: Option<String>,
    #[serde(skip_serializing)]
    pub encryption: RecordEncryption,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptedMachine {
    pub id: String,
    pub seq: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub active: bool,
    pub active_at: u64,
    pub metadata: Value,
    pub metadata_version: u64,
    pub daemon_state: Option<Value>,
    pub daemon_state_version: u64,
    #[serde(skip_serializing)]
    pub data_encryption_key: Option<String>,
    #[serde(skip_serializing)]
    pub encryption: RecordEncryption,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptedMessage {
    pub id: String,
    pub seq: u64,
    pub content: Value,
    pub local_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    http: Client,
    config: Config,
    credentials: Credentials,
}

#[derive(Debug, Error)]
#[error("{message}")]
pub struct ApiError {
    message: String,
}

impl ApiError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn from_http(status: StatusCode, context: &str, body: &str) -> Self {
        if status == StatusCode::UNAUTHORIZED {
            return Self::new(
                "Authentication expired. Run `vibe-agent auth login` to re-authenticate.",
            );
        }
        if status == StatusCode::FORBIDDEN {
            return Self::new(format!(
                "Forbidden: {context}. Check your account permissions."
            ));
        }
        if status == StatusCode::NOT_FOUND {
            return Self::new(format!("Not found: {context}"));
        }
        if status.is_client_error() {
            let detail = if body.trim().is_empty() {
                String::new()
            } else {
                format!(": {}", body.trim())
            };
            return Self::new(format!("Request failed ({status}){detail}"));
        }
        if status.is_server_error() {
            return Self::new(format!("Server error ({status}): {context}"));
        }

        Self::new(format!("Request failed ({status}): {context}"))
    }
}

#[derive(Debug, Deserialize)]
struct SessionListResponse {
    sessions: Vec<RawSession>,
}

#[derive(Debug, Deserialize)]
struct CreateSessionResponse {
    session: RawSession,
}

#[derive(Debug, Deserialize)]
struct SessionHistoryResponse {
    messages: Vec<SessionMessage>,
}

#[derive(Debug, Deserialize)]
struct MachineDetailResponse {
    machine: RawMachine,
}

impl ApiClient {
    pub fn new(config: Config, credentials: Credentials) -> Self {
        Self {
            http: Client::new(),
            config,
            credentials,
        }
    }

    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    pub async fn list_sessions(&self) -> Result<Vec<DecryptedSession>, ApiError> {
        let response: SessionListResponse =
            self.get_json("/v1/sessions", "listing sessions").await?;
        response
            .sessions
            .into_iter()
            .map(|session| decrypt_session(session, &self.credentials))
            .collect()
    }

    pub async fn list_active_sessions(&self) -> Result<Vec<DecryptedSession>, ApiError> {
        let response: SessionListResponse = self
            .get_json("/v2/sessions/active", "listing active sessions")
            .await?;
        response
            .sessions
            .into_iter()
            .map(|session| decrypt_session(session, &self.credentials))
            .collect()
    }

    pub async fn list_machines(&self) -> Result<Vec<DecryptedMachine>, ApiError> {
        let response: Vec<RawMachine> = self.get_json("/v1/machines", "listing machines").await?;
        response
            .into_iter()
            .map(|machine| decrypt_machine(machine, &self.credentials))
            .collect()
    }

    pub async fn get_machine(&self, machine_id: &str) -> Result<DecryptedMachine, ApiError> {
        let response: MachineDetailResponse = self
            .get_json(
                &format!("/v1/machines/{}", encode_path_component(machine_id)),
                &format!("machine {machine_id} details"),
            )
            .await?;
        decrypt_machine(response.machine, &self.credentials)
    }

    pub async fn create_session(
        &self,
        tag: &str,
        metadata: &Value,
    ) -> Result<DecryptedSession, ApiError> {
        let session_key = random_bytes::<32>();
        let encrypted_metadata = encrypt_json(&session_key, EncryptionVariant::DataKey, metadata)
            .map_err(ApiError::from)?;
        let data_encryption_key =
            wrap_data_encryption_key(&session_key, &self.credentials.content_key_pair.public_key);

        let response: CreateSessionResponse = self
            .post_json(
                "/v1/sessions",
                &serde_json::json!({
                    "tag": tag,
                    "metadata": crate::encryption::encode_base64(&encrypted_metadata),
                    "dataEncryptionKey": data_encryption_key,
                }),
                "creating session",
            )
            .await?;
        decrypt_session(response.session, &self.credentials)
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), ApiError> {
        let path = format!("/v1/sessions/{}", encode_path_component(session_id));
        self.delete_empty(&path, &format!("deleting session {session_id}"))
            .await
    }

    pub async fn get_session_messages(
        &self,
        session_id: &str,
        encryption: &RecordEncryption,
    ) -> Result<Vec<DecryptedMessage>, ApiError> {
        let path = format!(
            "/v1/sessions/{}/messages",
            encode_path_component(session_id)
        );
        let response: SessionHistoryResponse = self
            .get_json(&path, &format!("session {session_id} messages"))
            .await?;

        Ok(response
            .messages
            .into_iter()
            .map(|message| decrypt_message(message, encryption))
            .collect())
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        context: &str,
    ) -> Result<T, ApiError> {
        let request = self
            .http
            .get(format!("{}{}", self.config.server_url, path))
            .bearer_auth(&self.credentials.token);
        send_json(request, context).await
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &Value,
        context: &str,
    ) -> Result<T, ApiError> {
        let request = self
            .http
            .post(format!("{}{}", self.config.server_url, path))
            .bearer_auth(&self.credentials.token)
            .json(body);
        send_json(request, context).await
    }

    async fn delete_empty(&self, path: &str, context: &str) -> Result<(), ApiError> {
        let request = self
            .http
            .delete(format!("{}{}", self.config.server_url, path))
            .bearer_auth(&self.credentials.token);
        let response = request
            .send()
            .await
            .map_err(|error| ApiError::new(format!("Request failed: {error}")))?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(ApiError::from_http(status, context, &body));
        }
        Ok(())
    }
}

impl From<EncryptionError> for ApiError {
    fn from(error: EncryptionError) -> Self {
        Self::new(error.to_string())
    }
}

fn decrypt_session(
    raw: RawSession,
    credentials: &Credentials,
) -> Result<DecryptedSession, ApiError> {
    let encryption = resolve_record_encryption(
        &raw.id,
        raw.data_encryption_key.as_deref(),
        credentials,
        "session",
    )?;
    let metadata = decrypt_required_field(&raw.metadata, &encryption);
    let agent_state = raw
        .agent_state
        .as_deref()
        .and_then(|value| decrypt_optional_field(value, &encryption));

    Ok(DecryptedSession {
        id: raw.id,
        seq: raw.seq,
        created_at: raw.created_at,
        updated_at: raw.updated_at,
        active: raw.active,
        active_at: raw.active_at,
        metadata,
        metadata_version: raw.metadata_version,
        agent_state,
        agent_state_version: raw.agent_state_version,
        data_encryption_key: raw.data_encryption_key,
        encryption,
    })
}

fn decrypt_machine(
    raw: RawMachine,
    credentials: &Credentials,
) -> Result<DecryptedMachine, ApiError> {
    let encryption = resolve_record_encryption(
        &raw.id,
        raw.data_encryption_key.as_deref(),
        credentials,
        "machine",
    )?;
    let metadata = decrypt_required_field(&raw.metadata, &encryption);
    let daemon_state = raw
        .daemon_state
        .as_deref()
        .and_then(|value| decrypt_optional_field(value, &encryption));

    Ok(DecryptedMachine {
        id: raw.id,
        seq: raw.seq,
        created_at: raw.created_at,
        updated_at: raw.updated_at,
        active: raw.active,
        active_at: raw.active_at,
        metadata,
        metadata_version: raw.metadata_version,
        daemon_state,
        daemon_state_version: raw.daemon_state_version,
        data_encryption_key: raw.data_encryption_key,
        encryption,
    })
}

fn decrypt_message(message: SessionMessage, encryption: &RecordEncryption) -> DecryptedMessage {
    let content = decode_base64(&message.content.ciphertext)
        .ok()
        .and_then(|ciphertext| decrypt_json(&encryption.key, encryption.variant, &ciphertext))
        .unwrap_or(Value::Null);

    DecryptedMessage {
        id: message.id,
        seq: message.seq,
        content,
        local_id: message.local_id.flatten(),
        created_at: message.created_at,
        updated_at: message.updated_at,
    }
}

fn resolve_record_encryption(
    record_id: &str,
    data_encryption_key: Option<&str>,
    credentials: &Credentials,
    record_label: &str,
) -> Result<RecordEncryption, ApiError> {
    match data_encryption_key {
        Some(encoded) => {
            let key = unwrap_data_encryption_key(encoded, &credentials.content_key_pair.secret_key)
                .map_err(|_| {
                    ApiError::new(format!(
                        "Failed to decrypt {record_label} key for {record_label} {record_id}"
                    ))
                })?;
            Ok(RecordEncryption {
                key,
                variant: EncryptionVariant::DataKey,
            })
        }
        None => Ok(RecordEncryption {
            key: credentials.secret,
            variant: EncryptionVariant::Legacy,
        }),
    }
}

fn decrypt_required_field(encrypted: &str, encryption: &RecordEncryption) -> Value {
    decrypt_optional_field(encrypted, encryption).unwrap_or(Value::Null)
}

fn decrypt_optional_field(encrypted: &str, encryption: &RecordEncryption) -> Option<Value> {
    let ciphertext = decode_base64(encrypted).ok()?;
    decrypt_json(&encryption.key, encryption.variant, &ciphertext)
}

fn encode_path_component(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

async fn send_json<T: DeserializeOwned>(
    request: reqwest::RequestBuilder,
    context: &str,
) -> Result<T, ApiError> {
    let response = request
        .send()
        .await
        .map_err(|error| ApiError::new(format!("Request failed: {error}")))?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(ApiError::from_http(status, context, &body));
    }

    serde_json::from_str(&body).map_err(|error| ApiError::new(format!("Request failed: {error}")))
}

#[cfg(test)]
mod tests {
    use std::{
        net::TcpListener as StdTcpListener,
        sync::{Arc, Mutex},
    };

    use axum::{
        Json, Router,
        extract::{Path, State},
        http::{HeaderMap, StatusCode, header::AUTHORIZATION},
        routing::{delete, get, post},
    };
    use serde_json::{Value, json};
    use tempfile::TempDir;
    use tokio::{sync::oneshot, task::JoinHandle};

    use crate::{
        config::Config,
        credentials::Credentials,
        encryption::{
            decode_base64, derive_content_key_pair, encode_base64, encrypt_json,
            libsodium_encrypt_for_public_key,
        },
    };

    use vibe_wire::{SessionMessage, SessionMessageContent};

    use super::{
        ApiClient, EncryptionVariant, RawMachine, RawSession, decrypt_machine, decrypt_message,
        decrypt_session,
    };

    struct MockHttpServer {
        server_url: String,
        shutdown: Option<oneshot::Sender<()>>,
        task: Option<JoinHandle<()>>,
    }

    impl MockHttpServer {
        async fn start(router: Router) -> Self {
            let std_listener = StdTcpListener::bind("127.0.0.1:0").unwrap();
            let addr = std_listener.local_addr().unwrap();
            std_listener.set_nonblocking(true).unwrap();
            let listener = tokio::net::TcpListener::from_std(std_listener).unwrap();
            let (shutdown_tx, shutdown_rx) = oneshot::channel();
            let task = tokio::spawn(async move {
                axum::serve(listener, router)
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

        fn api_client(&self) -> (TempDir, ApiClient) {
            let temp_dir = TempDir::new().unwrap();
            let config = Config::from_sources(
                Some(self.server_url.clone()),
                Some(temp_dir.path().as_os_str().to_owned()),
            )
            .unwrap();
            let client = ApiClient::new(config, test_credentials());
            (temp_dir, client)
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

    impl Drop for MockHttpServer {
        fn drop(&mut self) {
            if let Some(sender) = self.shutdown.take() {
                let _ = sender.send(());
            }
            if let Some(task) = self.task.take() {
                task.abort();
            }
        }
    }

    fn test_credentials() -> Credentials {
        let secret = [7u8; 32];
        Credentials {
            token: "token".into(),
            secret,
            content_key_pair: derive_content_key_pair(&secret),
        }
    }

    #[test]
    fn session_encryption_resolves_data_key_records() {
        let credentials = test_credentials();
        let session_key = [11u8; 32];
        let wrapped = encode_base64(&{
            let mut data = vec![0];
            data.extend_from_slice(&libsodium_encrypt_for_public_key(
                &session_key,
                &credentials.content_key_pair.public_key,
            ));
            data
        });
        let metadata = encode_base64(
            &encrypt_json(
                &session_key,
                EncryptionVariant::DataKey,
                &json!({"tag": "session"}),
            )
            .unwrap(),
        );

        let session = decrypt_session(
            RawSession {
                id: "session-1".into(),
                seq: 1,
                created_at: 1,
                updated_at: 2,
                active: true,
                active_at: 2,
                metadata,
                metadata_version: 1,
                agent_state: None,
                agent_state_version: 0,
                data_encryption_key: Some(wrapped),
            },
            &credentials,
        )
        .unwrap();

        assert_eq!(session.encryption.variant, EncryptionVariant::DataKey);
        assert_eq!(session.metadata["tag"], "session");
    }

    #[test]
    fn session_encryption_resolves_legacy_records() {
        let credentials = test_credentials();
        let metadata = encode_base64(
            &encrypt_json(
                &credentials.secret,
                EncryptionVariant::Legacy,
                &json!({"tag": "legacy"}),
            )
            .unwrap(),
        );

        let session = decrypt_session(
            RawSession {
                id: "session-1".into(),
                seq: 1,
                created_at: 1,
                updated_at: 2,
                active: true,
                active_at: 2,
                metadata,
                metadata_version: 1,
                agent_state: None,
                agent_state_version: 0,
                data_encryption_key: None,
            },
            &credentials,
        )
        .unwrap();

        assert_eq!(session.encryption.variant, EncryptionVariant::Legacy);
        assert_eq!(session.metadata["tag"], "legacy");
    }

    #[test]
    fn session_metadata_decrypt_failure_falls_back_to_null() {
        let credentials = test_credentials();

        let session = decrypt_session(
            RawSession {
                id: "session-bad".into(),
                seq: 1,
                created_at: 1,
                updated_at: 2,
                active: true,
                active_at: 2,
                metadata: "not-base64".into(),
                metadata_version: 1,
                agent_state: Some("still-not-base64".into()),
                agent_state_version: 1,
                data_encryption_key: None,
            },
            &credentials,
        )
        .unwrap();

        assert_eq!(session.metadata, Value::Null);
        assert_eq!(session.agent_state, None);
    }

    #[test]
    fn machine_metadata_decrypt_failure_falls_back_to_null() {
        let credentials = test_credentials();

        let machine = decrypt_machine(
            RawMachine {
                id: "machine-bad".into(),
                seq: 1,
                created_at: 1,
                updated_at: 2,
                active: true,
                active_at: 2,
                metadata: "not-base64".into(),
                metadata_version: 1,
                daemon_state: Some("still-not-base64".into()),
                daemon_state_version: 1,
                data_encryption_key: None,
            },
            &credentials,
        )
        .unwrap();

        assert_eq!(machine.metadata, Value::Null);
        assert_eq!(machine.daemon_state, None);
    }

    #[test]
    fn message_decrypt_failure_falls_back_to_null() {
        let message = decrypt_message(
            SessionMessage {
                id: "msg-1".into(),
                seq: 1,
                content: SessionMessageContent::new("not-base64"),
                local_id: Some(Some("local-1".into())),
                created_at: 1,
                updated_at: 2,
            },
            &super::RecordEncryption {
                key: [9u8; 32],
                variant: EncryptionVariant::Legacy,
            },
        );

        assert_eq!(message.content, Value::Null);
        assert_eq!(message.local_id.as_deref(), Some("local-1"));
    }

    #[tokio::test]
    async fn delete_session_sends_authenticated_delete_request() {
        #[derive(Clone)]
        struct DeleteState {
            deleted_session_id: Arc<Mutex<Option<String>>>,
            authorization: Arc<Mutex<Option<String>>>,
        }

        let state = DeleteState {
            deleted_session_id: Arc::new(Mutex::new(None)),
            authorization: Arc::new(Mutex::new(None)),
        };
        let server = MockHttpServer::start(
            Router::new()
                .route(
                    "/v1/sessions/{session_id}",
                    delete(
                        |State(state): State<DeleteState>,
                         Path(session_id): Path<String>,
                         headers: HeaderMap| async move {
                            *state.deleted_session_id.lock().unwrap() = Some(session_id);
                            *state.authorization.lock().unwrap() = headers
                                .get(AUTHORIZATION)
                                .and_then(|value| value.to_str().ok())
                                .map(ToOwned::to_owned);
                            StatusCode::NO_CONTENT
                        },
                    ),
                )
                .with_state(state.clone()),
        )
        .await;
        let (_temp_dir, api) = server.api_client();

        api.delete_session("session-to-delete").await.unwrap();

        assert_eq!(
            state.deleted_session_id.lock().unwrap().as_deref(),
            Some("session-to-delete")
        );
        assert_eq!(
            state.authorization.lock().unwrap().as_deref(),
            Some("Bearer token")
        );

        server.shutdown().await;
    }

    #[tokio::test]
    async fn delete_session_maps_not_found_and_unauthorized_errors() {
        let not_found_server = MockHttpServer::start(Router::new().route(
            "/v1/sessions/{session_id}",
            delete(|| async {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({ "error": "Session not found" })),
                )
            }),
        ))
        .await;
        let (_temp_dir, api) = not_found_server.api_client();
        let error = api.delete_session("missing-session").await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "Not found: deleting session missing-session"
        );
        not_found_server.shutdown().await;

        let unauthorized_server = MockHttpServer::start(Router::new().route(
            "/v1/sessions/{session_id}",
            delete(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "Invalid token" })),
                )
            }),
        ))
        .await;
        let (_temp_dir, api) = unauthorized_server.api_client();
        let error = api.delete_session("expired-session").await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "Authentication expired. Run `vibe-agent auth login` to re-authenticate."
        );
        unauthorized_server.shutdown().await;
    }

    #[tokio::test]
    async fn list_sessions_maps_forbidden_and_server_errors() {
        let forbidden_server = MockHttpServer::start(Router::new().route(
            "/v1/sessions",
            get(|| async { (StatusCode::FORBIDDEN, Json(json!({ "error": "Forbidden" }))) }),
        ))
        .await;
        let (_temp_dir, api) = forbidden_server.api_client();
        let error = api.list_sessions().await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "Forbidden: listing sessions. Check your account permissions."
        );
        forbidden_server.shutdown().await;

        let server_error_server = MockHttpServer::start(Router::new().route(
            "/v1/sessions",
            get(|| async {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "boom" })),
                )
            }),
        ))
        .await;
        let (_temp_dir, api) = server_error_server.api_client();
        let error = api.list_sessions().await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "Server error (500 Internal Server Error): listing sessions"
        );
        server_error_server.shutdown().await;
    }

    #[tokio::test]
    async fn create_session_posts_encrypted_payload_and_decrypts_response() {
        #[derive(Clone)]
        struct CreateState {
            authorization: Arc<Mutex<Option<String>>>,
            captured_metadata: Arc<Mutex<Option<String>>>,
            captured_data_encryption_key: Arc<Mutex<Option<String>>>,
            captured_tag: Arc<Mutex<Option<String>>>,
        }

        let state = CreateState {
            authorization: Arc::new(Mutex::new(None)),
            captured_metadata: Arc::new(Mutex::new(None)),
            captured_data_encryption_key: Arc::new(Mutex::new(None)),
            captured_tag: Arc::new(Mutex::new(None)),
        };
        let server = MockHttpServer::start(
            Router::new()
                .route(
                    "/v1/sessions",
                    post(
                        |State(state): State<CreateState>,
                         headers: HeaderMap,
                         Json(body): Json<Value>| async move {
                            *state.authorization.lock().unwrap() = headers
                                .get(AUTHORIZATION)
                                .and_then(|value| value.to_str().ok())
                                .map(ToOwned::to_owned);
                            *state.captured_tag.lock().unwrap() = body
                                .get("tag")
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned);
                            *state.captured_metadata.lock().unwrap() = body
                                .get("metadata")
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned);
                            *state.captured_data_encryption_key.lock().unwrap() = body
                                .get("dataEncryptionKey")
                                .and_then(Value::as_str)
                                .map(ToOwned::to_owned);

                            Json(json!({
                                "session": {
                                    "id": "session-created",
                                    "seq": 1,
                                    "createdAt": 1,
                                    "updatedAt": 2,
                                    "active": true,
                                    "activeAt": 2,
                                    "metadata": body["metadata"],
                                    "metadataVersion": 1,
                                    "agentState": Value::Null,
                                    "agentStateVersion": 0,
                                    "dataEncryptionKey": body["dataEncryptionKey"],
                                }
                            }))
                        },
                    ),
                )
                .with_state(state.clone()),
        )
        .await;
        let (_temp_dir, api) = server.api_client();

        let created = api
            .create_session("demo-tag", &json!({"tag": "demo-tag", "path": "/tmp/demo"}))
            .await
            .unwrap();

        assert_eq!(created.id, "session-created");
        assert_eq!(created.metadata["tag"], "demo-tag");
        assert_eq!(created.metadata["path"], "/tmp/demo");
        assert_eq!(
            state.authorization.lock().unwrap().as_deref(),
            Some("Bearer token")
        );
        assert_eq!(
            state.captured_tag.lock().unwrap().as_deref(),
            Some("demo-tag")
        );

        let wrapped_key = state
            .captured_data_encryption_key
            .lock()
            .unwrap()
            .clone()
            .expect("wrapped key should be sent");
        let session_key = crate::encryption::unwrap_data_encryption_key(
            &wrapped_key,
            &api.credentials().content_key_pair.secret_key,
        )
        .unwrap();
        let metadata_ciphertext = state
            .captured_metadata
            .lock()
            .unwrap()
            .clone()
            .expect("metadata should be sent");
        let decrypted = crate::encryption::decrypt_json(
            &session_key,
            EncryptionVariant::DataKey,
            &decode_base64(&metadata_ciphertext).unwrap(),
        )
        .unwrap();
        assert_eq!(decrypted["tag"], "demo-tag");
        assert_eq!(decrypted["path"], "/tmp/demo");

        server.shutdown().await;
    }

    #[tokio::test]
    async fn list_active_sessions_uses_v2_route_and_decrypts_records() {
        #[derive(Clone)]
        struct ActiveState {
            authorization: Arc<Mutex<Option<String>>>,
        }

        let state = ActiveState {
            authorization: Arc::new(Mutex::new(None)),
        };
        let credentials = test_credentials();
        let metadata = encode_base64(
            &encrypt_json(
                &credentials.secret,
                EncryptionVariant::Legacy,
                &json!({"tag": "active-session"}),
            )
            .unwrap(),
        );
        let server = MockHttpServer::start(
            Router::new()
                .route(
                    "/v2/sessions/active",
                    get(
                        |State(state): State<ActiveState>, headers: HeaderMap| async move {
                            *state.authorization.lock().unwrap() = headers
                                .get(AUTHORIZATION)
                                .and_then(|value| value.to_str().ok())
                                .map(ToOwned::to_owned);
                            Json(json!({
                                "sessions": [{
                                    "id": "session-active",
                                    "seq": 1,
                                    "createdAt": 1,
                                    "updatedAt": 2,
                                    "active": true,
                                    "activeAt": 2,
                                    "metadata": metadata,
                                    "metadataVersion": 1,
                                    "agentState": Value::Null,
                                    "agentStateVersion": 0,
                                    "dataEncryptionKey": Value::Null,
                                }]
                            }))
                        },
                    ),
                )
                .with_state(state.clone()),
        )
        .await;
        let (_temp_dir, api) = server.api_client();

        let sessions = api.list_active_sessions().await.unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "session-active");
        assert_eq!(sessions[0].metadata["tag"], "active-session");
        assert_eq!(
            state.authorization.lock().unwrap().as_deref(),
            Some("Bearer token")
        );

        server.shutdown().await;
    }

    #[tokio::test]
    async fn machine_routes_use_auth_and_decrypt_records() {
        #[derive(Clone)]
        struct MachineState {
            machine_list_auth: Arc<Mutex<Option<String>>>,
            machine_detail_auth: Arc<Mutex<Option<String>>>,
            requested_machine_id: Arc<Mutex<Option<String>>>,
        }

        let state = MachineState {
            machine_list_auth: Arc::new(Mutex::new(None)),
            machine_detail_auth: Arc::new(Mutex::new(None)),
            requested_machine_id: Arc::new(Mutex::new(None)),
        };
        let credentials = test_credentials();
        let metadata = encode_base64(
            &encrypt_json(
                &credentials.secret,
                EncryptionVariant::Legacy,
                &json!({"host": "builder", "homeDir": "/srv/build"}),
            )
            .unwrap(),
        );
        let daemon_state = encode_base64(
            &encrypt_json(
                &credentials.secret,
                EncryptionVariant::Legacy,
                &json!({"status": "ready"}),
            )
            .unwrap(),
        );
        let server = MockHttpServer::start(
            Router::new()
                .route(
                    "/v1/machines",
                    get({
                        let metadata = metadata.clone();
                        let daemon_state = daemon_state.clone();
                        move |State(state): State<MachineState>, headers: HeaderMap| {
                            let metadata = metadata.clone();
                            let daemon_state = daemon_state.clone();
                            async move {
                                *state.machine_list_auth.lock().unwrap() = headers
                                    .get(AUTHORIZATION)
                                    .and_then(|value| value.to_str().ok())
                                    .map(ToOwned::to_owned);
                                Json(json!([{
                                    "id": "machine-1",
                                    "seq": 1,
                                    "createdAt": 1,
                                    "updatedAt": 2,
                                    "active": true,
                                    "activeAt": 2,
                                    "metadata": metadata,
                                    "metadataVersion": 1,
                                    "daemonState": daemon_state,
                                    "daemonStateVersion": 1,
                                    "dataEncryptionKey": Value::Null,
                                }]))
                            }
                        }
                    }),
                )
                .route(
                    "/v1/machines/{machine_id}",
                    get({
                        let metadata = metadata.clone();
                        let daemon_state = daemon_state.clone();
                        move |State(state): State<MachineState>,
                              Path(machine_id): Path<String>,
                              headers: HeaderMap| {
                            let metadata = metadata.clone();
                            let daemon_state = daemon_state.clone();
                            async move {
                                *state.machine_detail_auth.lock().unwrap() = headers
                                    .get(AUTHORIZATION)
                                    .and_then(|value| value.to_str().ok())
                                    .map(ToOwned::to_owned);
                                *state.requested_machine_id.lock().unwrap() = Some(machine_id);
                                Json(json!({
                                    "machine": {
                                        "id": "machine-1",
                                        "seq": 1,
                                        "createdAt": 1,
                                        "updatedAt": 2,
                                        "active": true,
                                        "activeAt": 2,
                                        "metadata": metadata,
                                        "metadataVersion": 1,
                                        "daemonState": daemon_state,
                                        "daemonStateVersion": 1,
                                        "dataEncryptionKey": Value::Null,
                                    }
                                }))
                            }
                        }
                    }),
                )
                .with_state(state.clone()),
        )
        .await;
        let (_temp_dir, api) = server.api_client();

        let machines = api.list_machines().await.unwrap();
        assert_eq!(machines.len(), 1);
        assert_eq!(machines[0].metadata["host"], "builder");
        assert_eq!(
            machines[0].daemon_state.as_ref().unwrap()["status"],
            "ready"
        );

        let machine = api.get_machine("machine-1").await.unwrap();
        assert_eq!(machine.metadata["homeDir"], "/srv/build");
        assert_eq!(
            state.machine_list_auth.lock().unwrap().as_deref(),
            Some("Bearer token")
        );
        assert_eq!(
            state.machine_detail_auth.lock().unwrap().as_deref(),
            Some("Bearer token")
        );
        assert_eq!(
            state.requested_machine_id.lock().unwrap().as_deref(),
            Some("machine-1")
        );

        server.shutdown().await;
    }

    #[tokio::test]
    async fn get_session_messages_uses_auth_and_decrypts_history() {
        #[derive(Clone)]
        struct HistoryState {
            authorization: Arc<Mutex<Option<String>>>,
            requested_session_id: Arc<Mutex<Option<String>>>,
        }

        let state = HistoryState {
            authorization: Arc::new(Mutex::new(None)),
            requested_session_id: Arc::new(Mutex::new(None)),
        };
        let credentials = test_credentials();
        let encryption = super::RecordEncryption {
            key: credentials.secret,
            variant: EncryptionVariant::Legacy,
        };
        let message_ciphertext = encode_base64(
            &encrypt_json(
                &credentials.secret,
                EncryptionVariant::Legacy,
                &json!({"role": "user", "content": {"type": "text", "text": "hello history"}}),
            )
            .unwrap(),
        );
        let server = MockHttpServer::start(
            Router::new()
                .route(
                    "/v1/sessions/{session_id}/messages",
                    get(
                        |State(state): State<HistoryState>,
                         Path(session_id): Path<String>,
                         headers: HeaderMap| async move {
                            *state.authorization.lock().unwrap() = headers
                                .get(AUTHORIZATION)
                                .and_then(|value| value.to_str().ok())
                                .map(ToOwned::to_owned);
                            *state.requested_session_id.lock().unwrap() = Some(session_id);
                            Json(json!({
                                "messages": [{
                                    "id": "msg-1",
                                    "seq": 1,
                                    "content": { "t": "encrypted", "c": message_ciphertext },
                                    "localId": "local-1",
                                    "createdAt": 1,
                                    "updatedAt": 2,
                                }]
                            }))
                        },
                    ),
                )
                .with_state(state.clone()),
        )
        .await;
        let (_temp_dir, api) = server.api_client();

        let messages = api
            .get_session_messages("session-1", &encryption)
            .await
            .unwrap();

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content["role"], "user");
        assert_eq!(messages[0].content["content"]["text"], "hello history");
        assert_eq!(messages[0].local_id.as_deref(), Some("local-1"));
        assert_eq!(
            state.authorization.lock().unwrap().as_deref(),
            Some("Bearer token")
        );
        assert_eq!(
            state.requested_session_id.lock().unwrap().as_deref(),
            Some("session-1")
        );

        server.shutdown().await;
    }
}
