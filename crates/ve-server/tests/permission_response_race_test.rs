use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::Duration;
use http_body_util::BodyExt;
use tokio::sync::mpsc;
use tower::ServiceExt;
use uuid::Uuid;
use ve_server::{
    api::permissions,
    build_app,
    config::Config,
    db,
    hub::{Hub, WsSender},
    state::AppState,
};
use ve_shared::{
    jwt::JwtManager,
    models::{PermissionDecision, PermissionRequest},
    proto::{DaemonMessage, WsEnvelope},
    types::PermissionStatus,
};

#[cfg(feature = "postgres")]
const POSTGRES_TEST_DATABASE_URL_ENV: &str = "VE_POSTGRES_TEST_DATABASE_URL";

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

fn jwt_manager(config: &Config) -> JwtManager {
    JwtManager::new(
        &config.jwt_secret,
        Duration::seconds(config.jwt_expiration_secs as i64),
    )
}

async fn respond_permission_via_route(
    state: Arc<AppState>,
    device_id: Uuid,
    permission_id: Uuid,
    decision: PermissionDecision,
) -> (StatusCode, String) {
    let token = jwt_manager(state.config.as_ref())
        .create_client_token(device_id, "device")
        .unwrap();
    let jwt_manager = Arc::new(jwt_manager(state.config.as_ref()));

    let response = build_app(state.clone(), jwt_manager, state.config.as_ref())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/permissions/{permission_id}/respond"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({ "decision": decision })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

fn parse_permission_response(body: &str) -> PermissionRequest {
    serde_json::from_str(body).unwrap()
}

async fn setup_state() -> Arc<AppState> {
    db::install_drivers();
    let db_name = format!("permission_response_race_{}.db", Uuid::new_v4());
    let db_url = format!("sqlite:/tmp/{}?mode=rwc", db_name);
    let config = test_config(db_url);
    let pool = db::create_pool(&config).await.unwrap();
    db::run_migrations(&pool, config.database_backend())
        .await
        .unwrap();
    Arc::new(AppState::new(pool, Hub::new(), config))
}

#[cfg(feature = "postgres")]
struct PostgresTestState {
    state: Arc<AppState>,
    admin_pool: db::DbPool,
    schema_name: String,
}

#[cfg(feature = "postgres")]
impl PostgresTestState {
    async fn cleanup(self) {
        self.state.db.close().await;
        let _ = sqlx::query(&format!(
            "DROP SCHEMA IF EXISTS \"{}\" CASCADE",
            self.schema_name
        ))
        .execute(&self.admin_pool)
        .await;
        self.admin_pool.close().await;
    }
}

#[cfg(feature = "postgres")]
async fn setup_postgres_state() -> Option<PostgresTestState> {
    db::install_drivers();
    let base_url = match std::env::var(POSTGRES_TEST_DATABASE_URL_ENV) {
        Ok(url) => url,
        Err(_) => return None,
    };

    let schema_name = format!("permission_race_{}", Uuid::new_v4().simple());
    let admin_pool = db::DbPool::connect(&base_url).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA \"{schema_name}\""))
        .execute(&admin_pool)
        .await
        .unwrap();

    let separator = if base_url.contains('?') { '&' } else { '?' };
    let database_url = format!("{base_url}{separator}options=-csearch_path%3D{schema_name}");
    let config = test_config(database_url);
    let pool = db::create_pool(&config).await.unwrap();
    db::run_migrations(&pool, config.database_backend())
        .await
        .unwrap();

    Some(PostgresTestState {
        state: Arc::new(AppState::new(pool, Hub::new(), config)),
        admin_pool,
        schema_name,
    })
}

