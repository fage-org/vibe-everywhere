use axum::{
    extract::{ConnectInfo, Extension, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use uuid::Uuid;

use ed25519_dalek::{Signer, SigningKey};
use ve_server::{
    api::auth::{
        daemon_hello, pair, pairing_status, DaemonHelloRequest, PairRequest, PairingStatusQuery,
    },
    config::{Config, DatabaseBackend},
    db::{install_drivers, run_migrations, DbPool},
    hub::Hub,
    state::AppState,
};
use ve_shared::{jwt::Claims, pairing_proof::PairingProof};

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
        std::env::temp_dir().join(format!("ve-pairing-status-test-{}.db", Uuid::new_v4()));
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
async fn pairing_status_returns_pending_before_pair_completes() {
    let state = setup_state().await;

    let hello = daemon_hello(
        State(state.clone()),
        remote_addr(),
        Json(DaemonHelloRequest {
            pair_code: "PARKAA".to_string(),
            host_name: "host-poll".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(1),
        }),
    )
    .await
    .unwrap()
    .0;

    let response = pairing_status(
        State(state),
        pairing_headers(&hello.pairing_secret),
        Query(PairingStatusQuery {
            host_id: hello.host_id,
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.status, "pending");
    assert!(response.daemon_token.is_none());
}

#[tokio::test]
async fn pairing_status_returns_daemon_token_once_after_pair_completes() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;

    let hello = daemon_hello(
        State(state.clone()),
        remote_addr(),
        Json(DaemonHelloRequest {
            pair_code: "PARKBB".to_string(),
            host_name: "host-paired".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(2),
        }),
    )
    .await
    .unwrap()
    .0;

    let claims = Claims::for_client_bootstrap(device_id, "device", chrono::Duration::hours(1));
    let _ = pair(
        State(state.clone()),
        Extension(claims),
        Json(PairRequest {
            pair_code: "PARKBB".to_string(),
        }),
    )
    .await
    .unwrap();

    let response = pairing_status(
        State(state.clone()),
        pairing_headers(&hello.pairing_secret),
        Query(PairingStatusQuery {
            host_id: hello.host_id,
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.status, "paired");
    assert!(response.daemon_token.is_some());

    let error = pairing_status(
        State(state),
        pairing_headers(&hello.pairing_secret),
        Query(PairingStatusQuery {
            host_id: hello.host_id,
        }),
    )
    .await
    .unwrap_err();

    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn pairing_status_rejects_wrong_secret() {
    let state = setup_state().await;

    let hello = daemon_hello(
        State(state.clone()),
        remote_addr(),
        Json(DaemonHelloRequest {
            pair_code: "PARKCC".to_string(),
            host_name: "host-secret-check".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(3),
        }),
    )
    .await
    .unwrap()
    .0;

    let error = pairing_status(
        State(state),
        pairing_headers("wrong-secret"),
        Query(PairingStatusQuery {
            host_id: hello.host_id,
        }),
    )
    .await
    .unwrap_err();

    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
}
