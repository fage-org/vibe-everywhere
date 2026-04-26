//! Tests for the workspace API module.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::{Extension, Json};
use tokio::sync::mpsc;
use uuid::Uuid;
use ve_shared::jwt::{Claims, TokenType};
use ve_shared::models::CreateWorkspaceRequest;
use ve_shared::proto::{AckPayload, ErrorPayload, WsEnvelope};

use super::{
    create_workspace, update_workspace, UpdateWorkspaceRequest,
};
use crate::authz::ClientAccess;
use crate::config::Config;
use crate::db;
use crate::error::ServerError;
use crate::hub::{Hub, WsSender};
use crate::state::AppState;
use crate::validation::{ValidationError, MAX_WORKSPACE_DISPLAY_NAME_LENGTH};

fn test_config(database_url: String) -> Config {
    const TEST_JWT_SECRET: &str = "test_secret_for_unit_tests_only_32chars!";
    Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url,
        jwt_secret: TEST_JWT_SECRET.to_string(),
        jwt_expiration_secs: 3600,
        pair_code_ttl_secs: 300,
        heartbeat_interval_secs: 30,
        connection_timeout_secs: 60,
        data_dir: std::env::temp_dir(),
        cors_origins: vec![],
        ack_timeout_ms: 3_000,
        ack_max_retries: 0,
        ack_retry_delay_ms: 0,
        permission_ttl_secs: 1800,
        permission_expiry_check_secs: 60,
        idempotency_ttl_secs: 86_400,
        idempotency_cleanup_secs: 3600,
        log_format: "pretty".to_string(),
        log_level: "info".to_string(),
    }
}

async fn setup_state() -> std::sync::Arc<crate::state::AppState> {
    db::install_drivers();
    let db_name = format!("workspace_create_{}.db", Uuid::new_v4());
    let db_url = format!("sqlite:/tmp/{}?mode=rwc", db_name);
    let config = test_config(db_url);
    let pool = db::create_pool(&config).await.unwrap();
    db::run_migrations(&pool, config.database_backend())
        .await
        .unwrap();
    let jwt_manager = std::sync::Arc::new(ve_shared::jwt::JwtManager::new(
        &config.jwt_secret,
        config.jwt_expiration(),
    ));
    std::sync::Arc::new(crate::state::AppState::new(pool, Hub::new(), config, jwt_manager))
}

async fn seed_device_and_host(state: &crate::state::AppState) -> (Uuid, Uuid) {
    let device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();

    sqlx::query(
        r#"INSERT INTO client_devices (device_id, device_name, device_type, server_url)
           VALUES ($1, 'device', 'desktop', 'http://localhost')"#,
    )
    .bind(device_id.to_string())
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO hosts (host_id, host_name, platform, pair_status)
           VALUES ($1, 'host', 'linux', 'paired')"#,
    )
    .bind(host_id.to_string())
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO device_host_access (device_id, host_id)
           VALUES ($1, $2)"#,
    )
    .bind(device_id.to_string())
    .bind(host_id.to_string())
    .execute(&state.db)
    .await
    .unwrap();

    (device_id, host_id)
}

fn client_claims(device_id: Uuid) -> Claims {
    Claims::for_client(device_id, "device", chrono::Duration::hours(1))
}

async fn register_fake_daemon(state: &crate::state::AppState, host_id: Uuid) -> mpsc::Receiver<WsEnvelope> {
    let (tx, rx): (WsSender, mpsc::Receiver<WsEnvelope>) = mpsc::channel(8);
    state.hub.register_daemon(host_id, tx).await;
    rx
}

