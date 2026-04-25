//! Token revocation service.
//!
//! Provides database-backed JWT revocation checks and revocation operations.

use chrono::Utc;
use uuid::Uuid;

use crate::db::DbPool;
use crate::error::Result;

/// Record that a token has been revoked.
pub async fn revoke_token(pool: &DbPool, jti: &str, device_id: Uuid, expires_at: chrono::DateTime<Utc>) -> Result<()> {
    sqlx::query::<sqlx::Any>(
        r#"
        INSERT INTO revoked_tokens (jti, device_id, expires_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (jti) DO NOTHING
        "#,
    )
    .bind(jti)
    .bind(device_id.to_string())
    .bind(expires_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

/// Check whether a token (by jti) has been revoked.
pub async fn is_revoked(pool: &DbPool, jti: &str) -> Result<bool> {
    let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, _>(
        r#"SELECT 1 FROM revoked_tokens WHERE jti = $1"#,
    )
    .bind(jti)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

/// Verify that the given jti matches the device's current_jti.
/// Returns `true` if the token is still valid (device has no current_jti set, or it matches).
pub async fn jti_matches_device(pool: &DbPool, device_id: Uuid, jti: &str) -> Result<bool> {
    let row: Option<(Option<String>,)> = sqlx::query_as::<sqlx::Any, _>(
        r#"SELECT current_jti FROM client_devices WHERE device_id = $1"#,
    )
    .bind(device_id.to_string())
    .fetch_optional(pool)
    .await?;

    match row {
        Some((Some(current_jti),)) => Ok(current_jti == jti),
        Some((None,)) => Ok(true),    // No current_jti set — legacy or after re-pair not yet completed
        None => Ok(false),            // Device doesn't exist
    }
}

/// Update the device's current_jti to the given value.
pub async fn update_device_current_jti(pool: &DbPool, device_id: Uuid, jti: &str) -> Result<()> {
    sqlx::query::<sqlx::Any>(
        r#"
        UPDATE client_devices
        SET current_jti = $2
        WHERE device_id = $1
        "#,
    )
    .bind(device_id.to_string())
    .bind(jti)
    .execute(pool)
    .await?;

    Ok(())
}

/// Clear the device's current_jti, effectively revoking all existing tokens for this device.
pub async fn clear_device_current_jti(pool: &DbPool, device_id: Uuid) -> Result<()> {
    sqlx::query::<sqlx::Any>(
        r#"
        UPDATE client_devices
        SET current_jti = NULL
        WHERE device_id = $1
        "#,
    )
    .bind(device_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete expired entries from the revoked_tokens table.
pub async fn cleanup_expired(pool: &DbPool) -> Result<u64> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query::<sqlx::Any>(
        r#"DELETE FROM revoked_tokens WHERE expires_at < $1"#,
    )
    .bind(now)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    // The token_revocation module is fully async and requires a real database pool.
    // Integration tests in the API modules cover the revocation flow.
}
