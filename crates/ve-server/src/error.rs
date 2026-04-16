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

    #[error("Invalid request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Pairing code expired")]
    PairCodeExpired,

    #[error("Pairing code already used")]
    PairCodeUsed,

    #[error("Session already archived")]
    SessionArchived,

    #[error("Permission already responded")]
    #[allow(dead_code)]
    PermissionResponded,

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
            ServerError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ServerError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            ServerError::PairCodeExpired => (StatusCode::GONE, "Pairing code expired".to_string()),
            ServerError::PairCodeUsed => (
                StatusCode::CONFLICT,
                "Pairing code already used".to_string(),
            ),
            ServerError::SessionArchived => {
                (StatusCode::CONFLICT, "Session already archived".to_string())
            }
            ServerError::PermissionResponded => (
                StatusCode::CONFLICT,
                "Permission already responded".to_string(),
            ),
            ServerError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            ServerError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ServerError::InternalUuidParse(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ServerError::Config(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Configuration error".to_string(),
            ),
            ServerError::Jwt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "JWT error".to_string()),
            ServerError::Time(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Time error".to_string()),
            ServerError::Json(_) => (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()),
            ServerError::InvalidJwtSecret(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ServerError::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };

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