#[tokio::test]
async fn create_workspace_waits_for_daemon_ack_before_persisting() {
    let state = setup_state().await;
    let (device_id, host_id) = seed_device_and_host(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;

    let req_path = format!("/tmp/ws-{}", Uuid::new_v4());
    let state_for_request = state.clone();
    let req_path_for_request = req_path.clone();
    let request_task = tokio::spawn(async move {
        create_workspace(
            ClientAccess { device_id },
            State(state_for_request),
            Json(CreateWorkspaceRequest {
                host_id,
                path: req_path_for_request,
                display_name: Some("ws".to_string()),
            }),
        )
        .await
    });

    let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
        .await
        .expect("timed out waiting for ensure_workspace command")
        .expect("daemon command");
    assert_eq!(outbound.r#type, "ensure_workspace");

    let row_count_before: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE host_id = $1 AND path = $2")
            .bind(host_id.to_string())
            .bind(&req_path)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(row_count_before.0, 0);
    assert!(!request_task.is_finished());

    state
        .hub
        .complete_with_ack(AckPayload {
            request_id: outbound.request_id.clone().unwrap(),
            success: true,
            error: None,
        })
        .await;

    let response = tokio::time::timeout(Duration::from_secs(1), request_task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    assert!(response.0.exists_on_host);
    assert_eq!(response.0.path, req_path);

    let row_count_after: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE host_id = $1 AND path = $2")
            .bind(host_id.to_string())
            .bind(&response.0.path)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(row_count_after.0, 1);
}

#[tokio::test]
async fn create_workspace_returns_bad_request_when_daemon_rejects_path() {
    let state = setup_state().await;
    let (device_id, host_id) = seed_device_and_host(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;

    let req_path = format!("/tmp/ws-{}", Uuid::new_v4());
    let state_for_request = state.clone();
    let req_path_for_request = req_path.clone();
    let request_task = tokio::spawn(async move {
        create_workspace(
            ClientAccess { device_id },
            State(state_for_request),
            Json(CreateWorkspaceRequest {
                host_id,
                path: req_path_for_request,
                display_name: Some("ws".to_string()),
            }),
        )
        .await
    });

    let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
        .await
        .expect("timed out waiting for ensure_workspace command")
        .expect("daemon command");
    assert_eq!(outbound.r#type, "ensure_workspace");

    state
        .hub
        .complete_with_error(ErrorPayload {
            request_id: outbound.request_id.clone().unwrap(),
            error_code: "WORKSPACE_INVALID".to_string(),
            error_message: "path rejected".to_string(),
        })
        .await;

    let error = tokio::time::timeout(Duration::from_secs(1), request_task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();

    assert!(matches!(error, ServerError::BadRequest(_)));

    let row_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE host_id = $1 AND path = $2")
            .bind(host_id.to_string())
            .bind(&req_path)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(row_count.0, 0);
}

#[tokio::test]
async fn create_workspace_rejects_too_long_path_before_daemon_dispatch() {
    let state = setup_state().await;
    let (device_id, host_id) = seed_device_and_host(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;
    let path = format!(
        "/{}",
        "a".repeat(crate::validation::MAX_WORKSPACE_PATH_LENGTH)
    );

    let error = create_workspace(
        ClientAccess { device_id },
        State(state.clone()),
        Json(CreateWorkspaceRequest {
            host_id,
            path: path.clone(),
            display_name: Some("ws".to_string()),
        }),
    )
    .await
    .unwrap_err();

    assert!(matches!(
        error,
        ServerError::Validation(ValidationError::TooLong {
            field: "workspace_path",
            max: crate::validation::MAX_WORKSPACE_PATH_LENGTH,
        })
    ));
    assert!(daemon_rx.try_recv().is_err());

    let row_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE host_id = $1 AND path = $2")
            .bind(host_id.to_string())
            .bind(&path)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(row_count.0, 0);
}

#[tokio::test]
async fn update_workspace_rejects_blank_display_name() {
    let state = setup_state().await;
    let (device_id, host_id) = seed_device_and_host(&state).await;
    let workspace_id = Uuid::new_v4();

    sqlx::query(
        r#"INSERT INTO workspaces (workspace_id, host_id, path, display_name)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind(workspace_id.to_string())
    .bind(host_id.to_string())
    .bind("/tmp/workspace")
    .bind("original-name")
    .execute(&state.db)
    .await
    .unwrap();

    let error = update_workspace(
        State(state.clone()),
        Extension(client_claims(device_id)),
        Path(workspace_id),
        Json(UpdateWorkspaceRequest {
            display_name: Some("   ".to_string()),
            is_favorited: None,
        }),
    )
    .await
    .unwrap_err();

    assert!(matches!(
        error,
        ServerError::Validation(ValidationError::Empty {
            field: "workspace_display_name",
        })
    ));

    let display_name: (String,) =
        sqlx::query_as("SELECT display_name FROM workspaces WHERE workspace_id = $1")
            .bind(workspace_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(display_name.0, "original-name");
}

#[tokio::test]
async fn update_workspace_rejects_too_long_display_name() {
    let state = setup_state().await;
    let (device_id, host_id) = seed_device_and_host(&state).await;
    let workspace_id = Uuid::new_v4();

    sqlx::query(
        r#"INSERT INTO workspaces (workspace_id, host_id, path, display_name)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind(workspace_id.to_string())
    .bind(host_id.to_string())
    .bind("/tmp/workspace")
    .bind("original-name")
    .execute(&state.db)
    .await
    .unwrap();

    let too_long = "a".repeat(MAX_WORKSPACE_DISPLAY_NAME_LENGTH + 1);
    let error = update_workspace(
        State(state.clone()),
        Extension(client_claims(device_id)),
        Path(workspace_id),
        Json(UpdateWorkspaceRequest {
            display_name: Some(too_long),
            is_favorited: None,
        }),
    )
    .await
    .unwrap_err();

    assert!(matches!(
        error,
        ServerError::Validation(ValidationError::TooLong {
            field: "workspace_display_name",
            max: MAX_WORKSPACE_DISPLAY_NAME_LENGTH,
        })
    ));
}

#[test]
fn client_claims_are_formal_client_tokens_for_workspace_tests() {
    let claims = client_claims(Uuid::new_v4());
    assert_eq!(claims.r#type, TokenType::Client);
}
