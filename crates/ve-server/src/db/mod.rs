//! Database Initialization
//!
//! Connection pool creation and migration execution using AnyPool for runtime database selection.

pub mod idempotency;

use sqlx::any::AnyPoolOptions;
use sqlx::migrate::MigrateDatabase;
use tracing::info;

use crate::config::{Config, DatabaseBackend};
use crate::error::Result;

/// Database pool type - uses AnyPool for runtime database selection
pub type DbPool = sqlx::AnyPool;

/// Install default database drivers for AnyPool
///
/// This must be called before creating any AnyPool connections.
pub fn install_drivers() {
    sqlx::any::install_default_drivers();
}

/// Create a database connection pool based on configuration
///
/// Detects database type from DATABASE_URL and creates appropriate pool.
/// Supports both SQLite and PostgreSQL at runtime.
pub async fn create_pool(config: &Config) -> Result<DbPool> {
    let backend = config.database_backend();

    match backend {
        DatabaseBackend::Sqlite => create_sqlite_pool(config).await,
        DatabaseBackend::Postgres => create_postgres_pool(config).await,
    }
}

/// Create a SQLite connection pool
async fn create_sqlite_pool(config: &Config) -> Result<DbPool> {
    // Ensure data directory exists
    tokio::fs::create_dir_all(&config.data_dir)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("Failed to create data directory: {}", e))
        })?;

    // Build database URL if not provided
    let db_url = if config.database_url.starts_with("sqlite:") {
        config.database_url.clone()
    } else {
        format!("sqlite:{}/vibe.db?mode=rwc", config.data_dir.display())
    };

    info!(url = %db_url, "Creating SQLite connection pool");

    // Create database if it doesn't exist
    if !sqlx::Sqlite::database_exists(&db_url)
        .await
        .unwrap_or(false)
    {
        info!("Creating new SQLite database");
        sqlx::Sqlite::create_database(&db_url).await.map_err(|e| {
            crate::error::ServerError::Internal(format!("Failed to create database: {}", e))
        })?;
    }

    let pool = AnyPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("Failed to connect to database: {}", e))
        })?;

    Ok(pool)
}

/// Create a PostgreSQL connection pool
async fn create_postgres_pool(config: &Config) -> Result<DbPool> {
    let db_url = &config.database_url;

    info!(url = %db_url, "Creating PostgreSQL connection pool");

    let pool = AnyPoolOptions::new()
        .max_connections(10)
        .connect(db_url)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("Failed to connect to PostgreSQL: {}", e))
        })?;

    info!("PostgreSQL connection pool created successfully");
    Ok(pool)
}

/// Run database migrations based on database type
pub async fn run_migrations(pool: &DbPool, backend: DatabaseBackend) -> Result<()> {
    match backend {
        DatabaseBackend::Sqlite => run_sqlite_migrations(pool).await,
        DatabaseBackend::Postgres => run_postgres_migrations(pool).await,
    }
}

/// Run SQLite database migrations
async fn run_sqlite_migrations(pool: &DbPool) -> Result<()> {
    info!("Running SQLite database migrations");

    // Migration 001: Initial schema
    info!("Running migration 001_initial.sql");
    sqlx::query(include_str!("migrations/sqlite/001_initial.sql"))
        .execute(pool)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("Migration 001 failed: {}", e))
        })?;

    // Migration 002: Supplemental fields
    // Note: SQLite ALTER TABLE ADD COLUMN fails if column exists
    // We catch and ignore "duplicate column name" errors for idempotency
    info!("Running migration 002_supplemental_fields.sql");
    let result = sqlx::query(include_str!("migrations/sqlite/002_supplemental_fields.sql"))
        .execute(pool)
        .await;

    match result {
        Ok(_) => info!("Migration 002 completed"),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("duplicate column name") {
                info!("Migration 002 columns already exist, skipping");
            } else {
                return Err(crate::error::ServerError::Internal(format!(
                    "Migration 002 failed: {}",
                    e
                )));
            }
        }
    }

    info!("All SQLite migrations completed successfully");
    Ok(())
}

/// Run PostgreSQL database migrations
async fn run_postgres_migrations(pool: &DbPool) -> Result<()> {
    info!("Running PostgreSQL migrations");

    sqlx::query(include_str!("migrations/postgres/001_initial.sql"))
        .execute(pool)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("PostgreSQL migration failed: {}", e))
        })?;

    info!("PostgreSQL migrations completed successfully");
    Ok(())
}
