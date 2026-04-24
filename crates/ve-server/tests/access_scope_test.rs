use axum::{
    body::Body,
    extract::{Extension, Query, State},
    http::{header, Request, StatusCode},
};

use futures::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message},
};
use tower::ServiceExt;
use uuid::Uuid;

use ve_server::{
    api::{
        files::{get_file_content, get_file_tree, FileContentQuery, FileTreeQuery},
        hosts::list_hosts,
        workspaces::list_workspaces,
    },
    authz::WorkspaceCollectionAccess,
    build_app,
    config::{Config, DatabaseBackend},
    db::{install_drivers, run_migrations, DbPool},
    hub::Hub,
    state::AppState,
};
use ve_shared::{
    jwt::{Claims, JwtManager},
    proto::{DaemonToServer, ErrorPayload},
};

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
    let temp_db = std::env::temp_dir().join(format!("ve-authz-surface-test-{}.db", Uuid::new_v4()));
    let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
    let pool = DbPool::connect(&database_url).await.unwrap();
    run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap();
    let config = test_config(database_url);
    let jwt_manager = Arc::new(JwtManager::new(&config.jwt_secret, config.jwt_expiration()));
    Arc::new(AppState::new(pool, Hub::new(), config, jwt_manager))
}

async fn setup_app() -> (axum::Router, Arc<AppState>, Arc<JwtManager>) {
    let state = setup_state().await;
    let jwt_manager = Arc::new(JwtManager::new(
        &state.config.jwt_secret,
        state.config.jwt_expiration(),
    ));
    let app = build_app(state.clone(), jwt_manager.clone(), &state.config);
    (app, state, jwt_manager)
}

async fn seed_visible_and_hidden_hosts(
    state: &Arc<AppState>,
    device_id: Uuid,
) -> (Uuid, Uuid, Uuid, Uuid) {
    let visible_host_id = Uuid::new_v4();
    let hidden_host_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, legacy_acl, server_url) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(device_id.to_string())
    .bind("device")
    .bind("desktop")
    .bind(0)
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    for host_id in [visible_host_id, hidden_host_id] {
        sqlx::query(
            "INSERT INTO hosts (host_id, host_name, platform, pair_status, created_at, updated_at) VALUES ($1, $2, $3, 'paired', $4, $4)",
        )
        .bind(host_id.to_string())
        .bind(format!("host-{host_id}"))
        .bind("linux")
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();
    }

    sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(visible_host_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    let visible_workspace_id = Uuid::new_v4();
    let hidden_workspace_id = Uuid::new_v4();
    for (workspace_id, host_id, path) in [
        (visible_workspace_id, visible_host_id, "/visible"),
        (hidden_workspace_id, hidden_host_id, "/hidden"),
    ] {
        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind(path)
        .bind(path)
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();
    }

    (
        visible_host_id,
        hidden_host_id,
        visible_workspace_id,
        hidden_workspace_id,
    )
}

