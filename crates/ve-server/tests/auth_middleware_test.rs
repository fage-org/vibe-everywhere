//! Authentication Middleware Tests
//!
//! Tests for JWT authentication middleware on HTTP API endpoints.

use axum::{
    body::Body,
    extract::Extension,
    http::{Request, StatusCode, header},
    Router,
    routing::get,
    response::IntoResponse,
    middleware::from_fn_with_state,
};
use tower::ServiceExt;
use ve_shared::jwt::{JwtManager, TokenType, Claims};
use chrono::Duration;
use std::sync::Arc;
use ve_server::middleware::auth::{auth_middleware, AuthError};

/// Helper to create a test JWT manager
fn test_jwt_manager() -> JwtManager {
    JwtManager::new("test_secret_key_at_least_32_characters!", Duration::hours(24))
}

/// Helper to create a valid client token
fn valid_client_token(jwt_manager: &JwtManager) -> String {
    jwt_manager
        .create_client_token(uuid::Uuid::new_v4(), "Test Device")
        .unwrap()
}

/// Helper to create a valid daemon token
fn valid_daemon_token(jwt_manager: &JwtManager) -> String {
    jwt_manager
        .create_daemon_token(uuid::Uuid::new_v4(), "Test Host")
        .unwrap()
}

/// Protected handler that requires authentication
async fn protected_handler(Extension(claims): Extension<Claims>) -> impl IntoResponse {
    (StatusCode::OK, format!("Hello {}!", claims.name))
}

/// Simple handler for public routes
async fn public_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

// ============================================================================
// Tests for auth middleware
// ============================================================================

#[tokio::test]
async fn auth_middleware_rejects_request_without_token() {
    let jwt_manager = Arc::new(test_jwt_manager());

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/protected")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should reject without token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_accepts_valid_client_token() {
    let jwt_manager = Arc::new(test_jwt_manager());
    let token = valid_client_token(&jwt_manager);

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should accept valid client token
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_accepts_valid_daemon_token() {
    let jwt_manager = Arc::new(test_jwt_manager());
    let token = valid_daemon_token(&jwt_manager);

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should accept valid daemon token
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_rejects_invalid_token() {
    let jwt_manager = Arc::new(test_jwt_manager());

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should reject invalid token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_rejects_malformed_auth_header() {
    let jwt_manager = Arc::new(test_jwt_manager());

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    // Missing "Bearer " prefix
    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, "some_token_without_bearer")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should reject malformed header
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_allows_health_endpoint() {
    // Health endpoint should NOT require authentication
    let jwt_manager = Arc::new(test_jwt_manager());

    let app = Router::new()
        .route("/healthz", get(public_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Health endpoint should be public
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_allows_auth_register_endpoint() {
    // Device registration should NOT require authentication
    let jwt_manager = Arc::new(test_jwt_manager());

    let app = Router::new()
        .route("/api/auth/register-device", get(public_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/api/auth/register-device")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Registration should be public (creates auth)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_allows_pair_endpoint() {
    // Pair endpoint should NOT require authentication
    let jwt_manager = Arc::new(test_jwt_manager());

    let app = Router::new()
        .route("/api/auth/pair", get(public_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/api/auth/pair")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Pairing should be public
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_allows_daemon_hello_endpoint() {
    // Daemon hello should NOT require authentication
    let jwt_manager = Arc::new(test_jwt_manager());

    let app = Router::new()
        .route("/api/auth/daemon-hello", get(public_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/api/auth/daemon-hello")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Daemon hello should be public (initiates pairing)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn auth_middleware_rejects_expired_token() {
    let jwt_manager = Arc::new(JwtManager::new("test_secret_key_at_least_32_characters!", Duration::seconds(-1)));
    let token = jwt_manager
        .create_client_token(uuid::Uuid::new_v4(), "Test Device")
        .unwrap();

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should reject expired token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_middleware_extracts_claims() {
    let jwt_manager = Arc::new(test_jwt_manager());
    let device_id = uuid::Uuid::new_v4();
    let token = jwt_manager
        .create_client_token(device_id, "TestDevice")
        .unwrap();

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(jwt_manager);

    let request = Request::builder()
        .uri("/protected")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should succeed and return the name
    assert_eq!(response.status(), StatusCode::OK);

    let body = http_body_util::BodyExt::collect(response.into_body()).await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("TestDevice"));
}

#[test]
fn auth_middleware_jwt_roundtrip() {
    let jwt_manager = test_jwt_manager();
    let device_id = uuid::Uuid::new_v4();
    let token = jwt_manager
        .create_client_token(device_id, "Test Device")
        .unwrap();

    let claims = jwt_manager.decode(&token).unwrap();
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