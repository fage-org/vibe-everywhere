//! Real ve-server startup and shutdown for integration testing

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::serve;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::info;
use uuid::Uuid;

use ve_server::build_app;
use ve_server::config::Config;
use ve_server::db::{self};
use ve_server::hub::Hub;
use ve_server::state::AppState;
use ve_server::tasks;
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
    task_handles: Vec<JoinHandle<()>>,
    postgres_admin_pool: Option<db::DbPool>,
    postgres_schema_name: Option<String>,
}

/// JWT secret used for integration testing.
const TEST_JWT_SECRET: &str = "test-integration-secret-key-32bytes!!";
const TEST_DATABASE_URL_ENV: &str = "VE_MOCK_CLIENT_DATABASE_URL";

struct PreparedDatabaseConfig {
    config: Config,
    db_path: std::path::PathBuf,
    postgres_admin_pool: Option<db::DbPool>,
    postgres_schema_name: Option<String>,
}

impl IntegrationServer {
    pub async fn start(
        temp_dir: &std::path::Path,
        database_url_override: Option<&str>,
    ) -> Result<Self> {
        // 1. Initialize drivers and DB config
        db::install_drivers();
        let prepared = prepare_database_config(temp_dir, database_url_override).await?;
        let config = prepared.config.clone();
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
        let state = AppState::new(pool.clone(), hub, config.clone(), Arc::clone(&jwt_manager));
        let hub_ref = Arc::clone(&state.hub);
        let app = build_app(Arc::new(state), Arc::clone(&jwt_manager), &config);
        let task_config = Arc::new(config.clone());
        let task_handles = vec![
            tasks::start_permission_expiry_task(pool.clone(), hub_ref.clone(), task_config.clone()),
            tasks::start_idempotency_cleanup_task(pool.clone(), task_config),
        ];

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
            db_path: prepared.db_path,
            config,
            hub: hub_ref,
            jwt_manager,
            pool,
            server_handle: Some(server_handle),
            task_handles,
            postgres_admin_pool: prepared.postgres_admin_pool,
            postgres_schema_name: prepared.postgres_schema_name,
        })
    }

    pub fn abort(&mut self) {
        if let Some(handle) = self.server_handle.take() {
            handle.abort();
        }
        for handle in self.task_handles.drain(..) {
            handle.abort();
        }

        if let (Some(admin_pool), Some(schema_name)) = (
            self.postgres_admin_pool.take(),
            self.postgres_schema_name.take(),
        ) {
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let _ =
                        sqlx::query(&format!("DROP SCHEMA IF EXISTS \"{schema_name}\" CASCADE"))
                            .execute(&admin_pool)
                            .await;
                    admin_pool.close().await;
                });
            }
        }
    }
}

fn build_test_config(temp_dir: &std::path::Path, database_url: &str) -> Result<Config> {
    let config_toml = format!(
        r#"
database_url = "{database_url}"
jwt_secret = "{TEST_JWT_SECRET}"
data_dir = "{}"
cors_origins = ["*"]
log_level = "debug"
permission_expiry_check_secs = 1
idempotency_cleanup_secs = 1
"#,
        temp_dir.display()
    );
    toml::from_str(&config_toml).context("parsing test config TOML")
}

async fn prepare_database_config(
    temp_dir: &std::path::Path,
    database_url_override: Option<&str>,
) -> Result<PreparedDatabaseConfig> {
    let selected_database_url = database_url_override
        .map(ToOwned::to_owned)
        .or_else(|| std::env::var(TEST_DATABASE_URL_ENV).ok());

    match selected_database_url {
        Some(database_url)
            if database_url.starts_with("postgres://")
                || database_url.starts_with("postgresql://") =>
        {
            prepare_postgres_database_config(temp_dir, &database_url).await
        }
        Some(database_url) => {
            let config = build_test_config(temp_dir, &database_url)?;
            Ok(PreparedDatabaseConfig {
                config,
                db_path: temp_dir.join("test.db"),
                postgres_admin_pool: None,
                postgres_schema_name: None,
            })
        }
        None => {
            let db_path = temp_dir.join("test.db");
            let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
            let config = build_test_config(temp_dir, &database_url)?;
            Ok(PreparedDatabaseConfig {
                config,
                db_path,
                postgres_admin_pool: None,
                postgres_schema_name: None,
            })
        }
    }
}

async fn prepare_postgres_database_config(
    temp_dir: &std::path::Path,
    base_url: &str,
) -> Result<PreparedDatabaseConfig> {
    let schema_name = format!("ve_mock_client_{}", Uuid::new_v4().simple());
    let admin_pool = db::DbPool::connect(base_url).await?;

    sqlx::query(&format!("CREATE SCHEMA \"{schema_name}\""))
        .execute(&admin_pool)
        .await
        .with_context(|| format!("creating postgres schema {schema_name}"))?;

    let scoped_database_url = scoped_postgres_database_url(base_url, &schema_name);
    let config = build_test_config(temp_dir, &scoped_database_url)?;

    Ok(PreparedDatabaseConfig {
        config,
        db_path: temp_dir.join(format!("{schema_name}.schema")),
        postgres_admin_pool: Some(admin_pool),
        postgres_schema_name: Some(schema_name),
    })
}

fn scoped_postgres_database_url(base_url: &str, schema_name: &str) -> String {
    let separator = if base_url.contains('?') { '&' } else { '?' };
    format!("{base_url}{separator}options=-csearch_path%3D{schema_name}")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_postgres_database_url_appends_search_path_option() {
        let scoped =
            scoped_postgres_database_url("postgres://user:pass@localhost:5432/vibe", "schema_a");

        assert!(scoped.contains("options=-csearch_path%3Dschema_a"));
        assert!(scoped.starts_with("postgres://user:pass@localhost:5432/vibe?"));
    }

    #[test]
    fn scoped_postgres_database_url_preserves_existing_query_string() {
        let scoped = scoped_postgres_database_url(
            "postgres://user:pass@localhost:5432/vibe?sslmode=disable",
            "schema_b",
        );

        assert!(scoped.contains("sslmode=disable"));
        assert!(scoped.contains("&options=-csearch_path%3Dschema_b"));
    }
}