async fn seed_visible_and_hidden_archives(state: &Arc<AppState>, device_id: Uuid) -> (Uuid, Uuid) {
    let (visible_host_id, hidden_host_id, visible_workspace_id, hidden_workspace_id) =
        seed_visible_and_hidden_hosts(state, device_id).await;
    let visible_session_id = Uuid::new_v4();
    let hidden_session_id = Uuid::new_v4();
    let visible_archive_id = Uuid::new_v4();
    let hidden_archive_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    for (session_id, host_id, workspace_id, title) in [
        (
            visible_session_id,
            visible_host_id,
            visible_workspace_id,
            "visible-archive",
        ),
        (
            hidden_session_id,
            hidden_host_id,
            hidden_workspace_id,
            "hidden-archive",
        ),
    ] {
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
    }

    sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(visible_session_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    for (archive_id, session_id, host_id, workspace_id, title) in [
        (
            visible_archive_id,
            visible_session_id,
            visible_host_id,
            visible_workspace_id,
            "visible-archive",
        ),
        (
            hidden_archive_id,
            hidden_session_id,
            hidden_host_id,
            hidden_workspace_id,
            "hidden-archive",
        ),
    ] {
        sqlx::query(
            "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $4)",
        )
        .bind(archive_id.to_string())
        .bind(session_id.to_string())
        .bind(title)
        .bind(&now)
        .bind("user_closed")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind(Option::<String>::None)
        .execute(&state.db)
        .await
        .unwrap();
    }

    (visible_archive_id, hidden_archive_id)
}

async fn seed_visible_and_hidden_permissions(
    state: &Arc<AppState>,
    device_id: Uuid,
) -> (Uuid, Uuid, Uuid) {
    let (visible_host_id, hidden_host_id, visible_workspace_id, hidden_workspace_id) =
        seed_visible_and_hidden_hosts(state, device_id).await;
    let visible_session_id = Uuid::new_v4();
    let hidden_session_id = Uuid::new_v4();
    let visible_permission_id = Uuid::new_v4();
    let hidden_permission_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    for (session_id, host_id, workspace_id, title) in [
        (
            visible_session_id,
            visible_host_id,
            visible_workspace_id,
            "visible-permission-session",
        ),
        (
            hidden_session_id,
            hidden_host_id,
            hidden_workspace_id,
            "hidden-permission-session",
        ),
    ] {
        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'waiting_approval', $6, $6)",
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
    }

    sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(visible_session_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    for (permission_id, session_id, summary) in [
        (
            visible_permission_id,
            visible_session_id,
            "visible permission request",
        ),
        (
            hidden_permission_id,
            hidden_session_id,
            "hidden permission request",
        ),
    ] {
        sqlx::query(
            "INSERT INTO permission_requests (permission_id, session_id, risk_type, summary, status, created_at) VALUES ($1, $2, 'exec_cmd', $3, 'pending', $4)",
        )
        .bind(permission_id.to_string())
        .bind(session_id.to_string())
        .bind(summary)
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();
    }

    (
        visible_permission_id,
        hidden_permission_id,
        hidden_session_id,
    )
}

async fn seed_legacy_hosts_without_acl(state: &Arc<AppState>) -> (Uuid, Uuid, Uuid, Uuid) {
    let paired_host_id = Uuid::new_v4();
    let pending_host_id = Uuid::new_v4();
    let paired_workspace_id = Uuid::new_v4();
    let pending_workspace_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    for (host_id, pair_status) in [(paired_host_id, "paired"), (pending_host_id, "pending")] {
        sqlx::query(
            "INSERT INTO hosts (host_id, host_name, platform, pair_status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)",
        )
        .bind(host_id.to_string())
        .bind(format!("host-{host_id}"))
        .bind("linux")
        .bind(pair_status)
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();
    }

    for (workspace_id, host_id, path) in [
        (paired_workspace_id, paired_host_id, "/legacy-paired"),
        (pending_workspace_id, pending_host_id, "/legacy-pending"),
    ] {
        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind(path)
        .bind(path)
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();
    }

    (
        paired_host_id,
        pending_host_id,
        paired_workspace_id,
        pending_workspace_id,
    )
}

async fn seed_legacy_sessions_without_acl(
    state: &Arc<AppState>,
) -> (Uuid, Uuid, Uuid, Uuid, Uuid, Uuid) {
    let (paired_host_id, pending_host_id, paired_workspace_id, pending_workspace_id) =
        seed_legacy_hosts_without_acl(state).await;
    let paired_session_id = Uuid::new_v4();
    let pending_session_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    for (session_id, host_id, workspace_id, title) in [
        (
            paired_session_id,
            paired_host_id,
            paired_workspace_id,
            "legacy-paired-session",
        ),
        (
            pending_session_id,
            pending_host_id,
            pending_workspace_id,
            "legacy-pending-session",
        ),
    ] {
        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
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
    }

    (
        paired_host_id,
        pending_host_id,
        paired_workspace_id,
        pending_workspace_id,
        paired_session_id,
        pending_session_id,
    )
}

async fn response_body_text(response: axum::response::Response) -> String {
    let body = response.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(body.to_vec()).unwrap()
}

async fn run_app_for_ws(app: axum::Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });
    (format!("ws://{addr}"), handle)
}