async fn seed_fixture(state: &AppState) -> (Uuid, Uuid, Uuid, Uuid, Uuid) {
    let device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let permission_id = Uuid::new_v4();
    let sibling_permission_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"INSERT INTO client_devices (device_id, device_name, device_type, legacy_acl, server_url)
           VALUES ($1, 'device', 'desktop', 0, 'http://localhost')"#,
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
        r#"INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, pending_permission_count, created_at, updated_at)
           VALUES ($1, 'test', $2, $3, 'claude_code', 'waiting_approval', 2, $4, $4)"#,
    )
    .bind(session_id.to_string())
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind(&now)
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

    for permission in [permission_id, sibling_permission_id] {
        sqlx::query(
            r#"INSERT INTO permission_requests (permission_id, session_id, risk_type, summary, status, created_at)
               VALUES ($1, $2, 'exec_cmd', 'approve command', 'pending', $3)"#,
        )
        .bind(permission.to_string())
        .bind(session_id.to_string())
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();
    }

    (
        device_id,
        host_id,
        session_id,
        permission_id,
        sibling_permission_id,
    )
}

async fn register_fake_daemon(state: &AppState, host_id: Uuid) -> mpsc::Receiver<WsEnvelope> {
    let (tx, rx): (WsSender, mpsc::Receiver<WsEnvelope>) = mpsc::channel(8);
    state.hub.register_daemon(host_id, tx).await;
    rx
}

fn decision_for_status(status: PermissionStatus) -> PermissionDecision {
    match status {
        PermissionStatus::ApprovedOnce => PermissionDecision::ApproveOnce,
        PermissionStatus::DeniedOnce => PermissionDecision::DenyOnce,
        other => panic!("unexpected permission status: {other:?}"),
    }
}

