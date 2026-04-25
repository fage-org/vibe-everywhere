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
use ve_shared::jwt::{JwtManager, TokenType};

use crate::state::AppState;
use crate::token_revocation;

/// Public routes that don't require authentication
const PUBLIC_ROUTES: &[&str] = &[
    "/healthz",
    "/api/auth/register-device",
    "/api/auth/daemon-hello",
    "/api/auth/pairing-status",
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

fn route_allows_token_type(uri: &Uri, token_type: &TokenType) -> bool {
    match uri.path() {
        "/api/auth/pair" => matches!(token_type, TokenType::ClientBootstrap),
        _ => matches!(token_type, TokenType::Client),
    }
}

/// Authentication middleware
///
/// Extracts and validates JWT token from Authorization header.
/// Injects Claims into request extensions for use by handlers.
///
/// Requires a tuple `(Arc<AppState>, Arc<JwtManager>)` as the injected state.
pub async fn auth_middleware(
    State((state, jwt_manager)): State<(Arc<AppState>, Arc<JwtManager>)>,
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
    let claims = jwt_manager
        .decode(token)
        .map_err(|_| AuthError::InvalidToken)?;

    // Check if token is expired
    if claims.is_expired() {
        return Err(AuthError::Expired);
    }

    if !route_allows_token_type(request.uri(), &claims.r#type) {
        return Err(AuthError::InvalidTokenType);
    }

    // Check token revocation for Client-type tokens
    if claims.r#type == TokenType::Client {
        if let Ok(device_id) = claims.subject_uuid() {
            if token_revocation::jti_matches_device(&state.db, device_id, &claims.jti)
                .await
                .unwrap_or(true)
            {
                let is_revoked = token_revocation::is_revoked(&state.db, &claims.jti)
                    .await
                    .unwrap_or(false);
                if is_revoked {
                    return Err(AuthError::TokenRevoked);
                }
            }
        }
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
    InvalidTokenType,
    Expired,
    TokenRevoked,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidFormat => (
                StatusCode::UNAUTHORIZED,
                "Invalid authorization header format",
            ),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::InvalidTokenType => (
                StatusCode::UNAUTHORIZED,
                "Token is not allowed for this route",
            ),
            AuthError::Expired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::TokenRevoked => (StatusCode::UNAUTHORIZED, "Token revoked"),
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

    #[test]
    fn test_route_allows_bootstrap_token_for_pair() {
        let uri: Uri = "/api/auth/pair".parse().unwrap();
        assert!(route_allows_token_type(&uri, &TokenType::ClientBootstrap));
        assert!(!route_allows_token_type(&uri, &TokenType::Client));
        assert!(!route_allows_token_type(&uri, &TokenType::Daemon));
    }

    #[test]
    fn test_route_allows_only_client_token_for_protected_routes() {
        let uri: Uri = "/api/sessions".parse().unwrap();
        assert!(route_allows_token_type(&uri, &TokenType::Client));
        assert!(!route_allows_token_type(&uri, &TokenType::ClientBootstrap));
        assert!(!route_allows_token_type(&uri, &TokenType::Daemon));
    }
}