#[tokio::test]
async fn get_archive_hidden_session_access_returns_masked_not_found_via_router() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (_visible_archive_id, hidden_archive_id) =
        seed_visible_and_hidden_archives(&state, device_id).await;
    let hidden_session_id: (String,) =
        sqlx::query_as("SELECT session_id FROM session_archives WHERE archive_id = $1")
            .bind(hidden_archive_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/archives/{hidden_archive_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_text = response_body_text(response).await;
    assert!(body_text.contains(&hidden_archive_id.to_string()));
    assert!(!body_text.contains(&hidden_session_id.0));
}

#[tokio::test]
async fn get_workspace_hidden_host_access_returns_masked_not_found_via_router() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (_visible_host_id, hidden_host_id, _visible_workspace_id, hidden_workspace_id) =
        seed_visible_and_hidden_hosts(&state, device_id).await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/workspaces/{hidden_workspace_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_text = response_body_text(response).await;
    assert!(body_text.contains(&hidden_workspace_id.to_string()));
    assert!(!body_text.contains(&hidden_host_id.to_string()));
}

#[tokio::test]
async fn get_permission_hidden_session_access_returns_masked_not_found_via_router() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (_visible_permission_id, hidden_permission_id, hidden_session_id) =
        seed_visible_and_hidden_permissions(&state, device_id).await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/permissions/{hidden_permission_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_text = response_body_text(response).await;
    assert!(body_text.contains(&hidden_permission_id.to_string()));
    assert!(!body_text.contains(&hidden_session_id.to_string()));
}

#[tokio::test]
async fn update_workspace_hidden_host_access_returns_masked_not_found_via_router() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (_visible_host_id, hidden_host_id, _visible_workspace_id, hidden_workspace_id) =
        seed_visible_and_hidden_hosts(&state, device_id).await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{hidden_workspace_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"display_name":"renamed"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_text = response_body_text(response).await;
    assert!(body_text.contains(&hidden_workspace_id.to_string()));
    assert!(!body_text.contains(&hidden_host_id.to_string()));
}

#[tokio::test]
async fn delete_workspace_hidden_host_access_returns_masked_not_found_via_router() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (_visible_host_id, hidden_host_id, _visible_workspace_id, hidden_workspace_id) =
        seed_visible_and_hidden_hosts(&state, device_id).await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{hidden_workspace_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_text = response_body_text(response).await;
    assert!(body_text.contains(&hidden_workspace_id.to_string()));
    assert!(!body_text.contains(&hidden_host_id.to_string()));
}

#[tokio::test]
async fn batch_delete_archives_hides_related_session_of_inaccessible_archive() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (visible_archive_id, hidden_archive_id) =
        seed_visible_and_hidden_archives(&state, device_id).await;
    let hidden_session_id: (String,) =
        sqlx::query_as("SELECT session_id FROM session_archives WHERE archive_id = $1")
            .bind(hidden_archive_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/archives/batch-delete")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(
                    r#"{{"archive_ids":["{visible_archive_id}","{hidden_archive_id}"]}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_text = response_body_text(response).await;
    let body_json: serde_json::Value = serde_json::from_str(&body_text).unwrap();
    assert_eq!(body_json["deleted_count"], 1);
    assert_eq!(
        body_json["failed_ids"],
        serde_json::json!([hidden_archive_id])
    );
    assert!(!body_text.contains(&hidden_session_id.0));

    let visible_archive_exists: Option<(String,)> =
        sqlx::query_as("SELECT archive_id FROM session_archives WHERE archive_id = $1")
            .bind(visible_archive_id.to_string())
            .fetch_optional(&state.db)
            .await
            .unwrap();
    assert!(visible_archive_exists.is_none());
}

#[tokio::test]
async fn list_hosts_only_returns_accessible_hosts() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let (visible_host_id, hidden_host_id, _visible_workspace_id, _hidden_workspace_id) =
        seed_visible_and_hidden_hosts(&state, device_id).await;

    let response = list_hosts(
        ve_server::authz::HostCollectionAccess { device_id },
        State(state.clone()),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.hosts.len(), 1);
    assert_eq!(response.hosts[0].host_id, visible_host_id);
    assert_ne!(response.hosts[0].host_id, hidden_host_id);
}

