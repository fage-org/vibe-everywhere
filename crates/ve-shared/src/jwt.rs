//! JWT Claims and Token Utilities
//!
//! JWT signing and verification for client and daemon authentication.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// JWT-specific errors
#[derive(Debug, Error)]
pub enum JwtError {
    #[error("JWT encoding error: {0}")]
    Encoding(String),

    #[error("JWT decoding error: {0}")]
    Decoding(String),

    #[error("Invalid token type")]
    InvalidType,

    #[error("Token expired")]
    Expired,
}

/// JWT token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Client,
    Daemon,
}

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject - device_id for client, host_id for daemon
    pub sub: String,
    /// Token type
    pub r#type: TokenType,
    /// Device name or host name
    pub name: String,
    /// Issued at
    pub iat: i64,
    /// Expiration
    pub exp: i64,
}

impl Claims {
    /// Create new claims for a client device
    pub fn for_client(device_id: Uuid, device_name: &str, expiration: Duration) -> Self {
        let now = Utc::now();
        Self {
            sub: device_id.to_string(),
            r#type: TokenType::Client,
            name: device_name.to_string(),
            iat: now.timestamp(),
            exp: (now + expiration).timestamp(),
        }
    }

    /// Create new claims for a daemon
    pub fn for_daemon(host_id: Uuid, host_name: &str, expiration: Duration) -> Self {
        let now = Utc::now();
        Self {
            sub: host_id.to_string(),
            r#type: TokenType::Daemon,
            name: host_name.to_string(),
            iat: now.timestamp(),
            exp: (now + expiration).timestamp(),
        }
    }

    /// Get the subject as UUID
    pub fn subject_uuid(&self) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&self.sub)
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// JWT manager for signing and verifying tokens
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration: Duration,
}

impl JwtManager {
    /// Create a new JWT manager with the given secret
    pub fn new(secret: &str, expiration: Duration) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiration,
        }
    }

    /// Sign and encode claims into a JWT token
    pub fn encode(&self, claims: &Claims) -> Result<String, JwtError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| JwtError::Encoding(e.to_string()))
    }

    /// Decode and verify a JWT token
    pub fn decode(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(
            token,
            &self.decoding_key,
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| JwtError::Decoding(e.to_string()))?;

        Ok(token_data.claims)
    }

    /// Create a token for a client device
    pub fn create_client_token(
        &self,
        device_id: Uuid,
        device_name: &str,
    ) -> Result<String, JwtError> {
        let claims = Claims::for_client(device_id, device_name, self.expiration);
        self.encode(&claims)
    }

    /// Create a token for a daemon
    pub fn create_daemon_token(&self, host_id: Uuid, host_name: &str) -> Result<String, JwtError> {
        let claims = Claims::for_daemon(host_id, host_name, self.expiration);
        self.encode(&claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_claims() {
        let device_id = Uuid::new_v4();
        let claims = Claims::for_client(device_id, "Test Device", Duration::hours(1));

        assert_eq!(claims.r#type, TokenType::Client);
        assert_eq!(claims.subject_uuid().unwrap(), device_id);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_daemon_claims() {
        let host_id = Uuid::new_v4();
        let claims = Claims::for_daemon(host_id, "Test Host", Duration::hours(1));

        assert_eq!(claims.r#type, TokenType::Daemon);
        assert_eq!(claims.subject_uuid().unwrap(), host_id);
    }

    #[test]
    fn test_jwt_roundtrip() {
        let manager = JwtManager::new("test_secret_key_for_testing", Duration::hours(24));
        let device_id = Uuid::new_v4();

        let token = manager
            .create_client_token(device_id, "Test Device")
            .unwrap();
        let claims = manager.decode(&token).unwrap();

        assert_eq!(claims.subject_uuid().unwrap(), device_id);
        assert_eq!(claims.r#type, TokenType::Client);
        assert_eq!(claims.name, "Test Device");
    }

    #[test]
    fn test_invalid_token() {
        let manager = JwtManager::new("test_secret_key", Duration::hours(24));

        let result = manager.decode("invalid_token");
        assert!(result.is_err());
    }
}
