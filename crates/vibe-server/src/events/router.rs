use std::{collections::HashMap, sync::Arc};

use parking_lot::RwLock;
use socketioxide::extract::SocketRef;
use vibe_wire::{
    CoreUpdateBody, CoreUpdateContainer, SessionMessage, UpdateMachineBody, UpdateNewMessageBody,
    UpdateSessionBody, VersionedEncryptedValue, VersionedMachineEncryptedValue,
    VersionedNullableEncryptedValue,
};

use crate::{
    auth::{SocketClientType, SocketConnectionAuth},
    events::socket_updates::{
        DurableUpdateBody, DurableUpdateContainer, EphemeralUpdate, KvBatchChange,
        LateDurableUpdate, LateVersionedValue,
    },
    storage::db::{
        ArtifactRecord, Database, FeedPostRecord, MachineRecord, SessionMessageRecord,
        SessionRecord, now_ms,
    },
};

#[derive(Debug, Clone)]
pub struct ClientConnection {
    pub socket_id: String,
    pub socket: SocketRef,
    pub user_id: String,
    pub auth: SocketConnectionAuth,
}

#[derive(Debug, Clone)]
pub enum RecipientFilter {
    AllInterestedInSession { session_id: String },
    UserScopedOnly,
    MachineScopedOnly { machine_id: String },
    AllUserAuthenticatedConnections,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPublishError {
    UpdateSequenceUnavailable,
}

#[derive(Debug, Clone, Default)]
pub struct EventRouter {
    inner: Arc<RwLock<HashMap<String, HashMap<String, ClientConnection>>>>,
}

impl EventRouter {
    pub fn add_connection(&self, connection: ClientConnection) {
        self.inner
            .write()
            .entry(connection.user_id.clone())
            .or_default()
            .insert(connection.socket_id.clone(), connection);
    }

    pub fn remove_connection(&self, user_id: &str, socket_id: &str) {
        let mut guard = self.inner.write();
        if let Some(connections) = guard.get_mut(user_id) {
            connections.remove(socket_id);
            if connections.is_empty() {
                guard.remove(user_id);
            }
        }
    }

    pub fn emit_update(
        &self,
        user_id: &str,
        payload: DurableUpdateContainer,
        filter: RecipientFilter,
        skip_socket_id: Option<&str>,
    ) {
        self.emit(user_id, "update", &payload, filter, skip_socket_id);
    }

    pub fn emit_ephemeral(
        &self,
        user_id: &str,
        payload: EphemeralUpdate,
        filter: RecipientFilter,
        skip_socket_id: Option<&str>,
    ) {
        self.emit(user_id, "ephemeral", &payload, filter, skip_socket_id);
    }

    pub fn publish_new_session(
        &self,
        db: &Database,
        user_id: &str,
        session: &SessionRecord,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_new_session_update(session, seq),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    pub fn publish_delete_session(
        &self,
        db: &Database,
        user_id: &str,
        session_id: &str,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_delete_session_update(session_id, seq),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    pub fn publish_new_message(
        &self,
        db: &Database,
        user_id: &str,
        session_id: &str,
        message: &SessionMessageRecord,
        skip_socket_id: Option<&str>,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_new_message_update(session_id, message, seq),
            RecipientFilter::AllInterestedInSession {
                session_id: session_id.to_string(),
            },
            skip_socket_id,
        );
        Ok(())
    }

    pub fn publish_session_metadata_update(
        &self,
        db: &Database,
        user_id: &str,
        session_id: &str,
        metadata: VersionedEncryptedValue,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_update_session_update(session_id, seq, Some(metadata), None),
            RecipientFilter::AllInterestedInSession {
                session_id: session_id.to_string(),
            },
            None,
        );
        Ok(())
    }

    pub fn publish_session_agent_state_update(
        &self,
        db: &Database,
        user_id: &str,
        session_id: &str,
        agent_state: VersionedNullableEncryptedValue,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_update_session_update(session_id, seq, None, Some(agent_state)),
            RecipientFilter::AllInterestedInSession {
                session_id: session_id.to_string(),
            },
            None,
        );
        Ok(())
    }

    pub fn publish_new_machine(
        &self,
        db: &Database,
        user_id: &str,
        machine: &MachineRecord,
    ) -> Result<(), EventPublishError> {
        let seq_new_machine = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_new_machine_update(machine, seq_new_machine),
            RecipientFilter::UserScopedOnly,
            None,
        );

