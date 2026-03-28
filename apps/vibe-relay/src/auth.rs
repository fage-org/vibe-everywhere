use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method},
    middleware::Next,
    response::Response,
};
use url::form_urlencoded;
use vibe_core::{ActorIdentity, UserRole};

use crate::{ApiError, AppState};

pub(crate) async fn require_relay_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    if request.method() == Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    let Some(expected_token) = state.config.access_token.as_deref() else {
        return Ok(next.run(request).await);
    };

    if request_access_token(&request).as_deref() != Some(expected_token) {
        return Err(ApiError::unauthorized(
            "auth_required",
            "Missing or invalid access token",
        ));
    }

    Ok(next.run(request).await)
}

pub(crate) fn request_access_token(request: &Request) -> Option<String> {
    if let Some(header_value) = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        let token = header_value
            .strip_prefix("Bearer ")
            .or_else(|| header_value.strip_prefix("bearer "))
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(token) = token {
            return Some(token.to_string());
        }
    }

    request.uri().query().and_then(query_access_token)
}

pub(crate) fn query_access_token(query: &str) -> Option<String> {
    form_urlencoded::parse(query.as_bytes())
        .find(|(key, _)| key == "access_token")
        .map(|(_, value)| value.into_owned())
        .filter(|value| !value.is_empty())
}

pub(crate) fn actor_from_headers(state: &AppState, headers: &HeaderMap) -> ActorIdentity {
    let default_actor = state.config.default_actor();

    ActorIdentity {
        tenant_id: read_actor_header(headers, "x-vibe-tenant-id")
            .unwrap_or(default_actor.tenant_id),
        user_id: read_actor_header(headers, "x-vibe-user-id").unwrap_or(default_actor.user_id),
        role: read_actor_header(headers, "x-vibe-user-role")
            .and_then(|value| parse_user_role(&value))
            .unwrap_or(default_actor.role),
    }
}

fn read_actor_header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn parse_user_role(value: &str) -> Option<UserRole> {
    match value.trim().to_lowercase().as_str() {
        "owner" => Some(UserRole::Owner),
        "admin" => Some(UserRole::Admin),
        "member" => Some(UserRole::Member),
        "viewer" => Some(UserRole::Viewer),
        "agent" => Some(UserRole::Agent),
        _ => None,
    }
}
