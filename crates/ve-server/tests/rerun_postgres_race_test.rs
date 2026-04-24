#![cfg(feature = "postgres")]

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
    api::sessions::{self, ControlRequest},
    config::{Config, DatabaseBackend},
    db::{self, DbPool},
    hub::Hub,
    state::AppState,
};
use ve_shared::jwt::Claims;

const POSTGRES_TEST_DATABASE_URL_ENV: &str = "VE_POSTGRES_TEST_DATABASE_URL";

struct PostgresTestState {
    state: Arc<AppState>,
    admin_pool: DbPool,
    schema_name: String,
}

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

async fn setup_postgres_state() -> Option<PostgresTestState> {
    db::install_drivers();
    let base_url = match std::env::var(POSTGRES_TEST_DATABASE_URL_ENV) {
        Ok(url) => url,
        Err(_) => return None,
    };

    let schema_name = format!("rerun_race_{}", Uuid::new_v4().simple());
    let admin_pool = DbPool::connect(&base_url).await.unwrap();
    sqlx::query(&format!("CREATE SCHEMA \"{schema_name}\""))
        .execute(&admin_pool)
        .await
        .unwrap();

    let separator = if base_url.contains('?') { '&' } else { '?' };
    let database_url = format!("{base_url}{separator}options=-csearch_path%3D{schema_name}");
    let config = test_config(database_url);
    let pool = db::create_pool(&config).await.unwrap();
    db::run_migrations(&pool, DatabaseBackend::Postgres)
        .await
        .unwrap();

    let jwt_manager = Arc::new(ve_shared::jwt::JwtManager::new(
        &config.jwt_secret,
        config.jwt_expiration(),
    ));

    Some(PostgresTestState {
        state: Arc::new(AppState::new(pool, Hub::new(), config, jwt_manager)),
        admin_pool,
        schema_name,
    })
}

async fn seed_registered_device(state: &Arc<AppState>, device_id: Uuid) {
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
}

async fn seed_host_with_access(state: &Arc<AppState>, device_id: Uuid, host_id: Uuid) {
    sqlx::query(
        "INSERT INTO hosts (host_id, host_name, platform, pair_status) VALUES ($1, $2, $3, 'paired')",
    )
    .bind(host_id.to_string())
    .bind("host")
    .bind("linux")
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(host_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();
}

async fn seed_workspace(state: &Arc<AppState>, host_id: Uuid, workspace_id: Uuid) {
    sqlx::query(
        r#"
        INSERT INTO workspaces (workspace_id, host_id, path, display_name, is_favorited, exists_on_host)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(workspace_id.to_string())
    .bind(host_id.to_string())
    .bind("/tmp/postgres-rerun-workspace")
    .bind("postgres-rerun-workspace")
    .bind(false)
    .bind(true)
    .execute(&state.db)
    .await
    .unwrap();
}

async fn seed_archived_session(
    state: &Arc<AppState>,
    device_id: Uuid,
    host_id: Uuid,
    workspace_id: Uuid,
    archived_session_id: Uuid,
) {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO sessions (
            session_id, title, host_id, workspace_id, agent_type, status,
            claude_session_id, can_resume_cross_device, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, 'archived', $6, $7, $8, $8)
        "#,
    )
    .bind(archived_session_id.to_string())
    .bind("archived rerun")
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind("claude_code")
    .bind("claude-session-1")
    .bind(true)
    .bind(&now)
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)",
    )
    .bind(device_id.to_string())
    .bind(archived_session_id.to_string())
    .execute(&state.db)
    .await
    .unwrap();
}

#[tokio::test]
async fn postgres_archived_rerun_handles_concurrent_requests_without_duplicate_live_sessions() {
    let Some(test_state) = setup_postgres_state().await else {
        eprintln!(
            "skipping postgres rerun race test because {} is not set",
            POSTGRES_TEST_DATABASE_URL_ENV
        );
        return;
    };

    let state = test_state.state.clone();
    let device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let archived_session_id = Uuid::new_v4();

    seed_registered_device(&state, device_id).await;
    seed_host_with_access(&state, device_id, host_id).await;
    seed_workspace(&state, host_id, workspace_id).await;
    seed_archived_session(&state, device_id, host_id, workspace_id, archived_session_id).await;

    let (daemon_tx, mut daemon_rx) = mpsc::channel(8);
    state.hub.register_daemon(host_id, daemon_tx).await;

    let _race_hook = sessions::test_support::install_archived_rerun_race_hook(
        archived_session_id.to_string(),
    );

    let spawn_request = || {
        let state = state.clone();
        tokio::spawn(async move {
            sessions::control_session(
                State(state),
                Extension(Claims::for_client(
                    device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        })
    };

    let first_task = spawn_request();
    let second_task = spawn_request();

    let outbound = daemon_rx.recv().await.unwrap();
    assert_eq!(outbound.r#type, "rerun_session");

    tokio::time::sleep(Duration::from_millis(100)).await;

    state
        .hub
        .complete_with_ack(ve_shared::proto::AckPayload {
            request_id: outbound.request_id.clone().unwrap(),
            success: true,
            error: None,
        })
        .await;

    let first = first_task.await.unwrap();
    let second = second_task.await.unwrap();

    let mut successful_session_ids = Vec::new();
    let mut saw_dispatching_conflict = false;

    for result in [first, second] {
        match result {
            Ok(response) => {
                let session_id = response.0["session_id"]
                    .as_str()
                    .unwrap()
                    .parse::<Uuid>()
                    .unwrap();
                successful_session_ids.push(session_id);
            }
            Err(ve_server::error::ServerError::Conflict(message))
                if message == "Archived session rerun is still dispatching; retry shortly" =>
            {
                saw_dispatching_conflict = true;
            }
            Err(other) => panic!("unexpected rerun result: {other:?}"),
        }
    }

    assert!(
        !successful_session_ids.is_empty(),
        "at least one concurrent rerun request should succeed"
    );

    let active_reruns: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT session_id, status
        FROM sessions
        WHERE rerun_from_session_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(archived_session_id.to_string())
    .fetch_all(&state.db)
    .await
    .unwrap();

    assert_eq!(active_reruns.len(), 1);
    assert_eq!(active_reruns[0].1, "pending");

    if successful_session_ids.len() == 2 {
        assert_eq!(successful_session_ids[0], successful_session_ids[1]);
    } else {
        assert!(
            saw_dispatching_conflict,
            "losing concurrent request should report the dispatching conflict"
        );
    }

    assert!(daemon_rx.try_recv().is_err());

    test_state.cleanup().await;
}
