//! Tests for the archive API module.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::Json;

use super::*;
use crate::config::{Config, DatabaseBackend};
use crate::db::{install_drivers, run_migrations, DbPool};
use crate::hub::Hub;
use crate::state::AppState;
use ve_shared::jwt::Claims;

fn test_config(database_url: String) -> Config {
    Config {
        listen_addr: "127.0.0.1:3000".parse().unwrap(),
        database_url,
        jwt_secret: "01234567890123456789012345678901".to_string(),
        jwt_expiration_secs: 3600,
        pair_code_ttl_secs: 300,
        heartbeat_interval_secs: 30,
        connection_timeout_secs: 60,
        data_dir: std::path::PathBuf::from("/tmp"),
        cors_origins: Vec::new(),
        ack_timeout_ms: 10000,
        ack_max_retries: 2,
        ack_retry_delay_ms: 500,
        permission_ttl_secs: 1800,
        permission_expiry_check_secs: 60,
        idempotency_ttl_secs: 86400,
        idempotency_cleanup_secs: 3600,
        log_format: "pretty".to_string(),
        log_level: "info".to_string(),
    }
}

async fn setup_state() -> Arc<AppState> {
    install_drivers();
    let temp_db =
        std::env::temp_dir().join(format!("ve-archives-api-test-{}.db", Uuid::new_v4()));
    let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
    let pool = DbPool::connect(&database_url).await.unwrap();
    run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap();

    let config = test_config(database_url);
    let jwt_manager = Arc::new(ve_shared::jwt::JwtManager::new(
        &config.jwt_secret,
        config.jwt_expiration(),
    ));
    Arc::new(AppState::new(pool, Hub::new(), config, jwt_manager))
}

async fn insert_archive_fixture(
    state: &Arc<AppState>,
    device_id: Uuid,
    host_id: Uuid,
    workspace_id: Uuid,
    session_id: Uuid,
    archive_id: Uuid,
) {
    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
    )
    .bind(device_id.to_string())
    .bind("device")
    .bind("desktop")
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
        .bind(host_id.to_string())
        .bind("host")
        .bind("linux")
        .execute(&state.db)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace_id.to_string())
    .bind(host_id.to_string())
    .bind("/tmp/project")
    .bind("project")
    .execute(&state.db)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
    )
    .bind(session_id.to_string())
    .bind("archived")
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind("claude_code")
    .bind(&now)
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(host_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(session_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(archive_id.to_string())
    .bind(session_id.to_string())
    .bind("archived")
    .bind(&now)
    .bind("user_closed")
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind(Option::<String>::None)
    .bind(&now)
    .execute(&state.db)
    .await
    .unwrap();
}

async fn insert_archive_for_existing_scope(
    state: &Arc<AppState>,
    device_id: Uuid,
    host_id: Uuid,
    workspace_id: Uuid,
    session_id: Uuid,
    archive_id: Uuid,
    title: &str,
) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
    )
    .bind(session_id.to_string())
    .bind(title)
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind("claude_code")
    .bind(&now)
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(session_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(archive_id.to_string())
    .bind(session_id.to_string())
    .bind(title)
    .bind(&now)
    .bind("user_closed")
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind(Option::<String>::None)
    .bind(&now)
    .execute(&state.db)
    .await
    .unwrap();
}

#[tokio::test]
async fn list_archives_excludes_sessions_without_session_access() {
    let state = setup_state().await;
    let visible_device_id = Uuid::new_v4();
    let hidden_device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let visible_session_id = Uuid::new_v4();
    let visible_archive_id = Uuid::new_v4();
    let hidden_session_id = Uuid::new_v4();
    let hidden_archive_id = Uuid::new_v4();

    insert_archive_fixture(
        &state,
        visible_device_id,
        host_id,
        workspace_id,
        visible_session_id,
        visible_archive_id,
    )
    .await;

    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
    )
    .bind(hidden_device_id.to_string())
    .bind("other")
    .bind("desktop")
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
        .bind(hidden_device_id.to_string())
        .bind(host_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
    )
    .bind(hidden_session_id.to_string())
    .bind("hidden")
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind("claude_code")
    .bind(&now)
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(hidden_archive_id.to_string())
    .bind(hidden_session_id.to_string())
    .bind("hidden")
    .bind(&now)
    .bind("user_closed")
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind(Option::<String>::None)
    .bind(&now)
    .execute(&state.db)
    .await
    .unwrap();

    let response = list_archives(
        State(state.clone()),
        Extension(Claims::for_client(
            visible_device_id,
            "device",
            chrono::Duration::hours(1),
        )),
        Query(ArchiveListQuery {
            host_id: Some(host_id),
            workspace_id: None,
            page: None,
            limit: None,
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].archive_id, visible_archive_id);
}

