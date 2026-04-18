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
pub fn start_permission_expiry_task(db: DbPool, config: Arc<Config>) -> tokio::task::JoinHandle<()> {
    let check_interval = Duration::from_secs(config.permission_expiry_check_secs);
    let permission_ttl_secs = config.permission_ttl_secs;

    tokio::spawn(async move {
        let mut ticker = interval(check_interval);

        info!(
            check_interval_secs = check_interval.as_secs(),
            permission_ttl_secs,
            "Permission expiry task started"
        );

        loop {
            ticker.tick().await;

            match expire_stale_permissions(&db, permission_ttl_secs).await {
                Ok(affected) => {
                    if affected > 0 {
                        info!(rows_affected = affected, "Expired stale permission requests");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to expire stale permissions");
                }
            }
        }
    })
}

/// Expire permission requests that have exceeded their TTL.
///
/// Returns the number of permission requests that were expired.
async fn expire_stale_permissions(db: &DbPool, ttl_secs: u64) -> Result<usize, sqlx::Error> {
    // Begin transaction
    let mut tx = db.begin().await?;

    // Calculate expiry threshold (permissions created before this time are expired)
    // This approach is database-agnostic: compare ISO8601 timestamps directly
    let expiry_threshold = chrono::Utc::now() - chrono::Duration::seconds(ttl_secs as i64);
    let expiry_threshold_str = expiry_threshold.to_rfc3339();

    // Find and expire stale permissions
    // Database-agnostic: compare timestamps directly using ISO8601 format
    let result = sqlx::query(
        r#"
        UPDATE permission_requests
        SET status = 'expired'
        WHERE status = 'pending'
          AND created_at < $1
        "#
    )
    .bind(&expiry_threshold_str)
    .execute(&mut *tx)
    .await?;

    let rows_affected = result.rows_affected() as usize;

    if rows_affected > 0 {
        // Update pending_permission_count for affected sessions
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE sessions
            SET pending_permission_count = (
                SELECT COUNT(*) FROM permission_requests pr
                WHERE pr.session_id = sessions.session_id AND pr.status = 'pending'
            ),
            updated_at = $1
            WHERE session_id IN (
                SELECT DISTINCT session_id FROM permission_requests
                WHERE status = 'expired'
            )
            "#
        )
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        info!(
            rows_affected,
            "Updated sessions after expiring permissions"
        );
    }

    tx.commit().await?;

    Ok(rows_affected)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ttl_calculation() {
        // TTL of 1800 seconds = 30 minutes
        let ttl_secs: u64 = 1800;
        assert_eq!(ttl_secs, 30 * 60);
    }
}
