//! Database Initialization
//!
//! Connection pool creation and migration execution using AnyPool for runtime database selection.

pub mod idempotency;

use sqlx::any::AnyPoolOptions;
use sqlx::migrate::MigrateDatabase;
use sqlx::{AnyConnection, Row};
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
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("PRAGMA foreign_keys = ON")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("PRAGMA busy_timeout = 5000")
                    .execute(&mut *conn)
                    .await?;
                Ok(())
            })
        })
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

    info!("Creating PostgreSQL connection pool");

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

async fn sqlite_hosts_supports_windows(pool: &DbPool) -> Result<bool> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'hosts'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        crate::error::ServerError::Internal(format!("Failed to inspect SQLite schema: {}", e))
    })?;

    Ok(row.map(|(sql,)| sql.contains("'windows'")).unwrap_or(false))
}

async fn sqlite_table_has_column(pool: &DbPool, table: &str, column: &str) -> Result<bool> {
    let pragma = format!("PRAGMA table_info('{table}')");
    let columns = sqlx::query(&pragma).fetch_all(pool).await.map_err(|e| {
        crate::error::ServerError::Internal(format!(
            "Failed to inspect SQLite table {table}: {}",
            e
        ))
    })?;

    Ok(columns.iter().any(|row| {
        row.try_get::<String, _>(1)
            .map(|name| name == column)
            .unwrap_or(false)
    }))
}

async fn sqlite_index_exists(pool: &DbPool, index_name: &str) -> Result<bool> {
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = $1",
    )
    .bind(index_name)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        crate::error::ServerError::Internal(format!(
            "Failed to inspect SQLite index {index_name}: {}",
            e
        ))
    })?;

    Ok(count > 0)
}

async fn sqlite_foreign_key_check(pool: &DbPool) -> Result<()> {
    let violations = sqlx::query("PRAGMA foreign_key_check")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!(
                "Failed to verify SQLite foreign keys: {}",
                e
            ))
        })?;

    if violations.is_empty() {
        return Ok(());
    }

    let details = violations
        .iter()
        .map(|row| {
            let table = row
                .try_get::<String, _>(0)
                .unwrap_or_else(|_| "<unknown>".to_string());
            let rowid = row.try_get::<i64, _>(1).unwrap_or(-1);
            let parent = row
                .try_get::<String, _>(2)
                .unwrap_or_else(|_| "<unknown>".to_string());
            let fk_id = row.try_get::<i64, _>(3).unwrap_or(-1);
            format!("table={table}, rowid={rowid}, parent={parent}, fk_id={fk_id}")
        })
        .collect::<Vec<_>>()
        .join("; ");

    Err(crate::error::ServerError::Internal(format!(
        "SQLite foreign key check failed after migration: {details}"
    )))
}

async fn run_sqlite_migration_004(conn: &mut AnyConnection) -> Result<()> {
    run_sqlite_migration_004_with_hook(conn, || Ok(())).await
}

async fn run_sqlite_migration_004_with_hook<F>(
    conn: &mut AnyConnection,
    after_script: F,
) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!(
                "Migration 004 failed to disable foreign keys: {}",
                e
            ))
        })?;

    let migration_result = async {
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                crate::error::ServerError::Internal(format!(
                    "Migration 004 failed to begin transaction: {}",
                    e
                ))
            })?;

        sqlx::query(include_str!("migrations/sqlite/004_windows_platform.sql"))
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                crate::error::ServerError::Internal(format!("Migration 004 failed: {}", e))
            })?;
        after_script()?;
        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(|e| {
                crate::error::ServerError::Internal(format!(
                    "Migration 004 failed to commit transaction: {}",
                    e
                ))
            })?;
        Ok(())
    }
    .await;

    if migration_result.is_err() {
        let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
    }

    let enable_result = sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!(
                "Migration 004 failed to re-enable foreign keys: {}",
                e
            ))
        });

    match (migration_result, enable_result) {
        (Ok(()), Ok(_)) => Ok(()),
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
    }
}

async fn run_sqlite_migration_003(pool: &DbPool) -> Result<()> {
    let result = sqlx::query(include_str!("migrations/sqlite/003_device_access.sql"))
        .execute(pool)
        .await;

    match result {
        Ok(_) => info!("Migration 003 completed"),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("already exists") {
                info!("Migration 003 tables already exist, skipping");
            } else {
                return Err(crate::error::ServerError::Internal(format!(
                    "Migration 003 failed: {}",
                    e
                )));
            }
        }
    }

    Ok(())
}

