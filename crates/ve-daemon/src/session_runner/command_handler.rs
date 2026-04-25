//! Command handling for the session runner.

use tracing::{debug, error, info, warn};
use ve_shared::models::PermissionDecision;
use ve_shared::proto::SessionControlAction;
use ve_shared::types::{CloseReason, SessionStatus};

use super::{ApprovalRule, BridgePermissionResult, DaemonError, PendingPermission,
    Result, RunnerCommand, RunnerState, SessionRunner};

impl SessionRunner {
    /// Handle a command
    pub(super) async fn handle_command(&mut self, cmd: RunnerCommand) -> Result<()> {
        match cmd {
            RunnerCommand::SendMessage { content, completion } => {
                let result = self.handle_send_message(content).await;
                Self::complete_command(completion, result);
            }

            RunnerCommand::Control { action, completion } => {
                let result = self.handle_control(action).await;
                Self::complete_command(completion, result);
            }

            RunnerCommand::Rerun { claude_session_id } => {
                if let Err(e) = self.handle_rerun(claude_session_id).await {
                    error!(error = %e, "Failed to handle rerun");
                    self.update_state(RunnerState::Error);
                    self.report_status(SessionStatus::Error, Some(e.to_string()), None)
                        .await;
                }
            }

            RunnerCommand::RegisterPermission {
                permission_id,
                risk_type,
                target,
                summary,
                bridge_response,
            } => {
                if self.check_approval_cache(&risk_type, target.as_deref()) {
                    info!(
                        session_id = %self.session_id,
                        %permission_id,
                        risk_type,
                        target = ?target,
                        "Permission auto-approved by cache"
                    );
                    if let Some(response_tx) = bridge_response {
                        let _ = response_tx
                            .send(BridgePermissionResult::Decision(
                                PermissionDecision::ApproveSession,
                            ));
                    } else {
                        self.driver
                            .respond_permission(
                                self.session_id,
                                permission_id,
                                PermissionDecision::ApproveSession,
                            )
                            .await?;
                    }
                } else {
                    self.pending_permissions.insert(
                        permission_id,
                        PendingPermission {
                            risk_type: risk_type.clone(),
                            target: target.clone(),
                            summary,
                            bridge_response,
                        },
                    );
                    let expires_at = std::time::Instant::now() + self.config.permission_timeout();
                    self.permission_timeouts.insert(permission_id, expires_at);
                    debug!(
                        session_id = %self.session_id,
                        %permission_id,
                        risk_type,
                        target = ?target,
                        timeout_secs = self.config.permission_timeout().as_secs(),
                        "Permission request registered with timeout"
                    );
                    if self.state == RunnerState::Running {
                        self.update_state(RunnerState::WaitingApproval);
                        self.report_status(SessionStatus::WaitingApproval, None, None)
                            .await;
                    }
                }
            }

            RunnerCommand::PermissionResponse {
                permission_id,
                decision,
            } => {
                if let Some(pending) = self.pending_permissions.remove(&permission_id) {
                    self.permission_timeouts.remove(&permission_id);

                    if let Some(response_tx) = pending.bridge_response {
                        let _ = response_tx.send(BridgePermissionResult::Decision(decision));
                    } else {
                        self.driver
                            .respond_permission(self.session_id, permission_id, decision)
                            .await?;
                    }

                    if decision == PermissionDecision::ApproveSession {
                        self.approval_cache.push(ApprovalRule {
                            risk_type: pending.risk_type.clone(),
                            target_pattern: pending.target.clone().unwrap_or("*".to_string()),
                        });
                        debug!(
                            session_id = %self.session_id,
                            risk_type = %pending.risk_type,
                            target = ?pending.target,
                            "Added approval rule to cache"
                        );
                    }

                    if self.pending_permissions.is_empty()
                        && self.permission_timeouts.is_empty()
                        && self.state == RunnerState::WaitingApproval
                    {
                        self.update_state(RunnerState::Running);
                        self.report_status(SessionStatus::Running, None, None).await;
                    }
                }
            }

            RunnerCommand::SetClaudeSessionId { claude_session_id } => {
                self.claude_session_id = Some(claude_session_id.clone());
                debug!(
                    session_id = %self.session_id,
                    claude_session_id = %claude_session_id,
                    "Stored Claude session ID for --resume support"
                );
            }

            RunnerCommand::Close { completion } => {
                let result = self.handle_close().await;
                Self::complete_command(completion, result);
            }
        }

        Ok(())
    }

    fn complete_command(completion: Option<tokio::sync::oneshot::Sender<Result<()>>>, result: Result<()>) {
        if let Some(tx) = completion {
            let _ = tx.send(result);
        } else if let Err(error) = result {
            error!(error = %error, "Failed to handle command");
        }
    }

    async fn handle_send_message(&mut self, content: String) -> Result<()> {
        if self.state != RunnerState::Running {
            return Err(DaemonError::SessionInvalidStatus {
                current: format!("{:?}", self.state),
                expected: "Running".to_string(),
            });
        }
        self.driver.send_message(self.session_id, &content).await?;
        debug!(session_id = %self.session_id, "Message sent to agent");
        Ok(())
    }

    /// Handle control action
    pub async fn handle_control(&mut self, action: SessionControlAction) -> Result<()> {
        match action {
            SessionControlAction::Pause => {
                self.update_state(RunnerState::Paused);
                self.driver.control(self.session_id, action).await?;
                self.report_status(SessionStatus::Paused, None, None).await;
            }
            SessionControlAction::Interrupt => {
                self.driver.control(self.session_id, action).await?;
            }
            SessionControlAction::Terminate => {
                self.driver.control(self.session_id, action).await?;
                self.update_state(RunnerState::Closed);
                self.report_status(SessionStatus::Archived, None, Some(CloseReason::Terminated))
                    .await;
            }
            SessionControlAction::Rerun => {
                return Err(DaemonError::SessionInvalidStatus {
                    current: format!("{:?}", self.state),
                    expected: "archived rerun path only".to_string(),
                });
            }
            SessionControlAction::Restart => {
                if let Some(claude_sid) = self.claude_session_id.clone() {
                    self.handle_rerun(claude_sid).await?;
                } else {
                    warn!(
                        session_id = %self.session_id,
                        "Restart requested but no claude_session_id available"
                    );
                    return Err(DaemonError::SessionRerunFailed {
                        reason: "No Claude session ID available".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    async fn handle_rerun(&mut self, claude_session_id: String) -> Result<()> {
        self.driver
            .rerun(self.session_id, &self.workspace_path, &claude_session_id)
            .await?;
        self.claude_session_id = Some(claude_session_id.clone());
        self.update_state(RunnerState::Running);
        self.report_status(
            SessionStatus::Running,
            Some("Session resumed".to_string()),
            None,
        )
        .await;
        info!(
            session_id = %self.session_id,
            claude_session_id = %claude_session_id,
            "Session rerun successful"
        );
        Ok(())
    }

    /// Handle close
    pub async fn handle_close(&mut self) -> Result<()> {
        self.update_state(RunnerState::Closing);
        self.driver.close(self.session_id).await?;
        self.update_state(RunnerState::Closed);
        self.report_status(SessionStatus::Archived, None, Some(CloseReason::UserClosed))
            .await;
        Ok(())
    }
}
