//! Session command helpers
//!
//! Low-level daemon command sending, error sanitization, and response handling.

use super::{AppState, DaemonResponse, Result, ServerError};
use uuid::Uuid;
use ve_shared::proto::{DaemonMessage, SessionControlAction};

pub async fn send_daemon_command_and_wait(
    state: &AppState,
    host_id: Uuid,
    request_id: String,
    request: DaemonMessage,
) -> Result<DaemonResponse> {
    state
        .hub
        .send_and_wait(
            &host_id,
            request,
            request_id,
            std::time::Duration::from_millis(state.config.ack_timeout_ms),
        )
        .await
        .map_err(|error| sanitize_session_command_transport_error(&error))
}

fn sanitize_session_command_transport_error(error: &dyn std::error::Error) -> ServerError {
    tracing::warn!(error = %error, "Sanitized daemon session command transport failure");
    ServerError::Conflict("Daemon command failed".to_string())
}

fn sanitize_session_command_response_error() -> ServerError {
    ServerError::Conflict("Daemon command failed".to_string())
}

pub async fn mark_session_error(state: &AppState, session_id: &str, message: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'error', latest_summary = $2, updated_at = CURRENT_TIMESTAMP
        WHERE session_id = $1
        "#,
    )
    .bind(session_id)
    .bind(message)
    .execute(&state.db)
    .await?;

    Ok(())
}

pub fn ensure_command_acked(response: DaemonResponse) -> Result<()> {
    match response {
        DaemonResponse::Ack(ack) if ack.success => Ok(()),
        DaemonResponse::Ack(ack) => {
            tracing::warn!(
                has_error = ack.error.is_some(),
                "Sanitized daemon session command ack failure"
            );
            Err(sanitize_session_command_response_error())
        }
        DaemonResponse::Error(error) => {
            tracing::warn!(error_code = %error.error_code, "Sanitized daemon session command error response");
            Err(sanitize_session_command_response_error())
        }
        DaemonResponse::Message(_) => Err(ServerError::Conflict(
            "Unexpected daemon response type".to_string(),
        )),
    }
}

pub async fn persist_control_success(
    state: &AppState,
    session_id: Uuid,
    action: SessionControlAction,
) -> Result<()> {
    let session_id_str = session_id.to_string();

    match action {
        SessionControlAction::Pause => {
            sqlx::query(
                r#"
                UPDATE sessions SET status = 'paused', updated_at = CURRENT_TIMESTAMP
                WHERE session_id = $1
                "#,
            )
            .bind(&session_id_str)
            .execute(&state.db)
            .await?;
        }
        SessionControlAction::Terminate => {}
        SessionControlAction::Restart => {
            sqlx::query(
                r#"
                UPDATE sessions SET status = 'running', updated_at = CURRENT_TIMESTAMP
                WHERE session_id = $1
                "#,
            )
            .bind(&session_id_str)
            .execute(&state.db)
            .await?;
        }
        SessionControlAction::Interrupt | SessionControlAction::Rerun => {}
    }

    Ok(())
}