#[tokio::test]
async fn list_workspaces_with_visible_host_filter_only_returns_accessible_workspaces() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let (visible_host_id, hidden_host_id, visible_workspace_id, hidden_workspace_id) =
        seed_visible_and_hidden_hosts(&state, device_id).await;

    let response = list_workspaces(
        WorkspaceCollectionAccess {
            device_id,
            host_id: Some(visible_host_id),
            page: 1,
            limit: 20,
        },
        State(state.clone()),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].workspace_id, visible_workspace_id);
    assert_ne!(response.items[0].workspace_id, hidden_workspace_id);
    assert_eq!(response.items[0].host_id, visible_host_id);
    assert_ne!(response.items[0].host_id, hidden_host_id);
}

#[tokio::test]
async fn list_workspaces_without_host_filter_only_returns_accessible_workspaces() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let (_visible_host_id, hidden_host_id, visible_workspace_id, hidden_workspace_id) =
        seed_visible_and_hidden_hosts(&state, device_id).await;

    let response = list_workspaces(
        WorkspaceCollectionAccess {
            device_id,
            host_id: None,
            page: 1,
            limit: 20,
        },
        State(state.clone()),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].workspace_id, visible_workspace_id);
    assert_ne!(response.items[0].workspace_id, hidden_workspace_id);
    assert_ne!(response.items[0].host_id, hidden_host_id);
}

#[tokio::test]
async fn list_workspaces_rejects_daemon_token_via_router() {
    let (app, _state, jwt_manager) = setup_app().await;
    let token = jwt_manager
        .create_daemon_token(Uuid::new_v4(), "host")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/workspaces")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_workspaces_hidden_host_filter_returns_not_found_via_router() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (_visible_host_id, hidden_host_id, _, _) =
        seed_visible_and_hidden_hosts(&state, device_id).await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/workspaces?host_id={hidden_host_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_workspace_rejects_hidden_host_with_not_found_via_router() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (_visible_host_id, hidden_host_id, _, _) =
        seed_visible_and_hidden_hosts(&state, device_id).await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/workspaces")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(
                    r#"{{"host_id":"{hidden_host_id}","path":"/hidden/new","display_name":"hidden"}}"#
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_session_rejects_workspace_from_other_host_with_not_found_via_router() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (visible_host_id, _hidden_host_id, _visible_workspace_id, hidden_workspace_id) =
        seed_visible_and_hidden_hosts(&state, device_id).await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/sessions")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(format!(
                    r#"{{"idempotency_key":"{}","host_id":"{visible_host_id}","workspace_id":"{hidden_workspace_id}","title":"test","initial_message":"hello"}}"#,
                    Uuid::new_v4()
                )))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_file_tree_rejects_hidden_host_before_daemon_lookup() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let (_visible_host_id, hidden_host_id, visible_workspace_id, _) =
        seed_visible_and_hidden_hosts(&state, device_id).await;

    let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
    let error = get_file_tree(
        State(state.clone()),
        Extension(claims),
        axum::extract::Path(hidden_host_id),
        Query(FileTreeQuery {
            workspace_id: visible_workspace_id,
            path: Some("src".to_string()),
        }),
    )
    .await
    .unwrap_err();

    let response = axum::response::IntoResponse::into_response(error);
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_file_tree_rejects_workspace_not_owned_by_host() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let (visible_host_id, _hidden_host_id, _visible_workspace_id, hidden_workspace_id) =
        seed_visible_and_hidden_hosts(&state, device_id).await;
    let (daemon_tx, _daemon_rx) = tokio::sync::mpsc::channel(1);
    state.hub.register_daemon(visible_host_id, daemon_tx).await;

    let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
    let error = get_file_tree(
        State(state.clone()),
        Extension(claims),
        axum::extract::Path(visible_host_id),
        Query(FileTreeQuery {
            workspace_id: hidden_workspace_id,
            path: Some("src".to_string()),
        }),
    )
    .await
    .unwrap_err();

    match error {
        ve_server::error::ServerError::NotFound(message) => {
            assert!(message.contains(&hidden_workspace_id.to_string()));
            assert!(message.contains(&visible_host_id.to_string()));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn get_file_content_rejects_hidden_host_before_daemon_lookup() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let (_visible_host_id, hidden_host_id, visible_workspace_id, _) =
        seed_visible_and_hidden_hosts(&state, device_id).await;

    let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
    let error = get_file_content(
        State(state.clone()),
        Extension(claims),
        axum::extract::Path(hidden_host_id),
        Query(FileContentQuery {
            workspace_id: visible_workspace_id,
            path: "secret.txt".to_string(),
        }),
    )
    .await
    .unwrap_err();

    let response = axum::response::IntoResponse::into_response(error);
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_file_tree_sanitizes_daemon_error_details() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let (visible_host_id, _hidden_host_id, visible_workspace_id, _) =
        seed_visible_and_hidden_hosts(&state, device_id).await;

    let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);
    state.hub.register_daemon(visible_host_id, daemon_tx).await;

    let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
    let pending_request = tokio::spawn(get_file_tree(
        State(state.clone()),
        Extension(claims),
        axum::extract::Path(visible_host_id),
        Query(FileTreeQuery {
            workspace_id: visible_workspace_id,
            path: Some("src".to_string()),
        }),
    ));

    let envelope = daemon_rx
        .recv()
        .await
        .expect("daemon should receive request");
    let request_id = envelope.request_id.expect("request_id should be set");

    state
        .hub
        .complete_with_error(ErrorPayload {
            request_id,
            error_code: "WORKSPACE_INVALID".to_string(),
            error_message: "Workspace path does not exist: /root/vibe-remote/secrets/project"
                .to_string(),
        })
        .await;

    let error = pending_request.await.unwrap().unwrap_err();
    let response = axum::response::IntoResponse::into_response(error);
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_text.contains("/root/vibe-remote/secrets/project"));
}