async fn run_sqlite_migration_006(pool: &DbPool) -> Result<()> {
    let rerun_select = if sqlite_table_has_column(pool, "sessions", "rerun_from_session_id").await?
    {
        "rerun_from_session_id"
    } else {
        "NULL"
    };

    let migration_sql = include_str!("migrations/sqlite/006_session_pending_status.sql")
        .replace("__RERUN_FROM_SESSION_ID_SELECT__", rerun_select);

    sqlx::query(&migration_sql)
        .execute(pool)
        .await
        .map_err(|e| crate::error::ServerError::Internal(format!("Migration 006 failed: {}", e)))?;

    Ok(())
}

async fn run_sqlite_migration_002(pool: &DbPool) -> Result<()> {
    let steps = [
        (
            "add idempotency_keys.request_hash",
            !sqlite_table_has_column(pool, "idempotency_keys", "request_hash").await?,
            "ALTER TABLE idempotency_keys ADD COLUMN request_hash TEXT",
        ),
        (
            "add idempotency_keys.result_type",
            !sqlite_table_has_column(pool, "idempotency_keys", "result_type").await?,
            "ALTER TABLE idempotency_keys ADD COLUMN result_type TEXT NOT NULL DEFAULT 'session'",
        ),
        (
            "add idempotency_keys.expires_at",
            !sqlite_table_has_column(pool, "idempotency_keys", "expires_at").await?,
            "ALTER TABLE idempotency_keys ADD COLUMN expires_at TEXT",
        ),
        (
            "drop idx_idempotency_keys_created_at",
            sqlite_index_exists(pool, "idx_idempotency_keys_created_at").await?,
            "DROP INDEX IF EXISTS idx_idempotency_keys_created_at",
        ),
        (
            "create idx_idempotency_keys_expires_at",
            !sqlite_index_exists(pool, "idx_idempotency_keys_expires_at").await?,
            "CREATE INDEX IF NOT EXISTS idx_idempotency_keys_expires_at ON idempotency_keys(expires_at)",
        ),
        (
            "add client_devices.legacy_acl",
            !sqlite_table_has_column(pool, "client_devices", "legacy_acl").await?,
            "ALTER TABLE client_devices ADD COLUMN legacy_acl INTEGER NOT NULL DEFAULT 1",
        ),
        (
            "add session_archives.metadata_json",
            !sqlite_table_has_column(pool, "session_archives", "metadata_json").await?,
            "ALTER TABLE session_archives ADD COLUMN metadata_json TEXT",
        ),
    ];

    for (label, should_run, statement) in steps {
        if !should_run {
            info!(
                step = label,
                "Migration 002 step already converged, skipping"
            );
            continue;
        }

        sqlx::query(statement).execute(pool).await.map_err(|e| {
            crate::error::ServerError::Internal(format!("Migration 002 failed at {label}: {}", e))
        })?;
        info!(step = label, "Migration 002 step completed");
    }

    Ok(())
}

