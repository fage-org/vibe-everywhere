//! WebSocket Client Module
//!
//! Manages the persistent WebSocket connection between ve-daemon and ve-server.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message as WsMessage},
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::{
    AckPayload, DaemonToServer, ErrorPayload, SessionControlAction, WsEnvelope,
};

use crate::agent::DriverEvent;
use crate::config::Config;
use crate::error::{AckError, DaemonError};
use crate::file_ops::FileOps;
use crate::session_registry::SessionRegistry;
use crate::Result;

/// Type alias for WebSocket sender
#[allow(clippy::type_complexity)]
type WsSender = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    WsMessage,
>;

/// WebSocket client for daemon-server communication
#[allow(dead_code)]
pub struct WsClient {
    /// Configuration reference
    config: Arc<Config>,
    /// Host UUID
    host_id: Uuid,
    /// Authentication token
    token: String,
    /// Session registry
    registry: Option<Arc<SessionRegistry>>,
    /// Broadcast sender for runner events (used to subscribe on each reconnect)
    event_tx: Option<broadcast::Sender<DriverEvent>>,
    /// WebSocket sender for sending acks (wrapped in Arc for sharing)
    ws_sender: Option<Arc<tokio::sync::Mutex<WsSender>>>,
    /// File operations handler (workspace roots collected from sessions)
    file_ops: Option<FileOps>,
}

/// Wait for SIGTERM shutdown signal.
/// Returns immediately on non-Unix platforms (no-op).
#[cfg(unix)]
async fn shutdown_signal() {
    let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to install SIGTERM handler");
    signal.recv().await;
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    std::future::pending::<()>().await;
}

impl WsClient {
    /// Create a new WebSocket client
    pub fn new(config: Arc<Config>, host_id: Uuid, token: String) -> Self {
        // Initialize FileOps with empty workspace roots (will be updated when sessions are created)
        let file_ops = FileOps::new(
            vec![],
            config.file_read_text_limit_bytes as usize,
            config.file_tree_max_nodes,
        );
        Self {
            config,
            host_id,
            token,
            registry: None,
            event_tx: None,
            ws_sender: None,
            file_ops: Some(file_ops),
        }
    }

    /// Create a new WebSocket client with session registry
    pub fn with_registry(
        config: Arc<Config>,
        host_id: Uuid,
        token: String,
        registry: Arc<SessionRegistry>,
        event_tx: broadcast::Sender<DriverEvent>,
    ) -> Self {
        // Initialize FileOps with empty workspace roots (will be updated when sessions are created)
        let file_ops = FileOps::new(
            vec![],
            config.file_read_text_limit_bytes as usize,
            config.file_tree_max_nodes,
        );
        Self {
            config,
            host_id,
            token,
            registry: Some(registry),
            event_tx: Some(event_tx),
            ws_sender: None,
            file_ops: Some(file_ops),
        }
    }

    /// Get event sender for session runners
    pub fn event_sender(&self) -> Option<broadcast::Sender<DriverEvent>> {
        self.event_tx.clone()
    }

    /// Send an ack response to the server
    async fn send_ack(&self, request_id: &str, success: bool, error: Option<&str>) {
        if let Some(ref ws_sender) = self.ws_sender {
            let ack = AckPayload {
                request_id: request_id.to_string(),
                success,
                error: error.map(|s| s.to_string()),
            };
            let envelope = WsEnvelope::new("ack", &ack);
            if let Ok(json) = serde_json::to_string(&envelope) {
                let mut sender = ws_sender.lock().await;
                if let Err(e) = sender.send(WsMessage::Text(json.into())).await {
                    warn!(error = %e, "Failed to send ack");
                } else {
                    debug!(%request_id, success, "Sent ack");
                }
            }
        } else {
            warn!(%request_id, "Cannot send ack: ws_sender not initialized");
        }
    }

    /// Send an error response to the server
    async fn send_error(&self, request_id: &str, error: &AckError, error_message: &str) {
        if let Some(ref ws_sender) = self.ws_sender {
            let error_payload = ErrorPayload {
                request_id: request_id.to_string(),
                error_code: error.as_error_code().to_string(),
                error_message: error_message.to_string(),
            };
            let envelope = WsEnvelope::new("error", &error_payload);
            if let Ok(json) = serde_json::to_string(&envelope) {
                let mut sender = ws_sender.lock().await;
                if let Err(e) = sender.send(WsMessage::Text(json.into())).await {
                    warn!(error = %e, "Failed to send error response");
                } else {
                    debug!(%request_id, error_code = %error.as_error_code(), "Sent error response");
                }
            }
        } else {
            warn!(%request_id, "Cannot send error: ws_sender not initialized");
        }
    }

