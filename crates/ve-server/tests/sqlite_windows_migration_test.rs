use sqlx::{any::AnyPoolOptions, AnyConnection, Row};
use uuid::Uuid;

use ve_server::{
    config::{Config, DatabaseBackend},
    db::{
        create_pool, install_drivers, run_migrations, test_run_sqlite_migration_004_with_hook,
        DbPool,
    },
    error::ServerError,
};

async fn setup_sqlite_pool(prefix: &str) -> DbPool {
    install_drivers();
    let temp_db = std::env::temp_dir().join(format!("{prefix}-{}.db", Uuid::new_v4()));
    let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
    AnyPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap()
}

fn sqlite_config(database_url: String, data_dir: std::path::PathBuf) -> Config {
    Config {
        listen_addr: "127.0.0.1:3000".parse().unwrap(),
        database_url,
        jwt_secret: "test-jwt-secret-that-is-long-enough".to_string(),
        jwt_expiration_secs: 30 * 24 * 60 * 60,
        pair_code_ttl_secs: 5 * 60,
        heartbeat_interval_secs: 30,
        connection_timeout_secs: 60,
        data_dir,
        cors_origins: Vec::new(),
        ack_timeout_ms: 10000,
        ack_max_retries: 2,
        ack_retry_delay_ms: 500,
        permission_ttl_secs: 30 * 60,
        permission_expiry_check_secs: 60,
        idempotency_ttl_secs: 24 * 60 * 60,
        idempotency_cleanup_secs: 60 * 60,
        log_format: "pretty".to_string(),
        log_level: "info".to_string(),
    }
}

