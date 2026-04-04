use vibe_wire::SessionMessageContent;

use crate::{
    api::types::ApiError,
    context::AppContext,
    events::EventPublishError,
    storage::db::{CompareAndSwap, SessionMessageRecord, SessionRecord},
};

#[derive(Clone)]
pub struct SessionsService {
    ctx: AppContext,
}

impl SessionsService {
    pub fn new(ctx: AppContext) -> Self {
        Self { ctx }
    }

    pub fn list_v1(&self, user_id: &str) -> Vec<SessionRecord> {
        self.ctx.db().list_sessions_by_updated_at(user_id, 150)
    }

    pub fn list_active(&self, user_id: &str, limit: usize) -> Vec<SessionRecord> {
        self.ctx.db().list_active_sessions(
            user_id,
            limit,
            crate::storage::db::now_ms().saturating_sub(900_000),
        )
    }

    pub fn list_v2(
        &self,
        user_id: &str,
        cursor: Option<&str>,
        limit: usize,
        changed_since: Option<u64>,
    ) -> Result<(Vec<SessionRecord>, Option<String>, bool), ApiError> {
        let cursor_id = match cursor {
            Some(cursor) if cursor.starts_with("cursor_v1_") => Some(&cursor[10..]),
            Some(_) => return Err(ApiError::bad_request("Invalid cursor format")),
            None => None,
        };
        let (sessions, has_next) =
            self.ctx
                .db()
                .page_sessions(user_id, cursor_id, changed_since, limit);
        let next_cursor = has_next
            .then(|| {
                sessions
                    .last()
                    .map(|session| format!("cursor_v1_{}", session.id))
            })
            .flatten();
        Ok((sessions, next_cursor, has_next))
    }

    pub fn create_or_load(
        &self,
        user_id: &str,
        tag: &str,
        metadata: &str,
        data_encryption_key: Option<String>,
    ) -> Result<SessionRecord, ApiError> {
        let (session, created) =
            self.ctx
                .db()
                .create_or_load_session(user_id, tag, metadata, data_encryption_key);
        if created {
            self.ctx
                .events()
                .publish_new_session(self.ctx.db(), user_id, &session)
                .map_err(map_event_publish_error)?;
        }
        Ok(session)
    }

