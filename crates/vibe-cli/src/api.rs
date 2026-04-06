use std::{sync::Arc, time::Duration};

use futures_util::FutureExt;
use rust_socketio::{
    Event, Payload, TransportType,
    asynchronous::{Client as SocketClient, ClientBuilder},
};
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;
use tokio::sync::{Mutex, Notify, oneshot};
use vibe_agent::{
    api::{
        DecryptedMachine, DecryptedMessage, DecryptedSession, RawMachine, RawSession,
        RecordEncryption,
    },
    config::Config as AgentConfig,
    credentials::read_credentials as read_agent_credentials,
    encryption::{
        EncryptionVariant, decode_base64, decrypt_json, encode_base64, encrypt_json, random_bytes,
        unwrap_data_encryption_key, wrap_data_encryption_key,
    },
    session::{SessionClient, SessionClientOptions},
};
use vibe_wire::SessionMessage;

use crate::{
    config::Config,
    credentials::{CredentialEncryption, Credentials},
    persistence::list_local_sessions,
    utils::machine_metadata::semantically_equal as machine_metadata_semantically_equal,
};

const SOCKET_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
const SOCKET_READY_TIMEOUT: Duration = Duration::from_secs(20);
const SOCKET_ACK_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Error)]
pub enum CliApiError {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Socket(#[from] rust_socketio::Error),
    #[error(transparent)]
    Encryption(#[from] vibe_agent::encryption::EncryptionError),
    #[error(transparent)]
    Session(#[from] vibe_agent::session::SessionError),
}

#[derive(Debug, Clone)]
pub struct CliApiClient {
    http: reqwest::Client,
    config: Config,
    credentials: Credentials,
}

#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub content: String,
    pub local_id: String,
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

#[derive(Debug, Deserialize)]
struct CreateMachineResponse {
    machine: CreateMachineRecord,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateMachineRecord {
    id: String,
    created_at: u64,
    updated_at: u64,
    active: bool,
    active_at: u64,
    metadata: String,
    metadata_version: u64,
    daemon_state: Option<String>,
    daemon_state_version: u64,
    data_encryption_key: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateStateAck {
    result: String,
    version: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct MachineUpdateStateAck {
    result: String,
    version: Option<u64>,
    #[serde(rename = "daemonState")]
    daemon_state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MachineUpdateMetadataAck {
    result: String,
    version: Option<u64>,
    metadata: Option<String>,
}

impl CliApiClient {
    pub fn new(config: Config, credentials: Credentials) -> Self {
        Self {
            http: reqwest::Client::new(),
            config,
            credentials,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn list_sessions(&self) -> Result<Vec<DecryptedSession>, CliApiError> {
        let response: SessionListResponse =
            self.get_json("/v1/sessions", "listing sessions").await?;
        response
            .sessions
            .into_iter()
            .map(|session| self.decrypt_session(session))
            .collect()
    }

    pub async fn list_active_sessions(&self) -> Result<Vec<DecryptedSession>, CliApiError> {
        let response: SessionListResponse = self
            .get_json("/v2/sessions/active", "listing active sessions")
            .await?;
        response
            .sessions
            .into_iter()
            .map(|session| self.decrypt_session(session))
            .collect()
    }

    pub async fn create_session(
        &self,
        tag: &str,
        metadata: &Value,
    ) -> Result<DecryptedSession, CliApiError> {
        let (encryption_key, variant, data_encryption_key) = match &self.credentials.encryption {
            CredentialEncryption::Legacy { secret, .. } => {
                (*secret, EncryptionVariant::Legacy, None)
            }
            CredentialEncryption::DataKey { public_key, .. } => {
                let key = random_bytes::<32>();
                (
                    key,
                    EncryptionVariant::DataKey,
                    Some(wrap_data_encryption_key(&key, public_key)),
                )
            }
        };
        let encrypted_metadata = encrypt_json(&encryption_key, variant, metadata)?;

        let response: CreateSessionResponse = self
            .post_json(
                "/v1/sessions",
                &serde_json::json!({
                    "tag": tag,
                    "metadata": encode_base64(&encrypted_metadata),
                    "dataEncryptionKey": data_encryption_key,
                }),
                "creating session",
            )
            .await?;

        self.decrypt_session_with_encryption(response.session, encryption_key, variant)
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), CliApiError> {
        let response = self
            .http
            .delete(format!(
                "{}/v1/sessions/{}",
                self.config.server_url, session_id
            ))
            .bearer_auth(self.token())
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliApiError::Message(format!(
                "deleting session failed: {status} {body}"
            )));
        }
        Ok(())
    }

    pub async fn get_session_messages(
        &self,
        session: &DecryptedSession,
    ) -> Result<Vec<DecryptedMessage>, CliApiError> {
        let response: SessionHistoryResponse = self
            .get_json(
                &format!("/v1/sessions/{}/messages", session.id),
                "fetching session messages",
            )
            .await?;
        Ok(response
            .messages
            .into_iter()
            .map(|message| decrypt_message(message, &session.encryption))
            .collect())
    }

    pub async fn list_machines(&self) -> Result<Vec<DecryptedMachine>, CliApiError> {
        let response: Vec<RawMachine> = self.get_json("/v1/machines", "listing machines").await?;
        response
            .into_iter()
            .map(|machine| self.decrypt_machine(machine))
            .collect()
    }

    pub async fn get_machine(&self, machine_id: &str) -> Result<DecryptedMachine, CliApiError> {
        let response: MachineDetailResponse = self
            .get_json(
                &format!("/v1/machines/{machine_id}"),
                "fetching machine details",
            )
            .await?;
        self.decrypt_machine(response.machine)
    }

    pub async fn create_or_load_machine(
        &self,
        machine_id: &str,
        metadata: &Value,
        daemon_state: Option<&Value>,
    ) -> Result<DecryptedMachine, CliApiError> {
        let encryption = self.resolve_machine_encryption();
        let encrypted_metadata = encode_base64(&encrypt_json(
            &encryption.key,
            encryption.variant,
            metadata,
        )?);
        let encrypted_daemon_state = daemon_state
            .map(|value| {
                encrypt_json(&encryption.key, encryption.variant, value)
                    .map(|bytes| encode_base64(&bytes))
            })
            .transpose()?;
        let data_encryption_key = match &self.credentials.encryption {
            CredentialEncryption::DataKey {
                public_key,
                machine_key,
            } => {
                if *machine_key == encryption.key {
                    Some(wrap_data_encryption_key(machine_key, public_key))
                } else {
                    None
                }
            }
            CredentialEncryption::Legacy { .. } => None,
        };

        let response: CreateMachineResponse = self
            .post_json(
                "/v1/machines",
                &serde_json::json!({
                    "id": machine_id,
                    "metadata": encrypted_metadata,
                    "daemonState": encrypted_daemon_state,
                    "dataEncryptionKey": data_encryption_key,
                }),
                "creating machine",
            )
            .await?;
        self.decrypt_created_machine(response.machine)
    }

    pub async fn register_vendor_token(
        &self,
        vendor: &str,
        token: &str,
    ) -> Result<(), CliApiError> {
        let response = self
            .http
            .post(format!(
                "{}/v1/connect/{}/register",
                self.config.server_url, vendor
            ))
            .bearer_auth(self.token())
            .json(&serde_json::json!({ "token": token }))
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliApiError::Message(format!(
                "registering vendor token failed: {status} {body}"
            )));
        }
        Ok(())
    }

    pub async fn list_vendor_tokens(&self) -> Result<Vec<String>, CliApiError> {
        let response: Value = self
            .get_json("/v1/connect/tokens", "listing vendor tokens")
            .await?;
        Ok(response
            .get("tokens")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|item| item.get("vendor")?.as_str().map(ToOwned::to_owned))
            .collect())
    }

    pub async fn delete_vendor_token(&self, vendor: &str) -> Result<(), CliApiError> {
        let response = self
            .http
            .delete(format!("{}/v1/connect/{}", self.config.server_url, vendor))
            .bearer_auth(self.token())
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliApiError::Message(format!(
                "deleting vendor token failed: {status} {body}"
            )));
        }
        Ok(())
    }

    pub async fn create_session_client(
        &self,
        session: &DecryptedSession,
    ) -> Result<SessionClient, CliApiError> {
        Ok(SessionClient::connect(SessionClientOptions {
            session_id: session.id.clone(),
            encryption_key: session.encryption.key,
            encryption_variant: session.encryption.variant,
            token: self.credentials.token.clone(),
            server_url: self.config.socket_url(),
            initial_metadata: Some(session.metadata.clone()),
            initial_metadata_version: session.metadata_version,
            initial_agent_state: session.agent_state.clone(),
            initial_agent_state_version: session.agent_state_version,
        })
        .await?)
    }

    pub async fn create_machine_sync_client(
        &self,
        machine: DecryptedMachine,
    ) -> Result<MachineSyncClient, CliApiError> {
        MachineSyncClient::connect(self.config.clone(), self.credentials.token.clone(), machine)
            .await
    }

    pub async fn post_messages_v3(
        &self,
        session_id: &str,
        messages: Vec<QueuedMessage>,
    ) -> Result<(), CliApiError> {
        let response = self
            .http
            .post(format!(
                "{}/v3/sessions/{}/messages",
                self.config.server_url, session_id
            ))
            .bearer_auth(self.token())
            .json(&serde_json::json!({
                "messages": messages
                    .into_iter()
                    .map(|message| serde_json::json!({
                        "content": message.content,
                        "localId": message.local_id,
                    }))
                    .collect::<Vec<_>>(),
            }))
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliApiError::Message(format!(
                "posting session messages failed: {status} {body}"
            )));
        }
        Ok(())
    }