#[tokio::test]
async fn get_file_tree_sanitizes_daemon_transport_error_details() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let (visible_host_id, _hidden_host_id, visible_workspace_id, _) =
        seed_visible_and_hidden_hosts(&state, device_id).await;

    let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);
    let connection_id = state.hub.register_daemon(visible_host_id, daemon_tx).await;

    let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
    let pending_request = tokio::spawn(get_file_tree(
        State(state.clone()),
        Extension(claims),
        axum::extract::Path(visible_host_id),
        Query(FileTreeQuery {
            workspace_id: visible_workspace_id,
            path: Some("src".to_string()),
        }),
    ));

    let _envelope = daemon_rx
        .recv()
        .await
        .expect("daemon should receive request");
    assert!(
        state
            .hub
            .unregister_daemon(&visible_host_id, connection_id)
            .await
    );
    state
        .hub
        .fail_pending_requests_for_connection(
            &visible_host_id,
            connection_id,
            "daemon transport lost while browsing /root/vibe-remote/secrets/project",
        )
        .await;

    let error = pending_request.await.unwrap().unwrap_err();
    let response = axum::response::IntoResponse::into_response(error);
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_text.contains("File operation failed"));
    assert!(!body_text.contains("/root/vibe-remote/secrets/project"));
}

#[tokio::test]
async fn get_file_content_sanitizes_daemon_error_details() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    let (visible_host_id, _hidden_host_id, visible_workspace_id, _) =
        seed_visible_and_hidden_hosts(&state, device_id).await;

    let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);
    state.hub.register_daemon(visible_host_id, daemon_tx).await;

    let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
    let pending_request = tokio::spawn(get_file_content(
        State(state.clone()),
        Extension(claims),
        axum::extract::Path(visible_host_id),
        Query(FileContentQuery {
            workspace_id: visible_workspace_id,
            path: "src/main.rs".to_string(),
        }),
    ));

    let envelope = daemon_rx
        .recv()
        .await
        .expect("daemon should receive request");
    let request_id = envelope.request_id.expect("request_id should be set");

    state
        .hub
        .handle_response(DaemonToServer::Error {
            request_id,
            error_code: "INTERNAL_ERROR".to_string(),
            error_message: "File read failed: /root/vibe-remote/private/main.rs".to_string(),
        })
        .await;

    let error = pending_request.await.unwrap().unwrap_err();
    let response = axum::response::IntoResponse::into_response(error);
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_text = String::from_utf8(body.to_vec()).unwrap();
    assert!(!body_text.contains("/root/vibe-remote/private/main.rs"));
}

