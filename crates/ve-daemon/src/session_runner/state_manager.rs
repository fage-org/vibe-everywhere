//! State management and permission timeout handling.

use std::time::Instant;

use tracing::{debug, warn};
use uuid::Uuid;
use ve_shared::types::SessionStatus;

use super::glob_match::matches_pattern;
use super::{BridgePermissionResult, DriverEvent, Result, RunnerState, SessionRunner};

impl SessionRunner {
    /// Validate a state transition.
    ///
    /// Returns `true` for allowed transitions, derived from actual
    /// command handler paths.
    ///
    /// Allowed transitions:
    /// - Starting  → Running (driver init ok)
    /// - Starting  → Error (driver init failed)
    /// - Starting  → Closed (close command during startup)
    /// - Running   → WaitingApproval (permission requested)
    /// - Running   → Paused (pause action)
    /// - Running   → Closing (close command)
    /// - Running   → Error (fatal error)
    /// - Running   → Closed (terminate / close)
    /// - WaitingApproval → Running (permission resolved)
    /// - WaitingApproval → Error (fatal error)
    /// - WaitingApproval → Closed (terminate / close)
    /// - Paused    → Running (resume / restart)
    /// - Paused    → Closing (close command)
    /// - Paused    → Error (fatal error)
    /// - Paused    → Closed (terminate / close)
    /// - Error     → Running (restart / rerun)
    /// - Error     → Closed (close command)
    /// - Closing   → Closed (close completes)
    /// - Closed    → (none)
    fn validate_transition(from: RunnerState, to: RunnerState) -> bool {
        matches!(
            (from, to),
            (RunnerState::Starting, RunnerState::Running | RunnerState::Error | RunnerState::Closed)
            | (
                RunnerState::Running,
                RunnerState::WaitingApproval
                    | RunnerState::Paused
                    | RunnerState::Closing
                    | RunnerState::Error
                    | RunnerState::Closed
            )
            | (
                RunnerState::WaitingApproval,
                RunnerState::Running | RunnerState::Error | RunnerState::Closed
            )
            | (
                RunnerState::Paused,
                RunnerState::Running
                    | RunnerState::Closing
                    | RunnerState::Error
                    | RunnerState::Closed
            )
            | (
                RunnerState::Error,
                RunnerState::Running | RunnerState::Closed
            )
            | (RunnerState::Closing, RunnerState::Closed)
        )
    }

    /// Update state with transition validation.
    /// Panics on invalid state transitions — these indicate a bug in the caller.
    pub(super) fn update_state(&mut self, new_state: RunnerState) {
        debug!(
            session_id = %self.session_id,
            old_state = ?self.state,
            new_state = ?new_state,
            "State transition"
        );
        assert!(
            Self::validate_transition(self.state, new_state),
            "Invalid state transition from {:?} to {:?} for session {}",
            self.state,
            new_state,
            self.session_id
        );
        self.state = new_state;
    }

    /// Report status change
    pub(super) async fn report_status(
        &self,
        status: SessionStatus,
        summary: Option<String>,
        close_reason: Option<ve_shared::types::CloseReason>,
    ) {
        let event = DriverEvent::StatusUpdate {
            session_id: self.session_id,
            status,
            summary,
            close_reason,
        };
        if self.event_tx.try_send(event).is_err() {
            warn!(session_id = %self.session_id, "Failed to send status update");
        }
    }

    /// Check approval cache
    ///
    /// Checks if a permission request matches any cached approval rules.
    /// risk_type must match exactly, target supports wildcard matching.
    pub fn check_approval_cache(&self, risk_type: &str, target: Option<&str>) -> bool {
        let target_str = target.unwrap_or("*");

        self.approval_cache.iter().any(|rule| {
            if rule.risk_type != risk_type {
                return false;
            }
            matches_pattern(&rule.target_pattern, target_str)
        })
    }

    /// Return the earliest pending permission timeout, if any.
    ///
    /// Used by the runner main loop to schedule `sleep_until` instead of
    /// fixed-interval polling.
    pub(super) fn earliest_permission_timeout(&self) -> Option<Instant> {
        self.permission_timeouts.values().copied().min()
    }

    /// Check permission timeouts
    ///
    /// Checks all pending permission requests and sends timeout responses for expired ones.
    pub(super) async fn check_permission_timeouts(&mut self) -> Result<()> {
        let now = Instant::now();

        let expired: Vec<Uuid> = self
            .permission_timeouts
            .iter()
            .filter(|(_, &expires_at)| now >= expires_at)
            .map(|(&id, _)| id)
            .collect();

        for permission_id in &expired {
            warn!(
                session_id = %self.session_id,
                permission_id = %permission_id,
                "Permission request timed out"
            );

            let pending = self.pending_permissions.remove(permission_id);
            self.permission_timeouts.remove(permission_id);

            if let Some(pending) = pending {
                if let Some(response_tx) = pending.bridge_response {
                    let _ = response_tx.send(BridgePermissionResult::Timeout);
                } else {
                    self.driver
                        .permission_timeout(self.session_id, *permission_id)
                        .await?;
                }
            }
        }

        if !expired.is_empty()
            && self.pending_permissions.is_empty()
            && self.permission_timeouts.is_empty()
            && self.state == RunnerState::WaitingApproval
        {
            self.update_state(RunnerState::Running);
            self.report_status(SessionStatus::Running, None, None).await;
        }

        Ok(())
    }
}
