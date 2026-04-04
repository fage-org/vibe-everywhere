use std::{sync::Arc, time::Duration};

use futures_util::FutureExt;
use rust_socketio::{
    Event, Payload, TransportType,
    asynchronous::{Client as SocketClient, ClientBuilder},
};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::{Mutex, Notify, broadcast};
use vibe_wire::{UpdateNewMessageBody, UpdateSessionBody};

use crate::encryption::{
    EncryptionError, EncryptionVariant, decode_base64, decrypt_json, encode_base64, encrypt_json,
};

#[derive(Debug, Clone)]
pub struct SessionClientOptions {
    pub session_id: String,
    pub encryption_key: [u8; 32],
    pub encryption_variant: EncryptionVariant,
    pub token: String,
    pub server_url: String,
    pub initial_metadata: Option<Value>,
    pub initial_metadata_version: u64,
    pub initial_agent_state: Option<Value>,
    pub initial_agent_state_version: u64,
}

#[derive(Debug, Clone)]
pub struct SessionMessageEvent {
    pub id: String,
    pub seq: u64,
    pub content: Value,
    pub local_id: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone)]
pub enum SessionEvent {
    Connected,
    Disconnected(String),
    StateChange {
        metadata: Option<Value>,
        agent_state: Option<Value>,
    },
    Message(SessionMessageEvent),
    Error(String),
}

