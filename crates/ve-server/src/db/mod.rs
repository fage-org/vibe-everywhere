//! Database Initialization
//!
//! Connection pool creation and migration execution.

use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use tracing::info;

use crate::config::Config;
use crate::error::Result;

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

/// Run database migrations
pub async fn run_migrations(pool: &sqlx::SqlitePool) -> Result<()> {
    info!("Running database migrations");

    // Run inline migrations for SQLite
    // In a production setup, these would be separate .sql files
    sqlx::query(include_str!("migrations/sqlite/001_initial.sql"))
        .execute(pool)
        .await
        .map_err(|e| crate::error::ServerError::Internal(format!("Migration failed: {}", e)))?;

    info!("Migrations completed successfully");
    Ok(())
}
