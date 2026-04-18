//! WebSocket Client Module
//!
//! Manages the persistent WebSocket connection between ve-daemon and ve-server.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message as WsMessage,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::{AckPayload, DaemonToServer, ErrorPayload, SessionControlAction, WsEnvelope};

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
    /// Event receiver from session runners
    event_rx: Option<mpsc::Receiver<DriverEvent>>,
    /// Event sender (clone for session runners)
    event_tx: Option<mpsc::Sender<DriverEvent>>,
    /// WebSocket sender for sending acks (wrapped in Arc for sharing)
    ws_sender: Option<Arc<tokio::sync::Mutex<WsSender>>>,
    /// File operations handler (workspace roots collected from sessions)
    file_ops: Option<FileOps>,
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
            event_rx: None,
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
    ) -> Self {
        let (event_tx, event_rx) = mpsc::channel(64);
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
            event_rx: Some(event_rx),
            event_tx: Some(event_tx),
            ws_sender: None,
            file_ops: Some(file_ops),
        }
    }

    /// Get event sender for session runners
    pub fn event_sender(&self) -> Option<mpsc::Sender<DriverEvent>> {
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
    pub async fn run(
        mut self,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
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
                Err(DaemonError::ConnectionTimeout)
                | Err(DaemonError::WsDisconnected) => {
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
                }
            }
        }

        Ok(())
    }

    /// Establish connection and run message loop
    async fn connect_and_run(&mut self) -> Result<()> {
        // Build WebSocket URL with token
        let ws_url = format!(
            "{}/ws/daemon?token={}",
            self.config.server_url.trim_end_matches('/'),
            self.token
        );

        // Log server URL without exposing token
        let server_display = self.config.server_url.trim_end_matches('/');
        info!(server = %server_display, "Connecting to server...");

        // Connect WebSocket
        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;

        info!("WebSocket connected");

        let (sender, mut receiver) = ws_stream.split();

        // Wrap sender in Arc<Mutex> for sharing between main loop and handlers
        let sender = Arc::new(tokio::sync::Mutex::new(sender));
        self.ws_sender = Some(sender.clone());

        // Send daemon_hello
        let hello = DaemonToServer::DaemonHello {
            host_id: self.host_id,
            host_name: self.config.host_name.clone(),
            platform: self.config.platform.clone(),
        };
        let envelope = WsEnvelope::new("daemon_hello", &hello);
        let json = serde_json::to_string(&envelope)
            .map_err(DaemonError::WsMessageParse)?;
        {
            let mut s = sender.lock().await;
            s.send(WsMessage::Text(json.into())).await
                .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;
        }

        info!(host_id = %self.host_id, "Sent daemon_hello");

        // Start heartbeat task
        let (heartbeat_tx, mut heartbeat_rx) = tokio::sync::mpsc::channel::<()>(1);
        let heartbeat_handle = tokio::spawn({
            let config = self.config.clone();
            async move {
                let mut interval =
                    tokio::time::interval(config.heartbeat_interval());
                loop {
                    interval.tick().await;
                    if heartbeat_tx.send(()).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Main message loop
        let mut last_pong = std::time::Instant::now();
        let heartbeat_timeout = self.config.heartbeat_timeout();

        // Take event_rx out of Option for the duration of this connection
        let mut event_rx = self.event_rx.take();

        loop {
            tokio::select! {
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
                event = async {
                    if let Some(ref mut rx) = event_rx {
                        rx.recv().await
                    } else {
                        std::future::pending().await
                    }
                } => {
                    if let Some(event) = event {
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
    fn handle_driver_event(
        &self,
        event: DriverEvent,
    ) -> Vec<(String, serde_json::Value)> {
        let mut messages = Vec::new();

        match &event {
            DriverEvent::PermissionRequest {
                permission_id: _,
                session_id,
                risk_type: _,
                summary: _,
                target: _,
            } => {
                // Permission registration happens in the async context of the caller
                // (see the select! branch where this is called)

                // Prepare message to forward to server
                let event_json = serde_json::to_value(&event).unwrap_or(serde_json::json!({}));
                messages.push(("session_event".to_string(), serde_json::json!({
                    "session_id": session_id,
                    "event_type": "permission_request",
                    "data": event_json,
                })));
            }
            DriverEvent::SessionEvent { session_id, event_type, data } => {
                messages.push(("session_event".to_string(), serde_json::json!({
                    "session_id": session_id,
                    "event_type": event_type,
                    "data": data,
                })));
            }
            DriverEvent::StatusUpdate { session_id, status, summary } => {
                messages.push(("session_status_update".to_string(), serde_json::json!({
                    "session_id": session_id,
                    "status": status,
                    "summary": summary,
                })));
            }
            DriverEvent::FatalError { session_id, message } => {
                messages.push(("session_event".to_string(), serde_json::json!({
                    "session_id": session_id,
                    "event_type": "fatal_error",
                    "data": { "message": message },
                })));
            }
            DriverEvent::ClaudeSessionId { session_id, claude_session_id } => {
                messages.push(("session_event".to_string(), serde_json::json!({
                    "session_id": session_id,
                    "event_type": "claude_session_id",
                    "data": { "claude_session_id": claude_session_id },
                })));
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

    async fn handle_create_session(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = envelope.request_id.clone()
            .ok_or(DaemonError::RequestIdMissing)?;

        let session_id = envelope.payload.get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let workspace_path = envelope.payload.get("workspace_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("workspace_path".to_string()))?
            .to_string();

        let agent_type = envelope.payload.get("agent_type")
            .and_then(|v| v.as_str())
            .unwrap_or("claude_code")
            .to_string();

        // Check if registry is available
        let registry = match &self.registry {
            Some(r) => r,
            None => {
                warn!("Received create_session but registry not configured");
                self.send_error(&request_id, &AckError::InternalError, "Registry not configured").await;
                return Ok(());
            }
        };

        // Create session
        match registry.create(session_id, workspace_path, agent_type).await {
            Ok(_) => {
                info!(%session_id, "Session created successfully");
                self.send_ack(&request_id, true, None).await;
            }
            Err(e) => {
                warn!(%session_id, error = %e, "Failed to create session");
                let ack_error = e.to_ack_error();
                self.send_error(&request_id, &ack_error, &e.to_string()).await;
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

        let session_id = envelope.payload.get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let content = envelope.payload.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("content".to_string()))?
            .to_string();

        let registry = match &self.registry {
            Some(r) => r,
            None => {
                self.send_error(&request_id, &AckError::InternalError, "Registry not configured").await;
                return Ok(());
            }
        };

        if let Some(handle) = registry.get(&session_id).await {
            match handle.send_message(content).await {
                Ok(()) => {
                    debug!(%session_id, "Message sent to session");
                    self.send_ack(&request_id, true, None).await;
                }
                Err(e) => {
                    warn!(%session_id, error = %e, "Failed to send message");
                    self.send_error(&request_id, &e.to_ack_error(), &e.to_string()).await;
                }
            }
        } else {
            warn!(%session_id, "Session not found for send_message");
            self.send_error(&request_id, &AckError::SessionNotFound, &format!("Session {} not found", session_id)).await;
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

        let session_id = envelope.payload.get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let action_str = envelope.payload.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("action".to_string()))?;

        let action = match action_str {
            "pause" => SessionControlAction::Pause,
            "terminate" => SessionControlAction::Terminate,
            "interrupt" => SessionControlAction::Interrupt,
            "rerun" => SessionControlAction::Rerun,
            _ => {
                warn!(action = %action_str, "Unknown session control action");
                self.send_error(&request_id, &AckError::InternalError, &format!("Unknown action: {}", action_str)).await;
                return Ok(());
            }
        };

        let registry = match &self.registry {
            Some(r) => r,
            None => {
                self.send_error(&request_id, &AckError::InternalError, "Registry not configured").await;
                return Ok(());
            }
        };

        if let Some(handle) = registry.get(&session_id).await {
            match handle.send_control(action).await {
                Ok(()) => {
                    debug!(%session_id, ?action, "Control sent to session");
                    self.send_ack(&request_id, true, None).await;
                }
                Err(e) => {
                    warn!(%session_id, error = %e, "Failed to send control");
                    self.send_error(&request_id, &e.to_ack_error(), &e.to_string()).await;
                }
            }
        } else {
            warn!(%session_id, "Session not found for session_control");
            self.send_error(&request_id, &AckError::SessionNotFound, &format!("Session {} not found", session_id)).await;
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

        let session_id = envelope.payload.get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let registry = match &self.registry {
            Some(r) => r,
            None => {
                self.send_error(&request_id, &AckError::InternalError, "Registry not configured").await;
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
                self.send_error(&request_id, &AckError::SessionNotFound, &format!("Session {} not found", session_id)).await;
            }
            Err(e) => {
                warn!(%session_id, error = %e, "Failed to close session");
                self.send_error(&request_id, &e.to_ack_error(), &e.to_string()).await;
            }
        }

        Ok(())
    }

    async fn handle_permission_response(&self, envelope: &WsEnvelope) -> Result<()> {
        let permission_id = envelope.payload.get("permission_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("permission_id".to_string()))?;
        let permission_id = Uuid::parse_str(permission_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid permission_id".to_string()))?;

        let session_id = envelope.payload.get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("session_id".to_string()))?;
        let session_id = Uuid::parse_str(session_id)
            .map_err(|_| DaemonError::WsPayloadMissing("invalid session_id".to_string()))?;

        let decision_str = envelope.payload.get("decision")
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
                handle.send_permission_response(permission_id, decision).await?;
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

        let workspace_path = envelope.payload.get("workspace_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let file_ops = match &self.file_ops {
            Some(ops) => ops,
            None => {
                self.send_error(&request_id, &AckError::InternalError, "FileOps not configured").await;
                return Ok(());
            }
        };

        // If workspace_path is provided, update FileOps with this root
        let effective_ops = if !workspace_path.is_empty() {
            let path = PathBuf::from(workspace_path);
            if path.exists() {
                FileOps::new(
                    vec![path],
                    self.config.file_read_text_limit_bytes as usize,
                    self.config.file_tree_max_nodes,
                )
            } else {
                self.send_error(&request_id, &AckError::WorkspaceInvalid, &format!("Workspace path does not exist: {}", workspace_path)).await;
                return Ok(());
            }
        } else {
            file_ops.clone()
        };

        let start_path = if workspace_path.is_empty() {
            PathBuf::from(".")
        } else {
            PathBuf::from(workspace_path)
        };

        match effective_ops.collect_tree(&start_path, 10) {
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
                self.send_error(&request_id, &e.to_ack_error(), &e.to_string()).await;
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

        let file_path = envelope.payload.get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if file_path.is_empty() {
            self.send_error(&request_id, &AckError::InternalError, "file_path is required").await;
            return Ok(());
        }

        // Extract workspace from file path (parent directory or use configured roots)
        let path = PathBuf::from(file_path);
        let workspace_root = path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        let file_ops = FileOps::new(
            vec![workspace_root],
            self.config.file_read_text_limit_bytes as usize,
            self.config.file_tree_max_nodes,
        );

        match file_ops.read_text_file(&path) {
            Ok(content) => {
                let response = DaemonToServer::FileContentResponse {
                    request_id,
                    file_path: file_path.to_string(),
                    content: content.content,
                    file_type: format!("{:?}", content.file_type).to_lowercase(),
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
                self.send_error(&request_id, &e.to_ack_error(), &e.to_string()).await;
            }
        }

        Ok(())
    }

    async fn handle_paired(&self, _envelope: &WsEnvelope) -> Result<()> {
        info!("Received paired notification");
        Ok(())
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
}
