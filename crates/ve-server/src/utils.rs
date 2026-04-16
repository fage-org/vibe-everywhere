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

/// Parse session status string with warning for unknown values
pub fn parse_session_status(s: &str) -> ve_shared::types::SessionStatus {
    match s {
        "running" => ve_shared::types::SessionStatus::Running,
        "waiting_approval" => ve_shared::types::SessionStatus::WaitingApproval,
        "paused" => ve_shared::types::SessionStatus::Paused,
        "error" => ve_shared::types::SessionStatus::Error,
        "closing" => ve_shared::types::SessionStatus::Closing,
        "archived" => ve_shared::types::SessionStatus::Archived,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown session status, defaulting to Running");
            ve_shared::types::SessionStatus::Running
        }
    }
}

/// Parse message type string with warning for unknown values
pub fn parse_message_type(s: &str) -> ve_shared::models::SessionMessageType {
    match s {
        "user" => ve_shared::models::SessionMessageType::User,
        "assistant" => ve_shared::models::SessionMessageType::Assistant,
        "system" => ve_shared::models::SessionMessageType::System,
        "tool" => ve_shared::models::SessionMessageType::Tool,
        "error" => ve_shared::models::SessionMessageType::Error,
        "permission" => ve_shared::models::SessionMessageType::Permission,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown message type, defaulting to System");
            ve_shared::models::SessionMessageType::System
        }
    }
}

/// Parse control action string with warning for unknown values
pub fn parse_control_action(s: &str) -> ve_shared::proto::SessionControlAction {
    match s {
        "pause" => ve_shared::proto::SessionControlAction::Pause,
        "terminate" => ve_shared::proto::SessionControlAction::Terminate,
        "interrupt" => ve_shared::proto::SessionControlAction::Interrupt,
        "rerun" => ve_shared::proto::SessionControlAction::Rerun,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown control action, defaulting to Interrupt");
            ve_shared::proto::SessionControlAction::Interrupt
        }
    }
}

/// Parse risk type string with warning for unknown values
pub fn parse_risk_type(s: &str) -> ve_shared::types::RiskType {
    match s {
        "write_fs" => ve_shared::types::RiskType::WriteFs,
        "exec_cmd" => ve_shared::types::RiskType::ExecCmd,
        "network" => ve_shared::types::RiskType::Network,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown risk type, defaulting to WriteFs");
            ve_shared::types::RiskType::WriteFs
        }
    }
}

/// Parse permission status string with warning for unknown values
pub fn parse_permission_status(s: &str) -> ve_shared::types::PermissionStatus {
    match s {
        "pending" => ve_shared::types::PermissionStatus::Pending,
        "approved_once" => ve_shared::types::PermissionStatus::ApprovedOnce,
        "denied_once" => ve_shared::types::PermissionStatus::DeniedOnce,
        "approved_session" => ve_shared::types::PermissionStatus::ApprovedSession,
        "expired" => ve_shared::types::PermissionStatus::Expired,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown permission status, defaulting to Pending");
            ve_shared::types::PermissionStatus::Pending
        }
    }
}

/// Parse close reason string with warning for unknown values
pub fn parse_close_reason(s: &str) -> ve_shared::types::CloseReason {
    match s {
        "user_closed" => ve_shared::types::CloseReason::UserClosed,
        "completed" => ve_shared::types::CloseReason::Completed,
        "failed" => ve_shared::types::CloseReason::Failed,
        "terminated" => ve_shared::types::CloseReason::Terminated,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown close reason, defaulting to UserClosed");
            ve_shared::types::CloseReason::UserClosed
        }
    }
}

/// Parse platform string with warning for unknown values
pub fn parse_platform(s: &str) -> ve_shared::types::Platform {
    match s {
        "linux" => ve_shared::types::Platform::Linux,
        "macos" => ve_shared::types::Platform::Macos,
        "wsl" => ve_shared::types::Platform::Wsl,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown platform, defaulting to Linux");
            ve_shared::types::Platform::Linux
        }
    }
}

/// Parse online status string with warning for unknown values
pub fn parse_online_status(s: &str) -> ve_shared::types::OnlineStatus {
    match s {
        "online" => ve_shared::types::OnlineStatus::Online,
        "offline" => ve_shared::types::OnlineStatus::Offline,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown online status, defaulting to Unknown");
            ve_shared::types::OnlineStatus::Unknown
        }
    }
}

/// Parse daemon status string with warning for unknown values
pub fn parse_daemon_status(s: &str) -> ve_shared::types::DaemonStatus {
    match s {
        "healthy" => ve_shared::types::DaemonStatus::Healthy,
        "connecting" => ve_shared::types::DaemonStatus::Connecting,
        "error" => ve_shared::types::DaemonStatus::Error,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown daemon status, defaulting to Disconnected");
            ve_shared::types::DaemonStatus::Disconnected
        }
    }
}

/// Parse pair status string with warning for unknown values
pub fn parse_pair_status(s: &str) -> ve_shared::types::PairStatus {
    match s {
        "paired" => ve_shared::types::PairStatus::Paired,
        "failed" => ve_shared::types::PairStatus::Failed,
        unknown => {
            tracing::warn!(input = %unknown, "Unknown pair status, defaulting to Pending");
            ve_shared::types::PairStatus::Pending
        }
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
    fn test_parse_session_status_unknown() {
        let status = parse_session_status("unknown_status");
        assert_eq!(status, ve_shared::types::SessionStatus::Running);
    }

    #[test]
    fn test_parse_session_status_valid() {
        assert_eq!(
            parse_session_status("running"),
            ve_shared::types::SessionStatus::Running
        );
        assert_eq!(
            parse_session_status("archived"),
            ve_shared::types::SessionStatus::Archived
        );
    }
}
