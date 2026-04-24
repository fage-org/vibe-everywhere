use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware::from_fn_with_state,
    routing::get,
    Router,
};
use chrono::Duration;
use tower::ServiceExt;
use uuid::Uuid;
use ve_server::{
    api::settings, config::Config, db, middleware::auth::auth_middleware, state::AppState,
};
use ve_shared::jwt::JwtManager;

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

async fn setup_app() -> (Router, Arc<AppState>, Arc<JwtManager>, Uuid) {
    db::install_drivers();
    let db_name = format!("settings_notifications_{}.db", Uuid::new_v4());
    let db_url = format!("sqlite:/tmp/{}?mode=rwc", db_name);
    let config = test_config(db_url.clone());
    let pool = db::create_pool(&config).await.unwrap();
    db::run_migrations(&pool, config.database_backend())
        .await
        .unwrap();

    let jwt_manager = Arc::new(JwtManager::new(
        &config.jwt_secret,
        Duration::seconds(config.jwt_expiration_secs as i64),
    ));
    let state = Arc::new(AppState::new(
        pool,
        ve_server::hub::Hub::new(),
        config.clone(),
        jwt_manager.clone(),
    ));

    let device_id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO client_devices (device_id, device_name, device_type, server_url)
           VALUES ($1, 'device', 'desktop', 'http://localhost')"#,
    )
    .bind(device_id.to_string())
    .execute(&state.db)
    .await
    .unwrap();

    let app = Router::new()
        .route(
            "/api/settings/notifications",
            get(settings::get_notification_preferences)
                .post(settings::update_notification_preferences),
        )
        .route_layer(from_fn_with_state(jwt_manager.clone(), auth_middleware))
        .with_state(state.clone())
        .with_state(jwt_manager.clone());

    (app, state, jwt_manager, device_id)
}

#[tokio::test]
async fn get_notification_preferences_reads_device_from_claims() {
    let (app, _state, jwt_manager, device_id) = setup_app().await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/settings/notifications")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn update_notification_preferences_reads_device_from_claims() {
    let (app, state, jwt_manager, device_id) = setup_app().await;
    let token = jwt_manager
        .create_client_token(device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/settings/notifications")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"enabled":false,"task_failed_enabled":false}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let row: (i64, i64) = sqlx::query_as(
        "SELECT enabled, task_failed_enabled FROM notification_preferences WHERE device_id = $1",
    )
    .bind(device_id.to_string())
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(row, (0, 0));
}

#[tokio::test]
async fn get_notification_preferences_returns_not_found_for_unknown_claim_device() {
    let (app, _state, jwt_manager, _seeded_device_id) = setup_app().await;
    let unknown_device_id = Uuid::new_v4();
    let token = jwt_manager
        .create_client_token(unknown_device_id, "device")
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/settings/notifications")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