#[tokio::test]
async fn list_hosts_does_not_grant_legacy_access_for_formal_client_when_acl_is_empty() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, legacy_acl, server_url) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(device_id.to_string())
    .bind("device")
    .bind("desktop")
    .bind(1)
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    let (paired_host_id, pending_host_id, _paired_workspace_id, _pending_workspace_id) =
        seed_legacy_hosts_without_acl(&state).await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/hosts")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_text = response_body_text(response).await;
    assert!(!body_text.contains(&paired_host_id.to_string()));
    assert!(!body_text.contains(&pending_host_id.to_string()));

    let paired_access_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM device_host_access WHERE device_id = $1 AND host_id = $2",
    )
    .bind(device_id.to_string())
    .bind(paired_host_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(paired_access_count, 0);

    let pending_access_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM device_host_access WHERE device_id = $1 AND host_id = $2",
    )
    .bind(device_id.to_string())
    .bind(pending_host_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(pending_access_count, 0);
}

#[tokio::test]
async fn get_archive_does_not_grant_legacy_session_access_for_formal_client_when_acl_is_empty() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, legacy_acl, server_url) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(device_id.to_string())
    .bind("device")
    .bind("desktop")
    .bind(1)
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    let (
        paired_host_id,
        pending_host_id,
        paired_workspace_id,
        pending_workspace_id,
        paired_session_id,
        pending_session_id,
    ) = seed_legacy_sessions_without_acl(&state).await;
    let paired_archive_id = Uuid::new_v4();
    let pending_archive_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    for (archive_id, session_id, host_id, workspace_id, title) in [
        (
            paired_archive_id,
            paired_session_id,
            paired_host_id,
            paired_workspace_id,
            "legacy-paired-archive",
        ),
        (
            pending_archive_id,
            pending_session_id,
            pending_host_id,
            pending_workspace_id,
            "legacy-pending-archive",
        ),
    ] {
        sqlx::query(
            "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $4)",
        )
        .bind(archive_id.to_string())
        .bind(session_id.to_string())
        .bind(title)
        .bind(&now)
        .bind("user_closed")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind(Option::<String>::None)
        .execute(&state.db)
        .await
        .unwrap();
    }

    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/archives/{paired_archive_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_text = response_body_text(response).await;
    assert!(body_text.contains(&paired_archive_id.to_string()));
    assert!(!body_text.contains(&pending_archive_id.to_string()));

    let paired_session_access_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
    )
    .bind(device_id.to_string())
    .bind(paired_session_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(paired_session_access_count, 0);

    let pending_session_access_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
    )
    .bind(device_id.to_string())
    .bind(pending_session_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(pending_session_access_count, 0);
}

#[tokio::test]
async fn list_hosts_does_not_grant_legacy_access_for_modern_device_without_acl() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, legacy_acl, server_url) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(device_id.to_string())
    .bind("device")
    .bind("desktop")
    .bind(0)
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    let (paired_host_id, _pending_host_id, _paired_workspace_id, _pending_workspace_id) =
        seed_legacy_hosts_without_acl(&state).await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/hosts")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_text = response_body_text(response).await;
    assert!(!body_text.contains(&paired_host_id.to_string()));

    let paired_access_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM device_host_access WHERE device_id = $1 AND host_id = $2",
    )
    .bind(device_id.to_string())
    .bind(paired_host_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(paired_access_count, 0);
}

#[tokio::test]
async fn list_hosts_does_not_backfill_older_paired_hosts_even_after_partial_legacy_acl_exists() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, legacy_acl, server_url) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(device_id.to_string())
    .bind("device")
    .bind("desktop")
    .bind(1)
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    let (paired_host_id, _pending_host_id, _paired_workspace_id, _pending_workspace_id) =
        seed_legacy_hosts_without_acl(&state).await;
    let newly_paired_host_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO hosts (host_id, host_name, platform, pair_status, created_at, updated_at) VALUES ($1, $2, $3, 'paired', $4, $4)",
    )
    .bind(newly_paired_host_id.to_string())
    .bind("new-host")
    .bind("linux")
    .bind(&now)
    .execute(&state.db)
    .await
    .unwrap();
    sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(newly_paired_host_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/hosts")
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_text = response_body_text(response).await;
    assert!(!body_text.contains(&paired_host_id.to_string()));
    assert!(body_text.contains(&newly_paired_host_id.to_string()));

    let backfilled_access_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM device_host_access WHERE device_id = $1 AND host_id = $2",
    )
    .bind(device_id.to_string())
    .bind(paired_host_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(backfilled_access_count, 0);
}

