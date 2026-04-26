//! Session Recovery on Startup
//!
//! Recovers from server crash scenarios where a session was left
//! in 'dispatching' status without the daemon having acknowledged.

use tracing::{error, info};

use crate::db::DbPool;

/// Recover orphaned dispatching sessions after a server restart.
///
/// Sessions in 'dispatching' status older than 5 minutes are transitioned
/// to 'error' so they no longer block future reruns of the same archived session.
pub async fn recover_orphaned_dispatching_sessions(db: &DbPool) {
    // Only recover sessions that have been dispatching for more than 5 minutes.
    // A legitimate dispatch takes at most a few seconds.
    let cutoff = (chrono::Utc::now() - chrono::Duration::minutes(5)).to_rfc3339();

    let result = sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'error',
            latest_summary = 'Server restarted during dispatch; session orphaned'
        WHERE status = 'dispatching'
          AND created_at < $1
        "#,
    )
    .bind(&cutoff)
    .execute(db)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => {
            info!(
                count = r.rows_affected(),
                cutoff = %cutoff,
                "Recovered orphaned dispatching sessions after server restart"
            );
        }
        Ok(_) => {}
        Err(e) => {
            error!(
                error = %e,
                "Failed to recover orphaned dispatching sessions"
            );
        }
    }
}
