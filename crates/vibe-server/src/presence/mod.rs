pub mod cache;
pub mod timeout;

use tokio_util::sync::CancellationToken;

use crate::{
    events::{EventRouter, RecipientFilter, build_machine_activity, build_session_activity},
    storage::{
        db::{Database, now_ms},
        redis::RedisStore,
    },
};

use self::{
    cache::{PresenceEntry, PresenceState},
    timeout::{
        BATCH_FLUSH_INTERVAL_MS, CACHE_TTL_MS, DB_UPDATE_THRESHOLD_MS, INACTIVITY_TIMEOUT_MS,
        TIMEOUT_SWEEP_INTERVAL_MS,
    },
};

#[derive(Clone)]
pub struct PresenceService {
    db: Database,
    redis: RedisStore,
    events: EventRouter,
    shutdown: CancellationToken,
}

enum HeartbeatAction {
    Ignore,
    Emit { active_at: u64 },
    Reactivate { active_at: u64 },
}

impl PresenceService {
    pub fn new(
        db: Database,
        redis: RedisStore,
        events: EventRouter,
        shutdown: CancellationToken,
    ) -> Self {
        Self {
            db,
            redis,
            events,
            shutdown,
        }
    }

    pub fn spawn_background_tasks(&self) {
        let flush_self = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = flush_self.shutdown.cancelled() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_millis(BATCH_FLUSH_INTERVAL_MS)) => {
                        flush_self.flush_pending().await;
                    }
                }
            }
        });

        let timeout_self = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = timeout_self.shutdown.cancelled() => break,
                    _ = tokio::time::sleep(std::time::Duration::from_millis(TIMEOUT_SWEEP_INTERVAL_MS)) => {
                        timeout_self.run_timeout_sweep().await;
                    }
                }
            }
        });
    }

    pub async fn flush_pending(&self) {
        let mut pending_sessions = Vec::new();
        let mut pending_machines = Vec::new();
        let now = now_ms();

        self.redis.write_presence_state(|state| {
            for ((user_id, session_id), entry) in &mut state.sessions {
                if let Some(timestamp) = entry.pending_update.take() {
                    entry.last_update_sent = timestamp;
                    pending_sessions.push((user_id.clone(), session_id.clone(), timestamp));
                }
            }

            for ((user_id, machine_id), entry) in &mut state.machines {
                if let Some(timestamp) = entry.pending_update.take() {
                    entry.last_update_sent = timestamp;
                    pending_machines.push((user_id.clone(), machine_id.clone(), timestamp));
                }
            }

            cleanup_expired(state, now);
        });

        for (user_id, session_id, timestamp) in pending_sessions {
            if let Some((updated, _)) =
                self.db
                    .set_session_presence(&user_id, &session_id, true, timestamp)
            {
                self.sync_presence_entry(true, &user_id, &session_id, true, updated.active_at);
            }
        }

        for (user_id, machine_id, timestamp) in pending_machines {
            if let Some((updated, _)) =
                self.db
                    .set_machine_presence(&user_id, &machine_id, true, timestamp)
            {
                self.sync_presence_entry(false, &user_id, &machine_id, true, updated.active_at);
            }
        }
    }

    pub async fn session_alive(&self, user_id: &str, session_id: &str, time: u64, thinking: bool) {
        let Some(timestamp) = normalize_timestamp(time) else {
            return;
        };
        if !self.validate_session(user_id, session_id) {
            return;
        }
        match prepare_heartbeat(&self.redis, true, user_id, session_id, timestamp) {
            HeartbeatAction::Ignore => {}
            HeartbeatAction::Emit { active_at } => {
                self.events.emit_ephemeral(
                    user_id,
                    build_session_activity(session_id, true, active_at, thinking),
                    RecipientFilter::UserScopedOnly,
                    None,
                );
            }
            HeartbeatAction::Reactivate { active_at } => {
                if let Some((updated, _)) = self
                    .db
                    .set_session_presence(user_id, session_id, true, active_at)
                {
                    self.sync_presence_entry(true, user_id, session_id, true, updated.active_at);
                    self.events.emit_ephemeral(
                        user_id,
                        build_session_activity(session_id, true, updated.active_at, thinking),
                        RecipientFilter::UserScopedOnly,
                        None,
                    );
                }
            }
        }
    }

    pub async fn session_end(&self, user_id: &str, session_id: &str, time: u64) {
        let Some(timestamp) = normalize_timestamp(time) else {
            return;
        };
        if let Some((updated, transitioned)) = self
            .db
            .set_session_presence(user_id, session_id, false, timestamp)
        {
            self.sync_presence_entry(true, user_id, session_id, updated.active, updated.active_at);
            if transitioned {
                self.events.emit_ephemeral(
                    user_id,
                    build_session_activity(session_id, false, updated.active_at, false),
                    RecipientFilter::UserScopedOnly,
                    None,
                );
            }
        }
    }

    pub async fn machine_alive(&self, user_id: &str, machine_id: &str, time: u64) {
        let Some(timestamp) = normalize_timestamp(time) else {
            return;
        };
        if !self.validate_machine(user_id, machine_id) {
            return;
        }
        match prepare_heartbeat(&self.redis, false, user_id, machine_id, timestamp) {
            HeartbeatAction::Ignore => {}
            HeartbeatAction::Emit { active_at } => {
                self.events.emit_ephemeral(
                    user_id,
                    build_machine_activity(machine_id, true, active_at),
                    RecipientFilter::UserScopedOnly,
                    None,
                );
            }
            HeartbeatAction::Reactivate { active_at } => {
                if let Some((updated, _)) = self
                    .db
                    .set_machine_presence(user_id, machine_id, true, active_at)
                {
                    self.sync_presence_entry(false, user_id, machine_id, true, updated.active_at);
                    self.events.emit_ephemeral(
                        user_id,
                        build_machine_activity(machine_id, true, updated.active_at),
                        RecipientFilter::UserScopedOnly,
                        None,
                    );
                }
            }
        }
    }

    pub async fn machine_connected(&self, user_id: &str, machine_id: &str, time: u64) {
        self.apply_machine_presence(user_id, machine_id, time, true)
            .await;
    }

    pub async fn machine_disconnected(&self, user_id: &str, machine_id: &str, time: u64) {
        self.apply_machine_presence(user_id, machine_id, time, false)
            .await;
    }

    pub fn sync_machine_presence(&self, user_id: &str, machine_id: &str, timestamp: u64) {
        self.sync_presence_entry(false, user_id, machine_id, true, timestamp);
    }

    pub fn invalidate_session(&self, user_id: &str, session_id: &str) {
        self.redis.write_presence_state(|state| {
            state
                .sessions
                .remove(&(user_id.to_string(), session_id.to_string()));
        });
    }

    pub fn validate_session(&self, user_id: &str, session_id: &str) -> bool {
        let now = now_ms();
        let key = (user_id.to_string(), session_id.to_string());
        if self.redis.read_presence_state(|state| {
            state
                .sessions
                .get(&key)
                .is_some_and(|entry| entry.valid_until > now)
        }) {
            return true;
        }

        let Some(record) = self.db.get_session_for_account(user_id, session_id) else {
            return false;
        };
        self.redis.write_presence_state(|state| {
            state.sessions.insert(
                key,
                PresenceEntry {
                    valid_until: now + CACHE_TTL_MS,
                    last_update_sent: record.active_at,
                    pending_update: None,
                    active: record.active,
                },
            );
        });
        true
    }

    pub fn validate_machine(&self, user_id: &str, machine_id: &str) -> bool {
        let now = now_ms();
        let key = (user_id.to_string(), machine_id.to_string());
        if self.redis.read_presence_state(|state| {
            state
                .machines
                .get(&key)
                .is_some_and(|entry| entry.valid_until > now)
        }) {
            return true;
        }

        let Some(record) = self.db.get_machine_for_account(user_id, machine_id) else {
            return false;
        };
        self.redis.write_presence_state(|state| {
            state.machines.insert(
                key,
                PresenceEntry {
                    valid_until: now + CACHE_TTL_MS,
                    last_update_sent: record.active_at,
                    pending_update: None,
                    active: record.active,
                },
            );
        });
        true
    }

    async fn run_timeout_sweep(&self) {
        let cutoff = now_ms().saturating_sub(INACTIVITY_TIMEOUT_MS);
        for session in self.db.timed_out_sessions(cutoff) {
            if let Some((updated, changed)) = self.db.set_session_presence(
                &session.account_id,
                &session.id,
                false,
                session.active_at,
            ) {
                self.clear_pending_offline(
                    true,
                    &updated.account_id,
                    &updated.id,
                    updated.active_at,
                );
                if changed {
                    self.events.emit_ephemeral(
                        &updated.account_id,
                        build_session_activity(&updated.id, false, updated.active_at, false),
                        RecipientFilter::UserScopedOnly,
                        None,
                    );
                }
            }
        }

        for machine in self.db.timed_out_machines(cutoff) {
            if let Some((updated, changed)) = self.db.set_machine_presence(
                &machine.account_id,
                &machine.id,
                false,
                machine.active_at,
            ) {
                self.clear_pending_offline(
                    false,
                    &updated.account_id,
                    &updated.id,
                    updated.active_at,
                );
                if changed {
                    self.events.emit_ephemeral(
                        &updated.account_id,
                        build_machine_activity(&updated.id, false, updated.active_at),
                        RecipientFilter::UserScopedOnly,
                        None,
                    );
                }
            }
        }
    }

    fn clear_pending_offline(&self, session: bool, user_id: &str, entity_id: &str, timestamp: u64) {
        self.sync_presence_entry(session, user_id, entity_id, false, timestamp);
    }

    fn sync_presence_entry(
        &self,
        session: bool,
        user_id: &str,
        entity_id: &str,
        active: bool,
        timestamp: u64,
    ) {
        let key = (user_id.to_string(), entity_id.to_string());
        self.redis.write_presence_state(|state| {
            let map = if session {
                &mut state.sessions
            } else {
                &mut state.machines
            };
            map.entry(key)
                .and_modify(|entry| {
                    entry.pending_update = None;
                    entry.last_update_sent = timestamp;
                    entry.valid_until = now_ms() + CACHE_TTL_MS;
                    entry.active = active;
                })
                .or_insert(PresenceEntry {
                    valid_until: now_ms() + CACHE_TTL_MS,
                    last_update_sent: timestamp,
                    pending_update: None,
                    active,
                });
        });
    }

    async fn apply_machine_presence(
        &self,
        user_id: &str,
        machine_id: &str,
        time: u64,
        active: bool,
    ) {
        let Some(timestamp) = normalize_timestamp(time) else {
            return;
        };
        if !self.validate_machine(user_id, machine_id) {
            return;
        }
        if let Some((updated, transitioned)) = self
            .db
            .set_machine_presence(user_id, machine_id, active, timestamp)
        {
            self.sync_presence_entry(
                false,
                user_id,
                machine_id,
                updated.active,
                updated.active_at,
            );
            if transitioned {
                self.events.emit_ephemeral(
                    user_id,
                    build_machine_activity(machine_id, updated.active, updated.active_at),
                    RecipientFilter::UserScopedOnly,
                    None,
                );
            }
        }
    }
}

