use axum::{
    extract::{ConnectInfo, Extension, State},
    response::IntoResponse,
    Json,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use uuid::Uuid;

use ve_server::{
    api::auth::{daemon_hello, pair, register_device, DaemonHelloRequest, PairRequest},
    config::{Config, DatabaseBackend},
    db::{install_drivers, run_migrations, DbPool},
    hub::Hub,
    state::AppState,
    validation::PAIR_CODE_LENGTH,
};
use ve_shared::{
    jwt::{Claims, JwtManager, TokenType},
    models::RegisterDeviceRequest,
    pairing_proof::PairingProof,
    types::DeviceType,
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
    let temp_db = std::env::temp_dir().join(format!("ve-auth-throttle-test-{}.db", Uuid::new_v4()));
    let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
    let pool = DbPool::connect(&database_url).await.unwrap();
    run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap();
    let config = test_config(database_url);
    let jwt_manager = Arc::new(JwtManager::new(&config.jwt_secret, config.jwt_expiration()));
    Arc::new(AppState::new(pool, Hub::new(), config, jwt_manager))
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

async fn pairing_count_for_host_name(state: &Arc<AppState>, host_name: &str) -> i64 {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM pairing_codes WHERE host_name = $1")
        .bind(host_name)
        .fetch_one(&state.db)
        .await
        .unwrap();
    row.0
}

fn remote_addr(octet: u8) -> ConnectInfo<SocketAddr> {
    ConnectInfo(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, octet)),
        4000,
    ))
}

fn valid_pair_code(index: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut value = index;
    let mut pair_code = String::with_capacity(PAIR_CODE_LENGTH);

    for _ in 0..PAIR_CODE_LENGTH {
        pair_code.push(CHARSET[value % CHARSET.len()] as char);
        value /= CHARSET.len();
    }

    pair_code
}

fn register_device_request(device_name: &str) -> RegisterDeviceRequest {
    RegisterDeviceRequest {
        device_name: device_name.to_string(),
        device_type: DeviceType::Desktop,
        server_url: "http://localhost".to_string(),
    }
}

fn test_pairing_proof(seed: u8) -> PairingProof {
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&[seed; 32]);
    let verifying_key = signing_key.verifying_key();
    let installation_id = hex::encode(verifying_key.to_bytes());
    let signature = ed25519_dalek::Signer::sign(&signing_key, installation_id.as_bytes());

    PairingProof {
        installation_id,
        public_key: hex::encode(verifying_key.to_bytes()),
        signature: hex::encode(signature.to_bytes()),
    }
}

