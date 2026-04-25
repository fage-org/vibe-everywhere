//! Session Runner
//!
//! Manages the runtime state of a single session.

mod command_handler;
mod glob_match;
mod handle_methods;
mod runner;
mod state_manager;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{broadcast, mpsc, oneshot};
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::SessionControlAction;

use crate::agent::{create_driver, AgentDriver, DriverConfig, DriverEvent};
use crate::config::Config;
use crate::error::DaemonError;
use crate::Result;

/// Runner internal state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerState {
    Starting,
    Running,
    WaitingApproval,
    Paused,
    Error,
    Closing,
    Closed,
}

/// Session runner command
#[derive(Debug)]
pub enum RunnerCommand {
    SendMessage {
        content: String,
        completion: Option<oneshot::Sender<Result<()>>>,
    },
    Control {
        action: SessionControlAction,
        completion: Option<oneshot::Sender<Result<()>>>,
    },
    Rerun { claude_session_id: String },
    RegisterPermission {
        permission_id: Uuid,
        risk_type: String,
        target: Option<String>,
        summary: String,
        bridge_response: Option<oneshot::Sender<BridgePermissionResult>>,
    },
    PermissionResponse {
        permission_id: Uuid,
        decision: PermissionDecision,
    },
    SetClaudeSessionId { claude_session_id: String },
    Close {
        completion: Option<oneshot::Sender<Result<()>>>,
    },
}

/// Session runner handle
#[derive(Clone)]
pub struct SessionRunnerHandle {
    pub command_tx: mpsc::Sender<RunnerCommand>,
    pub state: RunnerState,
    pub session_id: Uuid,
}

/// Session runner
#[allow(dead_code)]
pub struct SessionRunner {
    session_id: Uuid,
    workspace_path: String,
    agent_type: String,
    initial_message: Option<String>,
    config: Arc<Config>,
    startup_completion: Option<oneshot::Sender<Result<()>>>,

    state: RunnerState,
    driver: Box<dyn AgentDriver>,

    command_rx: mpsc::Receiver<RunnerCommand>,
    event_tx: broadcast::Sender<DriverEvent>,

    pending_permissions: HashMap<Uuid, PendingPermission>,
    permission_timeouts: HashMap<Uuid, Instant>,
    approval_cache: Vec<ApprovalRule>,

    claude_session_id: Option<String>,
    startup_claude_session_id: Option<String>,
}

/// Approval rule for session-level permission caching
#[derive(Debug, Clone)]
pub struct ApprovalRule {
    pub risk_type: String,
    pub target_pattern: String,
}

/// Result returned to the external permission-prompt MCP bridge.
#[derive(Debug)]
pub enum BridgePermissionResult {
    Decision(PermissionDecision),
    Timeout,
}

/// Pending permission request
#[derive(Debug)]
pub struct PendingPermission {
    pub risk_type: String,
    pub target: Option<String>,
    pub summary: String,
    pub bridge_response: Option<oneshot::Sender<BridgePermissionResult>>,
}

impl SessionRunner {
    fn build_driver_config(&self) -> DriverConfig {
        DriverConfig {
            session_id: self.session_id,
            workspace_path: self.workspace_path.clone(),
            agent_type: self.agent_type.clone(),
            initial_message: self.initial_message.clone(),
        }
    }

    /// Create a new session runner
    pub fn new(
        session_id: Uuid,
        workspace_path: String,
        agent_type: String,
        initial_message: Option<String>,
        config: Arc<Config>,
        event_tx: broadcast::Sender<DriverEvent>,
        startup_completion: Option<oneshot::Sender<Result<()>>>,
    ) -> Result<(Self, SessionRunnerHandle)> {
        let (command_tx, command_rx) = mpsc::channel(16);

        let handle = SessionRunnerHandle {
            command_tx,
            state: RunnerState::Starting,
            session_id,
        };

        let driver = create_driver(&agent_type, config.clone(), event_tx.clone())?;

        let runner = Self {
            session_id,
            workspace_path: workspace_path.clone(),
            agent_type: agent_type.clone(),
            initial_message,
            config: config.clone(),
            startup_completion,
            state: RunnerState::Starting,
            driver,
            command_rx,
            event_tx,
            pending_permissions: HashMap::new(),
            permission_timeouts: HashMap::new(),
            approval_cache: Vec::new(),
            claude_session_id: None,
            startup_claude_session_id: None,
        };

        Ok((runner, handle))
    }

    pub fn new_rerun(
        session_id: Uuid,
        workspace_path: String,
        agent_type: String,
        claude_session_id: String,
        config: Arc<Config>,
        event_tx: broadcast::Sender<DriverEvent>,
        startup_completion: Option<oneshot::Sender<Result<()>>>,
    ) -> Result<(Self, SessionRunnerHandle)> {
        let (mut runner, handle) = Self::new(
            session_id,
            workspace_path,
            agent_type,
            None,
            config,
            event_tx,
            startup_completion,
        )?;
        runner.startup_claude_session_id = Some(claude_session_id);
        Ok((runner, handle))
    }
}

async fn wait_for_command_completion(
    rx: oneshot::Receiver<Result<()>>,
    timeout: std::time::Duration,
) -> Result<()> {
    match tokio::time::timeout(timeout, rx).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => Err(DaemonError::InternalTaskError(
            "Command completion channel closed".to_string(),
        )),
        Err(_) => Err(DaemonError::SessionInvalidStatus {
            current: "timeout".to_string(),
            expected: "command completion".to_string(),
        }),
    }
}
