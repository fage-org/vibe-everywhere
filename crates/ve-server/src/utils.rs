//! Utility functions for parsing and validation
//!
//! Provides safe parsing helpers with proper error handling and logging.

use axum::http::HeaderMap;
use uuid::Uuid;

use crate::error::ServerError;

/// Generate a new request ID with prefix.
///
/// Format: `req-{uuid}` for WebSocket message-level IDs.
pub fn generate_request_id() -> String {
    format!("req-{}", Uuid::new_v4())
}

/// Generate a new trace ID with prefix.
///
/// Format: `tr-{uuid}` for cross-service trace IDs.
pub fn generate_trace_id() -> String {
    format!("tr-{}", Uuid::new_v4())
}

/// Extract `x-request-id` header from HTTP headers.
///
/// If not present, generates a new trace ID.
/// Returns the trace ID for logging correlation.
pub fn extract_request_id(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| {
            // If already prefixed, use it directly
            if s.starts_with("tr-") || s.starts_with("req-") {
                s.to_string()
            } else {
                // Add prefix if not present
                format!("tr-{}", s)
            }
        })
        .unwrap_or_else(generate_trace_id)
}

/// Parse a UUID string with a field name context for better error messages.
///
/// Returns a `ServerError::InternalUuidParse` error instead of panicking.
pub fn parse_uuid(input: &str, field_name: &str) -> Result<Uuid, ServerError> {
    Uuid::parse_str(input).map_err(|e| {
        tracing::error!(
            input = %input,
            field = %field_name,
            error = %e,
            "Failed to parse UUID"
        );
        ServerError::InternalUuidParse(format!("Invalid {}: '{}'", field_name, input))
    })
}

/// Parse a timestamp string that may be RFC3339, SQLite `datetime('now')`, or PostgreSQL TIMESTAMPTZ format.
///
/// Supports:
/// - RFC3339: `2026-04-23T00:20:11Z`
/// - SQLite: `2026-04-23 00:20:11` (space-separated, no T, no Z)
/// - PostgreSQL TIMESTAMPTZ: `2026-04-23 00:20:11.123456+00` (with microseconds and tz offset)
pub fn parse_sqlite_timestamp(
    s: &str,
) -> Result<chrono::DateTime<chrono::FixedOffset>, ServerError> {
    // Try RFC3339 first (e.g., "2026-04-23T00:20:11Z")
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt);
    }
    // Try PostgreSQL TIMESTAMPTZ format with microseconds and timezone (e.g., "2026-04-23 00:20:11.123456+00")
    if let Ok(dt) = chrono::DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f%#z") {
        return Ok(dt);
    }
    // Fallback: SQLite datetime('now') format (e.g., "2026-04-23 00:20:11")
    let dt = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map_err(|e| {
        tracing::error!(
            input = %s,
            error = %e,
            "Failed to parse timestamp (tried RFC3339, PostgreSQL TIMESTAMPTZ, and SQLite format)"
        );
        ServerError::Internal(format!("Invalid timestamp: '{}'", s))
    })?;
    // Assume UTC
    Ok(dt.and_utc().fixed_offset())
}

/// Parse a possibly-None SQLite timestamp, returning None for empty/missing values
pub fn parse_optional_sqlite_timestamp(
    s: &str,
) -> Option<chrono::DateTime<chrono::Utc>> {
    if s.is_empty() {
        return None;
    }
    parse_sqlite_timestamp(s).ok().map(|dt| dt.to_utc())
}

/// Parse session status string, returning error for unknown values
pub fn parse_session_status(s: &str) -> Result<ve_shared::types::SessionStatus, ServerError> {
    match s {
        "running" => Ok(ve_shared::types::SessionStatus::Running),
        "pending" => Ok(ve_shared::types::SessionStatus::Pending),
        "dispatching" => Ok(ve_shared::types::SessionStatus::Dispatching),
        "waiting_approval" => Ok(ve_shared::types::SessionStatus::WaitingApproval),
        "paused" => Ok(ve_shared::types::SessionStatus::Paused),
        "error" => Ok(ve_shared::types::SessionStatus::Error),
        "closing" => Ok(ve_shared::types::SessionStatus::Closing),
        "archived" => Ok(ve_shared::types::SessionStatus::Archived),
        unknown => Err(ServerError::BadRequest(format!(
            "Invalid session status: {}",
            unknown
        ))),
    }
}

