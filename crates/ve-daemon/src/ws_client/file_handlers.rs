//! File and permission message handlers for the WebSocket client.

use std::path::PathBuf;
use tracing::{debug, info, warn};
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::{DaemonToServer, WsEnvelope};

use super::{AckError, DaemonError, FileOps, Result, WsClient};

impl WsClient {
    pub(super) async fn handle_permission_response(&self, envelope: &WsEnvelope) -> Result<()> {
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

    pub(super) async fn handle_file_tree_request(&self, envelope: &WsEnvelope) -> Result<()> {
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

        // Verify workspace is associated with an active session
        if let Some(ref registry) = self.registry {
            if !registry.contains_workspace(&workspace_root).await {
                self.send_error(
                    &request_id,
                    &AckError::WorkspaceInvalid,
                    "Workspace not associated with any active session",
                )
                .await;
                return Ok(());
            }
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
                        use futures_util::SinkExt;
                        use tokio_tungstenite::tungstenite::Message as WsMessage;
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

    pub(super) async fn handle_file_content_request(&self, envelope: &WsEnvelope) -> Result<()> {
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

        // Verify workspace is associated with an active session
        if let Some(ref registry) = self.registry {
            if !registry.contains_workspace(&workspace_root).await {
                self.send_error(
                    &request_id,
                    &AckError::WorkspaceInvalid,
                    "Workspace not associated with any active session",
                )
                .await;
                return Ok(());
            }
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
                    content_may_be_corrupted: content.content_may_be_corrupted,
                };
                let envelope = WsEnvelope::new("file_content_response", &response);
                if let Ok(json) = serde_json::to_string(&envelope) {
                    if let Some(ref ws_sender) = self.ws_sender {
                        use futures_util::SinkExt;
                        use tokio_tungstenite::tungstenite::Message as WsMessage;
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

    pub(super) async fn handle_paired(&self, _envelope: &WsEnvelope) -> Result<()> {
        info!("Received paired notification");
        Ok(())
    }
}
