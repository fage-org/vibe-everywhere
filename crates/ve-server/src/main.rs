//! Vibe Everywhere Server
//!
//! Backend service for remote AI agent session management.

use std::sync::Arc;

use tracing::info;
use ve_server::{build_app, config::Config, db, error::Result, hub::Hub, state::AppState, tasks};
use ve_shared::jwt::JwtManager;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env().expect("Failed to load configuration");
    init_tracing(&config);

    info!(
        listen_addr = %config.listen_addr,
        log_format = %config.log_format,
        log_level = %config.log_level,
        "Starting Vibe Everywhere server"
    );

    db::install_drivers();
    let db_backend = config.database_backend();
    let db = db::create_pool(&config).await?;
    db::run_migrations(&db, db_backend).await?;

    tasks::recover_orphaned_dispatching_sessions(&db).await;

    let hub = Hub::new();
    let jwt_manager = Arc::new(JwtManager::new(&config.jwt_secret, config.jwt_expiration()));
    let state = AppState::new(db.clone(), hub, config.clone(), Arc::clone(&jwt_manager));

    let _expiry_task = tasks::start_permission_expiry_task(
        db.clone(),
        state.hub.clone(),
        Arc::new(config.clone()),
    );
    let _cleanup_task = tasks::start_idempotency_cleanup_task(db, Arc::new(config.clone()));
    info!("Background tasks started");

    let app = build_app(Arc::new(state), jwt_manager, &config);
    let addr = config.listen_addr;
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .expect("Server failed");

    Ok(())
}

fn init_tracing(config: &Config) {
    let level = config.log_level();

    if config.is_json_logging() {
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
