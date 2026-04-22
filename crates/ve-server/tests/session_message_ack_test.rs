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
    error::ServerError,
    hub::{Hub, WsSender},
    state::AppState,
};
use ve_shared::{
    jwt::Claims,
    proto::{AckPayload, ErrorPayload, WsEnvelope},
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
    let db_name = format!("session_message_ack_{}.db", Uuid::new_v4());
    let db_url = format!("sqlite:/tmp/{}?mode=rwc", db_name);
    let config = test_config(db_url);
    let pool = db::create_pool(&config).await.unwrap();
    db::run_migrations(&pool, config.database_backend())
        .await
        .unwrap();
    Arc::new(AppState::new(pool, Hub::new(), config))
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

#[tokio::test]
async fn send_message_waits_for_daemon_ack_before_returning_success() {
    let state = setup_state().await;
    let (device_id, host_id, _workspace_id, session_id) = seed_fixture(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;

    let content = "hello from test".to_string();
    let state_for_request = state.clone();
    let request_content = content.clone();
    let request_task = tokio::spawn(async move {
        sessions::send_message(
            State(state_for_request),
            Extension(client_claims(device_id)),
            HeaderMap::new(),
            Path(session_id),
            Json(sessions::SendMessageRequest {
                content: request_content,
            }),
        )
        .await
    });

    let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
        .await
        .expect("timed out waiting for send_message command")
        .expect("daemon command");
    assert_eq!(outbound.r#type, "send_message");

    let message_count_before: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM session_messages WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(message_count_before.0, 0);
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

    let stored_message: (String,) = sqlx::query_as(
        "SELECT content FROM session_messages WHERE session_id = $1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(session_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(stored_message.0, content);
}

#[tokio::test]
async fn send_message_returns_conflict_when_daemon_reports_error() {
    let state = setup_state().await;
    let (device_id, host_id, _workspace_id, session_id) = seed_fixture(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;

    let state_for_request = state.clone();
    let request_task = tokio::spawn(async move {
        sessions::send_message(
            State(state_for_request),
            Extension(client_claims(device_id)),
            HeaderMap::new(),
            Path(session_id),
            Json(sessions::SendMessageRequest {
                content: "will fail".to_string(),
            }),
        )
        .await
    });

    let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
        .await
        .expect("timed out waiting for send_message command")
        .expect("daemon command");
    assert_eq!(outbound.r#type, "send_message");
    assert!(!request_task.is_finished());

    state
        .hub
        .complete_with_error(ErrorPayload {
            request_id: outbound.request_id.clone().unwrap(),
            error_code: "CLI_NOT_RUNNING".to_string(),
            error_message: "CLI not running".to_string(),
        })
        .await;

    let error = tokio::time::timeout(Duration::from_secs(1), request_task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    match error {
        ServerError::Conflict(message) => assert_eq!(message, "Daemon command failed"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[tokio::test]
async fn send_message_sanitizes_failed_ack_error_details() {
    let state = setup_state().await;
    let (device_id, host_id, _workspace_id, session_id) = seed_fixture(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;

    let state_for_request = state.clone();
    let request_task = tokio::spawn(async move {
        sessions::send_message(
            State(state_for_request),
            Extension(client_claims(device_id)),
            HeaderMap::new(),
            Path(session_id),
            Json(sessions::SendMessageRequest {
                content: "will nack".to_string(),
            }),
        )
        .await
    });

    let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
        .await
        .expect("timed out waiting for send_message command")
        .expect("daemon command");
    assert_eq!(outbound.r#type, "send_message");
    assert!(!request_task.is_finished());

    state
        .hub
        .complete_with_ack(AckPayload {
            request_id: outbound.request_id.clone().unwrap(),
            success: false,
            error: Some(
                "daemon failed while reading /root/vibe-remote/private/prompt.txt".to_string(),
            ),
        })
        .await;

    let error = tokio::time::timeout(Duration::from_secs(1), request_task)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    match error {
        ServerError::Conflict(message) => assert_eq!(message, "Daemon command failed"),
        other => panic!("unexpected error: {other:?}"),
    }
}