async fn setup_pre_windows_sqlite_pool() -> DbPool {
    let pool = setup_sqlite_pool("ve-sqlite-windows-migration-test").await;

    let pre_windows_initial_schema = include_str!("../src/db/migrations/sqlite/001_initial.sql")
        .replace(
            "('linux', 'macos', 'windows', 'wsl')",
            "('linux', 'macos', 'wsl')",
        );

    sqlx::query(&pre_windows_initial_schema)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(include_str!(
        "../src/db/migrations/sqlite/002_supplemental_fields.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(include_str!(
        "../src/db/migrations/sqlite/003_device_access.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn setup_pre_windows_sqlite_pool_without_002() -> DbPool {
    let pool = setup_sqlite_pool("ve-sqlite-partial-002-test").await;

    let pre_windows_initial_schema = include_str!("../src/db/migrations/sqlite/001_initial.sql")
        .replace(
            "('linux', 'macos', 'windows', 'wsl')",
            "('linux', 'macos', 'wsl')",
        );

    sqlx::query(&pre_windows_initial_schema)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(include_str!(
        "../src/db/migrations/sqlite/003_device_access.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn setup_pre_007_sqlite_pool() -> DbPool {
    let pool = setup_sqlite_pool("ve-sqlite-pre-007-test").await;

    let pre_windows_initial_schema = include_str!("../src/db/migrations/sqlite/001_initial.sql")
        .replace(
            "('linux', 'macos', 'windows', 'wsl')",
            "('linux', 'macos', 'wsl')",
        );

    sqlx::query(&pre_windows_initial_schema)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(include_str!(
        "../src/db/migrations/sqlite/002_supplemental_fields.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(include_str!(
        "../src/db/migrations/sqlite/003_device_access.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(include_str!(
        "../src/db/migrations/sqlite/004_windows_platform.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(include_str!(
        "../src/db/migrations/sqlite/005_session_archive_uniqueness.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn insert_broken_workspace(pool: &DbPool) {
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
    )
    .bind("broken-workspace")
    .bind("missing-host")
    .bind("/tmp/broken")
    .bind("broken")
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn sqlite_create_pool_enables_foreign_keys_for_every_connection() {
    install_drivers();
    let data_dir = std::env::temp_dir().join(format!("ve-sqlite-pool-config-{}", Uuid::new_v4()));
    let database_path = data_dir.join("vibe.db");
    let database_url = format!("sqlite:{}?mode=rwc", database_path.display());
    let config = sqlite_config(database_url, data_dir.clone());

    let pool = create_pool(&config).await.unwrap();

    let mut connections = Vec::new();
    for _ in 0..5 {
        connections.push(pool.acquire().await.unwrap());
    }

    for (index, connection) in connections.iter_mut().enumerate() {
        let fk_enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&mut **connection)
            .await
            .unwrap();
        assert_eq!(
            fk_enabled, 1,
            "connection {index} should enforce foreign keys"
        );
    }

    drop(connections);

    sqlx::query("CREATE TABLE parent (id TEXT PRIMARY KEY NOT NULL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE child (id TEXT PRIMARY KEY NOT NULL, parent_id TEXT NOT NULL REFERENCES parent(id))",
    )
    .execute(&pool)
    .await
    .unwrap();

    let insert_result = sqlx::query("INSERT INTO child (id, parent_id) VALUES ($1, $2)")
        .bind("child-1")
        .bind("missing-parent")
        .execute(&pool)
        .await;
    assert!(insert_result.is_err());
}

#[tokio::test]
async fn sqlite_partial_migration_002_is_completed_on_retry() {
    let pool = setup_pre_windows_sqlite_pool_without_002().await;

    sqlx::query("ALTER TABLE idempotency_keys ADD COLUMN request_hash TEXT")
        .execute(&pool)
        .await
        .unwrap();

    run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap();

    let request_hash_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('idempotency_keys') WHERE name = 'request_hash'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(request_hash_exists, 1);

    let result_type_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('idempotency_keys') WHERE name = 'result_type'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(result_type_exists, 1);

    let expires_at_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('idempotency_keys') WHERE name = 'expires_at'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(expires_at_exists, 1);

    let metadata_json_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('session_archives') WHERE name = 'metadata_json'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(metadata_json_exists, 1);

    let expires_at_index_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_idempotency_keys_expires_at'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(expires_at_index_exists, 1);
}

#[tokio::test]
async fn sqlite_partial_migration_002_converges_without_duplicate_column_errors() {
    let pool = setup_pre_windows_sqlite_pool_without_002().await;

    sqlx::query("ALTER TABLE idempotency_keys ADD COLUMN request_hash TEXT")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("ALTER TABLE session_archives ADD COLUMN metadata_json TEXT")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_idempotency_keys_created_at ON idempotency_keys(created_at)")
        .execute(&pool)
        .await
        .unwrap();

    run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap();

    let request_hash_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('idempotency_keys') WHERE name = 'request_hash'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(request_hash_exists, 1);

    let result_type_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('idempotency_keys') WHERE name = 'result_type'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(result_type_exists, 1);

    let expires_at_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('idempotency_keys') WHERE name = 'expires_at'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(expires_at_exists, 1);

    let metadata_json_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('session_archives') WHERE name = 'metadata_json'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(metadata_json_exists, 1);

    let old_index_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_idempotency_keys_created_at'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(old_index_exists, 0);

    let expires_at_index_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_idempotency_keys_expires_at'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(expires_at_index_exists, 1);
}

#[tokio::test]
async fn sqlite_migrations_preserve_rerun_from_session_id_when_replaying_006_and_007() {
    let pool = setup_pre_007_sqlite_pool().await;

    sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
        .bind("host-rerun")
        .bind("host-rerun")
        .bind("linux")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
    )
    .bind("workspace-rerun")
    .bind("host-rerun")
    .bind("/tmp/rerun")
    .bind("rerun")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "ALTER TABLE sessions ADD COLUMN rerun_from_session_id TEXT REFERENCES sessions(session_id)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, claude_session_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $7, $7)",
    )
    .bind("archived-session")
    .bind("archived")
    .bind("host-rerun")
    .bind("workspace-rerun")
    .bind("claude_code")
    .bind("claude-archived")
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, claude_session_id, rerun_from_session_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $7, $8, $8)",
    )
    .bind("rerun-session")
    .bind("rerun")
    .bind("host-rerun")
    .bind("workspace-rerun")
    .bind("claude_code")
    .bind("claude-live")
    .bind("archived-session")
    .bind(&now)
    .execute(&pool)
    .await
    .unwrap();

    run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap();

    let rerun_from: Option<String> =
        sqlx::query_scalar("SELECT rerun_from_session_id FROM sessions WHERE session_id = $1")
            .bind("rerun-session")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(rerun_from.as_deref(), Some("archived-session"));

    let rerun_column_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('sessions') WHERE name = 'rerun_from_session_id'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(rerun_column_exists, 1);
}

#[tokio::test]
async fn sqlite_windows_migration_reenables_foreign_keys_when_begin_immediate_fails() {
    install_drivers();
    let temp_db = std::env::temp_dir().join(format!("ve-sqlite-004-lock-{}.db", Uuid::new_v4()));
    let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());

    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap();
    let lock_pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap();

    let pre_windows_initial_schema = include_str!("../src/db/migrations/sqlite/001_initial.sql")
        .replace(
            "('linux', 'macos', 'windows', 'wsl')",
            "('linux', 'macos', 'wsl')",
        );
    sqlx::query(&pre_windows_initial_schema)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(include_str!(
        "../src/db/migrations/sqlite/002_supplemental_fields.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(include_str!(
        "../src/db/migrations/sqlite/003_device_access.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();

    let mut locked_conn = lock_pool.acquire().await.unwrap();
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *locked_conn)
        .await
        .unwrap();

    let mut migration_conn: AnyConnection = pool.acquire().await.unwrap().detach();
    let error =
        test_run_sqlite_migration_004_with_hook(&mut migration_conn, || Ok::<(), ServerError>(()))
            .await
            .unwrap_err();
    assert!(error
        .to_string()
        .contains("Migration 004 failed to begin transaction"));

    let fk_enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut migration_conn)
        .await
        .unwrap();
    assert_eq!(fk_enabled, 1);

    sqlx::query("ROLLBACK")
        .execute(&mut *locked_conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn sqlite_windows_migration_rolls_back_when_004_fails_midway() {
    let pool = setup_pre_windows_sqlite_pool().await;

    sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
        .bind("host-atomic")
        .bind("host-atomic")
        .bind("linux")
        .execute(&pool)
        .await
        .unwrap();

    let mut conn: AnyConnection = pool.acquire().await.unwrap().detach();
    let error = test_run_sqlite_migration_004_with_hook(&mut conn, || -> Result<_, ServerError> {
        Err(ServerError::Internal(
            "forced failure after migration script".to_string(),
        ))
    })
    .await
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("forced failure after migration script"));

    let platform: String = sqlx::query("SELECT platform FROM hosts WHERE host_id = $1")
        .bind("host-atomic")
        .fetch_one(&pool)
        .await
        .unwrap()
        .get("platform");
    assert_eq!(platform, "linux");

    let hosts_new_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'hosts_new'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(hosts_new_exists, 0);

    let host_sql: String =
        sqlx::query("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'hosts'")
            .fetch_one(&pool)
            .await
            .unwrap()
            .get("sql");
    assert!(!host_sql.contains("'windows'"));
}

#[tokio::test]
async fn sqlite_windows_migration_preserves_foreign_keys_and_allows_windows_platform() {
    let pool = setup_pre_windows_sqlite_pool().await;

    let pre_migration_insert =
        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind("host-pre")
            .bind("host-pre")
            .bind("windows")
            .execute(&pool)
            .await;
    assert!(pre_migration_insert.is_err());

    sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
        .bind("host-1")
        .bind("host")
        .bind("linux")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
    )
    .bind("workspace-1")
    .bind("host-1")
    .bind("/tmp")
    .bind("tmp")
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO sessions (session_id, title, host_id, workspace_id) VALUES ($1, $2, $3, $4)",
    )
    .bind("session-1")
    .bind("title")
    .bind("host-1")
    .bind("workspace-1")
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
    )
    .bind("device-1")
    .bind("device")
    .bind("desktop")
    .bind("http://localhost")
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
        .bind("device-1")
        .bind("host-1")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
        .bind("device-1")
        .bind("session-1")
        .execute(&pool)
        .await
        .unwrap();

    run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap();

    let foreign_key_violations = sqlx::query("PRAGMA foreign_key_check")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert!(foreign_key_violations.is_empty());

    sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
        .bind("host-2")
        .bind("host-windows")
        .bind("windows")
        .execute(&pool)
        .await
        .unwrap();

    let stored_platform: String = sqlx::query("SELECT platform FROM hosts WHERE host_id = $1")
        .bind("host-2")
        .fetch_one(&pool)
        .await
        .unwrap()
        .get("platform");
    assert_eq!(stored_platform, "windows");

    let workspace_host: String =
        sqlx::query("SELECT host_id FROM workspaces WHERE workspace_id = $1")
            .bind("workspace-1")
            .fetch_one(&pool)
            .await
            .unwrap()
            .get("host_id");
    assert_eq!(workspace_host, "host-1");
}

#[tokio::test]
async fn sqlite_windows_migration_fails_when_foreign_keys_are_broken() {
    let pool = setup_pre_windows_sqlite_pool().await;
    insert_broken_workspace(&pool).await;

    let error = run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap_err();

    let message = error.to_string();
    assert!(message.contains("SQLite foreign key check failed after migration"));
    assert!(message.contains("table=workspaces"));
}

#[tokio::test]
async fn sqlite_windows_migration_still_fails_after_restart_when_foreign_keys_are_broken() {
    let pool = setup_pre_windows_sqlite_pool().await;
    insert_broken_workspace(&pool).await;

    let first_error = run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap_err();
    let first_message = first_error.to_string();
    assert!(first_message.contains("SQLite foreign key check failed after migration"));

    let second_error = run_migrations(&pool, DatabaseBackend::Sqlite)
        .await
        .unwrap_err();
    let second_message = second_error.to_string();
    assert!(second_message.contains("SQLite foreign key check failed after migration"));
    assert!(second_message.contains("table=workspaces"));
}