#[tokio::test]
async fn ws_session_subscription_stops_receiving_after_access_revocation() {
    let (app, state, jwt_manager) = setup_app().await;
    let device_id = Uuid::new_v4();
    let (
        _paired_host_id,
        _pending_host_id,
        _paired_workspace_id,
        _pending_workspace_id,
        session_id,
        _pending_session_id,
    ) = seed_legacy_sessions_without_acl(&state).await;

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

    sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(session_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();
    let (base_url, server_handle) = run_app_for_ws(app).await;
    let mut ws_request = format!("{base_url}/ws/client")
        .into_client_request()
        .unwrap();
    ws_request.headers_mut().insert(
        "Authorization",
        format!("Bearer {token}")
            .parse::<tokio_tungstenite::tungstenite::http::HeaderValue>()
            .unwrap(),
    );
    let (mut ws, _) = connect_async(ws_request).await.unwrap();

    ws.send(Message::Text(
        serde_json::json!({
            "type": "subscribe_session",
            "payload": { "session_id": session_id.to_string() }
        })
        .to_string()
        .into(),
    ))
    .await
    .unwrap();

    sqlx::query("DELETE FROM device_session_access WHERE device_id = $1 AND session_id = $2")
        .bind(device_id.to_string())
        .bind(session_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    state
        .hub
        .broadcast_to_session(
            &state.db,
            &session_id,
            ve_shared::proto::ClientMessage::SessionStatusChanged {
                session_id,
                new_status: ve_shared::types::SessionStatus::Running,
                close_reason: None,
            },
        )
        .await;

    let next = tokio::time::timeout(Duration::from_millis(200), ws.next()).await;
    server_handle.abort();

    assert!(
        next.is_err(),
        "revoked subscriber still received websocket message"
    );
}

#[tokio::test]
async fn ws_client_rejects_invalid_token_with_unauthorized_instead_of_jwt_500() {
    let (app, _state, _jwt_manager) = setup_app().await;
    let (base_url, server_handle) = run_app_for_ws(app).await;

    let url = format!("{base_url}/ws/client");
    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert(
        "Authorization",
        "Bearer invalid_token"
            .parse::<tokio_tungstenite::tungstenite::http::HeaderValue>()
            .unwrap(),
    );

    let error = connect_async(request).await.unwrap_err();
    server_handle.abort();

    let response = match error {
        tokio_tungstenite::tungstenite::Error::Http(response) => response,
        other => panic!("unexpected websocket error: {other:?}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body_text = String::from_utf8(response.into_body().unwrap_or_default()).unwrap();
    assert!(body_text.contains("Invalid token"));
    assert!(!body_text.contains("JWT error"));
}

#[tokio::test]
async fn ws_daemon_rejects_invalid_token_with_unauthorized_instead_of_jwt_500() {
    let (app, _state, _jwt_manager) = setup_app().await;
    let (base_url, server_handle) = run_app_for_ws(app).await;

    let url = format!("{base_url}/ws/daemon");
    let mut request = url.into_client_request().unwrap();
    request.headers_mut().insert(
        "Authorization",
        "Bearer invalid_token"
            .parse::<tokio_tungstenite::tungstenite::http::HeaderValue>()
            .unwrap(),
    );

    let error = connect_async(request).await.unwrap_err();
    server_handle.abort();

    let response = match error {
        tokio_tungstenite::tungstenite::Error::Http(response) => response,
        other => panic!("unexpected websocket error: {other:?}"),
    };

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body_text = String::from_utf8(response.into_body().unwrap_or_default()).unwrap();
    assert!(body_text.contains("Invalid token"));
    assert!(!body_text.contains("JWT error"));
}