/// Run SQLite database migrations
async fn run_sqlite_migrations(pool: &DbPool) -> Result<()> {
    info!("Running SQLite database migrations");

    // Migration 001: Initial schema
    info!("Running migration 001_initial.sql");
    sqlx::query(include_str!("migrations/sqlite/001_initial.sql"))
        .execute(pool)
        .await
        .map_err(|e| crate::error::ServerError::Internal(format!("Migration 001 failed: {}", e)))?;

    // Migration 002: Supplemental fields
    info!("Running migration 002_supplemental_fields.sql");
    run_sqlite_migration_002(pool).await?;

    // Migration 003: Device access control tables
    info!("Running migration 003_device_access.sql");
    run_sqlite_migration_003(pool).await?;

    // Migration 004: Windows platform support
    info!("Running migration 004_windows_platform.sql");
    if sqlite_hosts_supports_windows(pool).await? {
        info!("Migration 004 already applied, skipping");
    } else {
        let mut conn = pool.acquire().await.map_err(|e| {
            crate::error::ServerError::Internal(format!(
                "Migration 004 failed to acquire SQLite connection: {}",
                e
            ))
        })?;
        run_sqlite_migration_004(&mut conn).await?;
        info!("Migration 004 completed");
    }

    if sqlite_hosts_supports_windows(pool).await? {
        sqlite_foreign_key_check(pool).await?;
    }

    // Migration 005: Session archive uniqueness
    info!("Running migration 005_session_archive_uniqueness.sql");
    sqlx::query(include_str!(
        "migrations/sqlite/005_session_archive_uniqueness.sql"
    ))
    .execute(pool)
    .await
    .map_err(|e| crate::error::ServerError::Internal(format!("Migration 005 failed: {}", e)))?;

    // Migration 006: Session pending status
    info!("Running migration 006_session_pending_status.sql");
    run_sqlite_migration_006(pool).await?;

    // Migration 007: Session rerun idempotency
    info!("Running migration 007_session_rerun_idempotency.sql");
    sqlx::query(include_str!(
        "migrations/sqlite/007_session_rerun_idempotency.sql"
    ))
    .execute(pool)
    .await
    .map_err(|e| crate::error::ServerError::Internal(format!("Migration 007 failed: {}", e)))?;

    // Migration 008: Pairing polling secret
    info!("Running migration 008_pairing_secret.sql");
    let result = sqlx::query(include_str!("migrations/sqlite/008_pairing_secret.sql"))
        .execute(pool)
        .await;

    match result {
        Ok(_) => info!("Migration 008 completed"),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("duplicate column name") {
                info!("Migration 008 column already exists, skipping");
            } else {
                return Err(crate::error::ServerError::Internal(format!(
                    "Migration 008 failed: {}",
                    e
                )));
            }
        }
    }

    info!("All SQLite migrations completed successfully");
    Ok(())
}

async fn run_postgres_migration_002(pool: &DbPool) -> Result<()> {
    sqlx::query(include_str!("migrations/postgres/002_device_access.sql"))
        .execute(pool)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("PostgreSQL migration 002 failed: {}", e))
        })?;

    sqlx::query(
        "ALTER TABLE client_devices ADD COLUMN IF NOT EXISTS legacy_acl INTEGER NOT NULL DEFAULT 1",
    )
    .execute(pool)
    .await
    .map_err(|e| {
        crate::error::ServerError::Internal(format!(
            "PostgreSQL client_devices legacy_acl migration failed: {}",
            e
        ))
    })?;

    Ok(())
}

/// Run PostgreSQL database migrations
async fn run_postgres_migrations(pool: &DbPool) -> Result<()> {
    info!("Running PostgreSQL migrations");

    sqlx::query(include_str!("migrations/postgres/001_initial.sql"))
        .execute(pool)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("PostgreSQL migration 001 failed: {}", e))
        })?;

    run_postgres_migration_002(pool).await?;

    sqlx::query(include_str!("migrations/postgres/003_windows_platform.sql"))
        .execute(pool)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("PostgreSQL migration 003 failed: {}", e))
        })?;

    sqlx::query(include_str!(
        "migrations/postgres/004_session_archive_uniqueness.sql"
    ))
    .execute(pool)
    .await
    .map_err(|e| {
        crate::error::ServerError::Internal(format!("PostgreSQL migration 004 failed: {}", e))
    })?;

    sqlx::query(include_str!(
        "migrations/postgres/005_session_pending_status.sql"
    ))
    .execute(pool)
    .await
    .map_err(|e| {
        crate::error::ServerError::Internal(format!("PostgreSQL migration 005 failed: {}", e))
    })?;

    sqlx::query(include_str!(
        "migrations/postgres/006_session_rerun_idempotency.sql"
    ))
    .execute(pool)
    .await
    .map_err(|e| {
        crate::error::ServerError::Internal(format!("PostgreSQL migration 006 failed: {}", e))
    })?;

    sqlx::query(include_str!("migrations/postgres/007_pairing_secret.sql"))
        .execute(pool)
        .await
        .map_err(|e| {
            crate::error::ServerError::Internal(format!("PostgreSQL migration 007 failed: {}", e))
        })?;

    info!("PostgreSQL migrations completed successfully");
    Ok(())
}

pub async fn test_run_sqlite_migration_004_with_hook<F>(
    conn: &mut AnyConnection,
    after_script: F,
) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    run_sqlite_migration_004_with_hook(conn, after_script).await
}