/// Parse message type string, returning error for unknown values
pub fn parse_message_type(s: &str) -> Result<ve_shared::models::SessionMessageType, ServerError> {
    match s {
        "user" => Ok(ve_shared::models::SessionMessageType::User),
        "assistant" => Ok(ve_shared::models::SessionMessageType::Assistant),
        "system" => Ok(ve_shared::models::SessionMessageType::System),
        "tool" => Ok(ve_shared::models::SessionMessageType::Tool),
        "error" => Ok(ve_shared::models::SessionMessageType::Error),
        "permission" => Ok(ve_shared::models::SessionMessageType::Permission),
        unknown => Err(ServerError::BadRequest(format!(
            "Invalid message type: {}",
            unknown
        ))),
    }
}

/// Parse control action string, rejecting unknown values
pub fn parse_control_action(
    s: &str,
) -> Result<ve_shared::proto::SessionControlAction, ServerError> {
    match s {
        "pause" => Ok(ve_shared::proto::SessionControlAction::Pause),
        "terminate" => Ok(ve_shared::proto::SessionControlAction::Terminate),
        "interrupt" => Ok(ve_shared::proto::SessionControlAction::Interrupt),
        "rerun" => Ok(ve_shared::proto::SessionControlAction::Rerun),
        "restart" => Ok(ve_shared::proto::SessionControlAction::Restart),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown control action");
            Err(ServerError::BadRequest(format!(
                "Invalid control action: {}",
                unknown
            )))
        }
    }
}

/// Parse risk type string, returning error for unknown values
pub fn parse_risk_type(s: &str) -> Result<ve_shared::types::RiskType, ServerError> {
    match s {
        "write_fs" => Ok(ve_shared::types::RiskType::WriteFs),
        "exec_cmd" => Ok(ve_shared::types::RiskType::ExecCmd),
        "network" => Ok(ve_shared::types::RiskType::Network),
        unknown => Err(ServerError::BadRequest(format!(
            "Invalid risk type: {}",
            unknown
        ))),
    }
}

/// Parse permission status string, returning error for unknown values
pub fn parse_permission_status(s: &str) -> Result<ve_shared::types::PermissionStatus, ServerError> {
    match s {
        "pending" => Ok(ve_shared::types::PermissionStatus::Pending),
        "approved_once" => Ok(ve_shared::types::PermissionStatus::ApprovedOnce),
        "denied_once" => Ok(ve_shared::types::PermissionStatus::DeniedOnce),
        "approved_session" => Ok(ve_shared::types::PermissionStatus::ApprovedSession),
        "expired" => Ok(ve_shared::types::PermissionStatus::Expired),
        unknown => Err(ServerError::BadRequest(format!(
            "Invalid permission status: {}",
            unknown
        ))),
    }
}

/// Parse close reason string, returning error for unknown values
pub fn parse_close_reason(s: &str) -> Result<ve_shared::types::CloseReason, ServerError> {
    match s {
        "user_closed" => Ok(ve_shared::types::CloseReason::UserClosed),
        "completed" => Ok(ve_shared::types::CloseReason::Completed),
        "failed" => Ok(ve_shared::types::CloseReason::Failed),
        "terminated" => Ok(ve_shared::types::CloseReason::Terminated),
        unknown => Err(ServerError::BadRequest(format!(
            "Invalid close reason: {}",
            unknown
        ))),
    }
}

/// Parse platform string, returning error for unknown values
pub fn parse_platform(s: &str) -> Result<ve_shared::types::Platform, ServerError> {
    match s {
        "linux" => Ok(ve_shared::types::Platform::Linux),
        "macos" => Ok(ve_shared::types::Platform::Macos),
        "windows" => Ok(ve_shared::types::Platform::Windows),
        "wsl" => Ok(ve_shared::types::Platform::Wsl),
        unknown => Err(ServerError::BadRequest(format!(
            "Invalid platform: {}",
            unknown
        ))),
    }
}

/// Parse online status string, returning error for unknown values
pub fn parse_online_status(s: &str) -> Result<ve_shared::types::OnlineStatus, ServerError> {
    match s {
        "online" => Ok(ve_shared::types::OnlineStatus::Online),
        "offline" => Ok(ve_shared::types::OnlineStatus::Offline),
        "unknown" => Ok(ve_shared::types::OnlineStatus::Unknown),
        unknown => Err(ServerError::BadRequest(format!(
            "Invalid online status: {}",
            unknown
        ))),
    }
}