#[tokio::test]
async fn register_device_throttles_repeated_requests_from_same_source_ip() {
    let state = setup_state().await;

    for index in 0..5 {
        let response = register_device(
            State(state.clone()),
            remote_addr(1),
            Json(register_device_request(&format!("device-{index}"))),
        )
        .await;

        assert!(response.is_ok());
    }

    let error = register_device(
        State(state.clone()),
        remote_addr(1),
        Json(register_device_request("device-6")),
    )
    .await
    .unwrap_err();

    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn register_device_invalid_name_does_not_consume_ip_throttle() {
    let state = setup_state().await;

    for _ in 0..6 {
        let error = register_device(
            State(state.clone()),
            remote_addr(1),
            Json(register_device_request("   ")),
        )
        .await
        .unwrap_err();

        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    let response = register_device(
        State(state.clone()),
        remote_addr(1),
        Json(register_device_request("device-ok")),
    )
    .await
    .unwrap()
    .0;

    let claims = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration())
        .decode(&response.token)
        .unwrap();

    assert_eq!(claims.r#type, TokenType::ClientBootstrap);
}

#[tokio::test]
async fn register_device_does_not_share_limit_across_source_ips() {
    let state = setup_state().await;

    for index in 0..6 {
        let _ = register_device(
            State(state.clone()),
            remote_addr(1),
            Json(register_device_request(&format!("device-a-{index}"))),
        )
        .await;
    }

    let response = register_device(
        State(state.clone()),
        remote_addr(2),
        Json(register_device_request("device-b")),
    )
    .await
    .unwrap()
    .0;

    let claims = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration())
        .decode(&response.token)
        .unwrap();

    assert_eq!(claims.r#type, TokenType::ClientBootstrap);
}

#[tokio::test]
async fn pair_rejects_invalid_pair_code_before_throttle_state_is_consumed() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;
    let claims = Claims::for_client_bootstrap(device_id, "device", chrono::Duration::hours(1));
    let invalid_pair_code = "A".repeat(PAIR_CODE_LENGTH + 1);

    for _ in 0..6 {
        let error = pair(
            State(state.clone()),
            Extension(claims.clone()),
            Json(PairRequest {
                pair_code: invalid_pair_code.clone(),
            }),
        )
        .await
        .unwrap_err();

        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn pair_throttles_repeated_invalid_attempts_for_same_device_and_code() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;
    let claims = Claims::for_client_bootstrap(device_id, "device", chrono::Duration::hours(1));
    let pair_code = valid_pair_code(1);

    for _ in 0..5 {
        let error = pair(
            State(state.clone()),
            Extension(claims.clone()),
            Json(PairRequest {
                pair_code: pair_code.clone(),
            }),
        )
        .await
        .unwrap_err();

        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::GONE);
    }

    let error = pair(
        State(state.clone()),
        Extension(claims),
        Json(PairRequest { pair_code }),
    )
    .await
    .unwrap_err();

    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn pair_throttles_same_device_across_different_pair_codes() {
    let state = setup_state().await;
    let device_id = Uuid::new_v4();
    seed_registered_device(&state, device_id).await;
    let claims = Claims::for_client_bootstrap(device_id, "device", chrono::Duration::hours(1));

    for index in 0..5 {
        let error = pair(
            State(state.clone()),
            Extension(claims.clone()),
            Json(PairRequest {
                pair_code: valid_pair_code(index),
            }),
        )
        .await
        .unwrap_err();

        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::GONE);
    }

    let error = pair(
        State(state.clone()),
        Extension(claims),
        Json(PairRequest {
            pair_code: valid_pair_code(99),
        }),
    )
    .await
    .unwrap_err();

    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn daemon_hello_invalid_pair_code_does_not_consume_source_ip_throttle() {
    let state = setup_state().await;

    for _ in 0..5 {
        let error = daemon_hello(
            State(state.clone()),
            remote_addr(1),
            Json(DaemonHelloRequest {
                pair_code: " bad01! ".to_string(),
                host_name: "host-b".to_string(),
                platform: "linux".to_string(),
                pairing_proof: test_pairing_proof(1),
            }),
        )
        .await
        .unwrap_err();

        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    let response = daemon_hello(
        State(state.clone()),
        remote_addr(1),
        Json(DaemonHelloRequest {
            pair_code: valid_pair_code(42),
            host_name: "host-b".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(1),
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.status, "pending");
    assert_eq!(pairing_count_for_host_name(&state, "host-b").await, 1);
}

#[tokio::test]
async fn daemon_hello_rejects_invalid_pairing_proof_before_persisting_pairing() {
    let state = setup_state().await;
    let mut pairing_proof = test_pairing_proof(2);
    pairing_proof.signature = "00".repeat(64);

    let error = daemon_hello(
        State(state.clone()),
        remote_addr(1),
        Json(DaemonHelloRequest {
            pair_code: valid_pair_code(11),
            host_name: "host-proof".to_string(),
            platform: "linux".to_string(),
            pairing_proof,
        }),
    )
    .await
    .unwrap_err();

    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(pairing_count_for_host_name(&state, "host-proof").await, 0);
}

#[tokio::test]
async fn daemon_hello_throttles_same_source_ip_across_different_installations() {
    let state = setup_state().await;

    for index in 0..5 {
        let response = daemon_hello(
            State(state.clone()),
            remote_addr(1),
            Json(DaemonHelloRequest {
                pair_code: valid_pair_code(index),
                host_name: " Same-Host ".to_string(),
                platform: "linux".to_string(),
                pairing_proof: test_pairing_proof(index as u8 + 3),
            }),
        )
        .await;

        assert!(response.is_ok());
    }

    let error = daemon_hello(
        State(state.clone()),
        remote_addr(1),
        Json(DaemonHelloRequest {
            pair_code: valid_pair_code(99),
            host_name: "different-host".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(99),
        }),
    )
    .await
    .unwrap_err();

    let response = error.into_response();
    assert_eq!(response.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(pairing_count_for_host_name(&state, "Same-Host").await, 5);
    assert_eq!(
        pairing_count_for_host_name(&state, "different-host").await,
        0
    );
}

#[tokio::test]
async fn daemon_hello_accepts_supported_windows_platform() {
    let state = setup_state().await;

    let response = daemon_hello(
        State(state.clone()),
        remote_addr(1),
        Json(DaemonHelloRequest {
            pair_code: valid_pair_code(7),
            host_name: "host-c".to_string(),
            platform: "windows".to_string(),
            pairing_proof: test_pairing_proof(4),
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.status, "pending");
    assert_eq!(pairing_count_for_host_name(&state, "host-c").await, 1);
}
#[tokio::test]
async fn daemon_hello_does_not_share_limit_across_source_ips() {
    let state = setup_state().await;

    for index in 0..6 {
        let _ = daemon_hello(
            State(state.clone()),
            remote_addr(1),
            Json(DaemonHelloRequest {
                pair_code: valid_pair_code(index),
                host_name: "host-a".to_string(),
                platform: "linux".to_string(),
                pairing_proof: test_pairing_proof(5),
            }),
        )
        .await;
    }

    let response = daemon_hello(
        State(state.clone()),
        remote_addr(2),
        Json(DaemonHelloRequest {
            pair_code: valid_pair_code(199),
            host_name: "host-a".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(5),
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.status, "pending");
    assert_eq!(pairing_count_for_host_name(&state, "host-a").await, 6);
}

#[tokio::test]
async fn daemon_hello_returns_pairing_secret_for_polling_fallback() {
    let state = setup_state().await;

    let response = daemon_hello(
        State(state.clone()),
        remote_addr(1),
        Json(DaemonHelloRequest {
            pair_code: valid_pair_code(55),
            host_name: "host-secret".to_string(),
            platform: "linux".to_string(),
            pairing_proof: test_pairing_proof(7),
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(response.status, "pending");
    assert_eq!(response.pair_code, valid_pair_code(55));
    assert_eq!(
        response.qr_payload,
        format!("vibe://pair/{}", valid_pair_code(55))
    );
    assert!(!response.pairing_secret.is_empty());
    assert!(response.pairing_secret.len() >= 32);
}
