//! WebSocket Client Module
//!
//! Manages the persistent WebSocket connection between ve-daemon and ve-server.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{
    connect_async,
    tungstenite::Message as WsMessage,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use ve_shared::proto::{DaemonToServer, WsEnvelope};

use crate::config::Config;
use crate::error::DaemonError;
use crate::Result;

/// WebSocket client for daemon-server communication
pub struct WsClient {
    /// Configuration reference
    config: Arc<Config>,
    /// Host UUID
    host_id: Uuid,
    /// Authentication token
    token: String,
}

impl WsClient {
    /// Create a new WebSocket client
    pub fn new(config: Arc<Config>, host_id: Uuid, token: String) -> Self {
        Self {
            config,
            host_id,
            token,
        }
    }

    /// Connect to server and run the main message loop
    ///
    /// Handles automatic reconnection with exponential backoff.
    /// Returns when shutdown signal is received or max retries exceeded.
    pub async fn run(
        self,
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
    async fn connect_and_run(&self) -> Result<()> {
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

        let (mut sender, mut receiver) = ws_stream.split();

        // Send daemon_hello
        let hello = DaemonToServer::DaemonHello {
            host_id: self.host_id,
            host_name: self.config.host_name.clone(),
            platform: self.config.platform.clone(),
        };
        let envelope = WsEnvelope::new("daemon_hello", &hello);
        let json = serde_json::to_string(&envelope)
            .map_err(DaemonError::WsMessageParse)?;
        sender.send(WsMessage::Text(json.into())).await
            .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;

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

        loop {
            tokio::select! {
                // Heartbeat trigger
                _ = heartbeat_rx.recv() => {
                    let heartbeat = DaemonToServer::DaemonHeartbeat {
                        host_id: self.host_id,
                        active_sessions: vec![], // TODO: Get from session registry
                    };
                    let envelope = WsEnvelope::new("daemon_heartbeat", &heartbeat);
                    let json = serde_json::to_string(&envelope)
                        .map_err(DaemonError::WsMessageParse)?;
                    sender.send(WsMessage::Text(json.into())).await
                        .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;
                    debug!("Sent heartbeat");
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
                            sender.send(WsMessage::Pong(data)).await
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

    // Message handlers (to be implemented in next phase)

    async fn handle_create_session(&self, _envelope: &WsEnvelope) -> Result<()> {
        // TODO: Send ack, create SessionRunner
        info!("Received create_session (handler not implemented yet)");
        Ok(())
    }

    async fn handle_send_message(&self, _envelope: &WsEnvelope) -> Result<()> {
        info!("Received send_message (handler not implemented yet)");
        Ok(())
    }

    async fn handle_session_control(&self, _envelope: &WsEnvelope) -> Result<()> {
        info!("Received session_control (handler not implemented yet)");
        Ok(())
    }

    async fn handle_close_session(&self, _envelope: &WsEnvelope) -> Result<()> {
        info!("Received close_session (handler not implemented yet)");
        Ok(())
    }

    async fn handle_permission_response(&self, _envelope: &WsEnvelope) -> Result<()> {
        info!("Received permission_response (handler not implemented yet)");
        Ok(())
    }

    async fn handle_file_tree_request(&self, _envelope: &WsEnvelope) -> Result<()> {
        info!("Received file_tree_request (handler not implemented yet)");
        Ok(())
    }

    async fn handle_file_content_request(&self, _envelope: &WsEnvelope) -> Result<()> {
        info!("Received file_content_request (handler not implemented yet)");
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
        let backoff2 = calculate_backoff(min, max, 2);
        let backoff3 = calculate_backoff(min, max, 3);

        // Allow for jitter, but trend should be increasing
        // (With jitter, individual values might not always increase,
        // but the base values do: 1s, 2s, 4s)
        assert!(backoff1 < backoff3);
    }
}
