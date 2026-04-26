//! Session runner main loop.

use std::time::Duration;
use tokio::time::sleep_until;
use tracing::{error, info};

use super::{Result, RunnerState, SessionRunner};
use ve_shared::types::SessionStatus;

/// Fallback poll interval when no permissions are pending.
const IDLE_POLL_INTERVAL: Duration = Duration::from_secs(10);

impl SessionRunner {
    /// Run the session main loop
    pub async fn run(mut self) {
        info!(session_id = %self.session_id, "SessionRunner started");

        if let Some(claude_session_id) = self.startup_claude_session_id.clone() {
            if let Err(e) = self
                .driver
                .rerun(self.session_id, &self.workspace_path, &claude_session_id)
                .await
            {
                error!(error = %e, "Failed to rerun agent");
                self.update_state(RunnerState::Error);
                self.report_status(SessionStatus::Error, Some(e.to_string()), None)
                    .await;
                self.finish_startup(Err(e));
                return;
            }
            self.claude_session_id = Some(claude_session_id);
        } else {
            let config = self.build_driver_config();

            if let Err(e) = self.driver.start(config).await {
                error!(error = %e, "Failed to start agent");
                self.update_state(RunnerState::Error);
                self.report_status(SessionStatus::Error, Some(e.to_string()), None)
                    .await;
                self.finish_startup(Err(e));
                return;
            }
        }

        self.update_state(RunnerState::Running);
        self.report_status(SessionStatus::Running, None, None).await;
        self.finish_startup(Ok(()));

        loop {
            let deadline = self
                .earliest_permission_timeout()
                .map(|t| t.into())
                .unwrap_or_else(|| tokio::time::Instant::now() + IDLE_POLL_INTERVAL);

            tokio::select! {
                cmd = self.command_rx.recv() => {
                    match cmd {
                        Some(cmd) => {
                            if let Err(e) = self.handle_command(cmd).await {
                                error!(error = %e, "Failed to handle command");
                            }
                        }
                        None => {
                            info!(session_id = %self.session_id, "Command channel closed");
                            break;
                        }
                    }
                }

                _ = sleep_until(deadline) => {
                    if let Err(e) = self.check_permission_timeouts().await {
                        error!(error = %e, "Failed to check permission timeouts");
                    }
                }
            }

            if self.state == RunnerState::Closed || self.state == RunnerState::Error {
                break;
            }
        }

        info!(session_id = %self.session_id, state = ?self.state, "SessionRunner ended");
    }

    fn finish_startup(&mut self, result: Result<()>) {
        if let Some(tx) = self.startup_completion.take() {
            let _ = tx.send(result);
        }
    }
}
