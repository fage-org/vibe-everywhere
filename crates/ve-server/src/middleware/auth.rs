//! Authentication Middleware
//!
//! JWT authentication middleware for HTTP API endpoints.
//! Protects routes that require authentication while allowing public routes.

use axum::{
    extract::{Request, State},
    http::{header, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use ve_shared::jwt::JwtManager;

/// Public routes that don't require authentication
const PUBLIC_ROUTES: &[&str] = &[
    "/healthz",
    "/api/auth/register-device",
    "/api/auth/daemon-hello",
    "/api/auth/pair",
    "/ws/client",
    "/ws/daemon",
];

/// Check if a path matches a public route pattern
fn is_public_route(uri: &Uri) -> bool {
    let path = uri.path();

    // Exact matches
    if PUBLIC_ROUTES.contains(&path) {
        return true;
    }

    // WebSocket routes (start with /ws/)
    if path.starts_with("/ws/") {
        return true;
    }

    false
}

/// Authentication middleware
///
/// Extracts and validates JWT token from Authorization header.
/// Injects Claims into request extensions for use by handlers.
pub async fn auth_middleware(
    State(jwt_manager): State<Arc<JwtManager>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Skip authentication for public routes
    if is_public_route(request.uri()) {
        return Ok(next.run(request).await);
    }

    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    // Extract Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidFormat)?;

    // Validate token and extract claims
    let claims = jwt_manager.decode(token).map_err(|_| AuthError::InvalidToken)?;

    // Check if token is expired
    if claims.is_expired() {
        return Err(AuthError::Expired);
    }

    // Inject claims into request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Authentication error types
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidFormat,
    InvalidToken,
    Expired,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidFormat => (StatusCode::UNAUTHORIZED, "Invalid authorization header format"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::Expired => (StatusCode::UNAUTHORIZED, "Token expired"),
        };

        let body = axum::Json(serde_json::json!({
            "error": message,
            "code": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_public_route_health() {
        let uri: Uri = "/healthz".parse().unwrap();
        assert!(is_public_route(&uri));
    }

    #[test]
    fn test_is_public_route_auth() {
        let uri: Uri = "/api/auth/register-device".parse().unwrap();
        assert!(is_public_route(&uri));
    }

    #[test]
    fn test_is_public_route_protected() {
        let uri: Uri = "/api/sessions".parse().unwrap();
        assert!(!is_public_route(&uri));
    }

    #[test]
    fn test_is_public_route_ws() {
        let uri: Uri = "/ws/client".parse().unwrap();
        assert!(is_public_route(&uri));
    }

    #[test]
    fn test_is_public_route_nested_protected() {
        let uri: Uri = "/api/sessions/123".parse().unwrap();
        assert!(!is_public_route(&uri));
    }
}