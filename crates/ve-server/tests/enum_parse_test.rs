//! Tests for enum parsing with warning logs
//!
//! Tests for parse_* helper functions that log warnings for unknown values.

/// Simulated enum types for testing
#[derive(Debug, Clone, Copy, PartialEq)]
enum TestSessionStatus {
    Running,
    Pending,
    Dispatching,
    WaitingApproval,
    Paused,
    Error,
    Closing,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestRiskType {
    WriteFs,
    ExecCmd,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestPermissionStatus {
    Pending,
    ApprovedOnce,
    DeniedOnce,
    ApprovedSession,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestCloseReason {
    UserClosed,
    Completed,
    Failed,
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestPlatform {
    Linux,
    Macos,
    Windows,
    Wsl,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestOnlineStatus {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestDaemonStatus {
    Healthy,
    Connecting,
    Error,
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestPairStatus {
    Paired,
    Pending,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestMessageType {
    User,
    Assistant,
    System,
    Tool,
    Error,
    Permission,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TestControlAction {
    Pause,
    Terminate,
    Interrupt,
    Rerun,
    Restart,
}

/// Parse result containing the parsed value and whether it was unknown
#[derive(Debug)]
struct ParsedEnum<T> {
    value: T,
    was_unknown: bool,
    original_input: String,
}

impl<T> std::ops::Deref for ParsedEnum<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Parse session status with unknown detection
fn parse_session_status(s: &str) -> ParsedEnum<TestSessionStatus> {
    let (value, was_unknown) = match s {
        "running" => (TestSessionStatus::Running, false),
        "pending" => (TestSessionStatus::Pending, false),
        "dispatching" => (TestSessionStatus::Dispatching, false),
        "waiting_approval" => (TestSessionStatus::WaitingApproval, false),
        "paused" => (TestSessionStatus::Paused, false),
        "error" => (TestSessionStatus::Error, false),
        "closing" => (TestSessionStatus::Closing, false),
        "archived" => (TestSessionStatus::Archived, false),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown session status, defaulting to Running");
            (TestSessionStatus::Running, true)
        }
    };
    ParsedEnum {
        value,
        was_unknown,
        original_input: s.to_string(),
    }
}

/// Parse risk type with unknown detection
fn parse_risk_type(s: &str) -> ParsedEnum<TestRiskType> {
    let (value, was_unknown) = match s {
        "write_fs" => (TestRiskType::WriteFs, false),
        "exec_cmd" => (TestRiskType::ExecCmd, false),
        "network" => (TestRiskType::Network, false),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown risk type, defaulting to WriteFs");
            (TestRiskType::WriteFs, true)
        }
    };
    ParsedEnum {
        value,
        was_unknown,
        original_input: s.to_string(),
    }
}

/// Parse platform with unknown detection
fn parse_platform(s: &str) -> ParsedEnum<TestPlatform> {
    let (value, was_unknown) = match s {
        "linux" => (TestPlatform::Linux, false),
        "macos" => (TestPlatform::Macos, false),
        "windows" => (TestPlatform::Windows, false),
        "wsl" => (TestPlatform::Wsl, false),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown platform, defaulting to Linux");
            (TestPlatform::Linux, true)
        }
    };
    ParsedEnum {
        value,
        was_unknown,
        original_input: s.to_string(),
    }
}

/// Parse online status with unknown detection
fn parse_online_status(s: &str) -> ParsedEnum<TestOnlineStatus> {
    let (value, was_unknown) = match s {
        "online" => (TestOnlineStatus::Online, false),
        "offline" => (TestOnlineStatus::Offline, false),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown online status, defaulting to Unknown");
            (TestOnlineStatus::Unknown, true)
        }
    };
    ParsedEnum {
        value,
        was_unknown,
        original_input: s.to_string(),
    }
}

/// Parse daemon status with unknown detection
fn parse_daemon_status(s: &str) -> ParsedEnum<TestDaemonStatus> {
    let (value, was_unknown) = match s {
        "healthy" => (TestDaemonStatus::Healthy, false),
        "connecting" => (TestDaemonStatus::Connecting, false),
        "error" => (TestDaemonStatus::Error, false),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown daemon status, defaulting to Disconnected");
            (TestDaemonStatus::Disconnected, true)
        }
    };
    ParsedEnum {
        value,
        was_unknown,
        original_input: s.to_string(),
    }
}

/// Parse pair status with unknown detection
fn parse_pair_status(s: &str) -> ParsedEnum<TestPairStatus> {
    let (value, was_unknown) = match s {
        "paired" => (TestPairStatus::Paired, false),
        "failed" => (TestPairStatus::Failed, false),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown pair status, defaulting to Pending");
            (TestPairStatus::Pending, true)
        }
    };
    ParsedEnum {
        value,
        was_unknown,
        original_input: s.to_string(),
    }
}

/// Parse permission status with unknown detection
fn parse_permission_status(s: &str) -> ParsedEnum<TestPermissionStatus> {
    let (value, was_unknown) = match s {
        "pending" => (TestPermissionStatus::Pending, false),
        "approved_once" => (TestPermissionStatus::ApprovedOnce, false),
        "denied_once" => (TestPermissionStatus::DeniedOnce, false),
        "approved_session" => (TestPermissionStatus::ApprovedSession, false),
        "expired" => (TestPermissionStatus::Expired, false),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown permission status, defaulting to Pending");
            (TestPermissionStatus::Pending, true)
        }
    };
    ParsedEnum {
        value,
        was_unknown,
        original_input: s.to_string(),
    }
}

/// Parse close reason with unknown detection
fn parse_close_reason(s: &str) -> ParsedEnum<TestCloseReason> {
    let (value, was_unknown) = match s {
        "user_closed" => (TestCloseReason::UserClosed, false),
        "completed" => (TestCloseReason::Completed, false),
        "failed" => (TestCloseReason::Failed, false),
        "terminated" => (TestCloseReason::Terminated, false),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown close reason, defaulting to UserClosed");
            (TestCloseReason::UserClosed, true)
        }
    };
    ParsedEnum {
        value,
        was_unknown,
        original_input: s.to_string(),
    }
}

/// Parse message type with unknown detection
fn parse_message_type(s: &str) -> ParsedEnum<TestMessageType> {
    let (value, was_unknown) = match s {
        "user" => (TestMessageType::User, false),
        "assistant" => (TestMessageType::Assistant, false),
        "system" => (TestMessageType::System, false),
        "tool" => (TestMessageType::Tool, false),
        "error" => (TestMessageType::Error, false),
        "permission" => (TestMessageType::Permission, false),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown message type, defaulting to System");
            (TestMessageType::System, true)
        }
    };
    ParsedEnum {
        value,
        was_unknown,
        original_input: s.to_string(),
    }
}

