//! Session message handlers for the WebSocket client.

use tracing::{debug, info, warn};
use uuid::Uuid;
use ve_shared::proto::{SessionControlAction, WsEnvelope};

use super::{AckError, DaemonError, DriverEvent, Result, WsClient};

impl WsClient {
    /// Handle driver event from session runners and return messages to send
    pub(super) fn handle_driver_event(&self, event: DriverEvent) -> Vec<(String, serde_json::Value)> {
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
    pub(super) async fn handle_message(&self, text: &str) -> Result<()> {
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

    pub(super) async fn handle_ensure_workspace(&self, envelope: &WsEnvelope) -> Result<()> {
        let request_id = envelope
            .request_id
            .clone()
            .ok_or(DaemonError::RequestIdMissing)?;

        let workspace_path = envelope
            .payload
            .get("workspace_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DaemonError::WsPayloadMissing("workspace_path".to_string()))?;

        match super::utils::ensure_workspace_directory(workspace_path).await {
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

    pub(super) async fn handle_create_session(&self, envelope: &WsEnvelope) -> Result<()> {
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

    pub(super) async fn handle_rerun_session(&self, envelope: &WsEnvelope) -> Result<()> {
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

    pub(super) async fn handle_send_message(&self, envelope: &WsEnvelope) -> Result<()> {
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

    pub(super) async fn handle_session_control(&self, envelope: &WsEnvelope) -> Result<()> {
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

    pub(super) async fn handle_close_session(&self, envelope: &WsEnvelope) -> Result<()> {
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
}
