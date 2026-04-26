//! Error Handling
//!
//! Unified error types and HTTP error response mappings.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

/// Main error type for the server
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("JWT error: {0}")]
    Jwt(#[from] ve_shared::jwt::JwtError),

    #[error("Time error: {0}")]
    Time(#[from] chrono::OutOfRange),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication required")]
    #[allow(dead_code)]
    Unauthorized,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    #[allow(dead_code)]
    TokenExpired,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Host not found")]
    HostNotFound,

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Pairing code expired")]
    PairCodeExpired,

    #[error("Pairing code already used")]
    PairCodeUsed,

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Session already archived")]
    SessionArchived,

    #[error("Internal UUID parse error: {0}")]
    InternalUuidParse(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Invalid JWT secret: {0}")]
    InvalidJwtSecret(String),

    #[error("Validation error: {0}")]
    Validation(#[from] crate::validation::ValidationError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, error_message): (StatusCode, String) = match &self {
            ServerError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ),
            ServerError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            ServerError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired".to_string()),
            ServerError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ServerError::HostNotFound => (StatusCode::NOT_FOUND, "Host not found".to_string()),
            ServerError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ServerError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            ServerError::PairCodeExpired => (StatusCode::GONE, "Pairing code expired".to_string()),
            ServerError::PairCodeUsed => (
                StatusCode::CONFLICT,
                "Pairing code already used".to_string(),
            ),
            ServerError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, msg.clone()),
            ServerError::SessionArchived => {
                (StatusCode::CONFLICT, "Session already archived".to_string())
            }
            ServerError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            ServerError::Internal(_)
            | ServerError::InternalUuidParse(_)
            | ServerError::Config(_)
            | ServerError::Jwt(_)
            | ServerError::Time(_)
            | ServerError::InvalidJwtSecret(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            ServerError::Json(_) => (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()),
            ServerError::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

        if status.is_server_error() {
            error!(error = %self, "Request failed");
        }

        let body = Json(json!({
            "error": error_message,
            "code": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

/// Result type alias for server operations
pub type Result<T> = std::result::Result<T, ServerError>;

/// API response wrapper for successful responses
#[allow(dead_code)]
#[derive(Debug, serde::Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[allow(dead_code)]
impl<T: serde::Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

impl<T: serde::Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn internal_errors_do_not_expose_details_in_response() {
        let response =
            ServerError::Internal("secret db failure details".to_string()).into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_text = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_text.contains("Internal server error"));
        assert!(!body_text.contains("secret db failure details"));
    }
}