        let seq_backfill = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_update_machine_update(
                &machine.id,
                seq_backfill,
                Some(VersionedMachineEncryptedValue {
                    version: machine.metadata_version,
                    value: machine.metadata.clone(),
                }),
                None,
                None,
                None,
            ),
            RecipientFilter::MachineScopedOnly {
                machine_id: machine.id.clone(),
            },
            None,
        );
        Ok(())
    }

    pub fn publish_machine_metadata_update(
        &self,
        db: &Database,
        user_id: &str,
        machine_id: &str,
        metadata: VersionedMachineEncryptedValue,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_update_machine_update(machine_id, seq, Some(metadata), None, None, None),
            RecipientFilter::MachineScopedOnly {
                machine_id: machine_id.to_string(),
            },
            None,
        );
        Ok(())
    }

    pub fn publish_machine_daemon_state_update(
        &self,
        db: &Database,
        user_id: &str,
        machine_id: &str,
        daemon_state: VersionedMachineEncryptedValue,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_update_machine_update(machine_id, seq, None, Some(daemon_state), None, None),
            RecipientFilter::MachineScopedOnly {
                machine_id: machine_id.to_string(),
            },
            None,
        );
        Ok(())
    }

    pub fn publish_account_settings_update(
        &self,
        db: &Database,
        user_id: &str,
        version: u64,
        settings: Option<String>,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_update_account_settings(user_id, seq, settings, version),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    pub fn publish_account_profile_update(
        &self,
        db: &Database,
        user_id: &str,
        github: Option<Option<serde_json::Value>>,
        username: Option<Option<String>>,
        first_name: Option<Option<String>>,
        last_name: Option<Option<String>>,
        avatar: Option<Option<serde_json::Value>>,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_update_account_profile(
                user_id, seq, None, github, username, first_name, last_name, avatar,
            ),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    pub fn publish_new_artifact(
        &self,
        db: &Database,
        user_id: &str,
        artifact: &ArtifactRecord,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_new_artifact_update(artifact, seq),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    pub fn publish_update_artifact(
        &self,
        db: &Database,
        user_id: &str,
        artifact_id: &str,
        header: Option<LateVersionedValue>,
        body: Option<LateVersionedValue>,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_update_artifact_update(artifact_id, seq, header, body),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    pub fn publish_delete_artifact(
        &self,
        db: &Database,
        user_id: &str,
        artifact_id: &str,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_delete_artifact_update(artifact_id, seq),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    pub fn publish_relationship_update(
        &self,
        db: &Database,
        user_id: &str,
        target_user_id: &str,
        status: &str,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_relationship_updated(target_user_id, status, seq),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    pub fn publish_new_feed_post(
        &self,
        db: &Database,
        user_id: &str,
        post: &FeedPostRecord,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_new_feed_post(post, seq),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    pub fn publish_kv_batch_update(
        &self,
        db: &Database,
        user_id: &str,
        changes: Vec<KvBatchChange>,
    ) -> Result<(), EventPublishError> {
        let seq = next_user_seq(db, user_id)?;
        self.emit_update(
            user_id,
            build_kv_batch_update(seq, changes),
            RecipientFilter::UserScopedOnly,
            None,
        );
        Ok(())
    }

    fn emit<T: serde::Serialize>(
        &self,
        user_id: &str,
        event_name: &str,
        payload: &T,
        filter: RecipientFilter,
        skip_socket_id: Option<&str>,
    ) {
        let guard = self.inner.read();
        let Some(connections) = guard.get(user_id) else {
            return;
        };

        for connection in connections.values() {
            if skip_socket_id.is_some_and(|socket_id| socket_id == connection.socket_id) {
                continue;
            }
            if !should_send_to_connection(connection, &filter) {
                continue;
            }
            let _ = connection.socket.emit(event_name, payload);
        }
    }
}

fn next_user_seq(db: &Database, user_id: &str) -> Result<u64, EventPublishError> {
    db.allocate_account_seq(user_id)
        .ok_or(EventPublishError::UpdateSequenceUnavailable)
}

fn should_send_to_connection(connection: &ClientConnection, filter: &RecipientFilter) -> bool {
    should_send_to_auth(&connection.auth, filter)
}

fn should_send_to_auth(auth: &SocketConnectionAuth, filter: &RecipientFilter) -> bool {
    match filter {
        RecipientFilter::AllInterestedInSession { session_id } => match auth.client_type {
            SocketClientType::SessionScoped => auth.session_id.as_deref() == Some(session_id),
            SocketClientType::MachineScoped => false,
            SocketClientType::UserScoped => true,
        },
        RecipientFilter::UserScopedOnly => matches!(auth.client_type, SocketClientType::UserScoped),
        RecipientFilter::MachineScopedOnly { machine_id } => match auth.client_type {
            SocketClientType::UserScoped => true,
            SocketClientType::MachineScoped => {
                auth.machine_id.as_deref() == Some(machine_id.as_str())
            }
            SocketClientType::SessionScoped => false,
        },
        RecipientFilter::AllUserAuthenticatedConnections => true,
    }
}

pub fn build_new_session_update(session: &SessionRecord, seq: u64) -> DurableUpdateContainer {
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::NewSession {
            id: session.id.clone(),
            seq: session.seq,
            metadata: session.metadata.clone(),
            metadata_version: session.metadata_version,
            agent_state: session.agent_state.clone(),
            agent_state_version: session.agent_state_version,
            data_encryption_key: session.data_encryption_key.clone(),
            active: session.active,
            active_at: session.active_at,
            created_at: session.created_at,
            updated_at: session.updated_at,
        }),
        created_at: now_ms(),
    }
}

pub fn build_delete_session_update(session_id: &str, seq: u64) -> DurableUpdateContainer {
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::DeleteSession {
            sid: session_id.to_string(),
        }),
        created_at: now_ms(),
    }
}

pub fn build_new_machine_update(machine: &MachineRecord, seq: u64) -> DurableUpdateContainer {
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::NewMachine {
            machine_id: machine.id.clone(),
            seq: machine.seq,
            metadata: machine.metadata.clone(),
            metadata_version: machine.metadata_version,
            daemon_state: machine.daemon_state.clone(),
            daemon_state_version: machine.daemon_state_version,
            data_encryption_key: machine.data_encryption_key.clone(),
            active: machine.active,
            active_at: machine.active_at,
            created_at: machine.created_at,
            updated_at: machine.updated_at,
        }),
        created_at: now_ms(),
    }
}