#[tokio::test]
async fn list_archives_intersects_host_id_and_workspace_id_filters() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let other_host_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let other_workspace_id = Uuid::new_v4();
    let matching_session_id = Uuid::new_v4();
    let host_only_session_id = Uuid::new_v4();
    let workspace_only_session_id = Uuid::new_v4();
    let matching_archive_id = Uuid::new_v4();
    let host_only_archive_id = Uuid::new_v4();
    let workspace_only_archive_id = Uuid::new_v4();

    insert_archive_fixture(
        &state,
        device_id,
        host_id,
        workspace_id,
        matching_session_id,
        matching_archive_id,
    )
    .await;

    sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
        .bind(other_host_id.to_string())
        .bind("other-host")
        .bind("linux")
        .execute(&state.db)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(other_workspace_id.to_string())
    .bind(host_id.to_string())
    .bind("/tmp/other-project")
    .bind("other-project")
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(other_host_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    insert_archive_for_existing_scope(
        &state,
        device_id,
        host_id,
        other_workspace_id,
        host_only_session_id,
        host_only_archive_id,
        "host-only",
    )
    .await;

    insert_archive_for_existing_scope(
        &state,
        device_id,
        other_host_id,
        workspace_id,
        workspace_only_session_id,
        workspace_only_archive_id,
        "workspace-only",
    )
    .await;

    let response = list_archives(
        State(state.clone()),
        Extension(Claims::for_client(
            device_id,
            "device",
            chrono::Duration::hours(1),
        )),
        Query(ArchiveListQuery {
            host_id: Some(host_id),
            workspace_id: Some(workspace_id),
            page: None,
            limit: None,
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.total, 1);
    assert_eq!(response.items[0].archive_id, matching_archive_id);
}

#[tokio::test]
async fn get_archive_rejects_devices_without_session_access() {
    let state = setup_state().await;
    let owner_device_id = Uuid::new_v4();
    let other_device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let archive_id = Uuid::new_v4();

    insert_archive_fixture(
        &state,
        owner_device_id,
        host_id,
        workspace_id,
        session_id,
        archive_id,
    )
    .await;

    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
    )
    .bind(other_device_id.to_string())
    .bind("other")
    .bind("desktop")
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
        .bind(other_device_id.to_string())
        .bind(host_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    let error = get_archive(
        State(state.clone()),
        Extension(Claims::for_client(
            other_device_id,
            "other",
            chrono::Duration::hours(1),
        )),
        Path(archive_id),
    )
    .await
    .unwrap_err();

    assert!(
        matches!(error, ServerError::NotFound(message) if message == format!("Archive {}", archive_id))
    );
}

#[tokio::test]
async fn batch_delete_removes_archive_and_device_access_not_session() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let archive_id = Uuid::new_v4();

    insert_archive_fixture(
        &state,
        device_id,
        host_id,
        workspace_id,
        session_id,
        archive_id,
    )
    .await;

    let response = batch_delete_archives(
        State(state.clone()),
        Extension(Claims::for_client(
            device_id,
            "device",
            chrono::Duration::hours(1),
        )),
        Json(BatchDeleteRequest {
            archive_ids: vec![archive_id],
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.deleted_count, 1);
    assert!(response.failed_ids.is_empty());

    let archive_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE archive_id = $1")
            .bind(archive_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(archive_count.0, 0);

    // Session itself is NOT deleted (other devices may still have access).
    let session_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(session_count.0, 1);

    let access_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM device_session_access WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(access_count.0, 0);
}

#[tokio::test]
async fn list_archives_rejects_page_zero() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
    )
    .bind(device_id.to_string())
    .bind("device")
    .bind("desktop")
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    let error = list_archives(
        State(state.clone()),
        Extension(Claims::for_client(
            device_id,
            "device",
            chrono::Duration::hours(1),
        )),
        Query(ArchiveListQuery {
            host_id: None,
            workspace_id: None,
            page: Some(0),
            limit: None,
        }),
    )
    .await
    .unwrap_err();

    assert!(
        matches!(error, ServerError::BadRequest(message) if message == "page must be greater than 0")
    );
}
