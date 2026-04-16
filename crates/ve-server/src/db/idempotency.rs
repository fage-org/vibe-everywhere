//! Idempotency Key Storage
//!
//! Provides idempotency key management for duplicate request protection.
//! Keys are stored with request hashes to detect request body changes,
//! and have TTL-based expiration for cleanup.

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{Result, ServerError};

/// Idempotency key record from database
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IdempotencyKeyRecord {
    /// The idempotency key (client-provided)
    pub key: String,
    /// SHA-256 hash of the original request body
    pub request_hash: Option<String>,
    /// Reference to the created resource (session_id, etc.)
    pub result_ref: String,
    /// Type of resource (default: "session")
    pub result_type: String,
    /// Creation timestamp
    pub created_at: String,
    /// Expiration timestamp
    pub expires_at: Option<String>,
}

/// Idempotency key store for managing duplicate request protection
pub struct IdempotencyKeyStore {
    pool: SqlitePool,
    default_ttl_secs: u64,
}

impl IdempotencyKeyStore {
    /// Create a new idempotency key store
    pub fn new(pool: SqlitePool, default_ttl_secs: u64) -> Self {
        Self {
            pool,
            default_ttl_secs,
        }
    }

    /// Compute SHA-256 hash of a value for request body validation
    pub fn compute_hash(value: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if an idempotency key exists and return the associated record
    ///
    /// Returns:
    /// - `Ok(Some(record))` if key exists (may be expired or valid)
    /// - `Ok(None)` if key doesn't exist
    pub async fn get(&self, key: &str) -> Result<Option<IdempotencyKeyRecord>> {
        let row = sqlx::query_as::<_, (String, Option<String>, String, String, String, Option<String>)>(
            r#"
            SELECT key, request_hash, session_id,
                   COALESCE(result_type, 'session') as result_type,
                   created_at, expires_at
            FROM idempotency_keys
            WHERE key = ?
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| IdempotencyKeyRecord {
            key: r.0,
            request_hash: r.1,
            result_ref: r.2,
            result_type: r.3,
            created_at: r.4,
            expires_at: r.5,
        }))
    }

    /// Check if an existing key matches the request hash
    ///
    /// Returns:
    /// - `Ok(true)` if hashes match (same request)
    /// - `Ok(false)` if hashes don't match (different request body)
    /// - `Ok(true)` if no hash stored (backward compatibility)
    pub fn verify_hash(&self, record: &IdempotencyKeyRecord, request_hash: &str) -> bool {
        match &record.request_hash {
            Some(stored_hash) => stored_hash == request_hash,
            None => true, // No hash stored, accept for backward compatibility
        }
    }

    /// Store a new idempotency key
    ///
    /// Returns the stored record on success
    pub async fn store(
        &self,
        key: &str,
        request_hash: &str,
        result_ref: &Uuid,
        result_type: &str,
    ) -> Result<IdempotencyKeyRecord> {
        let result_ref_str = result_ref.to_string();
        let expires_at = self.compute_expires_at();

        // Try to insert, handling the case where columns might not exist
        // (for databases that haven't run the supplemental migration)
        let result = sqlx::query(
            r#"
            INSERT INTO idempotency_keys (key, request_hash, session_id, result_type, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(key)
        .bind(request_hash)
        .bind(&result_ref_str)
        .bind(result_type)
        .bind(&expires_at)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                tracing::debug!(
                    key = %key,
                    result_ref = %result_ref_str,
                    result_type = %result_type,
                    expires_at = %expires_at,
                    "Stored idempotency key"
                );
                Ok(IdempotencyKeyRecord {
                    key: key.to_string(),
                    request_hash: Some(request_hash.to_string()),
                    result_ref: result_ref_str,
                    result_type: result_type.to_string(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    expires_at: Some(expires_at),
                })
            }
            Err(e) => {
                // Check if it's a duplicate key error
                if e.to_string().contains("UNIQUE constraint") || e.to_string().contains("PRIMARY KEY") {
                    // Key already exists, fetch and return it
                    tracing::warn!(key = %key, "Idempotency key already exists, returning existing");
                    self.get(key).await?.ok_or_else(|| {
                        ServerError::Internal("Idempotency key disappeared after conflict".into())
                    })
                } else {
                    Err(ServerError::Internal(format!(
                        "Failed to store idempotency key: {}",
                        e
                    )))
                }
            }
        }
    }

    /// Delete expired idempotency keys
    ///
    /// Returns the number of keys deleted
    #[allow(dead_code)]
    pub async fn delete_expired(&self) -> Result<usize> {
        let now = chrono::Utc::now().to_rfc3339();

        let result = sqlx::query(
            r#"
            DELETE FROM idempotency_keys
            WHERE expires_at IS NOT NULL AND expires_at < ?
            "#,
        )
        .bind(&now)
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected() as usize;
        if deleted > 0 {
            tracing::info!(deleted, "Cleaned up expired idempotency keys");
        }
        Ok(deleted)
    }

    /// Compute expiration timestamp
    fn compute_expires_at(&self) -> String {
        let expires = chrono::Utc::now() + chrono::Duration::seconds(self.default_ttl_secs as i64);
        expires.to_rfc3339()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_computation() {
        let hash1 = IdempotencyKeyStore::compute_hash("test content");
        let hash2 = IdempotencyKeyStore::compute_hash("test content");
        let hash3 = IdempotencyKeyStore::compute_hash("different content");

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
        // Different content should produce different hash
        assert_ne!(hash1, hash3);
        // Hash should be 64 hex characters (SHA-256)
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_hash_deterministic() {
        let content = r#"{"title":"Test Session","host_id":"host-001","workspace_id":"ws-001","initial_message":"Hello"}"#;
        let hash = IdempotencyKeyStore::compute_hash(content);

        // SHA-256 produces 64 hex characters
        assert_eq!(hash.len(), 64);

        // Same content should produce same hash
        let hash2 = IdempotencyKeyStore::compute_hash(content);
        assert_eq!(hash, hash2);
    }
}