    /// Connect to server and run the main message loop
    ///
    /// Handles automatic reconnection with exponential backoff.
    /// Returns when shutdown signal is received or max retries exceeded.
    pub async fn run(mut self, mut shutdown_rx: broadcast::Receiver<()>) -> Result<()> {
        let mut retry_count = 0u32;
        let max_retries = 10;
        let min_backoff = self.config.reconnect_backoff_min();
        let max_backoff = self.config.reconnect_backoff_max();

        'connection_loop: loop {
            match self.connect_and_run().await {
                Ok(()) => {
                    info!("WebSocket connection closed normally");
                    break;
                }
                Err(DaemonError::ConnectionTimeout) | Err(DaemonError::WsDisconnected) => {
                    retry_count += 1;
                    if retry_count > max_retries {
                        error!(retry_count, "Max reconnection attempts reached");
                        return Err(DaemonError::ReconnectLimitExceeded {
                            attempts: max_retries,
                        });
                    }

                    let backoff = calculate_backoff(min_backoff, max_backoff, retry_count);
                    warn!(
                        retry_count,
                        backoff_ms = backoff.as_millis(),
                        "Connection lost, reconnecting..."
                    );

                    tokio::select! {
                        _ = tokio::time::sleep(backoff) => {}
                        _ = shutdown_rx.recv() => break 'connection_loop,
                    }
                }
                Err(DaemonError::TokenExpired) | Err(DaemonError::TokenInvalid { .. }) => {
                    error!("Token invalid or expired, need to re-pair");
                    return Err(DaemonError::TokenExpired);
                }
                Err(e) => {
                    error!(error = %e, "Unexpected error, reconnecting...");
                    retry_count += 1;
                    if retry_count > max_retries {
                        error!(retry_count, "Max reconnection attempts reached");
                        return Err(DaemonError::ReconnectLimitExceeded {
                            attempts: max_retries,
                        });
                    }
                    let backoff = calculate_backoff(min_backoff, max_backoff, retry_count);
                    tokio::select! {
                        _ = tokio::time::sleep(backoff) => {}
                        _ = shutdown_rx.recv() => break 'connection_loop,
                    }
                }
            }
        }

