//! Authentication Middleware Tests
//!
//! Tests for JWT authentication middleware on HTTP API endpoints.
//! These tests use a real AppState with an in-memory SQLite database.

use axum::{
    body::Body,
    extract::Extension,
    http::{header, Request, StatusCode},
    middleware::from_fn_with_state,
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Duration;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;
use ve_server::middleware::auth::{auth_middleware, AuthError};
use ve_server::{config::Config, db, hub::Hub, state::AppState};
use ve_shared::jwt::{Claims, JwtManager, TokenType};

fn test_config() -> Config {
    Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_string(),
        jwt_secret: "test_secret_key_at_least_32_characters!".to_string(),
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

async fn test_app() -> (Router, Arc<JwtManager>) {
    db::install_drivers();
    let db_name = format!("auth_test_{}.db", Uuid::new_v4());
    let db_url = format!("sqlite:/tmp/{}?mode=rwc", db_name);
    let config = Config {
        database_url: db_url.clone(),
        ..test_config()
    };
    let pool = db::create_pool(&config).await.unwrap();
    db::run_migrations(&pool, config.database_backend())
        .await
        .unwrap();

    let jwt_manager = Arc::new(JwtManager::new(
        &config.jwt_secret,
        Duration::hours(24),
    ));
    let state = Arc::new(AppState::new(pool, Hub::new(), config, jwt_manager.clone()));

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route("/healthz", get(public_handler))
        .route("/api/auth/register-device", get(public_handler))
        .route("/api/auth/daemon-hello", get(public_handler))
        .route("/api/auth/pairing-status", get(public_handler))
        .route("/api/auth/pair", get(public_handler))
        .route_layer(from_fn_with_state(
            (state.clone(), jwt_manager.clone()),
            auth_middleware,
        ))
        .with_state(state)
        .with_state(jwt_manager.clone());

    (app, jwt_manager)
}

async fn protected_handler(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    (StatusCode::OK, format!("Hello {}!", claims.name))
}

async fn public_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

fn valid_client_token(jm: &JwtManager) -> String {
    jm.create_client_token(Uuid::new_v4(), "Test Device").unwrap()
}

fn valid_bootstrap_token(jm: &JwtManager) -> String {
    jm.create_client_bootstrap_token(Uuid::new_v4(), "Test Device").unwrap()
}

fn valid_daemon_token(jm: &JwtManager) -> String {
    jm.create_daemon_token(Uuid::new_v4(), "Test Host").unwrap()
}

#[tokio::test]
async fn auth_middleware_rejects_request_without_token() {
    let (app, _) = test_app().await;
    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_accepts_valid_client_token() {
    let (app, jm) = test_app().await;
    let token = valid_client_token(&jm);
    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_rejects_bootstrap_token_on_generic_protected_route() {
    let (app, jm) = test_app().await;
    let token = valid_bootstrap_token(&jm);
    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_rejects_daemon_token_on_generic_protected_route() {
    let (app, jm) = test_app().await;
    let token = valid_daemon_token(&jm);
    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_rejects_invalid_token() {
    let (app, _) = test_app().await;
    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_rejects_malformed_auth_header() {
    let (app, _) = test_app().await;
    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, "some_token_without_bearer")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_allows_health_endpoint() {
    let (app, _) = test_app().await;
    let request = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_allows_auth_register_endpoint() {
    let (app, _) = test_app().await;
    let request = Request::builder()
        .uri("/api/auth/register-device")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_rejects_pair_endpoint_without_token() {
    let (app, _) = test_app().await;
    let request = Request::builder()
        .uri("/api/auth/pair")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_rejects_pair_endpoint_with_formal_client_token() {
    let (app, jm) = test_app().await;
    let token = valid_client_token(&jm);
    let request = Request::builder()
        .uri("/api/auth/pair")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_allows_pair_endpoint_with_bootstrap_token() {
    let (app, jm) = test_app().await;
    let token = valid_bootstrap_token(&jm);
    let request = Request::builder()
        .uri("/api/auth/pair")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_rejects_protected_route_with_bootstrap_token() {
    let (app, jm) = test_app().await;
    let token = valid_bootstrap_token(&jm);
    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_rejects_protected_route_with_daemon_token() {
    let (app, jm) = test_app().await;
    let token = valid_daemon_token(&jm);
    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_allows_daemon_hello_endpoint() {
    let (app, _) = test_app().await;
    let request = Request::builder()
        .uri("/api/auth/daemon-hello")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_rejects_expired_token() {
    db::install_drivers();
    let db_name = format!("auth_expired_test_{}.db", Uuid::new_v4());
    let db_url = format!("sqlite:/tmp/{}?mode=rwc", db_name);
    let mut config = test_config();
    config.database_url = db_url.clone();
    let pool = db::create_pool(&config).await.unwrap();
    db::run_migrations(&pool, config.database_backend()).await.unwrap();

    let jwt_manager = Arc::new(JwtManager::new(
        &config.jwt_secret,
        Duration::seconds(-1),
    ));
    let token = jwt_manager.create_client_token(Uuid::new_v4(), "Test Device").unwrap();
    let state = Arc::new(AppState::new(pool, Hub::new(), config, jwt_manager.clone()));

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(from_fn_with_state(
            (state.clone(), jwt_manager.clone()),
            auth_middleware,
        ))
        .with_state(state)
        .with_state(jwt_manager.clone());

    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_extracts_claims() {
    let (app, jm) = test_app().await;
    let device_id = Uuid::new_v4();
    let token = jm.create_client_token(device_id, "TestDevice").unwrap();
    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("TestDevice"));
}

#[tokio::test]
async fn auth_middleware_allows_pairing_status_endpoint() {
    let (app, _) = test_app().await;
    let request = Request::builder()
        .uri("/api/auth/pairing-status?host_id=00000000-0000-0000-0000-000000000000")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn auth_middleware_jwt_roundtrip() {
    let jm = JwtManager::new("test_secret_key_at_least_32_characters!", Duration::hours(24));
    let device_id = Uuid::new_v4();
    let token = jm.create_client_token(device_id, "Test Device").unwrap();
    let claims = jm.decode(&token).unwrap();
    assert_eq!(claims.r#type, TokenType::Client);
    assert_eq!(claims.subject_uuid().unwrap(), device_id);
}

#[test]
fn auth_error_into_response_missing_token() {
    let response = AuthError::MissingToken.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn auth_error_into_response_invalid_format() {
    let response = AuthError::InvalidFormat.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn auth_error_into_response_invalid_token() {
    let response = AuthError::InvalidToken.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn auth_error_into_response_expired() {
    let response = AuthError::Expired.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn auth_error_into_response_token_revoked() {
    let response = AuthError::TokenRevoked.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
