//! Database Initialization
//!
//! Connection pool creation and migration execution.

pub mod idempotency;

use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use tracing::info;

#[cfg(feature = "postgres")]
use sqlx::postgres::PgPoolOptions;

use crate::config::Config;
use crate::error::Result;

// Database pool type alias based on feature flags
// PostgreSQL takes priority when both features are enabled
#[cfg(feature = "postgres")]
pub type DbPool = sqlx::PgPool;

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub type DbPool = sqlx::SqlitePool;

/// Create a SQLite connection pool
pub async fn create_sqlite_pool(config: &Config) -> Result<sqlx::SqlitePool> {
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

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("Failed to connect to database: {}", e))
        })?;

    Ok(pool)
}

/// Create a PostgreSQL connection pool
#[cfg(feature = "postgres")]
pub async fn create_postgres_pool(config: &Config) -> Result<sqlx::PgPool> {
    let db_url = &config.database_url;

    info!(url = %db_url, "Creating PostgreSQL connection pool");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(db_url)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("Failed to connect to PostgreSQL: {}", e))
        })?;

    info!("PostgreSQL connection pool created successfully");
    Ok(pool)
}

/// Run SQLite database migrations
pub async fn run_sqlite_migrations(pool: &sqlx::SqlitePool) -> Result<()> {
    info!("Running database migrations");

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

    info!("All migrations completed successfully");
    Ok(())
}

/// Run PostgreSQL database migrations
#[cfg(feature = "postgres")]
pub async fn run_postgres_migrations(pool: &sqlx::PgPool) -> Result<()> {
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

/// Run migrations based on pool type
#[cfg(feature = "postgres")]
pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    run_postgres_migrations(pool).await
}

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    run_sqlite_migrations(pool).await
}
