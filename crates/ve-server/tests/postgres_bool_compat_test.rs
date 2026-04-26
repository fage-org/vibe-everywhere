#![cfg(feature = "postgres")]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Extension, Path, Query, State},
    http::HeaderMap,
    Json,
};
use ed25519_dalek::{Signer, SigningKey};
use uuid::Uuid;
use ve_server::{
    api::{
        auth::{
            daemon_hello, pair, pairing_status, DaemonHelloRequest, PairRequest, PairingStatusQuery,
        },
        sessions::get_session,
        settings::{
            get_notification_preferences, update_notification_preferences,
            UpdateNotificationPreferencesRequest,
        },
        workspaces::{get_workspace, update_workspace, UpdateWorkspaceRequest},
    },
    authz::WorkspaceAccess,
    config::{Config, DatabaseBackend},
    db::{self, DbPool},
    hub::Hub,
    state::AppState,
};
use ve_shared::{jwt::Claims, pairing_proof::PairingProof};

const POSTGRES_TEST_DATABASE_URL_ENV: &str = "VE_POSTGRES_TEST_DATABASE_URL";

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

async fn setup_postgres_state() -> Option<PostgresTestState> {
    db::install_drivers();
    let base_url = match std::env::var(POSTGRES_TEST_DATABASE_URL_ENV) {
        Ok(url) => url,
        Err(_) => return None,
    };

    let schema_name = format!("bool_compat_{}", Uuid::new_v4().simple());
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
    .bind("/tmp/postgres-workspace")
    .bind("postgres-workspace")
    .bind(false)
    .bind(true)
    .execute(&state.db)
    .await
    .unwrap();
}

fn workspace_access(device_id: Uuid, host_id: Uuid, workspace_id: Uuid) -> WorkspaceAccess {
    WorkspaceAccess {
        device_id,
        workspace_id,
        host_id,
        path: "/tmp/postgres-workspace".to_string(),
        display_name: "postgres-workspace".to_string(),
        is_favorited: false,
        last_used_at: None,
        exists_on_host: true,
        created_at: "2026-01-01 00:00:00".to_string(),
        updated_at: "2026-01-01 00:00:00".to_string(),
    }
}

async fn seed_session(
    state: &Arc<AppState>,
    device_id: Uuid,
    host_id: Uuid,
    workspace_id: Uuid,
    session_id: Uuid,
) {
    sqlx::query(
        r#"
        INSERT INTO sessions (
            session_id, title, host_id, workspace_id, agent_type, status,
            unread_event_count, pending_permission_count, can_resume_cross_device
        )
        VALUES ($1, $2, $3, $4, $5, 'running', 0, 0, $6)
        "#,
    )
    .bind(session_id.to_string())
    .bind("postgres-session")
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind("claude_code")
    .bind(true)
    .execute(&state.db)
    .await
    .unwrap();

    sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(session_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();
}

fn remote_addr() -> ConnectInfo<SocketAddr> {
    ConnectInfo(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        4000,
    ))
}

fn pairing_headers(secret: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-pairing-secret", secret.parse().unwrap());
    headers
}

fn test_pairing_proof(seed: u8) -> PairingProof {
    let signing_key = SigningKey::from_bytes(&[seed; 32]);
    let verifying_key = signing_key.verifying_key();
    let installation_id = hex::encode(verifying_key.to_bytes());
    let signature = signing_key.sign(installation_id.as_bytes());

    PairingProof {
        installation_id,
        public_key: hex::encode(verifying_key.to_bytes()),
        signature: hex::encode(signature.to_bytes()),
    }
}

