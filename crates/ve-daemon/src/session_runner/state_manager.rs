//! State management and permission timeout handling.

use std::time::Instant;

use tracing::{debug, warn};
use uuid::Uuid;
use ve_shared::types::SessionStatus;

use super::glob_match::matches_pattern;
use super::{BridgePermissionResult, DriverEvent, Result, RunnerState, SessionRunner};

impl SessionRunner {
    /// Update state
    pub(super) fn update_state(&mut self, new_state: RunnerState) {
        debug!(
            session_id = %self.session_id,
            old_state = ?self.state,
            new_state = ?new_state,
            "State transition"
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
        if self.event_tx.send(event).is_err() {
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
