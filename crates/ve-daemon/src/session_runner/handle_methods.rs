//! SessionRunnerHandle public methods.

use std::time::Duration;

use tokio::sync::oneshot;
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::SessionControlAction;

use super::{
    wait_for_command_completion, BridgePermissionResult, DaemonError, Result, RunnerCommand,
    SessionRunnerHandle,
};

impl SessionRunnerHandle {
    /// Send a message
    pub async fn send_message(&self, content: String) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::SendMessage {
                content,
                completion: None,
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    pub async fn send_message_and_wait(
        &self,
        content: String,
        timeout: Duration,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(RunnerCommand::SendMessage {
                content,
                completion: Some(tx),
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))?;
        wait_for_command_completion(rx, timeout).await
    }

    /// Send a control command
    pub async fn send_control(&self, action: SessionControlAction) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::Control {
                action,
                completion: None,
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    pub async fn send_control_and_wait(
        &self,
        action: SessionControlAction,
        timeout: Duration,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(RunnerCommand::Control {
                action,
                completion: Some(tx),
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))?;
        wait_for_command_completion(rx, timeout).await
    }

    /// Rerun with existing Claude session
    pub async fn send_rerun(&self, claude_session_id: String) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::Rerun { claude_session_id })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    /// Send close command
    pub async fn send_close(&self) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::Close { completion: None })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    pub async fn send_close_and_wait(&self, timeout: Duration) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(RunnerCommand::Close {
                completion: Some(tx),
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))?;
        wait_for_command_completion(rx, timeout).await
    }

    /// Register a permission request (stores metadata for session-level approval caching)
    pub async fn register_permission(
        &self,
        permission_id: Uuid,
        risk_type: String,
        target: Option<String>,
        summary: String,
    ) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::RegisterPermission {
                permission_id,
                risk_type,
                target,
                summary,
                bridge_response: None,
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    pub async fn register_bridge_permission(
        &self,
        permission_id: Uuid,
        risk_type: String,
        target: Option<String>,
        summary: String,
        bridge_response: oneshot::Sender<BridgePermissionResult>,
    ) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::RegisterPermission {
                permission_id,
                risk_type,
                target,
                summary,
                bridge_response: Some(bridge_response),
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    /// Send a permission response
    pub async fn send_permission_response(
        &self,
        permission_id: Uuid,
        decision: PermissionDecision,
    ) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::PermissionResponse {
                permission_id,
                decision,
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    /// Set Claude session ID (for --resume support)
    pub async fn set_claude_session_id(&self, claude_session_id: String) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::SetClaudeSessionId { claude_session_id })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }
}