#[tokio::test]
async fn postgres_auth_pairing_flow_handles_boolean_columns() {
    let Some(test_state) = setup_postgres_state().await else {
        eprintln!(
            "skipping postgres auth compatibility test because {} is not set",
            POSTGRES_TEST_DATABASE_URL_ENV
        );
        return;
    };

    let state = test_state.state.clone();
    let device_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;

    let hello = daemon_hello(
        State(state.clone()),
        remote_addr(),
        Json(DaemonHelloRequest {
            pair_code: "PARKPG".to_string(),
            host_name: "pg-host".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(7),
        }),
    )
    .await
    .unwrap()
    .0;

    let pending = pairing_status(
        State(state.clone()),
        pairing_headers(&hello.pairing_secret),
        Query(PairingStatusQuery {
            host_id: hello.host_id,
        }),
    )
    .await
    .unwrap()
    .0;
    assert_eq!(pending.status, "pending");
    assert!(pending.daemon_token.is_none());

    let bootstrap_claims =
        Claims::for_client_bootstrap(device_id, "device", chrono::Duration::hours(1));
    let paired = pair(
        State(state.clone()),
        Extension(bootstrap_claims),
        Json(PairRequest {
            pair_code: "PARKPG".to_string(),
        }),
    )
    .await
    .unwrap()
    .0;
    assert_eq!(paired.host_id, hello.host_id);

    let status = pairing_status(
        State(state),
        pairing_headers(&hello.pairing_secret),
        Query(PairingStatusQuery {
            host_id: hello.host_id,
        }),
    )
    .await
    .unwrap()
    .0;
    assert_eq!(status.status, "paired");
    assert!(status.daemon_token.is_some());

    test_state.cleanup().await;
}

#[tokio::test]
async fn postgres_workspace_routes_handle_boolean_columns() {
    let Some(test_state) = setup_postgres_state().await else {
        eprintln!(
            "skipping postgres workspace compatibility test because {} is not set",
            POSTGRES_TEST_DATABASE_URL_ENV
        );
        return;
    };

    let state = test_state.state.clone();
    let device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;
    seed_host_with_access(&state, device_id, host_id).await;
    seed_workspace(&state, host_id, workspace_id).await;

    let workspace = get_workspace(workspace_access(device_id, host_id, workspace_id))
    .await
    .unwrap()
    .0;
    assert!(!workspace.is_favorited);
    assert!(workspace.exists_on_host);

    let updated = update_workspace(
        State(state.clone()),
        workspace_access(device_id, host_id, workspace_id),
        Json(UpdateWorkspaceRequest {
            display_name: Some("renamed".to_string()),
            is_favorited: Some(true),
        }),
    )
    .await
    .unwrap()
    .0;
    assert!(updated.is_favorited);
    assert_eq!(updated.display_name, "renamed");

    let row: (bool,) =
        sqlx::query_as("SELECT is_favorited FROM workspaces WHERE workspace_id = $1")
            .bind(workspace_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert!(row.0);

    test_state.cleanup().await;
}

#[tokio::test]
async fn postgres_settings_routes_handle_boolean_columns() {
    let Some(test_state) = setup_postgres_state().await else {
        eprintln!(
            "skipping postgres settings compatibility test because {} is not set",
            POSTGRES_TEST_DATABASE_URL_ENV
        );
        return;
    };

    let state = test_state.state.clone();
    let device_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;
    let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));

    let updated = update_notification_preferences(
        Extension(claims.clone()),
        State(state.clone()),
        Json(UpdateNotificationPreferencesRequest {
            enabled: Some(false),
            permission_request_enabled: None,
            task_completed_enabled: None,
            task_failed_enabled: Some(false),
            session_error_enabled: None,
        }),
    )
    .await
    .unwrap()
    .0;
    assert!(!updated.enabled);
    assert!(!updated.task_failed_enabled);

    let fetched = get_notification_preferences(Extension(claims), State(state.clone()))
        .await
        .unwrap()
        .0;
    assert!(!fetched.enabled);
    assert!(!fetched.task_failed_enabled);

    let row: (bool, bool) = sqlx::query_as(
        "SELECT enabled, task_failed_enabled FROM notification_preferences WHERE device_id = $1",
    )
    .bind(device_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(row, (false, false));

    test_state.cleanup().await;
}

#[tokio::test]
async fn postgres_session_routes_handle_boolean_columns() {
    let Some(test_state) = setup_postgres_state().await else {
        eprintln!(
            "skipping postgres session compatibility test because {} is not set",
            POSTGRES_TEST_DATABASE_URL_ENV
        );
        return;
    };

    let state = test_state.state.clone();
    let device_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;
    seed_host_with_access(&state, device_id, host_id).await;
    seed_workspace(&state, host_id, workspace_id).await;
    seed_session(&state, device_id, host_id, workspace_id, session_id).await;

    let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
    let session = get_session(State(state.clone()), Extension(claims), Path(session_id))
        .await
        .unwrap()
        .0;

    assert_eq!(session.session_id, session_id);
    assert!(session.can_resume_cross_device);

    test_state.cleanup().await;
}
