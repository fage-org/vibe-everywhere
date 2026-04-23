//! Vibe Everywhere Server Library
//!
//! Backend service for remote AI agent session management.

use std::sync::Arc;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use ve_shared::jwt::JwtManager;

pub mod api;
pub mod authz;
pub mod config;
pub mod db;
pub mod error;
pub mod hub;
pub mod middleware;
pub mod state;
pub mod tasks;
pub mod utils;
pub mod validation;
pub(crate) mod ws;

use crate::{config::Config, state::AppState};

pub fn build_app(state: Arc<AppState>, jwt_manager: Arc<JwtManager>, config: &Config) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route(
            "/api/auth/register-device",
            post(api::auth::register_device),
        )
        .route("/api/auth/daemon-hello", post(api::auth::daemon_hello))
        .route("/api/auth/pairing-status", get(api::auth::pairing_status))
        .route("/api/auth/pair", post(api::auth::pair))
        .route("/api/hosts", get(api::hosts::list_hosts))
        .route("/api/hosts/{id}", get(api::hosts::get_host_route))
        .route("/api/hosts/{id}", post(api::hosts::unbind_host_route))
        .route(
            "/api/hosts/{id}/files/tree",
            get(api::files::get_file_tree_route),
        )
        .route(
            "/api/hosts/{id}/files/content",
            get(api::files::get_file_content_route),
        )
        .route("/api/workspaces", get(api::workspaces::list_workspaces))
        .route("/api/workspaces", post(api::workspaces::create_workspace))
        .route(
            "/api/workspaces/{id}",
            get(api::workspaces::get_workspace_route),
        )
        .route(
            "/api/workspaces/{id}",
            post(api::workspaces::update_workspace_route),
        )
        .route(
            "/api/workspaces/{id}",
            axum::routing::delete(api::workspaces::delete_workspace_route),
        )
        .route("/api/sessions", get(api::sessions::list_sessions))
        .route("/api/sessions", post(api::sessions::create_session))
        .route("/api/sessions/{id}", get(api::sessions::get_session_route))
        .route(
            "/api/sessions/{id}/messages",
            post(api::sessions::send_message_route),
        )
        .route(
            "/api/sessions/{id}/messages",
            get(api::sessions::list_messages_route),
        )
        .route(
            "/api/sessions/{id}/control",
            post(api::sessions::control_session_route),
        )
        .route(
            "/api/sessions/{id}/close",
            post(api::sessions::close_session_route),
        )
        .route("/api/permissions", get(api::permissions::list_permissions))
        .route(
            "/api/permissions/{id}",
            get(api::permissions::get_permission_route),
        )
        .route(
            "/api/permissions/{id}/respond",
            post(api::permissions::respond_permission_route),
        )
        .route("/api/archives", get(api::archives::list_archives_route))
        .route("/api/archives/{id}", get(api::archives::get_archive_route))
        .route(
            "/api/archives/batch-delete",
            post(api::archives::batch_delete_archives_route),
        )
        .route(
            "/api/settings/notifications",
            get(api::settings::get_notification_preferences),
        )
        .route(
            "/api/settings/notifications",
            axum::routing::put(api::settings::update_notification_preferences),
        )
        .route("/ws/client", get(ws::client_ws::ws_client_handler))
        .route("/ws/daemon", get(ws::daemon_ws::ws_daemon_handler))
        .route_layer(from_fn_with_state(
            jwt_manager.clone(),
            middleware::auth::auth_middleware,
        ))
        .layer(build_cors_layer(config))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .with_state(jwt_manager)
}

async fn healthz() -> &'static str {
    "OK"
}

fn build_cors_layer(config: &Config) -> CorsLayer {
    use axum::http::{header, HeaderValue, Method};
    use tower_http::cors::AllowOrigin;

    let allow_origin = if config.cors_origins.is_empty() {
        tracing::warn!("CORS origins not configured, using restrictive same-origin policy");
        AllowOrigin::exact(HeaderValue::from_static("same-origin"))
    } else if config.cors_origins.len() == 1 && config.cors_origins[0] == "*" {
        tracing::info!("CORS configured to allow all origins (*)");
        AllowOrigin::any()
    } else {
        tracing::info!(origins = ?config.cors_origins, "CORS configured with specific origins");
        let origins: Vec<HeaderValue> = config
            .cors_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
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
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
}
