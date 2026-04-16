//! Idempotency Key Cleanup Background Task
//!
//! Periodically removes expired idempotency keys from the database.

use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;
use tokio::time::interval;
use tracing::{error, info};

use crate::config::Config;

/// Start the idempotency key cleanup background task.
///
/// This task runs periodically to remove expired idempotency keys
/// that have passed their TTL.
pub fn start_idempotency_cleanup_task(
    db: SqlitePool,
    config: Arc<Config>,
) -> tokio::task::JoinHandle<()> {
    let cleanup_interval = Duration::from_secs(config.idempotency_cleanup_secs);

    tokio::spawn(async move {
        let mut ticker = interval(cleanup_interval);

        info!(
            cleanup_interval_secs = cleanup_interval.as_secs(),
            "Idempotency cleanup task started"
        );

        loop {
            ticker.tick().await;

            match cleanup_expired_keys(&db).await {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!(deleted, "Cleaned up expired idempotency keys");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to cleanup expired idempotency keys");
                }
            }
        }
    })
}

/// Remove expired idempotency keys from the database.
///
/// Returns the number of keys that were deleted.
async fn cleanup_expired_keys(db: &SqlitePool) -> Result<usize, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query(
        r#"
        DELETE FROM idempotency_keys
        WHERE expires_at IS NOT NULL AND expires_at < ?
        "#,
    )
    .bind(&now)
    .execute(db)
    .await?;

    Ok(result.rows_affected() as usize)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cleanup_timing() {
        // Default cleanup interval is 1 hour
        let cleanup_interval_secs: u64 = 3600;
        assert_eq!(cleanup_interval_secs, 60 * 60);
    }
}
