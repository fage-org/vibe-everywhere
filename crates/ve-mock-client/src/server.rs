//! Real ve-server startup and shutdown for integration testing

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::serve;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::info;

use ve_server::build_app;
use ve_server::config::Config;
use ve_server::db::{self};
use ve_server::hub::Hub;
use ve_server::state::AppState;
use ve_shared::jwt::JwtManager;

/// Integration test server
pub struct IntegrationServer {
    pub server_url: String,
    pub port: u16,
    pub db_path: std::path::PathBuf,
    pub config: Config,
    pub hub: Arc<Hub>,
    pub jwt_manager: Arc<JwtManager>,
    pub pool: db::DbPool,
    server_handle: Option<JoinHandle<()>>,
}

/// JWT secret used for integration testing.
const TEST_JWT_SECRET: &str = "test-integration-secret-key-32bytes!!";

impl IntegrationServer {
    pub async fn start(temp_dir: &std::path::Path) -> Result<Self> {
        // 1. Create temp DB file
        let db_path = temp_dir.join("test.db");
        let database_url = format!("sqlite:{}?mode=rwc", db_path.display());

        // 2. Build config via TOML deserialization (all serde defaults apply)
        let config_toml = format!(
            r#"
database_url = "{database_url}"
jwt_secret = "{TEST_JWT_SECRET}"
data_dir = "{}"
cors_origins = ["*"]
log_level = "debug"
"#,
            temp_dir.display()
        );
        let config: Config = toml::from_str(&config_toml).context("parsing test config TOML")?;

        // 3. Initialize drivers and DB
        db::install_drivers();
        let pool = db::create_pool(&config)
            .await
            .context("creating database pool")?;
        db::run_migrations(&pool, config.database_backend())
            .await
            .context("running migrations")?;

        // 4. Build app
        let hub = Hub::new();
        let jwt_manager = Arc::new(JwtManager::new(&config.jwt_secret, config.jwt_expiration()));
        // We need to access the hub after AppState consumes it, so we store Arc<Hub> directly
        // and extract it from AppState after construction
        let state = AppState::new(pool.clone(), hub, config.clone());
        let hub_ref = Arc::clone(&state.hub);
        let app = build_app(Arc::new(state), Arc::clone(&jwt_manager), &config);

        // 5. Bind to random port on loopback
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .context("binding to random port")?;
        let port = listener
            .local_addr()
            .context("getting local address")?
            .port();
        let server_url = format!("http://127.0.0.1:{port}");

        info!("Test server listening on {server_url}");

        // 6. Spawn server in background with ConnectInfo support
        let server_handle = tokio::spawn(async move {
            let _ = serve(
                listener,
                app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await;
        });

        // Wait for server to accept connections
        wait_for_server_ready(&server_url, Duration::from_secs(10)).await?;

        Ok(Self {
            server_url,
            port,
            db_path,
            config,
            hub: hub_ref,
            jwt_manager,
            pool,
            server_handle: Some(server_handle),
        })
    }

    pub fn abort(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
    }
}

/// Poll the server until it responds or times out.
async fn wait_for_server_ready(server_url: &str, timeout: Duration) -> Result<()> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("Server at {server_url} did not become ready within {timeout:?}");
        }
        if reqwest::get(format!("{server_url}/healthz")).await.is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

impl Drop for IntegrationServer {
    fn drop(&mut self) {
        self.abort();
    }
}
