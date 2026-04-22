use axum::{
    extract::{ConnectInfo, Extension, State},
    Json,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use uuid::Uuid;

use ed25519_dalek::{Signer, SigningKey};
use ve_server::{
    api::auth::{daemon_hello, pair, DaemonHelloRequest, PairRequest},
    config::{Config, DatabaseBackend},
    db::{install_drivers, run_migrations, DbPool},
    error::ServerError,
    hub::Hub,
    state::AppState,
    validation::PAIR_CODE_LENGTH,
};
use ve_shared::{
    jwt::{Claims, JwtManager, TokenType},
    pairing_proof::PairingProof,
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
    let temp_db = std::env::temp_dir().join(format!("ve-pair-atomic-test-{}.db", Uuid::new_v4()));
    let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
    let pool = DbPool::connect(&database_url).await.unwrap();
    run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap();
    Arc::new(AppState::new(pool, Hub::new(), test_config(database_url)))
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

fn remote_addr() -> ConnectInfo<SocketAddr> {
    ConnectInfo(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        4000,
    ))
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

async fn create_pending_pairing(state: &Arc<AppState>, pair_code: &str) -> Uuid {
    let response = daemon_hello(
        State(state.clone()),
        remote_addr(),
        Json(DaemonHelloRequest {
            pair_code: pair_code.to_string(),
            host_name: "host".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(1),
        }),
    )
    .await
    .unwrap()
    .0;

    response.host_id
}

async fn pairing_used_flag(state: &Arc<AppState>, pair_code: &str) -> i64 {
    let row: (i64,) = sqlx::query_as("SELECT used FROM pairing_codes WHERE pair_code = $1")
        .bind(pair_code)
        .fetch_one(&state.db)
        .await
        .unwrap();
    row.0
}

async fn host_pair_status(state: &Arc<AppState>, host_id: Uuid) -> String {
    let row: (String,) = sqlx::query_as("SELECT pair_status FROM hosts WHERE host_id = $1")
        .bind(host_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
    row.0
}

async fn host_access_count(state: &Arc<AppState>, device_id: Uuid, host_id: Uuid) -> i64 {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM device_host_access WHERE device_id = $1 AND host_id = $2",
    )
    .bind(device_id.to_string())
    .bind(host_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    row.0
}

#[tokio::test]
async fn pair_rejects_stale_client_token_before_consuming_code() {
    let state = setup_state().await;
    let host_id = create_pending_pairing(&state, "PARKAB").await;
    let stale_device_id = Uuid::new_v4();
    let claims =
        Claims::for_client_bootstrap(stale_device_id, "device", chrono::Duration::hours(1));

    for _ in 0..6 {
        let error = pair(
            State(state.clone()),
            Extension(claims.clone()),
            Json(PairRequest {
                pair_code: "PARKAB".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, ServerError::Unauthorized));
    }

    assert_eq!(pairing_used_flag(&state, "PARKAB").await, 0);
    assert_eq!(host_pair_status(&state, host_id).await, "pending");
    assert_eq!(host_access_count(&state, stale_device_id, host_id).await, 0);
}

#[tokio::test]
async fn pair_rejects_stale_client_token_before_validating_pair_code() {
    let state = setup_state().await;
    let stale_device_id = Uuid::new_v4();
    let claims =
        Claims::for_client_bootstrap(stale_device_id, "device", chrono::Duration::hours(1));
    let invalid_pair_code = "A".repeat(PAIR_CODE_LENGTH + 1);

    let error = pair(
        State(state),
        Extension(claims),
        Json(PairRequest {
            pair_code: invalid_pair_code,
        }),
    )
    .await
    .unwrap_err();

    assert!(matches!(error, ServerError::Unauthorized));
}

#[tokio::test]
async fn pair_rejects_formal_client_token() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;
    let _host_id = create_pending_pairing(&state, "PARKBC").await;
    let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));

    let error = pair(
        State(state.clone()),
        Extension(claims),
        Json(PairRequest {
            pair_code: "PARKBC".to_string(),
        }),
    )
    .await
    .unwrap_err();

    assert!(matches!(error, ServerError::Unauthorized));
    assert_eq!(pairing_used_flag(&state, "PARKBC").await, 0);
}

#[tokio::test]
async fn pair_rejects_second_redemption_after_success() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;
    let host_id = create_pending_pairing(&state, "PARKCD").await;
    let claims = Claims::for_client_bootstrap(device_id, "device", chrono::Duration::hours(1));

    let first = pair(
        State(state.clone()),
        Extension(claims.clone()),
        Json(PairRequest {
            pair_code: "PARKCD".to_string(),
        }),
    )
    .await
    .unwrap()
    .0;

    let client_claims = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration())
        .decode(&first.token)
        .unwrap();

    assert_eq!(client_claims.r#type, TokenType::Client);
    assert_eq!(client_claims.subject_uuid().unwrap(), device_id);
    assert_eq!(first.host_id, host_id);
    assert_eq!(pairing_used_flag(&state, "PARKCD").await, 1);
    assert_eq!(host_pair_status(&state, host_id).await, "paired");
    assert_eq!(host_access_count(&state, device_id, host_id).await, 1);

    let second = pair(
        State(state.clone()),
        Extension(claims),
        Json(PairRequest {
            pair_code: "PARKCD".to_string(),
        }),
    )
    .await
    .unwrap_err();

    assert!(matches!(second, ServerError::PairCodeUsed));
    assert_eq!(pairing_used_flag(&state, "PARKCD").await, 1);
    assert_eq!(host_pair_status(&state, host_id).await, "paired");
    assert_eq!(host_access_count(&state, device_id, host_id).await, 1);
}

