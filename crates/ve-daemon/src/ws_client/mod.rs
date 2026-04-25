//! WebSocket Client Module
//!
//! Manages the persistent WebSocket connection between ve-daemon and ve-server.

mod connection;
mod file_handlers;
mod handlers;
mod utils;
#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;

use futures_util::SinkExt;
use tokio::sync::{broadcast, oneshot};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{debug, warn};
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::{AckPayload, ErrorPayload, WsEnvelope};

use crate::agent::DriverEvent;
use crate::config::Config;
use crate::error::{AckError, DaemonError};
use crate::file_ops::FileOps;
use crate::session_registry::SessionRegistry;
use crate::session_runner::BridgePermissionResult;
use crate::Result;

#[derive(Debug, serde::Deserialize)]
struct PermissionBridgeRequest {
    request_id: String,
    session_id: Uuid,
    tool_name: String,
    input: serde_json::Value,
}

/// Type alias for WebSocket sender
#[allow(clippy::type_complexity)]
type WsSender = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    WsMessage,
>;

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

impl WsClient {
    /// Create a new WebSocket client
    pub fn new(config: Arc<Config>, host_id: Uuid, token: String) -> Self {
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

    fn permission_bridge_root(&self) -> PathBuf {
        self.config.config_dir.join("permission-bridge")
    }

    fn permission_bridge_requests_dir(&self) -> PathBuf {
        self.permission_bridge_root().join("requests")
    }

    fn permission_bridge_responses_dir(&self) -> PathBuf {
        self.permission_bridge_root().join("responses")
    }

    async fn ensure_permission_bridge_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.permission_bridge_requests_dir())
            .await
            .map_err(|e| DaemonError::Unknown(format!("Failed to create permission bridge requests dir: {}", e)))?;
        fs::create_dir_all(self.permission_bridge_responses_dir())
            .await
            .map_err(|e| DaemonError::Unknown(format!("Failed to create permission bridge responses dir: {}", e)))?;
        Ok(())
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

    fn permission_summary_and_target(
        tool_name: &str,
        input: &serde_json::Value,
    ) -> (String, Option<String>, String) {
        match tool_name {
            "Bash" => {
                let command = input
                    .get("command")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_string();
                (
                    "exec_cmd".to_string(),
                    Some(command.clone()),
                    format!("Claude requested Bash command execution: {}", command),
                )
            }
            "WebFetch" | "WebSearch" => {
                let target = input
                    .get("url")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string());
                (
                    "network".to_string(),
                    target.clone(),
                    format!("Claude requested network access via {}", tool_name),
                )
            }
            _ => {
                let target = input
                    .get("file_path")
                    .or_else(|| input.get("path"))
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string());
                (
                    "write_fs".to_string(),
                    target.clone(),
                    format!("Claude requested {} access", tool_name),
                )
            }
        }
    }

    async fn process_permission_bridge_requests(&self) -> Result<()> {
        let Some(registry) = &self.registry else {
            return Ok(());
        };
        let Some(ws_sender) = &self.ws_sender else {
            return Ok(());
        };

        let requests_dir = self.permission_bridge_requests_dir();
        let responses_dir = self.permission_bridge_responses_dir();
        let mut entries = match fs::read_dir(&requests_dir).await {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(err) => {
                return Err(DaemonError::Unknown(format!(
                    "Failed to read permission bridge requests dir: {}",
                    err
                )))
            }
        };

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            DaemonError::Unknown(format!("Failed to read permission bridge entry: {}", e))
        })? {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let request: PermissionBridgeRequest = match fs::read_to_string(&path)
                .await
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
            {
                Some(request) => request,
                None => {
                    warn!(path = %path.display(), "Failed to parse permission bridge request");
                    let _ = fs::remove_file(&path).await;
                    continue;
                }
            };
            let _ = fs::remove_file(&path).await;

            let Some(handle) = registry.get(&request.session_id).await else {
                self.write_permission_bridge_response(
                    &responses_dir,
                    &request.request_id,
                    serde_json::json!({
                        "behavior": "deny",
                        "message": format!("Session {} not found", request.session_id),
                    }),
                )
                .await?;
                continue;
            };

            let permission_id = Uuid::new_v4();
            let (risk_type, target, summary) =
                Self::permission_summary_and_target(&request.tool_name, &request.input);
            let (response_tx, response_rx) = oneshot::channel();
            handle
                .register_bridge_permission(
                    permission_id,
                    risk_type.clone(),
                    target.clone(),
                    summary.clone(),
                    response_tx,
                )
                .await?;

            let envelope = WsEnvelope::new(
                "permission_request",
                serde_json::json!({
                    "permission_id": permission_id,
                    "session_id": request.session_id,
                    "risk_type": risk_type,
                    "summary": summary,
                    "target": target,
                }),
            );
            let json = serde_json::to_string(&envelope).map_err(DaemonError::WsMessageParse)?;
            {
                let mut sender = ws_sender.lock().await;
                sender
                    .send(WsMessage::Text(json.into()))
                    .await
                    .map_err(|e| DaemonError::WsConnect(Box::new(e)))?;
            }

            let response_path = responses_dir.join(format!("{}.json", request.request_id));
            let original_input = request.input.clone();
            tokio::spawn(async move {
                let payload = match response_rx.await {
                    Ok(BridgePermissionResult::Decision(
                        PermissionDecision::ApproveOnce | PermissionDecision::ApproveSession,
                    )) => serde_json::json!({
                        "behavior": "allow",
                        "updatedInput": original_input,
                    }),
                    Ok(BridgePermissionResult::Decision(PermissionDecision::DenyOnce)) => {
                        serde_json::json!({
                            "behavior": "deny",
                            "message": "Denied by Vibe Everywhere user approval flow.",
                        })
                    }
                    Ok(BridgePermissionResult::Timeout) | Err(_) => serde_json::json!({
                        "behavior": "deny",
                        "message": "Timed out waiting for Vibe Everywhere user approval.",
                    }),
                };

                if let Err(err) = fs::write(
                    &response_path,
                    serde_json::to_string(&payload).unwrap_or_else(|_| {
                        "{\"behavior\":\"deny\",\"message\":\"Failed to serialize response\"}"
                            .to_string()
                    }),
                )
                .await
                {
                    warn!(error = %err, path = %response_path.display(), "Failed to write permission bridge response");
                }
            });
        }

        Ok(())
    }

    async fn write_permission_bridge_response(
        &self,
        responses_dir: &Path,
        request_id: &str,
        payload: serde_json::Value,
    ) -> Result<()> {
        let path = responses_dir.join(format!("{}.json", request_id));
        fs::write(
            &path,
            serde_json::to_string(&payload).map_err(DaemonError::WsMessageParse)?,
        )
        .await
        .map_err(|err| {
            DaemonError::Unknown(format!(
                "Failed to write permission bridge response {}: {}",
                path.display(),
                err
            ))
        })?;
        Ok(())
    }
}