    pub async fn update_agent_state(
        &self,
        session: &DecryptedSession,
        expected_version: u64,
        agent_state: Option<Option<String>>,
    ) -> Result<u64, CliApiError> {
        let socket =
            connect_scoped_socket(&self.config, &self.credentials.token, &session.id).await?;
        let mut payload = serde_json::Map::new();
        payload.insert("sid".into(), Value::String(session.id.clone()));
        payload.insert(
            "expectedVersion".into(),
            Value::Number(expected_version.into()),
        );
        match agent_state {
            Some(Some(agent_state)) => {
                payload.insert("agentState".into(), Value::String(agent_state));
            }
            Some(None) => {
                payload.insert("agentState".into(), Value::Null);
            }
            None => {}
        }
        let ack = emit_with_ack(
            &socket,
            "update-state",
            Value::Object(payload),
            SOCKET_ACK_TIMEOUT,
        )
        .await?;
        let _ = socket.disconnect().await;
        let ack: UpdateStateAck = serde_json::from_value(unwrap_singleton_array(ack))?;
        match ack.result.as_str() {
            "success" => ack
                .version
                .ok_or_else(|| CliApiError::Message("missing version in success ack".into())),
            "version-mismatch" => Err(CliApiError::Message(format!(
                "agent state version mismatch for session {}",
                session.id
            ))),
            _ => Err(CliApiError::Message(format!(
                "failed to update agent state for session {}",
                session.id
            ))),
        }
    }

