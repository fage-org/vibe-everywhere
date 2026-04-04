use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

use crate::storage::{
    db::Database,
    redis::{BearerTokenRecord, RedisStore},
};

#[derive(Debug, Clone)]
pub struct TokenClaims {
    pub user_id: String,
    pub extras: Option<Value>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Request not found")]
    RequestNotFound,
}

#[derive(Clone)]
pub struct AuthService {
    db: Database,
    redis: RedisStore,
    master_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedTokenPayload {
    user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extras: Option<Value>,
}

impl AuthService {
    pub fn new(db: Database, redis: RedisStore, master_secret: String) -> Self {
        Self {
            db,
            redis,
            master_secret,
        }
    }

    pub fn challenge_authenticate(
        &self,
        public_key_b64: &str,
        challenge_b64: &str,
        signature_b64: &str,
    ) -> Result<String, AuthError> {
        let public_key = decode_fixed(public_key_b64, 32).ok_or(AuthError::InvalidPublicKey)?;
        let challenge = STANDARD
            .decode(challenge_b64)
            .map_err(|_| AuthError::InvalidSignature)?;
        let signature = decode_fixed(signature_b64, 64).ok_or(AuthError::InvalidSignature)?;

        let verifying_key =
            VerifyingKey::from_bytes(&public_key).map_err(|_| AuthError::InvalidPublicKey)?;
        let signature = Signature::from_bytes(&signature);
        verifying_key
            .verify(&challenge, &signature)
            .map_err(|_| AuthError::InvalidSignature)?;

        let account = self
            .db
            .upsert_account_by_public_key(&hex::encode(public_key));
        Ok(self.create_token(&account.id, None))
    }

    pub fn create_token(&self, user_id: &str, extras: Option<Value>) -> String {
        let payload = SignedTokenPayload {
            user: user_id.to_string(),
            extras: extras.clone(),
        };
        let payload_raw = serde_json::to_vec(&payload).expect("token payload serializable");
        let payload_encoded = URL_SAFE_NO_PAD.encode(payload_raw);
        let signature = self.sign(payload_encoded.as_bytes());
        let token = format!("v1.{payload_encoded}.{}", URL_SAFE_NO_PAD.encode(signature));
        self.redis.cache_bearer_token(
            &token,
            BearerTokenRecord {
                user_id: user_id.to_string(),
                extras,
            },
        );
        token
    }

    pub fn verify_token(&self, token: &str) -> Option<TokenClaims> {
        if let Some(record) = self.redis.get_bearer_token(token) {
            return Some(TokenClaims {
                user_id: record.user_id,
                extras: record.extras,
            });
        }

        let (version, rest) = token.split_once('.')?;
        let (payload_encoded, signature_encoded) = rest.split_once('.')?;
        if version != "v1" {
            return None;
        }

        let signature = URL_SAFE_NO_PAD.decode(signature_encoded).ok()?;
        let expected = self.sign(payload_encoded.as_bytes());
        if signature.as_slice().ct_eq(expected.as_slice()).unwrap_u8() != 1 {
            return None;
        }

        let payload_raw = URL_SAFE_NO_PAD.decode(payload_encoded).ok()?;
        let payload: SignedTokenPayload = serde_json::from_slice(&payload_raw).ok()?;
        self.redis.cache_bearer_token(
            token,
            BearerTokenRecord {
                user_id: payload.user.clone(),
                extras: payload.extras.clone(),
            },
        );
        Some(TokenClaims {
            user_id: payload.user,
            extras: payload.extras,
        })
    }

    pub fn create_github_state_token(&self, user_id: &str) -> String {
        let state = format!("gh_{}", uuid::Uuid::now_v7());
        self.redis.set_json(
            &format!("github-state:{state}"),
            serde_json::json!({ "userId": user_id }),
            Some(5 * 60 * 1000),
        );
        state
    }

    pub fn verify_github_state_token(&self, state: &str) -> Option<String> {
        self.redis
            .get_json(&format!("github-state:{state}"))?
            .get("userId")?
            .as_str()
            .map(ToOwned::to_owned)
    }

    fn sign(&self, data: &[u8]) -> Vec<u8> {
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.master_secret.as_bytes()).expect("valid hmac key");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }
}

fn decode_fixed<const N: usize>(input: &str, expected_len: usize) -> Option<[u8; N]> {
    let decoded = STANDARD.decode(input).ok()?;
    if decoded.len() != expected_len {
        return None;
    }
    decoded.try_into().ok()
}

#[cfg(test)]
mod tests {
    use super::AuthService;
    use crate::storage::{db::Database, redis::RedisStore};

    fn auth(secret: &str) -> AuthService {
        AuthService::new(
            Database::default(),
            RedisStore::default(),
            secret.to_string(),
        )
    }

    #[test]
    fn signed_tokens_verify_across_service_instances() {
        let token = auth("secret").create_token("acct", Some(serde_json::json!({"role": "user"})));
        let claims = auth("secret").verify_token(&token).unwrap();
        assert_eq!(claims.user_id, "acct");
        assert_eq!(claims.extras.unwrap()["role"], "user");
    }

    #[test]
    fn tampered_tokens_are_rejected() {
        let token = auth("secret").create_token("acct", None);
        let tampered = format!("{token}x");
        assert!(auth("secret").verify_token(&tampered).is_none());
    }
}