fn prepare_heartbeat(
    redis: &RedisStore,
    session: bool,
    user_id: &str,
    entity_id: &str,
    timestamp: u64,
) -> HeartbeatAction {
    let key = (user_id.to_string(), entity_id.to_string());
    redis.write_presence_state(|state| {
        let map = if session {
            &mut state.sessions
        } else {
            &mut state.machines
        };
        let entry = map.entry(key).or_insert(PresenceEntry {
            valid_until: now_ms() + CACHE_TTL_MS,
            last_update_sent: 0,
            pending_update: None,
            active: false,
        });
        entry.valid_until = now_ms() + CACHE_TTL_MS;
        let latest_seen = entry.pending_update.unwrap_or(entry.last_update_sent);
        if !entry.active {
            if timestamp <= latest_seen {
                return HeartbeatAction::Ignore;
            }
            return HeartbeatAction::Reactivate {
                active_at: timestamp,
            };
        }
        let effective_active_at = latest_seen.max(timestamp);
        if timestamp > latest_seen
            && timestamp.saturating_sub(entry.last_update_sent) > DB_UPDATE_THRESHOLD_MS
        {
            entry.pending_update = Some(
                entry
                    .pending_update
                    .map_or(timestamp, |pending| pending.max(timestamp)),
            );
        }
        HeartbeatAction::Emit {
            active_at: effective_active_at,
        }
    })
}