    fn token(&self) -> &str {
        &self.credentials.token
    }

    async fn get_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        context: &str,
    ) -> Result<T, CliApiError> {
        let response = self
            .http
            .get(format!("{}{}", self.config.server_url, path))
            .bearer_auth(self.token())
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(CliApiError::Message(format!(
                "{context} failed: {status} {body}"
            )));
        }
        Ok(serde_json::from_str(&body)?)
    }

    async fn post_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: &Value,
        context: &str,
    ) -> Result<T, CliApiError> {
        let response = self
            .http
            .post(format!("{}{}", self.config.server_url, path))
            .bearer_auth(self.token())
            .json(body)
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(CliApiError::Message(format!(
                "{context} failed: {status} {body}"
            )));
        }
        Ok(serde_json::from_str(&body)?)
    }

    fn decrypt_session(&self, raw: RawSession) -> Result<DecryptedSession, CliApiError> {
        let encryption = self.resolve_session_encryption(&raw);
        let metadata =
            decrypt_optional_field(&raw.metadata, encryption.as_ref()).unwrap_or(Value::Null);
        let agent_state = raw
            .agent_state
            .as_deref()
            .and_then(|value| decrypt_optional_field(value, encryption.as_ref()));
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
            encryption: encryption.unwrap_or(RecordEncryption {
                key: [0u8; 32],
                variant: EncryptionVariant::Legacy,
            }),
        })
    }

    fn decrypt_session_with_encryption(
        &self,
        raw: RawSession,
        key: [u8; 32],
        variant: EncryptionVariant,
    ) -> Result<DecryptedSession, CliApiError> {
        let encryption = RecordEncryption { key, variant };
        let metadata =
            decrypt_optional_field(&raw.metadata, Some(&encryption)).unwrap_or(Value::Null);
        let agent_state = raw
            .agent_state
            .as_deref()
            .and_then(|value| decrypt_optional_field(value, Some(&encryption)));
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

    fn decrypt_machine(&self, raw: RawMachine) -> Result<DecryptedMachine, CliApiError> {
        let encryption = self.resolve_machine_encryption();
        let metadata =
            decrypt_optional_field(&raw.metadata, Some(&encryption)).unwrap_or(Value::Null);
        let daemon_state = raw
            .daemon_state
            .as_deref()
            .and_then(|value| decrypt_optional_field(value, Some(&encryption)));
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

    fn decrypt_created_machine(
        &self,
        raw: CreateMachineRecord,
    ) -> Result<DecryptedMachine, CliApiError> {
        let encryption = self.resolve_machine_encryption();
        let metadata =
            decrypt_optional_field(&raw.metadata, Some(&encryption)).unwrap_or(Value::Null);
        let daemon_state = raw
            .daemon_state
            .as_deref()
            .and_then(|value| decrypt_optional_field(value, Some(&encryption)));
        Ok(DecryptedMachine {
            id: raw.id,
            seq: 0,
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

    fn resolve_session_encryption(&self, raw: &RawSession) -> Option<RecordEncryption> {
        match &self.credentials.encryption {
            CredentialEncryption::Legacy { secret, .. } => Some(RecordEncryption {
                key: *secret,
                variant: EncryptionVariant::Legacy,
            }),
            CredentialEncryption::DataKey { .. } => {
                if let Some(local_state) = list_local_sessions(&self.config)
                    .ok()
                    .into_iter()
                    .flatten()
                    .find(|session| session.server_session_id == raw.id)
                    && let Some(key) = local_state.encryption_key
                    && let Ok(key) = decode_base64(&key)
                    && let Ok(key) = <[u8; 32]>::try_from(key.as_slice())
                {
                    return Some(RecordEncryption {
                        key,
                        variant: if local_state.encryption_variant == "dataKey" {
                            EncryptionVariant::DataKey
                        } else {
                            EncryptionVariant::Legacy
                        },
                    });
                }

                let agent_config = AgentConfig {
                    server_url: self.config.server_url.clone(),
                    home_dir: self.config.home_dir.clone(),
                    credential_path: self.config.home_dir.join("agent.key"),
                };
                let credentials = read_agent_credentials(&agent_config)?;
                let encoded = raw.data_encryption_key.as_deref()?;
                let key =
                    unwrap_data_encryption_key(encoded, &credentials.content_key_pair.secret_key)
                        .ok()?;
                Some(RecordEncryption {
                    key,
                    variant: EncryptionVariant::DataKey,
                })
            }
        }
    }

    fn resolve_machine_encryption(&self) -> RecordEncryption {
        let key = self.credentials.machine_key();
        let variant = match self.credentials.encryption {
            CredentialEncryption::Legacy { .. } => EncryptionVariant::Legacy,
            CredentialEncryption::DataKey { .. } => EncryptionVariant::DataKey,
        };
        RecordEncryption { key, variant }
    }
}

pub struct MachineSyncClient {
    machine: Arc<Mutex<DecryptedMachine>>,
    socket: SocketClient,
}

impl MachineSyncClient {
    async fn connect(
        config: Config,
        token: String,
        machine: DecryptedMachine,
    ) -> Result<Self, CliApiError> {
        let machine_id = machine.id.clone();
        let state = Arc::new(Mutex::new((false, None::<String>)));
        let ready = Arc::new(Notify::new());

        let connect_state = state.clone();
        let connect_ready = ready.clone();
        let error_state = state.clone();
        let error_ready = ready.clone();

        let builder = ClientBuilder::new(config.socket_url())
            .transport_type(TransportType::Websocket)
            .reconnect(false)
            .auth(serde_json::json!({
                "token": token,
                "clientType": "machine-scoped",
                "machineId": machine_id,
            }))
            .on(Event::Connect, move |_, _| {
                let state = connect_state.clone();
                let ready = connect_ready.clone();
                async move {
                    let mut guard = state.lock().await;
                    guard.0 = true;
                    guard.1 = None;
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
                    guard.1 = Some(message);
                    drop(guard);
                    ready.notify_waiters();
                }
                .boxed()
            });

        let socket = tokio::time::timeout(SOCKET_CONNECT_TIMEOUT, builder.connect())
            .await
            .map_err(|_| CliApiError::Message("timeout waiting for machine socket".into()))??;

        tokio::time::timeout(SOCKET_READY_TIMEOUT, async {
            loop {
                let notified = ready.notified();
                {
                    let guard = state.lock().await;
                    if guard.0 {
                        return Ok::<(), CliApiError>(());
                    }
                    if let Some(error) = guard.1.clone() {
                        return Err(CliApiError::Message(error));
                    }
                }
                notified.await;
            }
        })
        .await
        .map_err(|_| {
            CliApiError::Message("timeout waiting for machine socket readiness".into())
        })??;

        Ok(Self {
            machine: Arc::new(Mutex::new(machine)),
            socket,
        })
    }

    pub async fn machine_alive(&self) -> Result<(), CliApiError> {
        let machine_id = self.machine.lock().await.id.clone();
        self.socket
            .emit(
                "machine-alive",
                serde_json::json!({
                    "machineId": machine_id,
                    "time": crate::persistence::now_ms(),
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn sync_machine_metadata(
        &self,
        metadata: &Value,
    ) -> Result<Option<u64>, CliApiError> {
        let current_metadata = self.machine.lock().await.metadata.clone();
        if machine_metadata_semantically_equal(&current_metadata, metadata) {
            return Ok(None);
        }

        self.update_machine_metadata(metadata).await.map(Some)
    }

    pub async fn sync_machine_metadata_with_retry(
        &self,
        metadata: &Value,
        attempts: usize,
        retry_delay: Duration,
    ) -> Result<bool, CliApiError> {
        let attempts = attempts.max(1);
        let mut last_error = None;
        for attempt in 0..attempts {
            match self.sync_machine_metadata(metadata).await {
                Ok(updated) => return Ok(updated.is_some()),
                Err(error) => {
                    last_error = Some(error);
                    if attempt + 1 < attempts {
                        tokio::time::sleep(retry_delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            CliApiError::Message("machine metadata sync failed without an error".into())
        }))
    }

    pub async fn update_machine_metadata(&self, metadata: &Value) -> Result<u64, CliApiError> {
        let machine = self.machine.lock().await.clone();
        let encrypted = encode_base64(&encrypt_json(
            &machine.encryption.key,
            machine.encryption.variant,
            metadata,
        )?);
        let ack = emit_with_ack(
            &self.socket,
            "machine-update-metadata",
            serde_json::json!({
                "machineId": machine.id,
                "metadata": encrypted,
                "expectedVersion": machine.metadata_version,
            }),
            SOCKET_ACK_TIMEOUT,
        )
        .await?;
        let ack: MachineUpdateMetadataAck = serde_json::from_value(unwrap_singleton_array(ack))?;
        match ack.result.as_str() {
            "success" => {
                let version = ack.version.ok_or_else(|| {
                    CliApiError::Message("missing machine metadata version in success ack".into())
                })?;
                let metadata = ack
                    .metadata
                    .as_deref()
                    .and_then(|value| decrypt_optional_field(value, Some(&machine.encryption)))
                    .ok_or_else(|| {
                        CliApiError::Message("missing machine metadata in success ack".into())
                    })?;
                let mut guard = self.machine.lock().await;
                guard.metadata = metadata;
                guard.metadata_version = version;
                Ok(version)
            }
            "version-mismatch" => {
                if let Some(version) = ack.version
                    && version > machine.metadata_version
                    && let Some(metadata) = ack
                        .metadata
                        .as_deref()
                        .and_then(|value| decrypt_optional_field(value, Some(&machine.encryption)))
                {
                    let mut guard = self.machine.lock().await;
                    guard.metadata = metadata;
                    guard.metadata_version = version;
                }
                Err(CliApiError::Message(
                    "machine metadata version mismatch".into(),
                ))
            }
            _ => Err(CliApiError::Message(
                "failed to update machine metadata".into(),
            )),
        }
    }

    pub async fn update_daemon_state(&self, daemon_state: &Value) -> Result<u64, CliApiError> {
        let machine = self.machine.lock().await.clone();
        let encrypted = encode_base64(&encrypt_json(
            &machine.encryption.key,
            machine.encryption.variant,
            daemon_state,
        )?);
        let ack = emit_with_ack(
            &self.socket,
            "machine-update-state",
            serde_json::json!({
                "machineId": machine.id,
                "daemonState": encrypted,
                "expectedVersion": machine.daemon_state_version,
            }),
            SOCKET_ACK_TIMEOUT,
        )
        .await?;
        let ack: MachineUpdateStateAck = serde_json::from_value(unwrap_singleton_array(ack))?;
        match ack.result.as_str() {
            "success" => {
                let version = ack.version.ok_or_else(|| {
                    CliApiError::Message("missing machine version in success ack".into())
                })?;
                let mut guard = self.machine.lock().await;
                guard.daemon_state = ack
                    .daemon_state
                    .as_deref()
                    .and_then(|value| decrypt_optional_field(value, Some(&guard.encryption)));
                guard.daemon_state_version = version;
                Ok(version)
            }
            "version-mismatch" => Err(CliApiError::Message(
                "machine daemon state version mismatch".into(),
            )),
            _ => Err(CliApiError::Message(
                "failed to update machine daemon state".into(),
            )),
        }
    }

    pub async fn close(&self) {
        let _ = self.socket.disconnect().await;
    }
}

fn decrypt_optional_field(encrypted: &str, encryption: Option<&RecordEncryption>) -> Option<Value> {
    let encryption = encryption?;
    let ciphertext = decode_base64(encrypted).ok()?;
    decrypt_json(&encryption.key, encryption.variant, &ciphertext)
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

async fn connect_scoped_socket(
    config: &Config,
    token: &str,
    session_id: &str,
) -> Result<SocketClient, CliApiError> {
    let state = Arc::new(Mutex::new((false, None::<String>)));
    let ready = Arc::new(Notify::new());

    let connect_state = state.clone();
    let connect_ready = ready.clone();
    let error_state = state.clone();
    let error_ready = ready.clone();

    let builder = ClientBuilder::new(config.socket_url())
        .transport_type(TransportType::Websocket)
        .reconnect(false)
        .auth(serde_json::json!({
            "token": token,
            "clientType": "session-scoped",
            "sessionId": session_id,
        }))
        .on(Event::Connect, move |_, _| {
            let state = connect_state.clone();
            let ready = connect_ready.clone();
            async move {
                let mut guard = state.lock().await;
                guard.0 = true;
                guard.1 = None;
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
                guard.1 = Some(message);
                drop(guard);
                ready.notify_waiters();
            }
            .boxed()
        });

    let socket = tokio::time::timeout(SOCKET_CONNECT_TIMEOUT, builder.connect())
        .await
        .map_err(|_| CliApiError::Message("timeout waiting for socket connection".into()))??;

    tokio::time::timeout(SOCKET_READY_TIMEOUT, async {
        loop {
            let notified = ready.notified();
            {
                let guard = state.lock().await;
                if guard.0 {
                    return Ok::<(), CliApiError>(());
                }
                if let Some(error) = guard.1.clone() {
                    return Err(CliApiError::Message(error));
                }
            }
            notified.await;
        }
    })
    .await
    .map_err(|_| CliApiError::Message("timeout waiting for socket readiness".into()))??;

    Ok(socket)
}

async fn emit_with_ack(
    socket: &SocketClient,
    event: &str,
    payload: Value,
    timeout: Duration,
) -> Result<Value, CliApiError> {
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
        .map_err(|_| CliApiError::Message("timeout waiting for socket ack".into()))?
        .map_err(|_| CliApiError::Message("socket ack channel closed".into()))
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

fn unwrap_singleton_array(value: Value) -> Value {
    match value {
        Value::Array(mut values) if values.len() == 1 => values.pop().unwrap_or(Value::Null),
        other => other,
    }
}

fn payload_to_string(payload: &Payload) -> String {
    first_payload_value(payload)
        .and_then(|value| {
            value
                .get("message")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or_else(|| value.as_str().map(ToOwned::to_owned))
        })
        .unwrap_or_else(|| "Socket error".into())
}