pub fn build_new_message_update(
    session_id: &str,
    message: &SessionMessageRecord,
    seq: u64,
) -> DurableUpdateContainer {
    let core = CoreUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: CoreUpdateBody::NewMessage(UpdateNewMessageBody::new(
            session_id.to_string(),
            SessionMessage {
                id: message.id.clone(),
                seq: message.seq,
                local_id: Some(message.local_id.clone()),
                content: message.content.clone(),
                created_at: message.created_at,
                updated_at: message.updated_at,
            },
        )),
        created_at: now_ms(),
    };
    DurableUpdateContainer {
        id: core.id,
        seq: core.seq,
        body: DurableUpdateBody::Core(core.body),
        created_at: core.created_at,
    }
}

pub fn build_update_session_update(
    session_id: &str,
    seq: u64,
    metadata: Option<VersionedEncryptedValue>,
    agent_state: Option<VersionedNullableEncryptedValue>,
) -> DurableUpdateContainer {
    let mut body = UpdateSessionBody::new(session_id.to_string());
    body.metadata = metadata.map(Some);
    body.agent_state = agent_state.map(Some);
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Core(CoreUpdateBody::UpdateSession(body)),
        created_at: now_ms(),
    }
}

pub fn build_update_machine_update(
    machine_id: &str,
    seq: u64,
    metadata: Option<VersionedMachineEncryptedValue>,
    daemon_state: Option<VersionedMachineEncryptedValue>,
    active: Option<bool>,
    active_at: Option<u64>,
) -> DurableUpdateContainer {
    let mut body = UpdateMachineBody::new(machine_id.to_string());
    body.metadata = metadata.map(Some);
    body.daemon_state = daemon_state.map(Some);
    body.active = active;
    body.active_at = active_at;
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Core(CoreUpdateBody::UpdateMachine(body)),
        created_at: now_ms(),
    }
}

pub fn build_session_activity(
    session_id: &str,
    active: bool,
    active_at: u64,
    thinking: bool,
) -> EphemeralUpdate {
    EphemeralUpdate::Activity {
        id: session_id.to_string(),
        active,
        active_at,
        thinking: Some(thinking),
    }
}

pub fn build_machine_activity(machine_id: &str, active: bool, active_at: u64) -> EphemeralUpdate {
    EphemeralUpdate::MachineActivity {
        id: machine_id.to_string(),
        active,
        active_at,
    }
}

pub fn build_usage_update(
    session_id: &str,
    key: &str,
    tokens: std::collections::BTreeMap<String, u64>,
    cost: std::collections::BTreeMap<String, f64>,
) -> EphemeralUpdate {
    EphemeralUpdate::Usage {
        id: session_id.to_string(),
        key: key.to_string(),
        tokens,
        cost,
        timestamp: now_ms(),
    }
}

pub fn build_update_account_settings(
    user_id: &str,
    seq: u64,
    settings: Option<String>,
    version: u64,
) -> DurableUpdateContainer {
    build_update_account_profile(
        user_id,
        seq,
        settings.map(|value| LateVersionedValue { value, version }),
        None,
        None,
        None,
        None,
        None,
    )
}