fn cleanup_expired(state: &mut PresenceState, now: u64) {
    state.sessions.retain(|_, entry| entry.valid_until > now);
    state.machines.retain(|_, entry| entry.valid_until > now);
}

fn normalize_timestamp(timestamp: u64) -> Option<u64> {
    let now = now_ms();
    if timestamp > now {
        return Some(now);
    }
    if timestamp < now.saturating_sub(INACTIVITY_TIMEOUT_MS) {
        return None;
    }
    Some(timestamp)
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
    use tokio_util::sync::CancellationToken;

    use crate::{
        auth::{SocketClientType, SocketConnectionAuth},
        events::{ClientConnection, EventRouter},
        storage::{
            db::{Database, now_ms},
            redis::RedisStore,
        },
    };

    use super::{
        PresenceService,
        cache::PresenceEntry,
        timeout::{CACHE_TTL_MS, DB_UPDATE_THRESHOLD_MS, INACTIVITY_TIMEOUT_MS},
    };

    fn test_service(db: Database, events: EventRouter) -> PresenceService {
        PresenceService::new(db, RedisStore::default(), events, CancellationToken::new())
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

    async fn recv_until_event(
        rx: &mut tokio::sync::mpsc::Receiver<EioPacket>,
        event: &str,
    ) -> Packet {
        for _ in 0..4 {
            let message = recv_message(rx).await;
            let mut packet = decode_message(message);
            if let PacketData::Event(value, _) = &mut packet.inner
                && CommonParser.read_event(value).unwrap() == event
            {
                return packet;
            }
        }
        panic!("timed out waiting for event {event}");
    }

    fn decode_event_payload<T: DeserializeOwned>(packet: &mut Packet, with_event: bool) -> T {
        match &mut packet.inner {
            PacketData::Event(value, _) | PacketData::EventAck(value, _) => {
                CommonParser.decode_value(value, with_event).unwrap()
            }
            other => panic!("unexpected packet data: {other:?}"),
        }
    }

    async fn attach_user_scoped_connection(
        events: &EventRouter,
        user_id: &str,
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
        events.add_connection(ClientConnection {
            socket_id: socket.id.to_string(),
            socket,
            user_id: user_id.to_string(),
            auth: SocketConnectionAuth {
                user_id: user_id.to_string(),
                token: "token".into(),
                client_type: SocketClientType::UserScoped,
                session_id: None,
                machine_id: None,
            },
        });
        (io, client_rx)
    }

    #[tokio::test]
    async fn validation_cache_hits_after_first_lookup() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "meta", None);
        let service = test_service(db.clone(), EventRouter::default());

        assert!(service.validate_session(&account.id, &session.id));
        assert!(service.validate_session(&account.id, &session.id));
    }

    #[tokio::test]
    async fn machine_validation_cache_hits_after_first_lookup() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (machine, _) = db.create_or_load_machine(&account.id, "machine-1", "meta", None, None);
        let service = test_service(db.clone(), EventRouter::default());

        assert!(service.validate_machine(&account.id, &machine.id));
        assert!(service.validate_machine(&account.id, &machine.id));
    }

    #[tokio::test]
    async fn flush_persists_pending_updates_before_expired_entries_are_dropped() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "meta", None);
        let service = test_service(db.clone(), EventRouter::default());
        let timestamp = session.active_at.saturating_add(45_000);

        service.redis.write_presence_state(|state| {
            state.sessions.insert(
                (account.id.clone(), session.id.clone()),
                PresenceEntry {
                    valid_until: 0,
                    last_update_sent: 0,
                    pending_update: Some(timestamp),
                    active: true,
                },
            );
        });

        service.flush_pending().await;

        let updated = db
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert!(updated.active);
        assert_eq!(updated.active_at, timestamp);
    }

    #[tokio::test]
    async fn heartbeats_inside_threshold_do_not_persist_a_second_write() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "meta", None);
        let service = test_service(db.clone(), EventRouter::default());
        let first_timestamp = now_ms().saturating_sub(60_000);
        db.write(|state| {
            let session = state
                .sessions
                .get_mut(&session.id)
                .expect("session should exist");
            session.active_at = first_timestamp.saturating_sub(DB_UPDATE_THRESHOLD_MS + 1_000);
            session.active = true;
        });
        service.redis.write_presence_state(|state| {
            state.sessions.insert(
                (account.id.clone(), session.id.clone()),
                PresenceEntry {
                    valid_until: now_ms() + CACHE_TTL_MS,
                    last_update_sent: first_timestamp
                        .saturating_sub(DB_UPDATE_THRESHOLD_MS + 1_000),
                    pending_update: None,
                    active: true,
                },
            );
        });

        service
            .session_alive(&account.id, &session.id, first_timestamp, false)
            .await;
        service.flush_pending().await;

        let after_first_flush = db
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert_eq!(after_first_flush.active_at, first_timestamp);

        service
            .session_alive(
                &account.id,
                &session.id,
                first_timestamp.saturating_sub(1_000),
                false,
            )
            .await;
        service.flush_pending().await;

        let after_second_flush = db
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert_eq!(after_second_flush.active_at, first_timestamp);
    }

    #[tokio::test]
    async fn older_session_heartbeat_does_not_regress_active_at() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "meta", None);
        let events = EventRouter::default();
        let (_io, mut rx) = attach_user_scoped_connection(&events, &account.id).await;
        let service = test_service(db.clone(), events);
        let current_active_at = now_ms().saturating_sub(60_000);
        let older_heartbeat = current_active_at.saturating_sub(DB_UPDATE_THRESHOLD_MS + 5_000);

        db.write(|state| {
            let session = state
                .sessions
                .get_mut(&session.id)
                .expect("session should exist");
            session.active_at = current_active_at;
            session.active = true;
        });
        service.redis.write_presence_state(|state| {
            state.sessions.insert(
                (account.id.clone(), session.id.clone()),
                PresenceEntry {
                    valid_until: now_ms() + CACHE_TTL_MS,
                    last_update_sent: current_active_at,
                    pending_update: None,
                    active: true,
                },
            );
        });

        service
            .session_alive(&account.id, &session.id, older_heartbeat, false)
            .await;

        let mut packet = recv_until_event(&mut rx, "ephemeral").await;
        let payload: JsonValue = decode_event_payload(&mut packet, true);
        assert_eq!(payload["type"], "activity");
        assert_eq!(payload["id"], session.id);
        assert_eq!(payload["active"], true);
        assert_eq!(payload["activeAt"], current_active_at);

        service.flush_pending().await;

        let updated = db
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert_eq!(updated.active_at, current_active_at);
        assert!(service.redis.read_presence_state(|state| {
            state
                .sessions
                .get(&(account.id.clone(), session.id.clone()))
                .expect("cache entry should exist")
                .pending_update
                .is_none()
        }));
    }

    #[tokio::test]
    async fn session_alive_after_session_end_reactivates_persisted_state_immediately() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "meta", None);
        let start_active_at = now_ms().saturating_sub(60_000);
        let offline_at = start_active_at.saturating_add(5_000);
        let resume_at = offline_at.saturating_add(1_000);
        db.write(|state| {
            let session = state
                .sessions
                .get_mut(&session.id)
                .expect("session should exist");
            session.active = true;
            session.active_at = start_active_at;
        });
        let events = EventRouter::default();
        let (_io, mut rx) = attach_user_scoped_connection(&events, &account.id).await;
        let service = test_service(db.clone(), events);

        service
            .session_end(&account.id, &session.id, offline_at)
            .await;
        let _ = recv_until_event(&mut rx, "ephemeral").await;

        service
            .session_alive(&account.id, &session.id, resume_at, false)
            .await;

        let mut packet = recv_until_event(&mut rx, "ephemeral").await;
        let payload: JsonValue = decode_event_payload(&mut packet, true);
        assert_eq!(payload["type"], "activity");
        assert_eq!(payload["id"], session.id);
        assert_eq!(payload["active"], true);
        assert_eq!(payload["activeAt"], resume_at);

        let updated = db
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert!(updated.active);
        assert_eq!(updated.active_at, resume_at);
        let cache_entry = service.redis.read_presence_state(|state| {
            state
                .sessions
                .get(&(account.id.clone(), session.id.clone()))
                .expect("cache entry should exist")
                .clone()
        });
        assert!(cache_entry.active);
        assert!(cache_entry.pending_update.is_none());
        assert_eq!(cache_entry.last_update_sent, resume_at);
    }

    #[tokio::test]
    async fn stale_session_end_does_not_override_newer_online_state() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "meta", None);
        let base_active_at = now_ms().saturating_sub(60_000);
        let heartbeat_at = base_active_at.saturating_add(DB_UPDATE_THRESHOLD_MS + 5_000);
        let stale_end_at = heartbeat_at.saturating_sub(1_000);
        db.write(|state| {
            let session = state
                .sessions
                .get_mut(&session.id)
                .expect("session should exist");
            session.active = true;
            session.active_at = base_active_at;
        });
        let events = EventRouter::default();
        let (_io, mut rx) = attach_user_scoped_connection(&events, &account.id).await;
        let service = test_service(db.clone(), events);

        service
            .session_alive(&account.id, &session.id, heartbeat_at, false)
            .await;
        let _ = recv_until_event(&mut rx, "ephemeral").await;
        service.flush_pending().await;

        service
            .session_end(&account.id, &session.id, stale_end_at)
            .await;

        let updated = db
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert!(updated.active);
        assert_eq!(updated.active_at, heartbeat_at);
        let cache_entry = service.redis.read_presence_state(|state| {
            state
                .sessions
                .get(&(account.id.clone(), session.id.clone()))
                .expect("cache entry should exist")
                .clone()
        });
        assert!(cache_entry.active);
        assert!(cache_entry.pending_update.is_none());
        assert_eq!(cache_entry.last_update_sent, heartbeat_at);
        assert!(
            recv_packet_timeout(&mut rx, Duration::from_millis(30))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn timeout_sweep_emits_session_offline_once() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "meta", None);
        let offline_at = now_ms().saturating_sub(INACTIVITY_TIMEOUT_MS + 1_000);
        db.write(|state| {
            let session = state
                .sessions
                .get_mut(&session.id)
                .expect("session should exist");
            session.active = true;
            session.active_at = offline_at;
        });
        let events = EventRouter::default();
        let (_io, mut rx) = attach_user_scoped_connection(&events, &account.id).await;
        let service = test_service(db.clone(), events);

        service.run_timeout_sweep().await;

        let updated = db
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert!(!updated.active);
        let mut packet = recv_until_event(&mut rx, "ephemeral").await;
        let payload: JsonValue = decode_event_payload(&mut packet, true);
        assert_eq!(payload["type"], "activity");
        assert_eq!(payload["id"], session.id);
        assert_eq!(payload["active"], false);
        assert_eq!(payload["activeAt"], offline_at);

        service.run_timeout_sweep().await;
        assert!(
            recv_packet_timeout(&mut rx, Duration::from_millis(30))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn timeout_sweep_marks_machine_inactive() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (machine, _) = db.create_or_load_machine(&account.id, "machine-1", "meta", None, None);
        let offline_at = now_ms().saturating_sub(INACTIVITY_TIMEOUT_MS + 1_000);
        db.write(|state| {
            let machine = state
                .machines
                .get_mut(&(account.id.clone(), machine.id.clone()))
                .expect("machine should exist");
            machine.active = true;
            machine.active_at = offline_at;
        });
        let service = test_service(db.clone(), EventRouter::default());

        service.run_timeout_sweep().await;

        let updated = db
            .get_machine_for_account(&account.id, &machine.id)
            .expect("machine should exist");
        assert!(!updated.active);
        assert_eq!(updated.active_at, offline_at);
    }

    #[tokio::test]
    async fn session_end_clears_pending_updates_before_flush() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (session, _) = db.create_or_load_session(&account.id, "tag", "meta", None);
        let service = test_service(db.clone(), EventRouter::default());
        let pending_timestamp = now_ms().saturating_sub(60_000);
        let end_timestamp = pending_timestamp.saturating_add(5_000);
        db.write(|state| {
            let session = state
                .sessions
                .get_mut(&session.id)
                .expect("session should exist");
            session.active = true;
            session.active_at = pending_timestamp.saturating_sub(DB_UPDATE_THRESHOLD_MS + 1_000);
        });

        service.redis.write_presence_state(|state| {
            state.sessions.insert(
                (account.id.clone(), session.id.clone()),
                PresenceEntry {
                    valid_until: now_ms() + CACHE_TTL_MS,
                    last_update_sent: pending_timestamp
                        .saturating_sub(DB_UPDATE_THRESHOLD_MS + 1_000),
                    pending_update: Some(pending_timestamp),
                    active: true,
                },
            );
        });

        service
            .session_end(&account.id, &session.id, end_timestamp)
            .await;
        service.flush_pending().await;

        let updated = db
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert!(!updated.active);
        assert_eq!(updated.active_at, end_timestamp);
    }

    #[tokio::test]
    async fn timeout_sweep_clears_pending_machine_updates_before_flush() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (machine, _) = db.create_or_load_machine(&account.id, "machine-1", "meta", None, None);
        let offline_at = now_ms().saturating_sub(INACTIVITY_TIMEOUT_MS + 1_000);
        let pending_timestamp = now_ms().saturating_sub(60_000);
        db.write(|state| {
            let machine = state
                .machines
                .get_mut(&(account.id.clone(), machine.id.clone()))
                .expect("machine should exist");
            machine.active = true;
            machine.active_at = offline_at;
        });
        let service = test_service(db.clone(), EventRouter::default());

        service.redis.write_presence_state(|state| {
            state.machines.insert(
                (account.id.clone(), machine.id.clone()),
                PresenceEntry {
                    valid_until: now_ms() + CACHE_TTL_MS,
                    last_update_sent: offline_at,
                    pending_update: Some(pending_timestamp),
                    active: true,
                },
            );
        });

        service.run_timeout_sweep().await;
        service.flush_pending().await;

        let updated = db
            .get_machine_for_account(&account.id, &machine.id)
            .expect("machine should exist");
        assert!(!updated.active);
        assert_eq!(updated.active_at, offline_at);
    }

    #[tokio::test]
    async fn machine_disconnect_clears_pending_updates_and_persists_offline() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (machine, _) = db.create_or_load_machine(&account.id, "machine-1", "meta", None, None);
        let service = test_service(db.clone(), EventRouter::default());
        let connected_at = now_ms().saturating_sub(30_000);
        let disconnect_at = connected_at.saturating_add(20_000);
        db.write(|state| {
            let machine = state
                .machines
                .get_mut(&(account.id.clone(), machine.id.clone()))
                .expect("machine should exist");
            machine.active = false;
            machine.active_at = connected_at.saturating_sub(DB_UPDATE_THRESHOLD_MS + 1_000);
        });

        service
            .machine_connected(&account.id, &machine.id, connected_at)
            .await;
        service.redis.write_presence_state(|state| {
            state.machines.insert(
                (account.id.clone(), machine.id.clone()),
                PresenceEntry {
                    valid_until: now_ms() + CACHE_TTL_MS,
                    last_update_sent: connected_at,
                    pending_update: Some(connected_at.saturating_add(5_000)),
                    active: true,
                },
            );
        });

        service
            .machine_disconnected(&account.id, &machine.id, disconnect_at)
            .await;
        service.flush_pending().await;

        let updated = db
            .get_machine_for_account(&account.id, &machine.id)
            .expect("machine should exist");
        assert!(!updated.active);
        assert_eq!(updated.active_at, disconnect_at);
        assert!(service.redis.read_presence_state(|state| {
            state
                .machines
                .get(&(account.id.clone(), machine.id.clone()))
                .expect("cache entry should exist")
                .pending_update
                .is_none()
        }));
    }

    #[tokio::test]
    async fn machine_alive_after_disconnect_reactivates_persisted_state_immediately() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (machine, _) = db.create_or_load_machine(&account.id, "machine-1", "meta", None, None);
        let connected_at = now_ms().saturating_sub(60_000);
        let offline_at = connected_at.saturating_add(5_000);
        let resume_at = offline_at.saturating_add(1_000);
        db.write(|state| {
            let machine = state
                .machines
                .get_mut(&(account.id.clone(), machine.id.clone()))
                .expect("machine should exist");
            machine.active = true;
            machine.active_at = connected_at;
        });
        let events = EventRouter::default();
        let (_io, mut rx) = attach_user_scoped_connection(&events, &account.id).await;
        let service = test_service(db.clone(), events);

        service
            .machine_disconnected(&account.id, &machine.id, offline_at)
            .await;
        let _ = recv_until_event(&mut rx, "ephemeral").await;

        service
            .machine_alive(&account.id, &machine.id, resume_at)
            .await;

        let mut packet = recv_until_event(&mut rx, "ephemeral").await;
        let payload: JsonValue = decode_event_payload(&mut packet, true);
        assert_eq!(payload["type"], "machine-activity");
        assert_eq!(payload["id"], machine.id);
        assert_eq!(payload["active"], true);
        assert_eq!(payload["activeAt"], resume_at);

        let updated = db
            .get_machine_for_account(&account.id, &machine.id)
            .expect("machine should exist");
        assert!(updated.active);
        assert_eq!(updated.active_at, resume_at);
        let cache_entry = service.redis.read_presence_state(|state| {
            state
                .machines
                .get(&(account.id.clone(), machine.id.clone()))
                .expect("cache entry should exist")
                .clone()
        });
        assert!(cache_entry.active);
        assert!(cache_entry.pending_update.is_none());
        assert_eq!(cache_entry.last_update_sent, resume_at);
    }

    #[tokio::test]
    async fn stale_machine_disconnect_does_not_override_newer_connected_state() {
        let db = Database::default();
        let account = db.upsert_account_by_public_key("pk");
        let (machine, _) = db.create_or_load_machine(&account.id, "machine-1", "meta", None, None);
        let base_active_at = now_ms().saturating_sub(60_000);
        let connected_at = base_active_at.saturating_add(10_000);
        let stale_disconnect_at = connected_at.saturating_sub(1_000);
        db.write(|state| {
            let machine = state
                .machines
                .get_mut(&(account.id.clone(), machine.id.clone()))
                .expect("machine should exist");
            machine.active = false;
            machine.active_at = base_active_at;
        });
        let events = EventRouter::default();
        let (_io, mut rx) = attach_user_scoped_connection(&events, &account.id).await;
        let service = test_service(db.clone(), events);

        service
            .machine_connected(&account.id, &machine.id, connected_at)
            .await;
        let _ = recv_until_event(&mut rx, "ephemeral").await;

        service
            .machine_disconnected(&account.id, &machine.id, stale_disconnect_at)
            .await;

        let updated = db
            .get_machine_for_account(&account.id, &machine.id)
            .expect("machine should exist");
        assert!(updated.active);
        assert_eq!(updated.active_at, connected_at);
        let cache_entry = service.redis.read_presence_state(|state| {
            state
                .machines
                .get(&(account.id.clone(), machine.id.clone()))
                .expect("cache entry should exist")
                .clone()
        });
        assert!(cache_entry.active);
        assert!(cache_entry.pending_update.is_none());
        assert_eq!(cache_entry.last_update_sent, connected_at);
        assert!(
            recv_packet_timeout(&mut rx, Duration::from_millis(30))
                .await
                .is_none()
        );
    }
}
