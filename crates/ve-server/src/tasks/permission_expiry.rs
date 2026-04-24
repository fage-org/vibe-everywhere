//! Permission Expiry Background Task
//!
//! Periodically checks and expires stale permission requests.

use std::sync::Arc;
use std::time::Duration;

use crate::db::DbPool;
use tokio::time::interval;
use tracing::{error, info};

use crate::config::Config;

/// Start the permission expiry background task.
///
/// This task runs periodically to check for permission requests that have
/// exceeded their TTL and marks them as expired.
pub fn start_permission_expiry_task(
    db: DbPool,
    config: Arc<Config>,
) -> tokio::task::JoinHandle<()> {
    let check_interval = Duration::from_secs(config.permission_expiry_check_secs);
    let permission_ttl_secs = config.permission_ttl_secs;

    // Pre-compute database-specific SQL expressions.
    let (expiry_expr, now_expr) = match config.database_backend() {
        crate::config::DatabaseBackend::Postgres => (
            format!("NOW() - INTERVAL '{} seconds'", permission_ttl_secs),
            "NOW()".to_string(),
        ),
        crate::config::DatabaseBackend::Sqlite => (
            format!("datetime('now', '-{} seconds')", permission_ttl_secs),
            "datetime('now')".to_string(),
        ),
    };

    tokio::spawn(async move {
        let mut ticker = interval(check_interval);

        info!(
            check_interval_secs = check_interval.as_secs(),
            permission_ttl_secs, "Permission expiry task started"
        );

        // Run immediately on startup, then on each tick
        if let Err(e) = expire_stale_permissions(&db, &expiry_expr, &now_expr).await {
            error!(error = %e, "Failed to run initial permission expiry");
        }

        loop {
            ticker.tick().await;

            match expire_stale_permissions(&db, &expiry_expr, &now_expr).await {
                Ok(affected) => {
                    if affected > 0 {
                        info!(
                            rows_affected = affected,
                            "Expired stale permission requests"
                        );
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to expire stale permissions");
                }
            }
        }
    })
}

/// Expires permissions in the database and recomputes session counters.
///
/// This task only guarantees server-side state convergence for read paths.
/// It does not emit live client or daemon notifications.
///
/// `expiry_expr_sql` should be a SQL expression that evaluates to the threshold
/// time (e.g., `NOW() - INTERVAL '1800 seconds'` for PostgreSQL or
/// `datetime('now', '-1800 seconds')` for SQLite).
/// `now_expr_sql` should be a SQL expression for the current time
/// (e.g., `NOW()` for PostgreSQL or `datetime('now')` for SQLite).
pub async fn expire_stale_permissions(
    db: &DbPool,
    expiry_expr_sql: &str,
    now_expr_sql: &str,
) -> Result<usize, sqlx::Error> {
    let query = format!(
        "UPDATE permission_requests SET status = 'expired' WHERE status = 'pending' AND created_at < {}",
        expiry_expr_sql
    );

    let result = sqlx::raw_sql(&query).execute(db).await?;
    let rows_affected = result.rows_affected() as usize;

    if rows_affected > 0 {
        sqlx::raw_sql(&format!(
            r#"
            UPDATE sessions
            SET pending_permission_count = (
                SELECT COUNT(*) FROM permission_requests pr
                WHERE pr.session_id = sessions.session_id AND pr.status = 'pending'
            ),
            updated_at = {}
            WHERE session_id IN (
                SELECT DISTINCT session_id FROM permission_requests
                WHERE status = 'expired'
            )
            "#,
            now_expr_sql
        ))
        .execute(db)
        .await?;

        info!(rows_affected, "Updated sessions after expiring permissions");
    }

    Ok(rows_affected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseBackend};
    use crate::db::{install_drivers, run_migrations, DbPool};
    use uuid::Uuid;

    fn test_config(database_url: String) -> Config {
        Config {
            listen_addr: "127.0.0.1:3000".parse().unwrap(),
            database_url,
            jwt_secret: "01234567890123456789012345678901".to_string(),
            jwt_expiration_secs: 3600,
            pair_code_ttl_secs: 300,
            heartbeat_interval_secs: 30,
            connection_timeout_secs: 60,
            data_dir: std::path::PathBuf::from("/tmp"),
            cors_origins: Vec::new(),
            ack_timeout_ms: 10000,
            ack_max_retries: 2,
            ack_retry_delay_ms: 500,
            permission_ttl_secs: 1800,
            permission_expiry_check_secs: 60,
            idempotency_ttl_secs: 86400,
            idempotency_cleanup_secs: 3600,
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
        }
    }

    async fn setup_db() -> DbPool {
        install_drivers();
        let temp_db =
            std::env::temp_dir().join(format!("ve-permission-expiry-test-{}.db", Uuid::new_v4()));
        let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
        let _config = test_config(database_url.clone());
        let pool = DbPool::connect(&database_url).await.unwrap();
        run_migrations(&pool, DatabaseBackend::Sqlite)
            .await
            .unwrap();
        pool
    }

    #[test]
    fn test_ttl_calculation() {
        let ttl_secs: u64 = 1800;
        assert_eq!(ttl_secs, 30 * 60);
    }

    #[tokio::test]
    async fn expire_stale_permissions_updates_db_state_and_session_count_only() {
        let db = setup_db().await;
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let stale_permission_id = Uuid::new_v4();
        let fresh_permission_id = Uuid::new_v4();

        // Use SQLite datetime expressions for test fixtures to ensure comparable formats.
        let old_time: (String,) = sqlx::query_as("SELECT datetime('now', '-2 hours')")
            .fetch_one(&db)
            .await
            .unwrap();
        let now: (String,) = sqlx::query_as("SELECT datetime('now')")
            .fetch_one(&db)
            .await
            .unwrap();
        let old_time = old_time.0;
        let now = now.0;

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform, pair_status, created_at, updated_at) VALUES ($1, $2, $3, 'paired', $4, $4)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .bind(&now)
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp/ws")
            .bind("ws")
            .bind(&now)
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, pending_permission_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'waiting_approval', 2, $6, $6)")
            .bind(session_id.to_string())
            .bind("test")
            .bind(host_id.to_string())
            .bind(workspace_id.to_string())
            .bind("claude_code")
            .bind(&now)
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO permission_requests (permission_id, session_id, risk_type, summary, status, created_at) VALUES ($1, $2, 'exec_cmd', $3, 'pending', $4)")
            .bind(stale_permission_id.to_string())
            .bind(session_id.to_string())
            .bind("stale")
            .bind(&old_time)
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO permission_requests (permission_id, session_id, risk_type, summary, status, created_at) VALUES ($1, $2, 'exec_cmd', $3, 'pending', $4)")
            .bind(fresh_permission_id.to_string())
            .bind(session_id.to_string())
            .bind("fresh")
            .bind(&now)
            .execute(&db)
            .await
            .unwrap();

        let affected = expire_stale_permissions(
            &db,
            "datetime('now', '-1800 seconds')",
            "datetime('now')",
        )
        .await
        .unwrap();
        assert_eq!(affected, 1);

        let stale_status: (String,) =
            sqlx::query_as("SELECT status FROM permission_requests WHERE permission_id = $1")
                .bind(stale_permission_id.to_string())
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(stale_status.0, "expired");

        let fresh_status: (String,) =
            sqlx::query_as("SELECT status FROM permission_requests WHERE permission_id = $1")
                .bind(fresh_permission_id.to_string())
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(fresh_status.0, "pending");

        let pending_count: (i64,) =
            sqlx::query_as("SELECT pending_permission_count FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(pending_count.0, 1);
    }
}
