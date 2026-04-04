use crate::{
    api::types::ApiError,
    context::AppContext,
    events::EventPublishError,
    storage::db::{CompareAndSwap, MachineRecord},
};

#[derive(Clone)]
pub struct MachinesService {
    ctx: AppContext,
}

impl MachinesService {
    pub fn new(ctx: AppContext) -> Self {
        Self { ctx }
    }

    pub fn create_or_load(
        &self,
        user_id: &str,
        machine_id: &str,
        metadata: &str,
        daemon_state: Option<String>,
        data_encryption_key: Option<String>,
    ) -> Result<MachineRecord, ApiError> {
        let (machine, created) = self.ctx.db().create_or_load_machine(
            user_id,
            machine_id,
            metadata,
            daemon_state.clone(),
            data_encryption_key,
        );
        if created {
            self.ctx
                .events()
                .publish_new_machine(self.ctx.db(), user_id, &machine)
                .map_err(map_event_publish_error)?;
        }
        Ok(machine)
    }

    pub fn list(&self, user_id: &str) -> Vec<MachineRecord> {
        self.ctx.db().list_machines(user_id)
    }

    pub fn detail(&self, user_id: &str, machine_id: &str) -> Result<MachineRecord, ApiError> {
        self.ctx
            .db()
            .get_machine_for_account(user_id, machine_id)
            .ok_or_else(|| ApiError::not_found("Machine not found"))
    }

    pub fn update_metadata(
        &self,
        user_id: &str,
        machine_id: &str,
        expected_version: u64,
        metadata: String,
    ) -> Result<CompareAndSwap<String>, ApiError> {
        let result =
            self.ctx
                .db()
                .update_machine_metadata(user_id, machine_id, expected_version, &metadata);
        if matches!(result, CompareAndSwap::Success(_)) {
            self.ctx
                .events()
                .publish_machine_metadata_update(
                    self.ctx.db(),
                    user_id,
                    machine_id,
                    vibe_wire::VersionedMachineEncryptedValue {
                        version: expected_version + 1,
                        value: metadata.clone(),
                    },
                )
                .map_err(map_event_publish_error)?;
        }
        Ok(result)
    }

    pub fn update_daemon_state(
        &self,
        user_id: &str,
        machine_id: &str,
        expected_version: u64,
        daemon_state: String,
    ) -> Result<CompareAndSwap<Option<String>>, ApiError> {
        let result = self.ctx.db().update_machine_daemon_state(
            user_id,
            machine_id,
            expected_version,
            daemon_state.clone(),
        );
        if matches!(result, CompareAndSwap::Success(_)) {
            if let Some(machine) = self.ctx.db().get_machine_for_account(user_id, machine_id) {
                self.ctx
                    .presence()
                    .sync_machine_presence(user_id, machine_id, machine.active_at);
            }
            self.ctx
                .events()
                .publish_machine_daemon_state_update(
                    self.ctx.db(),
                    user_id,
                    machine_id,
                    vibe_wire::VersionedMachineEncryptedValue {
                        version: expected_version + 1,
                        value: daemon_state,
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

    use super::MachinesService;

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
    async fn create_or_load_emits_new_machine_and_scoped_update_shapes() {
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
        let (_matching_io, mut matching_rx) = attach_connection(
            &ctx,
            SocketConnectionAuth {
                user_id: account.id.clone(),
                token: "token".into(),
                client_type: SocketClientType::MachineScoped,
                session_id: None,
                machine_id: Some("machine-1".into()),
            },
        )
        .await;
        let (_other_io, mut other_rx) = attach_connection(
            &ctx,
            SocketConnectionAuth {
                user_id: account.id.clone(),
                token: "token".into(),
                client_type: SocketClientType::MachineScoped,
                session_id: None,
                machine_id: Some("machine-2".into()),
            },
        )
        .await;

        let service = MachinesService::new(ctx.clone());
        let machine = service
            .create_or_load(
                &account.id,
                "machine-1",
                "metadata",
                Some("daemon".into()),
                Some("key".into()),
            )
            .unwrap();

        let mut new_machine_packet = recv_update_packet(&mut user_rx).await;
        let new_machine_payload: JsonValue = decode_event_payload(&mut new_machine_packet, true);
        assert_eq!(new_machine_payload["body"]["t"], "new-machine");
        assert_eq!(new_machine_payload["body"]["machineId"], machine.id);
        assert_eq!(new_machine_payload["body"]["metadata"], "metadata");
        assert_eq!(new_machine_payload["body"]["metadataVersion"], 1);
        assert_eq!(new_machine_payload["body"]["daemonState"], "daemon");
        assert_eq!(new_machine_payload["body"]["daemonStateVersion"], 1);
        assert_eq!(new_machine_payload["body"]["dataEncryptionKey"], "key");

        let mut user_update_packet = recv_update_packet(&mut user_rx).await;
        let user_update_payload: JsonValue = decode_event_payload(&mut user_update_packet, true);
        assert_eq!(user_update_payload["body"]["t"], "update-machine");
        assert_eq!(user_update_payload["body"]["machineId"], machine.id);
        assert_eq!(user_update_payload["body"]["metadata"]["version"], 1);
        assert_eq!(user_update_payload["body"]["metadata"]["value"], "metadata");
        assert!(user_update_payload["body"].get("daemonState").is_none());

        let mut matching_update_packet = recv_update_packet(&mut matching_rx).await;
        let matching_update_payload: JsonValue =
            decode_event_payload(&mut matching_update_packet, true);
        assert_eq!(matching_update_payload["body"]["t"], "update-machine");
        assert_eq!(matching_update_payload["body"]["machineId"], machine.id);

        assert!(
            recv_packet_timeout(&mut other_rx, Duration::from_millis(30))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn update_daemon_state_surfaces_publish_error_when_user_sequence_is_unavailable() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let machine = ctx
            .db()
            .create_or_load_machine(&account.id, "machine-1", "meta", None, None)
            .0;
        ctx.db().write(|state| {
            state.accounts.remove(&account.id);
        });

        let service = MachinesService::new(ctx);
        assert!(
            service
                .update_daemon_state(&account.id, &machine.id, 0, "online".into())
                .is_err()
        );
    }
}
