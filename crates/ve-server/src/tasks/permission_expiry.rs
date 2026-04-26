//! Permission Expiry Background Task
//!
//! Periodically checks and expires stale permission requests.

use std::sync::Arc;
use std::time::Duration;

use crate::db::DbPool;
use tokio::time::interval;
use tracing::{error, info};
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::{ClientMessage, DaemonMessage};

use crate::config::Config;
use crate::hub::Hub;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpiredPermissionNotification {
    permission_id: Uuid,
    session_id: Uuid,
    host_id: Uuid,
}

/// Start the permission expiry background task.
///
/// This task runs periodically to check for permission requests that have
/// exceeded their TTL and marks them as expired.
pub fn start_permission_expiry_task(
    db: DbPool,
    hub: Arc<Hub>,
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
        if let Err(e) = expire_stale_permissions_and_notify(
            db.clone(),
            hub.clone(),
            expiry_expr.clone(),
            now_expr.clone(),
        )
        .await
        {
            error!(error = %e, "Failed to run initial permission expiry");
        }

        loop {
            ticker.tick().await;

            match expire_stale_permissions_and_notify(
                db.clone(),
                hub.clone(),
                expiry_expr.clone(),
                now_expr.clone(),
            )
            .await
            {
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
    Ok(
        expire_stale_permissions_in_transaction(db, expiry_expr_sql, now_expr_sql)
            .await?
            .len(),
    )
}

async fn expire_stale_permissions_and_notify(
    db: DbPool,
    hub: Arc<Hub>,
    expiry_expr_sql: String,
    now_expr_sql: String,
) -> Result<usize, sqlx::Error> {
    let expired_permissions =
        expire_stale_permissions_in_transaction(&db, &expiry_expr_sql, &now_expr_sql).await?;

    for expired in &expired_permissions {
        let _ = hub
            .send_to_daemon(
                &expired.host_id,
                DaemonMessage::PermissionResponse {
                    permission_id: expired.permission_id,
                    session_id: expired.session_id,
                    decision: PermissionDecision::DenyOnce,
                },
            )
            .await;

        hub.broadcast_to_session(
            &db,
            &expired.session_id,
            ClientMessage::PermissionExpired {
                permission_id: expired.permission_id,
                session_id: expired.session_id,
            },
        )
        .await;
    }

    Ok(expired_permissions.len())
}

async fn expire_stale_permissions_in_transaction(
    db: &DbPool,
    expiry_expr_sql: &str,
    now_expr_sql: &str,
) -> Result<Vec<ExpiredPermissionNotification>, sqlx::Error> {
    let expired_permissions: Vec<(String, String, String)> = sqlx::query_as(&format!(
        r#"
        UPDATE permission_requests
        SET status = 'expired', responded_at = {}
        WHERE status = 'pending' AND created_at < {}
        RETURNING
            permission_id,
            session_id,
            (
                SELECT host_id
                FROM sessions
                WHERE sessions.session_id = permission_requests.session_id
            ) AS host_id
        "#,
        now_expr_sql, expiry_expr_sql
    ))
    .fetch_all(db)
    .await?;

    if !expired_permissions.is_empty() {
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
    }

    let expired_permissions = expired_permissions
        .into_iter()
        .map(
            |(permission_id, session_id, host_id)| -> std::result::Result<_, sqlx::Error> {
                Ok(ExpiredPermissionNotification {
                    permission_id: Uuid::parse_str(&permission_id).map_err(|error| {
                        sqlx::Error::Protocol(format!(
                            "invalid permission_id returned from DB: {error}"
                        ))
                    })?,
                    session_id: Uuid::parse_str(&session_id).map_err(|error| {
                        sqlx::Error::Protocol(format!(
                            "invalid session_id returned from DB: {error}"
                        ))
                    })?,
                    host_id: Uuid::parse_str(&host_id).map_err(|error| {
                        sqlx::Error::Protocol(format!("invalid host_id returned from DB: {error}"))
                    })?,
                })
            },
        )
        .collect::<std::result::Result<Vec<_>, sqlx::Error>>()?;

    if !expired_permissions.is_empty() {
        info!(
            rows_affected = expired_permissions.len(),
            "Updated sessions after expiring permissions"
        );
    }

    Ok(expired_permissions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseBackend};
    use crate::db::{install_drivers, run_migrations, DbPool};
    use crate::hub::Hub;
    use tokio::sync::mpsc;
    use uuid::Uuid;

    fn test_config(database_url: String) -> Config {
        const TEST_JWT_SECRET: &str = "test_secret_for_unit_tests_only_32chars!";
        Config {
            listen_addr: "127.0.0.1:3000".parse().unwrap(),
            database_url,
            jwt_secret: TEST_JWT_SECRET.to_string(),
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

        let affected =
            expire_stale_permissions(&db, "datetime('now', '-1800 seconds')", "datetime('now')")
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

    #[tokio::test]
    async fn expire_stale_permissions_notifies_daemon_and_session_subscribers() {
        let db = setup_db().await;
        let hub = Hub::new();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let permission_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();

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

        sqlx::query("INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, pending_permission_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'waiting_approval', 1, $6, $6)")
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
            .bind(permission_id.to_string())
            .bind(session_id.to_string())
            .bind("stale")
            .bind(&old_time)
            .execute(&db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&db)
            .await
            .unwrap();

        let (daemon_tx, mut daemon_rx) = mpsc::channel(8);
        let (client_tx, mut client_rx) = mpsc::channel(8);
        hub.register_daemon(host_id, daemon_tx).await;
        hub.register_client(device_id, client_tx);
        hub.subscribe_session(device_id, session_id);

        let affected = expire_stale_permissions_and_notify(
            db.clone(),
            Arc::new(hub),
            "datetime('now', '-1800 seconds')".to_string(),
            "datetime('now')".to_string(),
        )
        .await
        .unwrap();
        assert_eq!(affected, 1);

        let daemon_message = daemon_rx.recv().await.unwrap();
        assert_eq!(daemon_message.r#type, "permission_response");
        assert_eq!(
            daemon_message.payload["permission_id"],
            permission_id.to_string()
        );
        assert_eq!(daemon_message.payload["decision"], "deny_once");

        let client_message = client_rx.recv().await.unwrap();
        assert_eq!(client_message.r#type, "permission_expired");
        assert_eq!(
            client_message.payload["payload"]["permission_id"],
            permission_id.to_string()
        );
        assert_eq!(
            client_message.payload["payload"]["session_id"],
            session_id.to_string()
        );
    }
}