pub fn build_update_account_profile(
    user_id: &str,
    seq: u64,
    settings: Option<LateVersionedValue>,
    github: Option<Option<serde_json::Value>>,
    username: Option<Option<String>>,
    first_name: Option<Option<String>>,
    last_name: Option<Option<String>>,
    avatar: Option<Option<serde_json::Value>>,
) -> DurableUpdateContainer {
    let mut changes = std::collections::BTreeMap::new();
    if let Some(settings) = settings {
        changes.insert("settings".into(), serde_json::json!(settings));
    }
    if let Some(github) = github {
        changes.insert("github".into(), github.unwrap_or(serde_json::Value::Null));
    }
    if let Some(username) = username {
        changes.insert(
            "username".into(),
            username
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );
    }
    if let Some(first_name) = first_name {
        changes.insert(
            "firstName".into(),
            first_name
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );
    }
    if let Some(last_name) = last_name {
        changes.insert(
            "lastName".into(),
            last_name
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );
    }
    if let Some(avatar) = avatar {
        changes.insert("avatar".into(), avatar.unwrap_or(serde_json::Value::Null));
    }
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::UpdateAccount {
            id: user_id.to_string(),
            changes,
        }),
        created_at: now_ms(),
    }
}

pub fn build_new_artifact_update(artifact: &ArtifactRecord, seq: u64) -> DurableUpdateContainer {
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::NewArtifact {
            artifact_id: artifact.id.clone(),
            seq: artifact.seq,
            header: artifact.header.clone(),
            header_version: artifact.header_version,
            body: artifact.body.clone(),
            body_version: artifact.body_version,
            data_encryption_key: artifact.data_encryption_key.clone(),
            created_at: artifact.created_at,
            updated_at: artifact.updated_at,
        }),
        created_at: now_ms(),
    }
}

pub fn build_update_artifact_update(
    artifact_id: &str,
    seq: u64,
    header: Option<LateVersionedValue>,
    body: Option<LateVersionedValue>,
) -> DurableUpdateContainer {
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::UpdateArtifact {
            artifact_id: artifact_id.to_string(),
            header,
            body,
        }),
        created_at: now_ms(),
    }
}

pub fn build_delete_artifact_update(artifact_id: &str, seq: u64) -> DurableUpdateContainer {
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::DeleteArtifact {
            artifact_id: artifact_id.to_string(),
        }),
        created_at: now_ms(),
    }
}

pub fn build_relationship_updated(
    target_user_id: &str,
    status: &str,
    seq: u64,
) -> DurableUpdateContainer {
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::RelationshipUpdated {
            uid: target_user_id.to_string(),
            status: status.to_string(),
            timestamp: now_ms(),
        }),
        created_at: now_ms(),
    }
}

pub fn build_new_feed_post(post: &FeedPostRecord, seq: u64) -> DurableUpdateContainer {
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::NewFeedPost {
            id: post.id.clone(),
            body: post.body.clone(),
            cursor: post.cursor.clone(),
            created_at: post.created_at,
        }),
        created_at: now_ms(),
    }
}

pub fn build_kv_batch_update(seq: u64, changes: Vec<KvBatchChange>) -> DurableUpdateContainer {
    DurableUpdateContainer {
        id: format!("upd_{}", uuid::Uuid::now_v7()),
        seq,
        body: DurableUpdateBody::Late(LateDurableUpdate::KvBatchUpdate { changes }),
        created_at: now_ms(),
    }
}

#[cfg(test)]
mod tests {
    use super::{RecipientFilter, build_update_account_profile, should_send_to_auth};
    use crate::auth::{SocketClientType, SocketConnectionAuth};

    #[test]
    fn session_filter_matches_user_and_session_scoped_only() {
        let auth = SocketConnectionAuth {
            user_id: "acct".into(),
            token: "token".into(),
            client_type: SocketClientType::SessionScoped,
            session_id: Some("sid".into()),
            machine_id: None,
        };

        assert!(should_send_to_auth(
            &auth,
            &RecipientFilter::AllInterestedInSession {
                session_id: "sid".into(),
            }
        ));
        assert!(!should_send_to_auth(
            &auth,
            &RecipientFilter::MachineScopedOnly {
                machine_id: "mid".into(),
            }
        ));
    }

    #[test]
    fn account_profile_update_serializes_explicit_null_fields() {
        let update =
            build_update_account_profile("acct", 7, None, Some(None), Some(None), None, None, None);
        let value = serde_json::to_value(update).unwrap();
        assert_eq!(value["body"]["t"], "update-account");
        assert_eq!(value["body"]["id"], "acct");
        assert_eq!(value["body"]["github"], serde_json::Value::Null);
        assert_eq!(value["body"]["username"], serde_json::Value::Null);
        assert!(!value["body"].as_object().unwrap().contains_key("settings"));
    }
}