#[derive(Debug, Clone, Default)]
struct SessionState {
    connected: bool,
    last_error: Option<String>,
    metadata: Option<Value>,
    metadata_version: u64,
    agent_state: Option<Value>,
    agent_state_version: u64,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Timeout waiting for socket connection")]
    ConnectionTimeout,
    #[error("{0}")]
    Connection(String),
    #[error("Timeout waiting for agent to become idle")]
    IdleTimeout,
    #[error("Timeout waiting for agent turn completion")]
    TurnCompletionTimeout,
    #[error("Session is archived")]
    SessionArchived,
    #[error("Socket disconnected while waiting for agent to become idle")]
    DisconnectedWhileWaitingIdle,
    #[error("Socket disconnected while waiting for agent turn completion")]
    DisconnectedWhileWaitingTurnCompletion,
    #[error(transparent)]
    Socket(#[from] rust_socketio::Error),
    #[error(transparent)]
    Encryption(#[from] EncryptionError),
}

pub struct SessionClient {
    session_id: String,
    encryption_key: [u8; 32],
    encryption_variant: EncryptionVariant,
    socket: SocketClient,
    state: Arc<Mutex<SessionState>>,
    ready: Arc<Notify>,
    events: broadcast::Sender<SessionEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdleState {
    Archived,
    Busy,
    Idle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TurnEvent {
    kind: String,
    turn_id: Option<String>,
}

impl SessionClient {
    pub async fn connect(options: SessionClientOptions) -> Result<Self, SessionError> {
        let state = Arc::new(Mutex::new(SessionState {
            connected: false,
            last_error: None,
            metadata: options.initial_metadata,
            metadata_version: options.initial_metadata_version,
            agent_state: options.initial_agent_state,
            agent_state_version: options.initial_agent_state_version,
        }));
        let ready = Arc::new(Notify::new());
        let (events, _) = broadcast::channel(128);

        let connect_state = state.clone();
        let connect_ready = ready.clone();
        let connect_events = events.clone();

        let disconnect_state = state.clone();
        let disconnect_ready = ready.clone();
        let disconnect_events = events.clone();

        let error_state = state.clone();
        let error_ready = ready.clone();
        let error_events = events.clone();

        let update_state = state.clone();
        let update_events = events.clone();
        let encryption_key = options.encryption_key;
        let encryption_variant = options.encryption_variant;

        let builder = ClientBuilder::new(options.server_url)
            .transport_type(TransportType::Websocket)
            .auth(serde_json::json!({
                "token": options.token,
                "clientType": "session-scoped",
                "sessionId": options.session_id,
            }))
            .reconnect(true)
            .reconnect_on_disconnect(true)
            .reconnect_delay(1_000, 5_000)
            .on(Event::Connect, move |_, _| {
                let state = connect_state.clone();
                let ready = connect_ready.clone();
                let events = connect_events.clone();
                async move {
                    let mut guard = state.lock().await;
                    guard.connected = true;
                    guard.last_error = None;
                    drop(guard);
                    ready.notify_waiters();
                    let _ = events.send(SessionEvent::Connected);
                }
                .boxed()
            })
            .on(Event::Close, move |_, _| {
                let state = disconnect_state.clone();
                let ready = disconnect_ready.clone();
                let events = disconnect_events.clone();
                async move {
                    let mut guard = state.lock().await;
                    guard.connected = false;
                    guard.last_error = Some("Socket disconnected".into());
                    drop(guard);
                    ready.notify_waiters();
                    let _ = events.send(SessionEvent::Disconnected("Socket disconnected".into()));
                }
                .boxed()
            })
            .on(Event::Error, move |payload, _| {
                let state = error_state.clone();
                let ready = error_ready.clone();
                let events = error_events.clone();
                let message = payload_to_string(&payload);
                async move {
                    let mut guard = state.lock().await;
                    guard.last_error = Some(message.clone());
                    drop(guard);
                    ready.notify_waiters();
                    let _ = events.send(SessionEvent::Error(message));
                }
                .boxed()
            })
            .on("update", move |payload, _| {
                let state = update_state.clone();
                let events = update_events.clone();
                async move {
                    if let Some(value) = first_payload_value(&payload)
                        && let Err(error) = handle_update(
                            value,
                            encryption_key,
                            encryption_variant,
                            state,
                            events.clone(),
                        )
                        .await
                    {
                        let _ = events.send(SessionEvent::Error(error.to_string()));
                    }
                }
                .boxed()
            });

        let socket = tokio::time::timeout(Duration::from_secs(10), builder.connect())
            .await
            .map_err(|_| SessionError::ConnectionTimeout)??;

        Ok(Self {
            session_id: options.session_id,
            encryption_key: options.encryption_key,
            encryption_variant: options.encryption_variant,
            socket,
            state,
            ready,
            events,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.events.subscribe()
    }

    pub async fn wait_for_connect(&self, timeout: Duration) -> Result<(), SessionError> {
        let wait = async {
            let mut receiver = self.events.subscribe();
            loop {
                let notified = self.ready.notified();
                {
                    let state = self.state.lock().await;
                    if state.connected {
                        return Ok(());
                    }
                    if let Some(error) = state.last_error.clone() {
                        return Err(SessionError::Connection(error));
                    }
                }

                tokio::select! {
                    _ = notified => {},
                    event = receiver.recv() => {
                        match event {
                            Ok(SessionEvent::Connected) => return Ok(()),
                            Ok(SessionEvent::Error(error)) => return Err(SessionError::Connection(error)),
                            Ok(SessionEvent::Disconnected(reason)) => return Err(SessionError::Connection(reason)),
                            _ => {}
                        }
                    }
                }
            }
        };

        tokio::time::timeout(timeout, wait)
            .await
            .map_err(|_| SessionError::ConnectionTimeout)?
    }

    pub async fn send_message(
        &self,
        text: &str,
        meta: Option<serde_json::Map<String, Value>>,
    ) -> Result<(), SessionError> {
        let mut message = serde_json::Map::new();
        message.insert("role".into(), Value::String("user".into()));
        message.insert(
            "content".into(),
            serde_json::json!({
                "type": "text",
                "text": text,
            }),
        );

        let mut merged_meta = serde_json::Map::new();
        merged_meta.insert("sentFrom".into(), Value::String("vibe-agent".into()));
        if let Some(meta) = meta {
            for (key, value) in meta {
                merged_meta.insert(key, value);
            }
        }
        message.insert("meta".into(), Value::Object(merged_meta));

        let encrypted = encrypt_json(
            &self.encryption_key,
            self.encryption_variant,
            &Value::Object(message),
        )?;
        self.socket
            .emit(
                "message",
                serde_json::json!({
                    "sid": self.session_id,
                    "message": encode_base64(&encrypted),
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn send_stop(&self) -> Result<(), SessionError> {
        self.socket
            .emit(
                "session-end",
                serde_json::json!({
                    "sid": self.session_id,
                    "time": now_ms(),
                }),
            )
            .await?;
        Ok(())
    }

    pub async fn get_metadata(&self) -> Option<Value> {
        self.state.lock().await.metadata.clone()
    }

    pub async fn get_agent_state(&self) -> Option<Value> {
        self.state.lock().await.agent_state.clone()
    }

    pub async fn wait_for_idle(&self, timeout: Duration) -> Result<(), SessionError> {
        let mut receiver = self.subscribe();
        self.wait_for_idle_on(&mut receiver, timeout).await
    }

    pub async fn wait_for_idle_on(
        &self,
        receiver: &mut broadcast::Receiver<SessionEvent>,
        timeout: Duration,
    ) -> Result<(), SessionError> {
        wait_for_idle_events(self.state.clone(), receiver, timeout).await
    }

    pub async fn wait_for_turn_completion(&self, timeout: Duration) -> Result<(), SessionError> {
        let mut receiver = self.subscribe();
        self.wait_for_turn_completion_on(&mut receiver, timeout)
            .await
    }

    pub async fn wait_for_turn_completion_on(
        &self,
        receiver: &mut broadcast::Receiver<SessionEvent>,
        timeout: Duration,
    ) -> Result<(), SessionError> {
        wait_for_turn_completion_events(self.state.clone(), receiver, timeout).await
    }

    pub async fn close(&self) {
        let _ = self.socket.disconnect().await;
    }
}

async fn handle_update(
    value: Value,
    encryption_key: [u8; 32],
    encryption_variant: EncryptionVariant,
    state: Arc<Mutex<SessionState>>,
    events: broadcast::Sender<SessionEvent>,
) -> Result<(), SessionError> {
    let Some(body) = value.get("body").cloned() else {
        return Ok(());
    };
    let Some(kind) = body.get("t").and_then(Value::as_str) else {
        return Ok(());
    };

    match kind {
        "new-message" => {
            let update: UpdateNewMessageBody = serde_json::from_value(body)
                .map_err(|error| SessionError::Connection(error.to_string()))?;
            let ciphertext = decode_base64(&update.message.content.ciphertext)?;
            let Some(content) = decrypt_json(&encryption_key, encryption_variant, &ciphertext)
            else {
                return Ok(());
            };

            let _ = events.send(SessionEvent::Message(SessionMessageEvent {
                id: update.message.id,
                seq: update.message.seq,
                content,
                local_id: update.message.local_id.flatten(),
                created_at: update.message.created_at,
                updated_at: update.message.updated_at,
            }));
        }
        "update-session" => {
            let update: UpdateSessionBody = serde_json::from_value(body)
                .map_err(|error| SessionError::Connection(error.to_string()))?;
            let mut guard = state.lock().await;

            if let Some(metadata) = update.metadata.flatten()
                && metadata.version > guard.metadata_version
            {
                let ciphertext = decode_base64(&metadata.value)?;
                guard.metadata = decrypt_json(&encryption_key, encryption_variant, &ciphertext);
                guard.metadata_version = metadata.version;
            }

            if let Some(agent_state) = update.agent_state.flatten()
                && agent_state.version > guard.agent_state_version
            {
                guard.agent_state = match agent_state.value {
                    Some(value) => {
                        let ciphertext = decode_base64(&value)?;
                        decrypt_json(&encryption_key, encryption_variant, &ciphertext)
                    }
                    None => None,
                };
                guard.agent_state_version = agent_state.version;
            }

            let metadata = guard.metadata.clone();
            let agent_state = guard.agent_state.clone();
            drop(guard);

            let _ = events.send(SessionEvent::StateChange {
                metadata,
                agent_state,
            });
        }
        _ => {}
    }

    Ok(())
}

async fn wait_for_idle_events(
    state: Arc<Mutex<SessionState>>,
    receiver: &mut broadcast::Receiver<SessionEvent>,
    timeout: Duration,
) -> Result<(), SessionError> {
    {
        let state = state.lock().await;
        match check_idle_state(state.metadata.as_ref(), state.agent_state.as_ref()) {
            IdleState::Archived => return Err(SessionError::SessionArchived),
            IdleState::Idle => return Ok(()),
            IdleState::Busy => {}
        }
    }

    let wait = async {
        loop {
            match receiver.recv().await {
                Ok(SessionEvent::StateChange { .. }) => {
                    let state = state.lock().await;
                    match check_idle_state(state.metadata.as_ref(), state.agent_state.as_ref()) {
                        IdleState::Archived => return Err(SessionError::SessionArchived),
                        IdleState::Idle => return Ok(()),
                        IdleState::Busy => {}
                    }
                }
                Ok(SessionEvent::Disconnected(_)) => {
                    return Err(SessionError::DisconnectedWhileWaitingIdle);
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(SessionError::DisconnectedWhileWaitingIdle);
                }
            }
        }
    };

    tokio::time::timeout(timeout, wait)
        .await
        .map_err(|_| SessionError::IdleTimeout)?
}

async fn wait_for_turn_completion_events(
    state: Arc<Mutex<SessionState>>,
    receiver: &mut broadcast::Receiver<SessionEvent>,
    timeout: Duration,
) -> Result<(), SessionError> {
    let wait = async {
        let mut saw_activity = false;
        let mut active_turn_id: Option<String> = None;
        let mut saw_turn_start = false;
        let mut saw_non_ready_message = false;

        loop {
            match receiver.recv().await {
                Ok(SessionEvent::Message(message)) => {
                    saw_activity = true;

                    if let Some(turn_event) = get_turn_event(&message.content) {
                        if turn_event.kind == "turn-start" {
                            saw_turn_start = true;
                            saw_non_ready_message = true;
                            active_turn_id = turn_event.turn_id;
                            continue;
                        }

                        if active_turn_id.is_none()
                            || turn_event.turn_id.is_none()
                            || turn_event.turn_id == active_turn_id
                        {
                            return Ok(());
                        }
                        continue;
                    }

                    if is_ready_event(&message.content) {
                        if saw_turn_start || saw_non_ready_message {
                            return Ok(());
                        }
                        continue;
                    }

                    saw_non_ready_message = true;
                }
                Ok(SessionEvent::StateChange { .. }) => {
                    if !saw_activity || saw_turn_start {
                        continue;
                    }
                    let state = state.lock().await;
                    match check_idle_state(state.metadata.as_ref(), state.agent_state.as_ref()) {
                        IdleState::Archived => return Err(SessionError::SessionArchived),
                        IdleState::Idle => return Ok(()),
                        IdleState::Busy => {}
                    }
                }
                Ok(SessionEvent::Disconnected(_)) => {
                    return Err(SessionError::DisconnectedWhileWaitingTurnCompletion);
                }
                Ok(_) => {}
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(SessionError::DisconnectedWhileWaitingTurnCompletion);
                }
            }
        }
    };

    tokio::time::timeout(timeout, wait)
        .await
        .map_err(|_| SessionError::TurnCompletionTimeout)?
}

fn check_idle_state(metadata: Option<&Value>, agent_state: Option<&Value>) -> IdleState {
    if metadata
        .and_then(Value::as_object)
        .and_then(|meta| meta.get("lifecycleState"))
        .and_then(Value::as_str)
        == Some("archived")
    {
        return IdleState::Archived;
    }

    let Some(state) = agent_state.and_then(Value::as_object) else {
        return IdleState::Busy;
    };

    let controlled_by_user = state
        .get("controlledByUser")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let has_requests = state
        .get("requests")
        .and_then(Value::as_object)
        .is_some_and(|requests| !requests.is_empty());

    if !controlled_by_user && !has_requests {
        IdleState::Idle
    } else {
        IdleState::Busy
    }
}

fn get_turn_event(content: &Value) -> Option<TurnEvent> {
    let envelope = content.as_object()?;
    if envelope.get("role").and_then(Value::as_str) != Some("session") {
        return None;
    }
    let body = envelope.get("content")?.as_object()?;
    let event = body.get("ev")?.as_object()?;
    let kind = event.get("t")?.as_str()?;
    if kind != "turn-start" && kind != "turn-end" {
        return None;
    }

    Some(TurnEvent {
        kind: kind.to_owned(),
        turn_id: body
            .get("turn")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

fn is_ready_event(content: &Value) -> bool {
    let envelope = match content.as_object() {
        Some(envelope) => envelope,
        None => return false,
    };
    if envelope.get("role").and_then(Value::as_str) != Some("agent") {
        return false;
    }
    let body = match envelope.get("content").and_then(Value::as_object) {
        Some(body) => body,
        None => return false,
    };
    body.get("type").and_then(Value::as_str) == Some("event")
        && body
            .get("data")
            .and_then(Value::as_object)
            .and_then(|data| data.get("type"))
            .and_then(Value::as_str)
            == Some("ready")
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

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use serde_json::json;
    use tokio::sync::{Mutex, broadcast};

    use super::{
        IdleState, SessionError, SessionEvent, SessionMessageEvent, SessionState, check_idle_state,
        get_turn_event, is_ready_event, wait_for_idle_events, wait_for_turn_completion_events,
    };

    fn idle_state_value() -> serde_json::Value {
        json!({
            "controlledByUser": false,
            "requests": {}
        })
    }

    fn busy_state_value() -> serde_json::Value {
        json!({
            "controlledByUser": true,
            "requests": {}
        })
    }

    fn message_event(content: serde_json::Value) -> SessionEvent {
        SessionEvent::Message(SessionMessageEvent {
            id: "msg-1".into(),
            seq: 1,
            content,
            local_id: None,
            created_at: 1,
            updated_at: 1,
        })
    }

    async fn set_state_and_emit(
        state: &Arc<Mutex<SessionState>>,
        events: &broadcast::Sender<SessionEvent>,
        metadata: Option<serde_json::Value>,
        agent_state: Option<serde_json::Value>,
    ) {
        let mut guard = state.lock().await;
        guard.metadata = metadata.clone();
        guard.agent_state = agent_state.clone();
        drop(guard);
        let _ = events.send(SessionEvent::StateChange {
            metadata,
            agent_state,
        });
    }

    #[test]
    fn idle_state_detects_archived_sessions() {
        let metadata = json!({"lifecycleState": "archived"});
        assert_eq!(
            check_idle_state(Some(&metadata), Some(&json!({}))),
            IdleState::Archived
        );
    }

    #[test]
    fn idle_state_detects_idle_agent() {
        let state = idle_state_value();
        assert_eq!(check_idle_state(None, Some(&state)), IdleState::Idle);
    }

    #[tokio::test]
    async fn wait_for_idle_resolves_when_agent_becomes_idle() {
        let state = Arc::new(Mutex::new(SessionState::default()));
        let (events, _) = broadcast::channel(8);
        let mut receiver = events.subscribe();
        let state_for_update = state.clone();
        let events_for_update = events.clone();

        let update_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            set_state_and_emit(
                &state_for_update,
                &events_for_update,
                None,
                Some(idle_state_value()),
            )
            .await;
        });

        wait_for_idle_events(state, &mut receiver, Duration::from_millis(200))
            .await
            .unwrap();
        update_task.await.unwrap();
    }

    #[tokio::test]
    async fn wait_for_idle_times_out_when_agent_stays_busy() {
        let state = Arc::new(Mutex::new(SessionState {
            agent_state: Some(busy_state_value()),
            ..SessionState::default()
        }));
        let (events, _) = broadcast::channel(8);
        let mut receiver = events.subscribe();

        let error = wait_for_idle_events(state, &mut receiver, Duration::from_millis(50))
            .await
            .unwrap_err();
        assert!(matches!(error, SessionError::IdleTimeout));
    }

    #[test]
    fn turn_event_is_extracted_from_session_protocol_messages() {
        let content = json!({
            "role": "session",
            "content": {
                "turn": "turn-1",
                "ev": {
                    "t": "turn-start"
                }
            }
        });

        let event = get_turn_event(&content).unwrap();
        assert_eq!(event.kind, "turn-start");
        assert_eq!(event.turn_id.as_deref(), Some("turn-1"));
    }

    #[test]
    fn ready_event_is_detected_from_agent_messages() {
        let content = json!({
            "role": "agent",
            "content": {
                "type": "event",
                "data": {
                    "type": "ready"
                }
            }
        });

        assert!(is_ready_event(&content));
    }

    #[tokio::test]
    async fn wait_for_turn_completion_resolves_on_turn_end_after_turn_start() {
        let state = Arc::new(Mutex::new(SessionState::default()));
        let (events, _) = broadcast::channel(8);
        let mut receiver = events.subscribe();
        let events_for_update = events.clone();

        let update_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = events_for_update.send(message_event(json!({
                "role": "session",
                "content": {
                    "turn": "turn-1",
                    "ev": { "t": "turn-start" },
                }
            })));
            let _ = events_for_update.send(message_event(json!({
                "role": "session",
                "content": {
                    "turn": "turn-1",
                    "ev": { "t": "turn-end", "status": "completed" },
                }
            })));
        });

        wait_for_turn_completion_events(state, &mut receiver, Duration::from_millis(200))
            .await
            .unwrap();
        update_task.await.unwrap();
    }

    #[tokio::test]
    async fn wait_for_turn_completion_resolves_on_ready_after_activity() {
        let state = Arc::new(Mutex::new(SessionState::default()));
        let (events, _) = broadcast::channel(8);
        let mut receiver = events.subscribe();
        let events_for_update = events.clone();

        let update_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = events_for_update.send(message_event(json!({
                "role": "session",
                "content": {
                    "ev": { "t": "text", "text": "Thinking..." },
                }
            })));
            let _ = events_for_update.send(message_event(json!({
                "role": "agent",
                "content": {
                    "type": "event",
                    "data": { "type": "ready" },
                }
            })));
        });

        wait_for_turn_completion_events(state, &mut receiver, Duration::from_millis(200))
            .await
            .unwrap();
        update_task.await.unwrap();
    }

    #[tokio::test]
    async fn wait_for_turn_completion_ignores_idle_before_any_activity() {
        let state = Arc::new(Mutex::new(SessionState::default()));
        let (events, _) = broadcast::channel(8);
        let mut receiver = events.subscribe();
        let state_for_update = state.clone();
        let events_for_update = events.clone();

        let update_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            set_state_and_emit(
                &state_for_update,
                &events_for_update,
                None,
                Some(idle_state_value()),
            )
            .await;
        });

        let error =
            wait_for_turn_completion_events(state, &mut receiver, Duration::from_millis(80))
                .await
                .unwrap_err();
        assert!(matches!(error, SessionError::TurnCompletionTimeout));
        update_task.await.unwrap();
    }

    #[tokio::test]
    async fn wait_for_turn_completion_falls_back_to_idle_after_activity() {
        let state = Arc::new(Mutex::new(SessionState::default()));
        let (events, _) = broadcast::channel(8);
        let mut receiver = events.subscribe();
        let state_for_update = state.clone();
        let events_for_update = events.clone();

        let update_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = events_for_update.send(message_event(json!({
                "role": "assistant",
                "content": {
                    "type": "text",
                    "text": "Working on it",
                }
            })));
            set_state_and_emit(
                &state_for_update,
                &events_for_update,
                None,
                Some(idle_state_value()),
            )
            .await;
        });

        wait_for_turn_completion_events(state, &mut receiver, Duration::from_millis(200))
            .await
            .unwrap();
        update_task.await.unwrap();
    }

    #[tokio::test]
    async fn wait_for_turn_completion_requires_turn_end_once_turn_started() {
        let state = Arc::new(Mutex::new(SessionState::default()));
        let (events, _) = broadcast::channel(8);
        let mut receiver = events.subscribe();
        let state_for_update = state.clone();
        let events_for_update = events.clone();

        let update_task = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = events_for_update.send(message_event(json!({
                "role": "session",
                "content": {
                    "turn": "turn-2",
                    "ev": { "t": "turn-start" },
                }
            })));
            set_state_and_emit(
                &state_for_update,
                &events_for_update,
                None,
                Some(idle_state_value()),
            )
            .await;
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = events_for_update.send(message_event(json!({
                "role": "session",
                "content": {
                    "turn": "turn-2",
                    "ev": { "t": "turn-end", "status": "completed" },
                }
            })));
        });

        let wait =
            wait_for_turn_completion_events(state, &mut receiver, Duration::from_millis(200));
        tokio::pin!(wait);
        assert!(
            tokio::time::timeout(Duration::from_millis(40), &mut wait)
                .await
                .is_err()
        );
        wait.await.unwrap();
        update_task.await.unwrap();
    }
}