async fn assert_single_winner_under_concurrency(state: Arc<AppState>) {
    let (device_id, host_id, session_id, permission_id, _sibling_permission_id) =
        seed_fixture(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;
    let _race_hook =
        permissions::test_support::install_permission_response_race_hook([permission_id]);

    let spawn_request = |decision: PermissionDecision| {
        let state = state.clone();
        tokio::spawn(async move {
            respond_permission_via_route(state, device_id, permission_id, decision).await
        })
    };

    let approve_task = spawn_request(PermissionDecision::ApproveOnce);
    let deny_task = spawn_request(PermissionDecision::DenyOnce);

    let approve_result = approve_task.await.unwrap();
    let deny_result = deny_task.await.unwrap();

    let mut success_statuses = Vec::new();
    for (status, body) in [approve_result, deny_result] {
        assert_eq!(status, StatusCode::OK);
        success_statuses.push(parse_permission_response(&body).status);
    }

    assert_eq!(success_statuses.len(), 2);

    let final_state: (String, i64,) = sqlx::query_as(
        "SELECT permission_requests.status, sessions.pending_permission_count FROM permission_requests JOIN sessions ON sessions.session_id = permission_requests.session_id WHERE permission_id = $1"
    )
    .bind(permission_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();

    let winning_status = match final_state.0.as_str() {
        "approved_once" => PermissionStatus::ApprovedOnce,
        "denied_once" => PermissionStatus::DeniedOnce,
        other => panic!("unexpected persisted winning status: {other}"),
    };
    assert!(success_statuses
        .iter()
        .all(|status| *status == winning_status));
    assert_eq!(final_state.1, 1);

    let outbound = tokio::time::timeout(std::time::Duration::from_secs(1), daemon_rx.recv())
        .await
        .expect("timed out waiting for permission response")
        .expect("daemon command");
    assert_eq!(outbound.r#type, "permission_response");

    let payload: DaemonMessage = serde_json::from_value(outbound.payload.clone()).unwrap();
    match payload {
        DaemonMessage::PermissionResponse {
            permission_id: actual_permission_id,
            session_id: actual_session_id,
            decision,
        } => {
            assert_eq!(actual_permission_id, permission_id);
            assert_eq!(actual_session_id, session_id);
            assert_eq!(decision, decision_for_status(winning_status));
        }
        other => panic!("unexpected daemon message: {other:?}"),
    }

    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(100), daemon_rx.recv())
            .await
            .is_err()
    );
}

async fn assert_distinct_permissions_keep_count(state: Arc<AppState>) {
    let (device_id, host_id, session_id, permission_id, sibling_permission_id) =
        seed_fixture(&state).await;
    let mut daemon_rx = register_fake_daemon(&state, host_id).await;
    let _race_hook = permissions::test_support::install_permission_response_race_hook([
        permission_id,
        sibling_permission_id,
    ]);

    let spawn_request = |permission_id: Uuid, decision: PermissionDecision| {
        let state = state.clone();
        tokio::spawn(async move {
            respond_permission_via_route(state, device_id, permission_id, decision).await
        })
    };

    let first_task = spawn_request(permission_id, PermissionDecision::ApproveOnce);
    let second_task = spawn_request(sibling_permission_id, PermissionDecision::DenyOnce);

    let (first_status, first_body) = first_task.await.unwrap();
    let (second_status, second_body) = second_task.await.unwrap();
    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(second_status, StatusCode::OK);

    let first_response = parse_permission_response(&first_body);
    let second_response = parse_permission_response(&second_body);

    assert_eq!(first_response.permission_id, permission_id);
    assert_eq!(first_response.session_id, session_id);
    assert_eq!(first_response.status, PermissionStatus::ApprovedOnce);
    assert_eq!(second_response.permission_id, sibling_permission_id);
    assert_eq!(second_response.session_id, session_id);
    assert_eq!(second_response.status, PermissionStatus::DeniedOnce);

    let final_state: (i64,) =
        sqlx::query_as("SELECT pending_permission_count FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(final_state.0, 0);

    let mut received_messages = Vec::new();
    for _ in 0..2 {
        let outbound = tokio::time::timeout(std::time::Duration::from_secs(1), daemon_rx.recv())
            .await
            .expect("timed out waiting for permission response")
            .expect("daemon command");
        assert_eq!(outbound.r#type, "permission_response");

        let payload: DaemonMessage = serde_json::from_value(outbound.payload.clone()).unwrap();
        match payload {
            DaemonMessage::PermissionResponse {
                permission_id: actual_permission_id,
                session_id: actual_session_id,
                decision,
            } => {
                assert_eq!(actual_session_id, session_id);
                received_messages.push((actual_permission_id, decision));
            }
            other => panic!("unexpected daemon message: {other:?}"),
        }
    }

    received_messages.sort_by_key(|(permission_id, _)| *permission_id);
    let mut expected_messages = vec![
        (permission_id, PermissionDecision::ApproveOnce),
        (sibling_permission_id, PermissionDecision::DenyOnce),
    ];
    expected_messages.sort_by_key(|(permission_id, _)| *permission_id);
    assert_eq!(received_messages, expected_messages);

    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(100), daemon_rx.recv())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn respond_permission_allows_only_one_winner_under_concurrency() {
    assert_single_winner_under_concurrency(setup_state().await).await;
}

#[tokio::test]
async fn respond_permission_concurrently_updates_two_distinct_permissions_without_restoring_stale_count(
) {
    assert_distinct_permissions_keep_count(setup_state().await).await;
}

#[tokio::test]
async fn respond_permission_route_rejects_hidden_permission_before_handler_logic() {
    let state = setup_state().await;
    let (device_id, _host_id, hidden_session_id, permission_id, _sibling_permission_id) =
        seed_fixture(&state).await;

    sqlx::query("DELETE FROM device_session_access WHERE device_id = $1")
        .bind(device_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

    let (status, body) = respond_permission_via_route(
        state,
        device_id,
        permission_id,
        PermissionDecision::ApproveOnce,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains(&permission_id.to_string()));
    assert!(!body.contains(&hidden_session_id.to_string()));
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn respond_permission_allows_only_one_winner_under_concurrency_on_postgres() {
    let Some(state) = setup_postgres_state().await else {
        eprintln!(
            "skipping postgres race test because {} is not set",
            POSTGRES_TEST_DATABASE_URL_ENV
        );
        return;
    };

    assert_single_winner_under_concurrency(state.state.clone()).await;
    state.cleanup().await;
}

#[cfg(feature = "postgres")]
#[tokio::test]
async fn respond_permission_keeps_pending_count_stable_on_postgres() {
    let Some(state) = setup_postgres_state().await else {
        eprintln!(
            "skipping postgres race test because {} is not set",
            POSTGRES_TEST_DATABASE_URL_ENV
        );
        return;
    };

    assert_distinct_permissions_keep_count(state.state.clone()).await;
    state.cleanup().await;
}
