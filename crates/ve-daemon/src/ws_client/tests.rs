//! Tests for the WebSocket client module.

use std::sync::Arc;

use std::time::Duration;

use super::utils::calculate_backoff;
use super::*;
use tempfile::tempdir;

#[test]
fn test_calculate_backoff_first_retry() {
    let min = Duration::from_millis(1000);
    let max = Duration::from_millis(30000);

    // Full jitter: uniform random in [0, capped_exponential].
    // For retry 1: capped = 1000 * 2^0 = 1000ms.
    let backoff = calculate_backoff(min, max, 1);
    assert!(backoff <= min); // Never exceeds the exponential cap
}

#[test]
fn test_calculate_backoff_caps_at_max() {
    let min = Duration::from_millis(1000);
    let max = Duration::from_millis(5000);

    // Full jitter: random(0, min(max, base * 2^(retry-1))).
    // Even with high retry count, should be capped at max.
    let backoff = calculate_backoff(min, max, 10);
    assert!(backoff <= max); // Always within cap
}

#[test]
fn test_calculate_backoff_respects_retry_growth() {
    let min = Duration::from_millis(1000);
    let max = Duration::from_millis(30000);

    // The expected value (mean) grows exponentially even though
    // individual samples are random.
    let mut total1 = 0u64;
    let mut total3 = 0u64;
    let iterations = 500u64;
    for _ in 0..iterations {
        total1 += calculate_backoff(min, max, 1).as_millis() as u64;
        total3 += calculate_backoff(min, max, 3).as_millis() as u64;
    }
    // Average of retry 3 (cap = 4000ms) should be ~4x retry 1 (cap = 1000ms)
    let avg1 = total1 / iterations;
    let avg3 = total3 / iterations;
    assert!(avg3 > avg1, "avg retry 3 ({avg3}ms) should exceed avg retry 1 ({avg1}ms)");
}

#[test]
fn handle_driver_event_emits_dedicated_permission_request_with_same_id() {
    let client = WsClient::new(
        Arc::new(crate::config::Config {
            server_url: "https://example.com".to_string(),
            host_name: "host".to_string(),
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
        }),
        Uuid::new_v4(),
        "token".to_string(),
    );
    let permission_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();

    let messages = client.handle_driver_event(DriverEvent::PermissionRequest {
        permission_id,
        session_id,
        risk_type: "write_fs".to_string(),
        summary: "need access".to_string(),
        target: Some("/tmp".to_string()),
    });

    assert_eq!(messages.len(), 1);
    let (msg_type, payload) = &messages[0];
    assert_eq!(msg_type, "permission_request");
    assert_eq!(payload["permission_id"], permission_id.to_string());
    assert_eq!(payload["session_id"], session_id.to_string());
}

#[tokio::test]
async fn ensure_workspace_directory_creates_missing_absolute_path() {
    let dir = tempdir().unwrap();
    let workspace = dir.path().join("new-workspace");

    super::utils::ensure_workspace_directory(workspace.to_str().unwrap())
        .await
        .unwrap();

    assert!(workspace.exists());
    assert!(workspace.is_dir());
}

#[tokio::test]
async fn ensure_workspace_directory_rejects_relative_path() {
    let error =
        super::utils::ensure_workspace_directory("relative/workspace")
            .await
            .unwrap_err();

    assert!(matches!(error, DaemonError::WorkspaceInvalid { .. }));
}

#[tokio::test]
async fn ensure_workspace_directory_rejects_existing_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("not-a-directory");
    std::fs::write(&file_path, "hello").unwrap();

    let error =
        super::utils::ensure_workspace_directory(file_path.to_str().unwrap())
            .await
            .unwrap_err();

    assert!(matches!(error, DaemonError::WorkspaceInvalid { .. }));
}
