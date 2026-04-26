//! Tests for the session runner module.

use std::sync::Arc;

use tokio::sync::mpsc;
use uuid::Uuid;

use super::*;
use crate::agent::DriverEvent;
use crate::config::Config;
use crate::error::DaemonError;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::SessionControlAction;
use ve_shared::types::{CloseReason, SessionStatus};
use super::glob_match::matches_pattern;

    fn create_test_config(mock_mode: bool) -> Arc<Config> {
        Arc::new(Config {
            server_url: "https://test.com".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: std::path::PathBuf::from("/tmp"),
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 4,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            permission_mode: "default".to_string(),
            mock_mode,
        })
    }

    #[test]
    fn test_build_driver_config_preserves_initial_message() {
        let config = Arc::new(Config {
            server_url: "https://test.com".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: std::path::PathBuf::from("/tmp"),
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 4,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            permission_mode: "default".to_string(),
            mock_mode: false,
        });
        let (event_tx, _event_rx) = mpsc::channel(16);
        let session_id = Uuid::new_v4();
        let (runner, _handle) = SessionRunner::new(
            session_id,
            "/workspace".to_string(),
            "claude_code".to_string(),
            Some("Hello".to_string()),
            config,
            event_tx,
            None,
        ).unwrap();

        let driver_config = runner.build_driver_config();

        assert_eq!(driver_config.session_id, session_id);
        assert_eq!(driver_config.initial_message, Some("Hello".to_string()));
        assert_eq!(driver_config.workspace_path, "/workspace");
    }

    #[test]
    fn rerun_control_is_rejected_for_live_runner_path() {
        let error = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                let config = Arc::new(Config {
                    server_url: "https://test.com".to_string(),
                    host_name: "test".to_string(),
                    platform: "linux".to_string(),
                    config_dir: std::path::PathBuf::from("/tmp"),
                    log_format: "pretty".to_string(),
                    log_level: "info".to_string(),
                    heartbeat_interval_secs: 30,
                    heartbeat_timeout_secs: 90,
                    ack_timeout_secs: 30,
                    permission_timeout_secs: 60,
                    reconnect_backoff_min_ms: 1000,
                    reconnect_backoff_max_ms: 30000,
                    max_parallel_sessions: 4,
                    file_read_text_limit_bytes: 262_144,
                    file_tree_max_nodes: 20_000,
                    claude_command: "claude".to_string(),
                    default_model: "claude-sonnet-4-20250514".to_string(),
                    permission_mode: "default".to_string(),
                    mock_mode: false,
                });
                let (event_tx, _event_rx) = mpsc::channel(16);
                let session_id = Uuid::new_v4();
                let (mut runner, _handle) = SessionRunner::new(
                    session_id,
                    "/workspace".to_string(),
                    "claude_code".to_string(),
                    None,
                    config,
                    event_tx,
                    None,
                ).unwrap();
                runner.handle_control(SessionControlAction::Rerun).await
            })
            .unwrap_err();

        assert!(matches!(error, DaemonError::SessionInvalidStatus { .. }));
    }

    #[tokio::test]
    async fn terminate_reports_archived_with_terminated_reason() {
        let config = Arc::new(Config {
            server_url: "https://test.com".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: std::path::PathBuf::from("/tmp"),
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 4,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            permission_mode: "default".to_string(),
            mock_mode: false,
        });
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let session_id = Uuid::new_v4();
        let (mut runner, _handle) = SessionRunner::new(
            session_id,
            "/workspace".to_string(),
            "claude_code".to_string(),
            None,
            config,
            event_tx,
            None,
        ).unwrap();
        runner.state = RunnerState::Running;

        runner
            .handle_control(SessionControlAction::Terminate)
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        assert!(matches!(
            event,
            DriverEvent::StatusUpdate {
                session_id: actual_session_id,
                status: SessionStatus::Archived,
                close_reason: Some(CloseReason::Terminated),
                ..
            } if actual_session_id == session_id
        ));
        assert_eq!(runner.state, RunnerState::Closed);
    }

    #[tokio::test]
    async fn close_reports_archived_with_user_closed_reason() {
        let config = create_test_config(false);
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let session_id = Uuid::new_v4();
        let (mut runner, _handle) = SessionRunner::new(
            session_id,
            "/workspace".to_string(),
            "claude_code".to_string(),
            None,
            config,
            event_tx,
            None,
        ).unwrap();
        runner.state = RunnerState::Running;

        runner.handle_close().await.unwrap();

        let event = event_rx.recv().await.unwrap();
        assert!(matches!(
            event,
            DriverEvent::StatusUpdate {
                session_id: actual_session_id,
                status: SessionStatus::Archived,
                close_reason: Some(CloseReason::UserClosed),
                ..
            } if actual_session_id == session_id
        ));
        assert_eq!(runner.state, RunnerState::Closed);
    }

    #[tokio::test]
    async fn register_permission_transitions_runner_to_waiting_approval() {
        let config = create_test_config(true);
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let session_id = Uuid::new_v4();
        let permission_id = Uuid::new_v4();
        let (mut runner, _handle) = SessionRunner::new(
            session_id,
            "/workspace".to_string(),
            "claude_code".to_string(),
            None,
            config,
            event_tx,
            None,
        ).unwrap();
        runner.state = RunnerState::Running;

        runner
            .handle_command(RunnerCommand::RegisterPermission {
                permission_id,
                risk_type: "exec_cmd".to_string(),
                target: Some("/tmp/mock-command".to_string()),
                summary: "needs approval".to_string(),
                bridge_response: None,
            })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        assert!(matches!(
            event,
            DriverEvent::StatusUpdate {
                session_id: actual_session_id,
                status: SessionStatus::WaitingApproval,
                close_reason: None,
                ..
            } if actual_session_id == session_id
        ));
        assert_eq!(runner.state, RunnerState::WaitingApproval);
        assert!(runner.pending_permissions.contains_key(&permission_id));
    }

    #[tokio::test]
    async fn permission_response_transitions_runner_back_to_running_after_last_pending() {
        let config = create_test_config(true);
        let (event_tx, mut event_rx) = mpsc::channel(16);
        let session_id = Uuid::new_v4();
        let permission_id = Uuid::new_v4();
        let (mut runner, _handle) = SessionRunner::new(
            session_id,
            "/workspace".to_string(),
            "claude_code".to_string(),
            None,
            config,
            event_tx,
            None,
        ).unwrap();
        runner.state = RunnerState::Running;

        runner
            .handle_command(RunnerCommand::RegisterPermission {
                permission_id,
                risk_type: "exec_cmd".to_string(),
                target: Some("/tmp/mock-command".to_string()),
                summary: "needs approval".to_string(),
                bridge_response: None,
            })
            .await
            .unwrap();
        let waiting_event = event_rx.recv().await.unwrap();
        assert!(matches!(
            waiting_event,
            DriverEvent::StatusUpdate {
                status: SessionStatus::WaitingApproval,
                ..
            }
        ));

        runner
            .handle_command(RunnerCommand::PermissionResponse {
                permission_id,
                decision: PermissionDecision::DenyOnce,
            })
            .await
            .unwrap();

        let resumed_event = event_rx.recv().await.unwrap();
        assert!(matches!(
            resumed_event,
            DriverEvent::StatusUpdate {
                session_id: actual_session_id,
                status: SessionStatus::Running,
                close_reason: None,
                ..
            } if actual_session_id == session_id
        ));
        assert_eq!(runner.state, RunnerState::Running);
        assert!(runner.pending_permissions.is_empty());
    }

    #[test]
    fn test_approval_rule_debug() {
        let rule = ApprovalRule {
            risk_type: "write_fs".to_string(),
            target_pattern: "*".to_string(),
        };
        assert!(format!("{:?}", rule).contains("write_fs"));
    }

    #[test]
    fn test_pending_permission_stores_metadata() {
        let pending = PendingPermission {
            risk_type: "write_fs".to_string(),
            target: Some("/workspace/test.txt".to_string()),
            summary: "Write to file".to_string(),
            bridge_response: None,
        };
        assert_eq!(pending.risk_type, "write_fs");
        assert_eq!(pending.target, Some("/workspace/test.txt".to_string()));
    }

    #[test]
    fn test_approval_rule_from_pending_permission() {
        let pending = PendingPermission {
            risk_type: "execute_bash".to_string(),
            target: Some("npm test".to_string()),
            summary: "Run tests".to_string(),
            bridge_response: None,
        };
        let rule = ApprovalRule {
            risk_type: pending.risk_type.clone(),
            target_pattern: pending.target.clone().unwrap_or("*".to_string()),
        };
        assert_eq!(rule.risk_type, "execute_bash");
        assert_eq!(rule.target_pattern, "npm test");
    }

    // ========== matches_pattern tests ==========

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(matches_pattern("*", "anything"));
        assert!(matches_pattern("*", ""));
        assert!(matches_pattern("*", "/some/long/path"));
    }

    #[test]
    fn test_matches_pattern_extension() {
        assert!(matches_pattern("*.txt", "file.txt"));
        assert!(matches_pattern("*.txt", "test.txt"));
        assert!(!matches_pattern("*.txt", "file.rs"));
        assert!(!matches_pattern("*.txt", "txt"));
    }

    #[test]
    fn test_matches_pattern_question() {
        assert!(matches_pattern("file?.txt", "file1.txt"));
        assert!(matches_pattern("file?.txt", "fileA.txt"));
        assert!(!matches_pattern("file?.txt", "file.txt"));
        assert!(!matches_pattern("file?.txt", "file12.txt"));
    }

    #[test]
    fn test_matches_pattern_path() {
        assert!(matches_pattern("/home/user/*", "/home/user/project"));
        assert!(matches_pattern("/home/user/*", "/home/user/"));
        assert!(!matches_pattern("/home/user/*", "/home/other/file"));
    }

    #[test]
    fn test_matches_pattern_empty() {
        assert!(matches_pattern("", ""));
        assert!(!matches_pattern("", "x"));
        assert!(!matches_pattern("x", ""));
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("exact", "exact"));
        assert!(!matches_pattern("exact", "Exact"));
        assert!(!matches_pattern("exact", "exact "));
    }

    #[test]
    fn test_matches_pattern_multiple_wildcards() {
        assert!(matches_pattern("*.rs", "lib.rs"));
        // Note: Our simple glob doesn't support ** special behavior
        // * matches any characters including /, so this works:
        assert!(matches_pattern("src/*", "src/main.rs"));
        assert!(matches_pattern("src/*", "src/lib/mod.rs"));
        // Pattern with multiple * works:
        assert!(matches_pattern("*.txt", "file.txt"));
        assert!(matches_pattern("*test*", "mytestfile"));
    }

    #[tokio::test]
    async fn send_message_and_wait_times_out_without_runner_completion() {
        let (command_tx, mut command_rx) = mpsc::channel(1);
        let handle = SessionRunnerHandle {
            command_tx,
            state: RunnerState::Running,
            session_id: Uuid::new_v4(),
        };

        let request = tokio::spawn(async move {
            handle
                .send_message_and_wait("hello".to_string(), std::time::Duration::from_millis(20))
                .await
        });

        let command = command_rx.recv().await.expect("runner command");
        match command {
            RunnerCommand::SendMessage {
                content,
                completion: Some(_),
            } => assert_eq!(content, "hello"),
            other => panic!("unexpected command: {other:?}"),
        }

        let error = request.await.unwrap().unwrap_err();
        assert!(matches!(
            error,
            DaemonError::SessionInvalidStatus {
                current,
                expected,
            } if current == "timeout" && expected == "command completion"
        ));
    }
