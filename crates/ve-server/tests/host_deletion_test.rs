//! Tests for host deletion validation

use ve_server::validation::{validate_host_can_be_deleted, HostDeletionStatus};

/// Test that host with no dependencies can be deleted
#[test]
fn validate_host_can_be_deleted_no_dependencies() {
    let status = HostDeletionStatus {
        session_count: 0,
        archive_count: 0,
        workspace_count: 0,
    };
    assert!(validate_host_can_be_deleted(&status));
}

/// Test that host with active sessions cannot be deleted
#[test]
fn validate_host_cannot_be_deleted_with_sessions() {
    let status = HostDeletionStatus {
        session_count: 5,
        archive_count: 0,
        workspace_count: 0,
    };
    assert!(!validate_host_can_be_deleted(&status));
}

/// Test that host with archives cannot be deleted
#[test]
fn validate_host_cannot_be_deleted_with_archives() {
    let status = HostDeletionStatus {
        session_count: 0,
        archive_count: 3,
        workspace_count: 0,
    };
    assert!(!validate_host_can_be_deleted(&status));
}

/// Test that host with only workspaces can be deleted (cascaded)
#[test]
fn validate_host_can_be_deleted_with_workspaces_only() {
    let status = HostDeletionStatus {
        session_count: 0,
        archive_count: 0,
        workspace_count: 2,
    };
    assert!(validate_host_can_be_deleted(&status));
}

/// Test that sessions take precedence over workspaces
#[test]
fn validate_host_cannot_be_deleted_with_sessions_and_workspaces() {
    let status = HostDeletionStatus {
        session_count: 1,
        archive_count: 0,
        workspace_count: 5,
    };
    assert!(!validate_host_can_be_deleted(&status));
}