/// Parse daemon status string, returning error for unknown values
pub fn parse_daemon_status(s: &str) -> Result<ve_shared::types::DaemonStatus, ServerError> {
    match s {
        "healthy" => Ok(ve_shared::types::DaemonStatus::Healthy),
        "connecting" => Ok(ve_shared::types::DaemonStatus::Connecting),
        "disconnected" => Ok(ve_shared::types::DaemonStatus::Disconnected),
        "error" => Ok(ve_shared::types::DaemonStatus::Error),
        unknown => Err(ServerError::BadRequest(format!(
            "Invalid daemon status: {}",
            unknown
        ))),
    }
}

/// Parse pair status string, returning error for unknown values
pub fn parse_pair_status(s: &str) -> Result<ve_shared::types::PairStatus, ServerError> {
    match s {
        "paired" => Ok(ve_shared::types::PairStatus::Paired),
        "failed" => Ok(ve_shared::types::PairStatus::Failed),
        unknown => Err(ServerError::BadRequest(format!(
            "Invalid pair status: {}",
            unknown
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{header::HeaderName, HeaderValue};

    #[test]
    fn test_generate_request_id_format() {
        let id = generate_request_id();
        assert!(id.starts_with("req-"));
        assert!(id.len() > 4); // Should have UUID after prefix
    }

    #[test]
    fn test_generate_trace_id_format() {
        let id = generate_trace_id();
        assert!(id.starts_with("tr-"));
        assert!(id.len() > 3); // Should have UUID after prefix
    }

    #[test]
    fn test_extract_request_id_from_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_static("tr-abc123"),
        );
        let id = extract_request_id(&headers);
        assert_eq!(id, "tr-abc123");
    }

    #[test]
    fn test_extract_request_id_adds_prefix() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_static("abc123"),
        );
        let id = extract_request_id(&headers);
        assert_eq!(id, "tr-abc123");
    }

    #[test]
    fn test_extract_request_id_generates_when_missing() {
        let headers = HeaderMap::new();
        let id = extract_request_id(&headers);
        assert!(id.starts_with("tr-"));
    }

    #[test]
    fn test_parse_uuid_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_uuid(uuid_str, "test_field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let result = parse_uuid("not-a-uuid", "test_field");
        assert!(result.is_err());
        if let Err(ServerError::InternalUuidParse(msg)) = result {
            assert!(msg.contains("test_field"));
        } else {
            panic!("Expected InternalUuidParse error");
        }
    }

    #[test]
    fn test_parse_session_status_unknown_returns_error() {
        let result = parse_session_status("unknown_status");
        assert!(result.is_err());
        assert!(matches!(result, Err(ServerError::BadRequest(msg)) if msg.contains("Invalid session status")));
    }

    #[test]
    fn test_parse_session_status_valid() {
        let status = parse_session_status("running").unwrap();
        assert_eq!(status, ve_shared::types::SessionStatus::Running);
    }

    #[test]
    fn test_parse_online_status_accepts_unknown() {
        let status = parse_online_status("unknown").unwrap();
        assert_eq!(status, ve_shared::types::OnlineStatus::Unknown);
    }

    #[test]
    fn test_parse_daemon_status_accepts_disconnected() {
        let status = parse_daemon_status("disconnected").unwrap();
        assert_eq!(status, ve_shared::types::DaemonStatus::Disconnected);
    }

    #[test]
    fn test_parse_control_action_valid() {
        assert_eq!(
            parse_control_action("pause").unwrap(),
            ve_shared::proto::SessionControlAction::Pause
        );
        assert_eq!(
            parse_control_action("rerun").unwrap(),
            ve_shared::proto::SessionControlAction::Rerun
        );
        assert_eq!(
            parse_control_action("restart").unwrap(),
            ve_shared::proto::SessionControlAction::Restart
        );
    }

    #[test]
    fn test_parse_control_action_invalid() {
        let result = parse_control_action("abort");
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ServerError::BadRequest(message)) if message == "Invalid control action: abort"
        ));
    }
}

/// Compute SHA-256 hex digest of a string.
pub fn sha256_hex(input: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}
