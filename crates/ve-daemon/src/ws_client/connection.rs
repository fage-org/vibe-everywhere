//! WebSocket connection and reconnection logic.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message as WsMessage},
};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use super::utils::calculate_backoff;
use super::{shutdown_signal, DaemonError, DriverEvent, Result, WsClient, WsEnvelope};
use ve_shared::proto::DaemonToServer;

impl WsClient {
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
        self.ensure_permission_bridge_dirs().await?;

        let ws_base = self
            .config
            .server_url
            .trim_end_matches('/')
            .replacen("http://", "ws://", 1)
            .replacen("https://", "wss://", 1);
        let ws_url = format!("{}/ws/daemon", ws_base);

        let server_display = self.config.server_url.trim_end_matches('/');
        info!(server = %server_display, "Connecting to server...");

        let mut request = ws_url
            .into_client_request()
            .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;
        let auth_value = format!("Bearer {}", self.token)
            .parse::<tokio_tungstenite::tungstenite::http::HeaderValue>()
            .map_err(|e| {
                DaemonError::TokenInvalid {
                    reason: format!("invalid header value: {e}"),
                }
            })?;
        request.headers_mut().insert("Authorization", auth_value);

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;

        info!("WebSocket connected");

        // Take the event receiver. It is consumed directly by this connection.
        // On reconnect failure, it is NOT restored (the channel is single-consumer).
        let mut event_rx = self.event_rx.take();

        let (sender, mut receiver) = ws_stream.split();
        let sender = std::sync::Arc::new(tokio::sync::Mutex::new(sender));
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

        // Send active session list for reconciliation after (re)connection
        let active_sessions = if let Some(ref registry) = self.registry {
            registry.list_active_session_ids().await
        } else {
            vec![]
        };
        let sync = DaemonToServer::SyncSessions {
            host_id: self.host_id,
            active_sessions: active_sessions.clone(),
        };
        let envelope = WsEnvelope::new("sync_sessions", &sync);
        let json = serde_json::to_string(&envelope).map_err(DaemonError::WsMessageParse)?;
        {
            let mut s = sender.lock().await;
            s.send(WsMessage::Text(json.into()))
                .await
                .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;
        }
        info!(host_id = %self.host_id, count = active_sessions.len(), "Sent sync_sessions");

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
        let mut bridge_tick = tokio::time::interval(Duration::from_millis(250));

        loop {
            tokio::select! {
                _ = shutdown_signal() => {
                    info!("Shutdown signal received during connection, closing WebSocket");
                    break;
                }

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

                _ = bridge_tick.tick() => {
                    if let Err(error) = self.process_permission_bridge_requests().await {
                        warn!(error = %error, "Failed to process permission bridge requests");
                    }
                }

                event = event_rx.as_mut().unwrap().recv() => {
                    match event {
                        Some(event) => {
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
                        None => {
                            warn!("Event channel closed — no more driver events will be forwarded");
                        }
                    }
                }

                msg = receiver.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            match self.handle_message(&text).await {
                                Ok(()) => {}
                                Err(DaemonError::SessionArchived { .. }) => {}
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
}
