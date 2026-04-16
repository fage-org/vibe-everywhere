//! Vibe Everywhere Server
//!
//! Backend service for remote AI agent session management.

mod api;
mod config;
mod db;
mod error;
mod hub;
mod middleware;
mod state;
mod tasks;
mod utils;
mod validation;
mod ws;

use std::sync::Arc;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use ve_shared::jwt::JwtManager;

use crate::config::Config;
use crate::error::Result;
use crate::hub::Hub;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration first (needed for logging setup)
    let config = Config::from_env().expect("Failed to load configuration");

    // Initialize tracing with environment-aware configuration
    init_tracing(&config);

    info!(
        listen_addr = %config.listen_addr,
        log_format = %config.log_format,
        log_level = %config.log_level,
        "Starting Vibe Everywhere server"
    );

    // Initialize database
    let db = db::create_sqlite_pool(&config).await?;
    db::run_migrations(&db).await?;

    // Initialize WebSocket hub
    let hub = Hub::new();

    // Create JWT manager for authentication
    let jwt_manager = Arc::new(JwtManager::new(
        &config.jwt_secret,
        config.jwt_expiration(),
    ));

    // Create application state
    let state = AppState::new(db.clone(), hub, config.clone());

    // Start background tasks
    let _expiry_task = tasks::start_permission_expiry_task(db.clone(), Arc::new(config.clone()));
    let _cleanup_task = tasks::start_idempotency_cleanup_task(db, Arc::new(config.clone()));
    info!("Background tasks started");

    // Build router
    let app = Router::new()
        // Health check (public)
        .route("/healthz", get(healthz))
        // Auth routes (public - they create auth)
        .route(
            "/api/auth/register-device",
            post(api::auth::register_device),
        )
        .route("/api/auth/daemon-hello", post(api::auth::daemon_hello))
        .route("/api/auth/pair", post(api::auth::pair))
        // Protected API routes
        .route("/api/hosts", get(api::hosts::list_hosts))
        .route("/api/hosts/:id", get(api::hosts::get_host))
        .route("/api/hosts/:id", post(api::hosts::unbind_host))
        .route("/api/workspaces", get(api::workspaces::list_workspaces))
        .route("/api/workspaces", post(api::workspaces::create_workspace))
        .route("/api/workspaces/:id", get(api::workspaces::get_workspace))
        .route(
            "/api/workspaces/:id",
            post(api::workspaces::update_workspace),
        )
        .route(
            "/api/workspaces/:id",
            axum::routing::delete(api::workspaces::delete_workspace),
        )
        .route("/api/sessions", get(api::sessions::list_sessions))
        .route("/api/sessions", post(api::sessions::create_session))
        .route("/api/sessions/:id", get(api::sessions::get_session))
        .route(
            "/api/sessions/:id/messages",
            post(api::sessions::send_message),
        )
        .route(
            "/api/sessions/:id/messages",
            get(api::sessions::list_messages),
        )
        .route(
            "/api/sessions/:id/control",
            post(api::sessions::control_session),
        )
        .route(
            "/api/sessions/:id/close",
            post(api::sessions::close_session),
        )
        .route("/api/permissions", get(api::permissions::list_permissions))
        .route(
            "/api/permissions/:id",
            get(api::permissions::get_permission),
        )
        .route(
            "/api/permissions/:id/respond",
            post(api::permissions::respond_permission),
        )
        .route("/api/archives", get(api::archives::list_archives))
        .route("/api/archives/:id", get(api::archives::get_archive))
        .route(
            "/api/archives/batch-delete",
            post(api::archives::batch_delete_archives),
        )
        .route(
            "/api/settings/notifications",
            get(api::settings::get_notification_preferences),
        )
        .route(
            "/api/settings/notifications",
            post(api::settings::update_notification_preferences),
        )
        // WebSocket routes (public - they handle their own auth)
        .route("/ws/client", get(ws::client_ws::ws_client_handler))
        .route("/ws/daemon", get(ws::daemon_ws::ws_daemon_handler))
        // Apply authentication middleware
        .route_layer(from_fn_with_state(
            jwt_manager.clone(),
            middleware::auth::auth_middleware,
        ))
        // Other middleware layers
        .layer(build_cors_layer(&config))
        .layer(TraceLayer::new_for_http())
        // Provide state
        .with_state(Arc::new(state))
        // Also provide jwt_manager as state for middleware
        .with_state(jwt_manager);

    // Start server
    let addr = config.listen_addr;
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server failed");

    Ok(())
}

/// Health check endpoint
async fn healthz() -> &'static str {
    "OK"
}

/// Initialize tracing subscriber with environment-aware configuration
///
/// Development (log_format=pretty): Human-readable format with colors and file:line
/// Production (log_format=json): Structured JSON format for log aggregation
fn init_tracing(config: &Config) {
    let level = config.log_level();

    if config.is_json_logging() {
        // Production: JSON format for log aggregation (ELK, Loki, CloudWatch)
        tracing_subscriber::fmt()
            .json()
            .with_max_level(level)
            .with_target(true)
            .with_current_span(false)
            .with_span_list(true)
            .with_file(true)
            .with_line_number(true)
            .init();
    } else {
        // Development: Pretty format for human readability
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(true)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true)
            .pretty()
            .init();
    }
}

/// Build CORS layer from configuration
fn build_cors_layer(config: &Config) -> CorsLayer {
    use axum::http::{header, Method, HeaderValue};
    use tower_http::cors::AllowOrigin;

    let allow_origin = if config.cors_origins.is_empty() {
        // No origins configured = most restrictive (same-origin only)
        // For development, you might want to use ["*"]
        tracing::warn!("CORS origins not configured, using restrictive same-origin policy");
        AllowOrigin::exact(HeaderValue::from_static("same-origin"))
    } else if config.cors_origins.len() == 1 && config.cors_origins[0] == "*" {
        // Wildcard = allow all origins
        tracing::info!("CORS configured to allow all origins (*)");
        AllowOrigin::any()
    } else {
        // Specific origins
        tracing::info!(origins = ?config.cors_origins, "CORS configured with specific origins");
        let origins: Vec<HeaderValue> = config
            .cors_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        AllowOrigin::list(origins)
    };

    CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
}