use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Extension, Path, State},
    http::HeaderMap,
    Json,
};
use tokio::sync::mpsc;
use uuid::Uuid;
use ve_server::{
    api::sessions,
    config::Config,
    db,
    hub::{Hub, WsSender},
    state::AppState,
};
use ve_shared::{
    jwt::Claims,
    proto::{AckPayload, SessionControlAction, WsEnvelope},
};

fn test_config(database_url: String) -> Config {
    Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url,
        jwt_secret: "super_secure_test_secret_key_32_chars!!".to_string(),
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

async fn setup_state() -> Arc<AppState> {
    db::install_drivers();
    let db_name = format!("session_ack_{}.db", Uuid::new_v4());
    let db_url = format!("sqlite:/tmp/{}?mode=rwc", db_name);
    let config = test_config(db_url);
    let pool = db::create_pool(&config).await.unwrap();
    db::run_migrations(&pool, config.database_backend())
        .await
        .unwrap();
    let jwt_manager = Arc::new(ve_shared::jwt::JwtManager::new(
        &config.jwt_secret,
        config.jwt_expiration(),
    ));
    Arc::new(AppState::new(pool, Hub::new(), config, jwt_manager))
}

fn client_claims(device_id: Uuid) -> Claims {
    Claims::for_client(device_id, "device", chrono::Duration::hours(1))
}

async fn seed_fixture(state: &AppState) -> (Uuid, Uuid, Uuid, Uuid) {
    let device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

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

    sqlx::query(
        r#"INSERT INTO workspaces (workspace_id, host_id, path, display_name)
           VALUES ($1, $2, '/tmp/ws', 'ws')"#,
    )
    .bind(workspace_id.to_string())
    .bind(host_id.to_string())
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at)
           VALUES ($1, 'test', $2, $3, 'claude_code', 'running', $4, $4)"#,
    )
    .bind(session_id.to_string())
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind(now)
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query(
        r#"INSERT INTO device_session_access (device_id, session_id)
           VALUES ($1, $2)"#,
    )
    .bind(device_id.to_string())
    .bind(session_id.to_string())
    .execute(&state.db)
    .await
    .unwrap();

    (device_id, host_id, workspace_id, session_id)
}

async fn register_fake_daemon(state: &AppState, host_id: Uuid) -> mpsc::Receiver<WsEnvelope> {
    let (tx, rx): (WsSender, mpsc::Receiver<WsEnvelope>) = mpsc::channel(8);
    state.hub.register_daemon(host_id, tx).await;
    rx
}

async fn assert_control_ack_updates_status(
    initial_status: &str,
    action: SessionControlAction,
    expected_status: &str,
) {
    let state = setup_state().await;
    let (device_id, host_id, _workspace_id, session_id) = seed_fixture(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;

    if initial_status != "running" {
        sqlx::query(r#"UPDATE sessions SET status = $2 WHERE session_id = $1"#)
            .bind(session_id.to_string())
            .bind(initial_status)
            .execute(&state.db)
            .await
            .unwrap();
    }

    let state_for_request = state.clone();
    let claims = client_claims(device_id);
    let action_name = match action {
        SessionControlAction::Pause => "pause",
        SessionControlAction::Terminate => "terminate",
        SessionControlAction::Interrupt => "interrupt",
        SessionControlAction::Rerun => "rerun",
        SessionControlAction::Restart => "restart",
    }
    .to_string();

    let request_task = tokio::spawn(async move {
        sessions::control_session(
            State(state_for_request),
            Extension(claims),
            HeaderMap::new(),
            Path(session_id),
            Json(sessions::ControlRequest {
                action: action_name,
            }),
        )
        .await
    });

    let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
        .await
        .expect("timed out waiting for session_control command")
        .expect("daemon command");
    assert_eq!(outbound.r#type, "session_control");

    let status_before: (String,) =
        sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(status_before.0, initial_status);
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
    assert!(response.0.get("success").and_then(|v| v.as_bool()).unwrap());

    let status_after: (String,) =
        sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(status_after.0, expected_status);
}

#[tokio::test]
async fn close_session_waits_for_daemon_ack_before_archiving() {
    let state = setup_state().await;
    let (device_id, host_id, _workspace_id, session_id) = seed_fixture(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;

    let state_for_request = state.clone();
    let claims = client_claims(device_id);
    let request_task = tokio::spawn(async move {
        sessions::close_session(
            State(state_for_request),
            Extension(claims),
            HeaderMap::new(),
            Path(session_id),
        )
        .await
    });

    let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
        .await
        .expect("timed out waiting for close_session command")
        .expect("daemon command");
    assert_eq!(outbound.r#type, "close_session");

    let status_before: (String,) =
        sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(status_before.0, "running");

    let archive_count_before: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(archive_count_before.0, 0);
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
    assert!(response.0.get("success").and_then(|v| v.as_bool()).unwrap());
    assert!(response
        .0
        .get("close_requested")
        .and_then(|v| v.as_bool())
        .unwrap());
    assert!(response.0.get("archive_id").is_none());

    let status_after: (String,) =
        sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(status_after.0, "running");

    let archive_count_after: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(archive_count_after.0, 0);
}

#[tokio::test]
async fn pause_control_waits_for_daemon_ack_before_pausing_session() {
    assert_control_ack_updates_status("running", SessionControlAction::Pause, "paused").await;
}

#[tokio::test]
async fn terminate_control_waits_for_daemon_ack_before_archiving_session() {
    let state = setup_state().await;
    let (device_id, host_id, _workspace_id, session_id) = seed_fixture(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;

    let state_for_request = state.clone();
    let claims = client_claims(device_id);
    let request_task = tokio::spawn(async move {
        sessions::control_session(
            State(state_for_request),
            Extension(claims),
            HeaderMap::new(),
            Path(session_id),
            Json(sessions::ControlRequest {
                action: "terminate".to_string(),
            }),
        )
        .await
    });

    let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
        .await
        .expect("timed out waiting for session_control command")
        .expect("daemon command");
    assert_eq!(outbound.r#type, "session_control");

    let status_before: (String,) =
        sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(status_before.0, "running");

    let archive_count_before: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(archive_count_before.0, 0);
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
    assert!(response.0.get("success").and_then(|v| v.as_bool()).unwrap());

    let status_after: (String,) =
        sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(status_after.0, "running");

    let archive_count_after: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(archive_count_after.0, 0);
}

#[tokio::test]
async fn restart_control_waits_for_daemon_ack_before_marking_session_running() {
    assert_control_ack_updates_status("paused", SessionControlAction::Restart, "running").await;
}