/// Parse control action with unknown detection
fn parse_control_action(s: &str) -> Result<ParsedEnum<TestControlAction>, String> {
    match s {
        "pause" => Ok(ParsedEnum {
            value: TestControlAction::Pause,
            was_unknown: false,
            original_input: s.to_string(),
        }),
        "terminate" => Ok(ParsedEnum {
            value: TestControlAction::Terminate,
            was_unknown: false,
            original_input: s.to_string(),
        }),
        "interrupt" => Ok(ParsedEnum {
            value: TestControlAction::Interrupt,
            was_unknown: false,
            original_input: s.to_string(),
        }),
        "rerun" => Ok(ParsedEnum {
            value: TestControlAction::Rerun,
            was_unknown: false,
            original_input: s.to_string(),
        }),
        "restart" => Ok(ParsedEnum {
            value: TestControlAction::Restart,
            was_unknown: false,
            original_input: s.to_string(),
        }),
        unknown => {
            tracing::warn!(input = %unknown, "Unknown control action");
            Err(format!("Invalid control action: {}", unknown))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Session Status Tests
    #[test]
    fn parse_valid_session_status_running() {
        let result = parse_session_status("running");
        assert_eq!(*result, TestSessionStatus::Running);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_session_status_dispatching() {
        let result = parse_session_status("dispatching");
        assert_eq!(*result, TestSessionStatus::Dispatching);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_session_status_waiting_approval() {
        let result = parse_session_status("waiting_approval");
        assert_eq!(*result, TestSessionStatus::WaitingApproval);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_session_status_archived() {
        let result = parse_session_status("archived");
        assert_eq!(*result, TestSessionStatus::Archived);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_session_status_defaults_to_running() {
        let result = parse_session_status("unknown_status");
        assert_eq!(*result, TestSessionStatus::Running);
        assert!(result.was_unknown);
        assert_eq!(result.original_input, "unknown_status");
    }

    #[test]
    fn parse_empty_session_status_defaults_to_running() {
        let result = parse_session_status("");
        assert_eq!(*result, TestSessionStatus::Running);
        assert!(result.was_unknown);
    }

    #[test]
    fn parse_case_mismatch_session_status_is_unknown() {
        let result = parse_session_status("RUNNING");
        assert_eq!(*result, TestSessionStatus::Running);
        assert!(result.was_unknown); // Case-sensitive!
    }

    // Risk Type Tests
    #[test]
    fn parse_valid_risk_type_write_fs() {
        let result = parse_risk_type("write_fs");
        assert_eq!(*result, TestRiskType::WriteFs);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_risk_type_exec_cmd() {
        let result = parse_risk_type("exec_cmd");
        assert_eq!(*result, TestRiskType::ExecCmd);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_risk_type_network() {
        let result = parse_risk_type("network");
        assert_eq!(*result, TestRiskType::Network);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_risk_type_defaults_to_write_fs() {
        let result = parse_risk_type("unknown_type");
        assert_eq!(*result, TestRiskType::WriteFs);
        assert!(result.was_unknown);
    }

    // Platform Tests
    #[test]
    fn parse_valid_platform_linux() {
        let result = parse_platform("linux");
        assert_eq!(*result, TestPlatform::Linux);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_platform_macos() {
        let result = parse_platform("macos");
        assert_eq!(*result, TestPlatform::Macos);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_platform_wsl() {
        let result = parse_platform("wsl");
        assert_eq!(*result, TestPlatform::Wsl);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_platform_windows() {
        let result = parse_platform("windows");
        assert_eq!(*result, TestPlatform::Windows);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_platform_defaults_to_linux() {
        let result = parse_platform("unknown");
        assert_eq!(*result, TestPlatform::Linux);
        assert!(result.was_unknown);
    }

    // Online Status Tests
    #[test]
    fn parse_valid_online_status_online() {
        let result = parse_online_status("online");
        assert_eq!(*result, TestOnlineStatus::Online);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_online_status_offline() {
        let result = parse_online_status("offline");
        assert_eq!(*result, TestOnlineStatus::Offline);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_online_status_defaults_to_unknown() {
        let result = parse_online_status("away");
        assert_eq!(*result, TestOnlineStatus::Unknown);
        assert!(result.was_unknown);
    }

    // Daemon Status Tests
    #[test]
    fn parse_valid_daemon_status_healthy() {
        let result = parse_daemon_status("healthy");
        assert_eq!(*result, TestDaemonStatus::Healthy);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_daemon_status_error() {
        let result = parse_daemon_status("error");
        assert_eq!(*result, TestDaemonStatus::Error);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_daemon_status_defaults_to_disconnected() {
        let result = parse_daemon_status("crashed");
        assert_eq!(*result, TestDaemonStatus::Disconnected);
        assert!(result.was_unknown);
    }

    // Pair Status Tests
    #[test]
    fn parse_valid_pair_status_paired() {
        let result = parse_pair_status("paired");
        assert_eq!(*result, TestPairStatus::Paired);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_pair_status_defaults_to_pending() {
        let result = parse_pair_status("unknown");
        assert_eq!(*result, TestPairStatus::Pending);
        assert!(result.was_unknown);
    }

    // Permission Status Tests
    #[test]
    fn parse_valid_permission_status_pending() {
        let result = parse_permission_status("pending");
        assert_eq!(*result, TestPermissionStatus::Pending);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_permission_status_approved_once() {
        let result = parse_permission_status("approved_once");
        assert_eq!(*result, TestPermissionStatus::ApprovedOnce);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_permission_status_defaults_to_pending() {
        let result = parse_permission_status("revoked");
        assert_eq!(*result, TestPermissionStatus::Pending);
        assert!(result.was_unknown);
    }

    // Close Reason Tests
    #[test]
    fn parse_valid_close_reason_user_closed() {
        let result = parse_close_reason("user_closed");
        assert_eq!(*result, TestCloseReason::UserClosed);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_close_reason_completed() {
        let result = parse_close_reason("completed");
        assert_eq!(*result, TestCloseReason::Completed);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_close_reason_defaults_to_user_closed() {
        let result = parse_close_reason("timeout");
        assert_eq!(*result, TestCloseReason::UserClosed);
        assert!(result.was_unknown);
    }

    // Message Type Tests
    #[test]
    fn parse_valid_message_type_user() {
        let result = parse_message_type("user");
        assert_eq!(*result, TestMessageType::User);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_message_type_defaults_to_system() {
        let result = parse_message_type("debug");
        assert_eq!(*result, TestMessageType::System);
        assert!(result.was_unknown);
    }

    // Control Action Tests
    #[test]
    fn parse_valid_control_action_pause() {
        let result = parse_control_action("pause").unwrap();
        assert_eq!(*result, TestControlAction::Pause);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_valid_control_action_terminate() {
        let result = parse_control_action("terminate").unwrap();
        assert_eq!(*result, TestControlAction::Terminate);
        assert!(!result.was_unknown);
    }

    #[test]
    fn parse_unknown_control_action_returns_error() {
        let result = parse_control_action("abort");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid control action: abort");
    }
}