    pub fn history(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<Vec<SessionMessageRecord>, ApiError> {
        self.ctx
            .db()
            .get_session_for_account(user_id, session_id)
            .ok_or_else(|| ApiError::not_found("Session not found"))?;
        Ok(self.ctx.db().list_session_messages_desc(session_id, 150))
    }

    pub fn delete(&self, user_id: &str, session_id: &str) -> Result<(), ApiError> {
        let deleted = self
            .ctx
            .db()
            .delete_session_for_account(user_id, session_id)
            .ok_or_else(|| ApiError::not_found("Session not found or not owned by user"))?;
        self.ctx.presence().invalidate_session(user_id, &deleted.id);
        self.ctx
            .events()
            .publish_delete_session(self.ctx.db(), user_id, &deleted.id)
            .map_err(map_event_publish_error)?;
        Ok(())
    }

    pub fn page_messages(
        &self,
        user_id: &str,
        session_id: &str,
        after_seq: u64,
        limit: usize,
    ) -> Result<(Vec<SessionMessageRecord>, bool), ApiError> {
        self.ctx
            .db()
            .get_session_for_account(user_id, session_id)
            .ok_or_else(|| ApiError::not_found("Session not found"))?;
        Ok(self
            .ctx
            .db()
            .page_session_messages(session_id, after_seq, limit))
    }

    pub fn append_bulk_messages(
        &self,
        user_id: &str,
        session_id: &str,
        messages: Vec<(String, String)>,
    ) -> Result<(Vec<SessionMessageRecord>, Vec<SessionMessageRecord>), ApiError> {
        self.ctx
            .db()
            .append_messages_idempotent(user_id, session_id, messages)
            .ok_or_else(|| ApiError::not_found("Session not found"))
    }

    pub fn emit_new_message(
        &self,
        user_id: &str,
        session_id: &str,
        message: &SessionMessageRecord,
        skip_socket_id: Option<&str>,
    ) -> Result<(), ApiError> {
        self.ctx
            .events()
            .publish_new_message(self.ctx.db(), user_id, session_id, message, skip_socket_id)
            .map_err(map_event_publish_error)?;
        Ok(())
    }

    pub fn append_single_message(
        &self,
        user_id: &str,
        session_id: &str,
        ciphertext: String,
        local_id: Option<String>,
        skip_socket_id: Option<&str>,
    ) -> Result<(), ApiError> {
        let (message, created) = self
            .ctx
            .db()
            .append_message(
                user_id,
                session_id,
                SessionMessageContent::new(ciphertext),
                local_id,
            )
            .ok_or_else(|| ApiError::not_found("Session not found"))?;
        if created {
            self.emit_new_message(user_id, session_id, &message, skip_socket_id)?;
        }
        Ok(())
    }

    pub fn update_metadata(
        &self,
        user_id: &str,
        session_id: &str,
        expected_version: u64,
        metadata: String,
    ) -> Result<CompareAndSwap<String>, ApiError> {
        let result =
            self.ctx
                .db()
                .update_session_metadata(user_id, session_id, expected_version, &metadata);
        if matches!(result, CompareAndSwap::Success(_)) {
            self.ctx
                .events()
                .publish_session_metadata_update(
                    self.ctx.db(),
                    user_id,
                    session_id,
                    vibe_wire::VersionedEncryptedValue {
                        version: expected_version + 1,
                        value: metadata.clone(),
                    },
                )
                .map_err(map_event_publish_error)?;
        }
        Ok(result)
    }

    pub fn update_agent_state(
        &self,
        user_id: &str,
        session_id: &str,
        expected_version: u64,
        agent_state: Option<String>,
    ) -> Result<CompareAndSwap<Option<String>>, ApiError> {
        let result = self.ctx.db().update_session_agent_state(
            user_id,
            session_id,
            expected_version,
            agent_state.clone(),
        );
        if matches!(result, CompareAndSwap::Success(_)) {
            self.ctx
                .events()
                .publish_session_agent_state_update(
                    self.ctx.db(),
                    user_id,
                    session_id,
                    vibe_wire::VersionedNullableEncryptedValue {
                        version: expected_version + 1,
                        value: agent_state.clone(),
                    },
                )
                .map_err(map_event_publish_error)?;
        }
        Ok(result)
    }
}

fn map_event_publish_error(error: EventPublishError) -> ApiError {
    match error {
        EventPublishError::UpdateSequenceUnavailable => {
            ApiError::internal("Failed to allocate update sequence")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use engineioxide::Packet as EioPacket;
    use serde::de::DeserializeOwned;
    use serde_json::Value as JsonValue;
    use socketioxide::{SocketIo, extract::SocketRef};
    use socketioxide_core::{
        packet::{Packet, PacketData},
        parser::{Parse, ParserState},
    };
    use socketioxide_parser_common::CommonParser;

    use crate::{
        auth::{SocketClientType, SocketConnectionAuth},
        config::Config,
        context::AppContext,
        events::ClientConnection,
    };

    use super::SessionsService;

    fn test_config() -> Config {
        Config {
            host: "127.0.0.1".parse().unwrap(),
            port: 3005,
            master_secret: "secret".into(),
            ios_up_to_date: ">=1.4.1".into(),
            android_up_to_date: ">=1.4.1".into(),
            ios_store_url: "ios-store".into(),
            android_store_url: "android-store".into(),
            webapp_url: "https://app.vibe.engineering".into(),
        }
    }

    async fn recv_message(rx: &mut tokio::sync::mpsc::Receiver<EioPacket>) -> String {
        let packet = tokio::time::timeout(Duration::from_millis(50), rx.recv())
            .await
            .unwrap()
            .unwrap();
        match packet {
            EioPacket::Message(message) => message.to_string(),
            other => panic!("unexpected packet: {other:?}"),
        }
    }

    fn decode_message(message: String) -> Packet {
        CommonParser
            .decode_str(&ParserState::default(), message.into())
            .unwrap()
    }

    async fn recv_update_packet(rx: &mut tokio::sync::mpsc::Receiver<EioPacket>) -> Packet {
        for _ in 0..4 {
            let mut packet = decode_message(recv_message(rx).await);
            if let PacketData::Event(value, _) = &mut packet.inner
                && CommonParser.read_event(value).unwrap() == "update"
            {
                return packet;
            }
        }
        panic!("timed out waiting for update packet");
    }

    async fn recv_packet_timeout(
        rx: &mut tokio::sync::mpsc::Receiver<EioPacket>,
        timeout: Duration,
    ) -> Option<Packet> {
        let packet = tokio::time::timeout(timeout, rx.recv()).await.ok()??;
        match packet {
            EioPacket::Message(message) => Some(decode_message(message.to_string())),
            other => panic!("unexpected packet: {other:?}"),
        }
    }

    fn decode_event_payload<T: DeserializeOwned>(packet: &mut Packet, with_event: bool) -> T {
        match &mut packet.inner {
            PacketData::Event(value, _) | PacketData::EventAck(value, _) => {
                CommonParser.decode_value(value, with_event).unwrap()
            }
            other => panic!("unexpected packet data: {other:?}"),
        }
    }

    async fn attach_connection(
        ctx: &AppContext,
        auth: SocketConnectionAuth,
    ) -> (SocketIo, tokio::sync::mpsc::Receiver<EioPacket>) {
        let (_svc, io) = SocketIo::new_svc();
        let (socket_tx, mut socket_rx) = tokio::sync::mpsc::channel::<SocketRef>(1);
        io.ns("/", move |socket: SocketRef| {
            let socket_tx = socket_tx.clone();
            async move {
                socket_tx.send(socket).await.unwrap();
            }
        });

        let (_client_tx, mut client_rx) = io.new_dummy_sock("/", ()).await;
        let _ = recv_message(&mut client_rx).await;
        let socket = socket_rx.recv().await.unwrap();
        ctx.events().add_connection(ClientConnection {
            socket_id: socket.id.to_string(),
            socket,
            user_id: auth.user_id.clone(),
            auth,
        });
        (io, client_rx)
    }

    #[tokio::test]
    async fn create_or_load_emits_new_session_update_to_user_scoped_only() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let (_user_io, mut user_rx) = attach_connection(
            &ctx,
            SocketConnectionAuth {
                user_id: account.id.clone(),
                token: "token".into(),
                client_type: SocketClientType::UserScoped,
                session_id: None,
                machine_id: None,
            },
        )
        .await;
        let (_session_io, mut session_rx) = attach_connection(
            &ctx,
            SocketConnectionAuth {
                user_id: account.id.clone(),
                token: "token".into(),
                client_type: SocketClientType::SessionScoped,
                session_id: Some("session-other".into()),
                machine_id: None,
            },
        )
        .await;

        let service = SessionsService::new(ctx.clone());
        let session = service
            .create_or_load(&account.id, "tag-1", "ciphertext", Some("key".into()))
            .unwrap();

        let mut packet = recv_update_packet(&mut user_rx).await;
        let payload: JsonValue = decode_event_payload(&mut packet, true);
        assert_eq!(payload["body"]["t"], "new-session");
        assert_eq!(payload["body"]["id"], session.id);
        assert_eq!(payload["body"]["metadata"], "ciphertext");
        assert_eq!(payload["body"]["metadataVersion"], 0);
        assert_eq!(payload["body"]["dataEncryptionKey"], "key");
        assert!(payload["createdAt"].is_number());

        assert!(
            recv_packet_timeout(&mut session_rx, Duration::from_millis(30))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn delete_emits_delete_session_update_to_user_scoped_only() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let (session, _) =
            ctx.db()
                .create_or_load_session(&account.id, "tag-1", "ciphertext", None);
        let (_user_io, mut user_rx) = attach_connection(
            &ctx,
            SocketConnectionAuth {
                user_id: account.id.clone(),
                token: "token".into(),
                client_type: SocketClientType::UserScoped,
                session_id: None,
                machine_id: None,
            },
        )
        .await;
        let (_session_io, mut session_rx) = attach_connection(
            &ctx,
            SocketConnectionAuth {
                user_id: account.id.clone(),
                token: "token".into(),
                client_type: SocketClientType::SessionScoped,
                session_id: Some(session.id.clone()),
                machine_id: None,
            },
        )
        .await;

        let service = SessionsService::new(ctx.clone());
        service.delete(&account.id, &session.id).unwrap();

        let mut packet = recv_update_packet(&mut user_rx).await;
        let payload: JsonValue = decode_event_payload(&mut packet, true);
        assert_eq!(payload["body"]["t"], "delete-session");
        assert_eq!(payload["body"]["sid"], session.id);
        assert!(payload["createdAt"].is_number());

        assert!(
            recv_packet_timeout(&mut session_rx, Duration::from_millis(30))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn update_metadata_surfaces_publish_error_when_user_sequence_is_unavailable() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let (session, _) =
            ctx.db()
                .create_or_load_session(&account.id, "tag-1", "ciphertext", None);
        ctx.db().write(|state| {
            state.accounts.remove(&account.id);
        });

        let service = SessionsService::new(ctx);
        assert!(
            service
                .update_metadata(&account.id, &session.id, 0, "next".into())
                .is_err()
        );
    }
}