        Ok(())
    }

    /// Run a single connected session.
    ///
    /// On connection failure, restores `event_rx` so the caller can retry
    /// without losing the event channel.
    async fn connect_and_run(&mut self) -> Result<()> {
        // Build WebSocket URL, converting http(s):// to ws(s)://
        let ws_base = self
            .config
            .server_url
            .trim_end_matches('/')
            .replacen("http://", "ws://", 1)
            .replacen("https://", "wss://", 1);
        let ws_url = format!("{}/ws/daemon", ws_base);

        // Log server URL without exposing token
        let server_display = self.config.server_url.trim_end_matches('/');
        info!(server = %server_display, "Connecting to server...");

        // Build request with Authorization header (no token in URL)
        let mut request = ws_url
            .into_client_request()
            .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", self.token)
                .parse::<tokio_tungstenite::tungstenite::http::HeaderValue>()
                .expect("Bearer token should be a valid header value"),
        );

        // Connect WebSocket
        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;

        info!("WebSocket connected");

        // Subscribe a fresh broadcast receiver for this connection.
        // Because broadcast receivers are cloneable, each reconnect gets
        // a new receiver without consuming any persistent state.
        let mut event_rx = self
            .event_tx
            .as_ref()
            .map(|tx| tx.subscribe())
            .unwrap_or_else(|| {
                let (_tx, rx) = broadcast::channel(1);
                rx
            });

        let (sender, mut receiver) = ws_stream.split();
        let sender = Arc::new(tokio::sync::Mutex::new(sender));
        self.ws_sender = Some(sender.clone());

        let hello = DaemonToServer::DaemonHello {
            host_id: self.host_id,
            host_name: self.config.host_name.clone(),
            platform: self.config.platform.clone(),
        };
        let envelope = WsEnvelope::new("daemon_hello", &hello);
        let json = serde_json::to_string(&envelope).map_err(DaemonError::WsMessageParse)?;
        {
            let mut s = sender.lock().await;
            s.send(WsMessage::Text(json.into()))
                .await
                .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;
        }
        info!(host_id = %self.host_id, "Sent daemon_hello");

        let (heartbeat_tx, mut heartbeat_rx) = tokio::sync::mpsc::channel::<()>(1);
        let heartbeat_handle = tokio::spawn({
            let config = self.config.clone();
            async move {
                let mut interval = tokio::time::interval(config.heartbeat_interval());
                loop {
                    interval.tick().await;
                    if heartbeat_tx.send(()).await.is_err() {
                        break;
                    }
                }
            }
        });

        let mut last_pong = std::time::Instant::now();
        let heartbeat_timeout = self.config.heartbeat_timeout();

        loop {
            tokio::select! {
                // Shutdown signal — exit immediately so daemon reconnection test passes
                _ = shutdown_signal() => {
                    info!("Shutdown signal received during connection, closing WebSocket");
                    break;
                }

                // Heartbeat trigger
                _ = heartbeat_rx.recv() => {
                    let active_sessions = if let Some(ref registry) = self.registry {
                        registry.list_active_session_ids().await
                    } else {
                        vec![]
                    };
                    let heartbeat = DaemonToServer::DaemonHeartbeat {
                        host_id: self.host_id,
                        active_sessions,
                    };
                    let envelope = WsEnvelope::new("daemon_heartbeat", &heartbeat);
                    let json = serde_json::to_string(&envelope)
                        .map_err(DaemonError::WsMessageParse)?;
                    {
                        let mut s = sender.lock().await;
                        s.send(WsMessage::Text(json.into())).await
                            .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;
                    }
                    debug!("Sent heartbeat");
                }

                // Handle events from session runners
                event = event_rx.recv() => {
                    match event {
                        Ok(event) => {
                            // Handle permission registration for session-level approval cache
                            if let DriverEvent::PermissionRequest {
                                permission_id,
                                session_id,
                                risk_type,
                                summary,
                                target,
                            } = &event
                            {
                                if let Some(ref registry) = self.registry {
                                    if let Some(handle) = registry.get(session_id).await {
                                        let _ = handle.register_permission(
                                            *permission_id,
                                            risk_type.clone(),
                                            target.clone(),
                                            summary.clone(),
                                        ).await;
                                    }
                                }
                            }

                            // Handle ClaudeSessionId event for --resume support
                            if let DriverEvent::ClaudeSessionId {
                                session_id,
                                claude_session_id,
                            } = &event
                            {
                                if let Some(ref registry) = self.registry {
                                    if let Some(handle) = registry.get(session_id).await {
                                        let _ = handle.set_claude_session_id(claude_session_id.clone()).await;
                                    }
                                }
                            }

                            // Process event and send messages
                            let messages = self.handle_driver_event(event);
                            for (msg_type, payload) in messages {
                                let envelope = WsEnvelope::new(&msg_type, &payload);
                                if let Ok(json) = serde_json::to_string(&envelope) {
                                    let mut s = sender.lock().await;
                                    if s.send(WsMessage::Text(json.into())).await.is_err() {
                                        warn!("Failed to forward event to server");
                                    }
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            warn!(lagged = n, "Event receiver lagged, missed events");
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            warn!("Event channel closed");
                        }
                    }
                }

                // Receive message
                msg = receiver.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            match self.handle_message(&text).await {
                                Ok(()) => {}
                                Err(DaemonError::SessionArchived { .. }) => {
                                    // Ignore errors for archived sessions
                                }
                                Err(e) => {
                                    warn!(error = %e, "Failed to handle message");
                                }
                            }
                            last_pong = std::time::Instant::now();
                        }
                        Some(Ok(WsMessage::Ping(data))) => {
                            let mut s = sender.lock().await;
                            s.send(WsMessage::Pong(data)).await
                                .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            info!("Server closed connection");
                            break;
                        }
                        Some(Ok(WsMessage::Pong(_))) => {
                            last_pong = std::time::Instant::now();
                        }
                        Some(Err(e)) => {
                            error!(error = %e, "WebSocket error");
                            break;
                        }
                        None => {
                            info!("WebSocket stream ended");
                            break;
                        }
                        _ => {}
                    }
                }

                // Heartbeat timeout check (every 10 seconds)
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    if last_pong.elapsed() > heartbeat_timeout {
                        warn!(
                            timeout_seconds = heartbeat_timeout.as_secs(),
                            "Heartbeat timeout"
                        );
                        heartbeat_handle.abort();
                        return Err(DaemonError::HeartbeatTimeout {
                            timeout_seconds: heartbeat_timeout.as_secs() as u32,
                        });
                    }
                }
            }
        }

        heartbeat_handle.abort();
        Ok(())
    }

    /// Handle driver event from session runners and return messages to send
    fn handle_driver_event(&self, event: DriverEvent) -> Vec<(String, serde_json::Value)> {
        let mut messages = Vec::new();

        match &event {
            DriverEvent::PermissionRequest {
                permission_id,
                session_id,
                risk_type,
                summary,
                target,
            } => {
                messages.push((
                    "permission_request".to_string(),
                    serde_json::json!({
                        "permission_id": permission_id,
                        "session_id": session_id,
                        "risk_type": risk_type,
                        "summary": summary,
                        "target": target,
                    }),
                ));
            }
            DriverEvent::SessionEvent {
                session_id,
                event_type,
                data,
            } => {
                messages.push((
                    "session_event".to_string(),
                    serde_json::json!({
                        "session_id": session_id,
                        "event_type": event_type,
                        "data": data,
                    }),
                ));
            }
            DriverEvent::StatusUpdate {
                session_id,
                status,
                summary,
                close_reason,
            } => {
                messages.push((
                    "session_status_update".to_string(),
                    serde_json::json!({
                        "session_id": session_id,
                        "status": status,
                        "summary": summary,
                        "close_reason": close_reason,
                    }),
                ));

                if let Some(registry) = &self.registry {
                    if matches!(
                        status,
                        ve_shared::types::SessionStatus::Archived
                            | ve_shared::types::SessionStatus::Error
                    ) {
                        let session_id = *session_id;
                        let registry = registry.clone();
                        tokio::spawn(async move {
                            registry.remove(&session_id).await;
                        });
                    }
                }
            }
            DriverEvent::FatalError {
                session_id,
                message,
            } => {
                messages.push((
                    "session_event".to_string(),
                    serde_json::json!({
                        "session_id": session_id,
                        "event_type": "fatal_error",
                        "data": { "message": message },
                    }),
                ));
            }
            DriverEvent::ClaudeSessionId {
                session_id,
                claude_session_id,
            } => {
                messages.push((
                    "session_event".to_string(),
                    serde_json::json!({
                        "session_id": session_id,
                        "event_type": "claude_session_id",
                        "data": { "claude_session_id": claude_session_id },
                    }),
                ));
            }
        }

        messages
    }

    /// Handle received message
    async fn handle_message(&self, text: &str) -> Result<()> {
        let envelope: WsEnvelope =
            serde_json::from_str(text).map_err(DaemonError::WsMessageParse)?;

        debug!(type = %envelope.r#type, "Received message");

        match envelope.r#type.as_str() {
            "create_session" => {
                self.handle_create_session(&envelope).await?;
            }
            "ensure_workspace" => {
                self.handle_ensure_workspace(&envelope).await?;
            }
            "rerun_session" => {
                self.handle_rerun_session(&envelope).await?;
            }
            "send_message" => {
                self.handle_send_message(&envelope).await?;
            }
            "session_control" => {
                self.handle_session_control(&envelope).await?;
            }
            "close_session" => {
                self.handle_close_session(&envelope).await?;
            }
            "permission_response" => {
                self.handle_permission_response(&envelope).await?;
            }
            "file_tree_request" => {
                self.handle_file_tree_request(&envelope).await?;
            }
            "file_content_request" => {
                self.handle_file_content_request(&envelope).await?;
            }
            "paired" => {
                self.handle_paired(&envelope).await?;
            }
            "pong" => {
                debug!("Received pong");
            }
            _ => {
                warn!(type = %envelope.r#type, "Unknown message type");
            }
        }

        Ok(())
    }

    // Message handlers

    async fn handle_ensure_workspace(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = envelope
            .request_id
            .clone()
            .ok_or(DaemonError::RequestIdMissing)?;

        let workspace_path = envelope
            .payload
            .get("workspace_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("workspace_path".to_string()))?;

        match ensure_workspace_directory(workspace_path) {
            Ok(()) => {
                debug!(workspace_path, "Workspace prepared successfully");
                self.send_ack(&request_id, true, None).await;
            }
            Err(error) => {
                warn!(workspace_path, error = %error, "Failed to prepare workspace");
                self.send_error(&request_id, &error.to_ack_error(), &error.to_string())
                    .await;
            }
        }

        Ok(())
    }

    async fn handle_create_session(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = envelope
            .request_id
            .clone()
            .ok_or(DaemonError::RequestIdMissing)?;

        let session_id = envelope
            .payload
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let workspace_path = envelope
            .payload
            .get("workspace_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("workspace_path".to_string()))?
            .to_string();

        let agent_type = envelope
            .payload
            .get("agent_type")
            .and_then(|v| v.as_str())
            .unwrap_or("claude_code")
            .to_string();

        let initial_message = envelope
            .payload
            .get("initial_message")
            .and_then(|v| v.as_str())
            .map(ToString::to_string);

        // Check if registry is available
        let registry = match &self.registry {
            Some(r) => r,
            None => {
                warn!("Received create_session but registry not configured");
                self.send_error(
                    &request_id,
                    &AckError::InternalError,
                    "Registry not configured",
                )
                .await;
                return Ok(());
            }
        };

        // Create session
        match registry
            .create(session_id, workspace_path, agent_type, initial_message)
            .await
        {
            Ok(_) => {
                info!(%session_id, "Session created successfully");
                self.send_ack(&request_id, true, None).await;
            }
            Err(e) => {
                warn!(%session_id, error = %e, "Failed to create session");
                let ack_error = e.to_ack_error();
                self.send_error(&request_id, &ack_error, &e.to_string())
                    .await;
            }
        }

        Ok(())
    }

    async fn handle_rerun_session(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = envelope
            .request_id
            .clone()
            .ok_or(DaemonError::RequestIdMissing)?;

        let session_id = envelope
            .payload
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let workspace_path = envelope
            .payload
            .get("workspace_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("workspace_path".to_string()))?
            .to_string();

        let agent_type = envelope
            .payload
            .get("agent_type")
            .and_then(|v| v.as_str())
            .unwrap_or("claude_code")
            .to_string();

        let claude_session_id = envelope
            .payload
            .get("claude_session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("claude_session_id".to_string()))?
            .to_string();

        let registry = match &self.registry {
            Some(r) => r,
            None => {
                warn!("Received rerun_session but registry not configured");
                self.send_error(
                    &request_id,
                    &AckError::InternalError,
                    "Registry not configured",
                )
                .await;
                return Ok(());
            }
        };

        match registry
            .create_rerun(session_id, workspace_path, agent_type, claude_session_id)
            .await
        {
            Ok(_) => {
                info!(%session_id, "Session rerun created successfully");
                self.send_ack(&request_id, true, None).await;
            }
            Err(e) => {
                warn!(%session_id, error = %e, "Failed to create rerun session");
                let ack_error = e.to_ack_error();
                self.send_error(&request_id, &ack_error, &e.to_string())
                    .await;
            }
        }

        Ok(())
    }

    async fn handle_send_message(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = match &envelope.request_id {
            Some(id) => id.clone(),
            None => {
                warn!("send_message missing request_id");
                return Ok(());
            }
        };

        let session_id = envelope
            .payload
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let content = envelope
            .payload
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("content".to_string()))?
            .to_string();

        let registry = match &self.registry {
            Some(r) => r,
            None => {
                self.send_error(
                    &request_id,
                    &AckError::InternalError,
                    "Registry not configured",
                )
                .await;
                return Ok(());
            }
        };

        if let Some(handle) = registry.get(&session_id).await {
            match handle
                .send_message_and_wait(content, self.config.ack_timeout())
                .await
            {
                Ok(()) => {
                    debug!(%session_id, "Message sent to session");
                    self.send_ack(&request_id, true, None).await;
                }
                Err(e) => {
                    warn!(%session_id, error = %e, "Failed to send message");
                    self.send_error(&request_id, &e.to_ack_error(), &e.to_string())
                        .await;
                }
            }
        } else {
            warn!(%session_id, "Session not found for send_message");
            self.send_error(
                &request_id,
                &AckError::SessionNotFound,
                &format!("Session {} not found", session_id),
            )
            .await;
        }

        Ok(())
    }

    async fn handle_session_control(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = match &envelope.request_id {
            Some(id) => id.clone(),
            None => {
                warn!("session_control missing request_id");
                return Ok(());
            }
        };

        let session_id = envelope
            .payload
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let action_str = envelope
            .payload
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("action".to_string()))?;

        let action = match action_str {
            "pause" => SessionControlAction::Pause,
            "terminate" => SessionControlAction::Terminate,
            "interrupt" => {
                self.send_error(
                    &request_id,
                    &AckError::SessionInvalidState,
                    "Interrupt is not supported safely for Claude Code sessions",
                )
                .await;
                return Ok(());
            }
            "rerun" => SessionControlAction::Rerun,
            "restart" => SessionControlAction::Restart,
            _ => {
                warn!(action = %action_str, "Unknown session control action");
                self.send_error(
                    &request_id,
                    &AckError::InternalError,
                    &format!("Unknown action: {}", action_str),
                )
                .await;
                return Ok(());
            }
        };

        let registry = match &self.registry {
            Some(r) => r,
            None => {
                self.send_error(
                    &request_id,
                    &AckError::InternalError,
                    "Registry not configured",
                )
                .await;
                return Ok(());
            }
        };

        if let Some(handle) = registry.get(&session_id).await {
            match handle
                .send_control_and_wait(action, self.config.ack_timeout())
                .await
            {
                Ok(()) => {
                    debug!(%session_id, ?action, "Control sent to session");
                    self.send_ack(&request_id, true, None).await;
                }
                Err(e) => {
                    warn!(%session_id, error = %e, "Failed to send control");
                    self.send_error(&request_id, &e.to_ack_error(), &e.to_string())
                        .await;
                }
            }
        } else {
            warn!(%session_id, "Session not found for session_control");
            self.send_error(
                &request_id,
                &AckError::SessionNotFound,
                &format!("Session {} not found", session_id),
            )
            .await;
        }

        Ok(())
    }

    async fn handle_close_session(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = match &envelope.request_id {
            Some(id) => id.clone(),
            None => {
                warn!("close_session missing request_id");
                return Ok(());
            }
        };

        let session_id = envelope
            .payload
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let registry = match &self.registry {
            Some(r) => r,
            None => {
                self.send_error(
                    &request_id,
                    &AckError::InternalError,
                    "Registry not configured",
                )
                .await;
                return Ok(());
            }
        };

        // 使用原子性的 close_and_remove 方法
        match registry.close_and_remove(&session_id).await {
            Ok(()) => {
                info!(%session_id, "Session closed");
                self.send_ack(&request_id, true, None).await;
            }
            Err(DaemonError::SessionNotFound { .. }) => {
                warn!(%session_id, "Session not found for close_session");
                self.send_error(
                    &request_id,
                    &AckError::SessionNotFound,
                    &format!("Session {} not found", session_id),
                )
                .await;
            }
            Err(e) => {
                warn!(%session_id, error = %e, "Failed to close session");
                self.send_error(&request_id, &e.to_ack_error(), &e.to_string())
                    .await;
            }
        }

        Ok(())
    }

    async fn handle_permission_response(&self, envelope: &WsEnvelope) -> Result<()> {
        let permission_id = envelope
            .payload
            .get("permission_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("permission_id".to_string()))?;
        let permission_id = Uuid::parse_str(permission_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid permission_id".to_string()))?;

        let session_id = envelope
            .payload
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let decision_str = envelope
            .payload
            .get("decision")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("decision".to_string()))?;

        let decision = match decision_str {
            "approve_once" => PermissionDecision::ApproveOnce,
            "deny_once" => PermissionDecision::DenyOnce,
            "approve_session" => PermissionDecision::ApproveSession,
            _ => {
                warn!(decision = %decision_str, "Unknown permission decision");
                return Ok(());
            }
        };

        // Get session handle and send permission response
        if let Some(ref registry) = self.registry {
            if let Some(handle) = registry.get(&session_id).await {
                handle
                    .send_permission_response(permission_id, decision)
                    .await?;
                debug!(%session_id, %permission_id, ?decision, "Permission response sent to session");
            } else {
                warn!(%session_id, "Session not found for permission_response");
            }
        }

        Ok(())
    }

    async fn handle_file_tree_request(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = match &envelope.request_id {
            Some(id) => id.clone(),
            None => {
                warn!("file_tree_request missing request_id");
                return Ok(());
            }
        };

        let workspace_path = envelope
            .payload
            .get("workspace_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let relative_path = envelope
            .payload
            .get("relative_path")
            .and_then(|v| v.as_str());

        if workspace_path.is_empty() {
            self.send_error(
                &request_id,
                &AckError::WorkspaceInvalid,
                "workspace_path is required",
            )
            .await;
            return Ok(());
        }

        let workspace_root = PathBuf::from(workspace_path);
        if !workspace_root.exists() {
            self.send_error(
                &request_id,
                &AckError::WorkspaceInvalid,
                &format!("Workspace path does not exist: {workspace_path}"),
            )
            .await;
            return Ok(());
        }

        let file_ops = FileOps::new(
            vec![workspace_root.clone()],
            self.config.file_read_text_limit_bytes as usize,
            self.config.file_tree_max_nodes,
        );
        let start_path = match relative_path {
            Some(path) if !path.is_empty() => workspace_root.join(path),
            _ => workspace_root.clone(),
        };

        match file_ops.collect_tree(&start_path, 10) {
            Ok(tree) => {
                let tree_json = serde_json::to_value(&tree).unwrap_or(serde_json::Value::Null);
                let response = DaemonToServer::FileTreeResponse {
                    request_id,
                    session_id: Uuid::nil(),
                    tree: tree_json,
                };
                let envelope = WsEnvelope::new("file_tree_response", &response);
                if let Ok(json) = serde_json::to_string(&envelope) {
                    if let Some(ref ws_sender) = self.ws_sender {
                        let mut sender = ws_sender.lock().await;
                        if let Err(e) = sender.send(WsMessage::Text(json.into())).await {
                            warn!(error = %e, "Failed to send file tree response");
                        }
                    }
                }
                debug!("Sent file tree response");
            }
            Err(e) => {
                warn!(error = %e, "Failed to collect file tree");
                self.send_error(&request_id, &e.to_ack_error(), &e.to_string())
                    .await;
            }
        }

        Ok(())
    }

    async fn handle_file_content_request(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = match &envelope.request_id {
            Some(id) => id.clone(),
            None => {
                warn!("file_content_request missing request_id");
                return Ok(());
            }
        };

        let workspace_path = envelope
            .payload
            .get("workspace_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let relative_path = envelope
            .payload
            .get("relative_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if workspace_path.is_empty() {
            self.send_error(
                &request_id,
                &AckError::WorkspaceInvalid,
                "workspace_path is required",
            )
            .await;
            return Ok(());
        }

        if relative_path.is_empty() {
            self.send_error(
                &request_id,
                &AckError::InternalError,
                "relative_path is required",
            )
            .await;
            return Ok(());
        }

        let workspace_root = PathBuf::from(workspace_path);
        if !workspace_root.exists() {
            self.send_error(
                &request_id,
                &AckError::WorkspaceInvalid,
                &format!("Workspace path does not exist: {workspace_path}"),
            )
            .await;
            return Ok(());
        }

        let file_ops = FileOps::new(
            vec![workspace_root.clone()],
            self.config.file_read_text_limit_bytes as usize,
            self.config.file_tree_max_nodes,
        );
        let path = workspace_root.join(relative_path);

        match file_ops.read_text_file(&path) {
            Ok(content) => {
                let response = DaemonToServer::FileContentResponse {
                    request_id,
                    file_path: relative_path.to_string(),
                    content: content.content,
                    file_type: format!("{:?}", content.file_type).to_lowercase(),
                    truncated: content.truncated,
                    total_size: content.total_size,
                };
                let envelope = WsEnvelope::new("file_content_response", &response);
                if let Ok(json) = serde_json::to_string(&envelope) {
                    if let Some(ref ws_sender) = self.ws_sender {
                        let mut sender = ws_sender.lock().await;
                        if let Err(e) = sender.send(WsMessage::Text(json.into())).await {
                            warn!(error = %e, "Failed to send file content response");
                        }
                    }
                }
                debug!("Sent file content response");
            }
            Err(e) => {
                warn!(error = %e, "Failed to read file content");
                self.send_error(&request_id, &e.to_ack_error(), &e.to_string())
                    .await;
            }
        }

        Ok(())
    }

    async fn handle_paired(&self, _envelope: &WsEnvelope) -> Result<()> {
        info!("Received paired notification");
        Ok(())
    }
}

fn ensure_workspace_directory(workspace_path: &str) -> Result<()> {
    let trimmed_path = workspace_path.trim();
    if trimmed_path.is_empty() {
        return Err(DaemonError::WorkspaceInvalid {
            path: workspace_path.to_string(),
        });
    }

    let path = Path::new(trimmed_path);
    if !path.is_absolute() {
        return Err(DaemonError::WorkspaceInvalid {
            path: trimmed_path.to_string(),
        });
    }

    if path.exists() {
        return if path.is_dir() {
            Ok(())
        } else {
            Err(DaemonError::WorkspaceInvalid {
                path: trimmed_path.to_string(),
            })
        };
    }

    std::fs::create_dir_all(path).map_err(|_| DaemonError::WorkspaceInvalid {
        path: trimmed_path.to_string(),
    })?;

    if path.is_dir() {
        Ok(())
    } else {
        Err(DaemonError::WorkspaceInvalid {
            path: trimmed_path.to_string(),
        })
    }
}

/// Calculate exponential backoff duration
///
/// Uses exponential growth with random jitter (±20%).
fn calculate_backoff(min: Duration, max: Duration, retry_count: u32) -> Duration {
    let base = min.as_millis() as f64;
    let multiplier = 2_f64.powi(retry_count as i32 - 1);
    let backoff = base * multiplier;
    let backoff = backoff.min(max.as_millis() as f64);

    // Add random jitter (±20%)
    let jitter = backoff * 0.2 * (rand::random::<f64>() - 0.5) * 2.0;
    Duration::from_millis((backoff + jitter) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_calculate_backoff_first_retry() {
        let min = Duration::from_millis(1000);
        let max = Duration::from_millis(30000);

        // First retry should be close to min
        let backoff = calculate_backoff(min, max, 1);
        assert!(backoff >= Duration::from_millis(800)); // Allow jitter
        assert!(backoff <= Duration::from_millis(1200));
    }

    #[test]
    fn test_calculate_backoff_caps_at_max() {
        let min = Duration::from_millis(1000);
        let max = Duration::from_millis(5000);

        // Even with high retry count, should be capped
        let backoff = calculate_backoff(min, max, 10);
        assert!(backoff <= Duration::from_millis(6000)); // max + jitter
    }

    #[test]
    fn test_calculate_backoff_exponential_growth() {
        let min = Duration::from_millis(1000);
        let max = Duration::from_millis(30000);

        // Should grow exponentially
        let backoff1 = calculate_backoff(min, max, 1);
        let _backoff2 = calculate_backoff(min, max, 2);
        let backoff3 = calculate_backoff(min, max, 3);

        // Allow for jitter, but trend should be increasing
        // (With jitter, individual values might not always increase,
        // but the base values do: 1s, 2s, 4s)
        assert!(backoff1 < backoff3);
    }

    #[test]
    fn handle_driver_event_emits_dedicated_permission_request_with_same_id() {
        let client = WsClient::new(
            Arc::new(crate::config::Config {
                server_url: "https://example.com".to_string(),
                host_name: "host".to_string(),
                platform: "linux".to_string(),
                config_dir: std::path::PathBuf::from("/tmp"),
                log_format: "pretty".to_string(),
                log_level: "info".to_string(),
                heartbeat_interval_secs: 30,
                heartbeat_timeout_secs: 90,
                ack_timeout_secs: 30,
                permission_timeout_secs: 60,
                reconnect_backoff_min_ms: 1000,
                reconnect_backoff_max_ms: 30000,
                max_parallel_sessions: 4,
                file_read_text_limit_bytes: 262_144,
                file_tree_max_nodes: 20_000,
                claude_command: "claude".to_string(),
                default_model: "claude-sonnet-4-20250514".to_string(),
                mock_mode: false,
            }),
            Uuid::new_v4(),
            "token".to_string(),
        );
        let permission_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        let messages = client.handle_driver_event(DriverEvent::PermissionRequest {
            permission_id,
            session_id,
            risk_type: "write_fs".to_string(),
            summary: "need access".to_string(),
            target: Some("/tmp".to_string()),
        });

        assert_eq!(messages.len(), 1);
        let (msg_type, payload) = &messages[0];
        assert_eq!(msg_type, "permission_request");
        assert_eq!(payload["permission_id"], permission_id.to_string());
        assert_eq!(payload["session_id"], session_id.to_string());
    }

    #[test]
    fn ensure_workspace_directory_creates_missing_absolute_path() {
        let dir = tempdir().unwrap();
        let workspace = dir.path().join("new-workspace");

        ensure_workspace_directory(workspace.to_str().unwrap()).unwrap();

        assert!(workspace.exists());
        assert!(workspace.is_dir());
    }

    #[test]
    fn ensure_workspace_directory_rejects_relative_path() {
        let error = ensure_workspace_directory("relative/workspace").unwrap_err();

        assert!(matches!(error, DaemonError::WorkspaceInvalid { .. }));
    }

    #[test]
    fn ensure_workspace_directory_rejects_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("not-a-directory");
        std::fs::write(&file_path, "hello").unwrap();

        let error = ensure_workspace_directory(file_path.to_str().unwrap()).unwrap_err();

        assert!(matches!(error, DaemonError::WorkspaceInvalid { .. }));
    }
}