#[tokio::test]
async fn pair_grants_new_host_access_to_legacy_devices_and_requester_only() {
    let state = setup_state().await;
    let legacy_device_id = Uuid::new_v4();
    let requester_device_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, legacy_acl, server_url) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(legacy_device_id.to_string())
    .bind("legacy-device")
    .bind("desktop")
    .bind(1)
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, legacy_acl, server_url) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(requester_device_id.to_string())
    .bind("requester-device")
    .bind("desktop")
    .bind(1)
    .bind("http://localhost")
    .execute(&state.db)
    .await
    .unwrap();

    let host_id = create_pending_pairing(&state, "PARKLG").await;
    let claims = Claims::for_client_bootstrap(
        requester_device_id,
        "requester-device",
        chrono::Duration::hours(1),
    );

    let _ = pair(
        State(state.clone()),
        Extension(claims),
        Json(PairRequest {
            pair_code: "PARKLG".to_string(),
        }),
    )
    .await
    .unwrap();

    assert_eq!(
        host_access_count(&state, legacy_device_id, host_id).await,
        1
    );
    assert_eq!(
        host_access_count(&state, requester_device_id, host_id).await,
        1
    );

    let requester_legacy_acl: i64 =
        sqlx::query_scalar("SELECT legacy_acl FROM client_devices WHERE device_id = $1")
            .bind(requester_device_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(requester_legacy_acl, 0);
}

#[tokio::test]
async fn daemon_hello_rejects_unrecoverable_supplied_pair_code() {
    let state = setup_state().await;

    let error = daemon_hello(
        State(state.clone()),
        remote_addr(),
        Json(DaemonHelloRequest {
            pair_code: " pair01! ".to_string(),
            host_name: "host".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(1),
        }),
    )
    .await
    .unwrap_err();

    assert!(matches!(error, ServerError::Validation(_)));
}

#[tokio::test]
async fn pair_accepts_canonicalized_pair_code_from_daemon_hello() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;
    let claims = Claims::for_client_bootstrap(device_id, "device", chrono::Duration::hours(1));

    let host_id = daemon_hello(
        State(state.clone()),
        remote_addr(),
        Json(DaemonHelloRequest {
            pair_code: "  parkef  ".to_string(),
            host_name: "host".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(1),
        }),
    )
    .await
    .unwrap()
    .0
    .host_id;

    let paired = pair(
        State(state.clone()),
        Extension(claims),
        Json(PairRequest {
            pair_code: "PARKEF".to_string(),
        }),
    )
    .await
    .unwrap()
    .0;

    let client_claims = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration())
        .decode(&paired.token)
        .unwrap();

    assert_eq!(client_claims.r#type, TokenType::Client);
    assert_eq!(client_claims.subject_uuid().unwrap(), device_id);
    assert_eq!(paired.host_id, host_id);
    assert_eq!(pairing_used_flag(&state, "PARKEF").await, 1);
    assert_eq!(host_pair_status(&state, host_id).await, "paired");
    assert_eq!(host_access_count(&state, device_id, host_id).await, 1);
}
