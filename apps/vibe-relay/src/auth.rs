use axum::http::{HeaderMap, Uri};
use url::form_urlencoded;
use vibe_core::{ActorIdentity, UserRole};

use crate::{ApiError, AppState};

#[cfg(test)]
use axum::extract::Request;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthenticatedDevice {
    pub(crate) device_id: String,
    pub(crate) actor: ActorIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AuthenticatedSubject {
    Control(ActorIdentity),
    Device(AuthenticatedDevice),
}

impl AuthenticatedSubject {
    pub(crate) fn actor(&self) -> &ActorIdentity {
        match self {
            Self::Control(actor) => actor,
            Self::Device(device) => &device.actor,
        }
    }
}

pub(crate) async fn require_control_actor(
    state: &AppState,
    headers: &HeaderMap,
    uri: Option<&Uri>,
) -> Result<ActorIdentity, ApiError> {
    let default_actor = state.config.default_actor();
    let Some(expected_token) = state.config.access_token.as_deref() else {
        return Ok(default_actor);
    };

    match request_access_token_from_parts(headers, uri.and_then(Uri::query)).as_deref() {
        Some(actual) if actual == expected_token => Ok(default_actor),
        _ => Err(ApiError::unauthorized(
            "auth_required",
            "Missing or invalid control-plane access token",
        )),
    }
}

pub(crate) async fn require_registration_actor(
    state: &AppState,
    headers: &HeaderMap,
    uri: Option<&Uri>,
) -> Result<ActorIdentity, ApiError> {
    let default_actor = state.config.default_actor();
    let control_token = state.config.access_token.as_deref();
    let enrollment_token = state.config.enrollment_token.as_deref();

    if control_token.is_none() && enrollment_token.is_none() {
        return Ok(default_actor);
    }

    let Some(actual_token) = request_access_token_from_parts(headers, uri.and_then(Uri::query))
    else {
        return Err(ApiError::unauthorized(
            "agent_enrollment_required",
            "Missing or invalid enrollment token",
        ));
    };

    if control_token == Some(actual_token.as_str())
        || enrollment_token == Some(actual_token.as_str())
    {
        Ok(default_actor)
    } else {
        Err(ApiError::unauthorized(
            "agent_enrollment_required",
            "Missing or invalid enrollment token",
        ))
    }
}

pub(crate) async fn require_device_auth(
    state: &AppState,
    headers: &HeaderMap,
    uri: Option<&Uri>,
) -> Result<AuthenticatedDevice, ApiError> {
    let Some(actual_token) = request_access_token_from_parts(headers, uri.and_then(Uri::query))
    else {
        return Err(ApiError::unauthorized(
            "device_auth_required",
            "Missing or invalid device credential",
        ));
    };

    let store = state.store.read().await;
    let Some(credential) = store
        .device_credentials
        .values()
        .find(|credential| credential.token == actual_token)
    else {
        return Err(ApiError::unauthorized(
            "device_auth_required",
            "Missing or invalid device credential",
        ));
    };

    let Some(device) = store.devices.get(&credential.device_id) else {
        return Err(ApiError::unauthorized(
            "device_auth_required",
            "Missing or invalid device credential",
        ));
    };

    Ok(AuthenticatedDevice {
        device_id: device.id.clone(),
        actor: device_actor(device),
    })
}

pub(crate) async fn authenticate_control_or_device(
    state: &AppState,
    headers: &HeaderMap,
    uri: Option<&Uri>,
) -> Result<AuthenticatedSubject, ApiError> {
    let default_actor = state.config.default_actor();
    let Some(actual_token) = request_access_token_from_parts(headers, uri.and_then(Uri::query))
    else {
        if state.config.access_token.is_none() {
            return Ok(AuthenticatedSubject::Control(default_actor));
        }

        return Err(ApiError::unauthorized(
            "auth_required",
            "Missing or invalid access token",
        ));
    };

    if state.config.access_token.as_deref() == Some(actual_token.as_str()) {
        return Ok(AuthenticatedSubject::Control(default_actor));
    }

    let store = state.store.read().await;
    let Some(credential) = store
        .device_credentials
        .values()
        .find(|credential| credential.token == actual_token)
    else {
        return Err(ApiError::unauthorized(
            "auth_required",
            "Missing or invalid access token",
        ));
    };

    let Some(device) = store.devices.get(&credential.device_id) else {
        return Err(ApiError::unauthorized(
            "auth_required",
            "Missing or invalid access token",
        ));
    };

    Ok(AuthenticatedSubject::Device(AuthenticatedDevice {
        device_id: device.id.clone(),
        actor: device_actor(device),
    }))
}

pub(crate) fn ensure_authenticated_device_matches(
    auth: &AuthenticatedDevice,
    device_id: &str,
) -> Result<(), ApiError> {
    if auth.device_id == device_id {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "device_forbidden",
            "The current device credential cannot access another device",
        ))
    }
}

pub(crate) fn issue_device_credential() -> String {
    format!(
        "vda_{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

#[cfg(test)]
pub(crate) fn request_access_token(request: &Request) -> Option<String> {
    request_access_token_from_parts(request.headers(), request.uri().query())
}

pub(crate) fn request_access_token_from_parts(
    headers: &HeaderMap,
    query: Option<&str>,
) -> Option<String> {
    if let Some(header_value) = headers
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

    query.and_then(query_access_token)
}

pub(crate) fn query_access_token(query: &str) -> Option<String> {
    form_urlencoded::parse(query.as_bytes())
        .find(|(key, _)| key == "access_token")
        .map(|(_, value)| value.into_owned())
        .filter(|value| !value.is_empty())
}

fn device_actor(device: &vibe_core::DeviceRecord) -> ActorIdentity {
    ActorIdentity {
        tenant_id: device.tenant_id.clone(),
        user_id: device.user_id.clone(),
        role: UserRole::Agent,
    }
}
