pub mod account;
pub mod middleware;
pub mod token;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Deserialize;
use thiserror::Error;

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};

use crate::{
    api::types::{
        AccountAuthRequestBody, ApiError, AuthRequestBody, AuthResponseBody, AuthStatusQuery,
        ChallengeAuthBody,
    },
    context::AppContext,
};

pub use middleware::AuthenticatedUser;
pub use token::{AuthError, AuthService, TokenClaims};

use self::account::{
    AuthRequestResponse, AuthRequestStatusResponse, SuccessResponse, TokenResponse,
};

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SocketClientType {
    SessionScoped,
    #[default]
    UserScoped,
    MachineScoped,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketAuthPayload {
    pub token: Option<String>,
    pub client_type: Option<SocketClientType>,
    pub session_id: Option<String>,
    pub machine_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SocketConnectionAuth {
    pub user_id: String,
    pub token: String,
    pub client_type: SocketClientType,
    pub session_id: Option<String>,
    pub machine_id: Option<String>,
}

#[derive(Debug, Error)]
pub enum SocketAuthError {
    #[error("Missing authentication token")]
    MissingToken,
    #[error("Invalid authentication token")]
    InvalidToken,
    #[error("Session ID required for session-scoped clients")]
    MissingSessionId,
    #[error("Machine ID required for machine-scoped clients")]
    MissingMachineId,
    #[error("Invalid socket auth payload")]
    InvalidPayload,
}

pub fn routes() -> Router<AppContext> {
    Router::new()
        .route("/v1/auth", post(post_auth))
        .route("/v1/auth/request", post(post_terminal_auth_request))
        .route(
            "/v1/auth/request/status",
            get(get_terminal_auth_request_status),
        )
        .route("/v1/auth/response", post(post_terminal_auth_response))
        .route("/v1/auth/account/request", post(post_account_auth_request))
        .route(
            "/v1/auth/account/response",
            post(post_account_auth_response),
        )
}

pub async fn validate_socket_auth(
    payload: Result<SocketAuthPayload, socketioxide::ParserError>,
    ctx: &AppContext,
) -> Result<SocketConnectionAuth, SocketAuthError> {
    let SocketAuthPayload {
        token,
        client_type,
        session_id,
        machine_id,
    } = payload.map_err(|_| SocketAuthError::InvalidPayload)?;
    let token = token
        .as_deref()
        .filter(|token| !token.is_empty())
        .ok_or(SocketAuthError::MissingToken)?;
    let client_type = client_type.unwrap_or_default();
    let session_id = session_id.filter(|session_id| !session_id.is_empty());
    let machine_id = machine_id.filter(|machine_id| !machine_id.is_empty());
    if matches!(client_type, SocketClientType::SessionScoped) && session_id.is_none() {
        return Err(SocketAuthError::MissingSessionId);
    }
    if matches!(client_type, SocketClientType::MachineScoped) && machine_id.is_none() {
        return Err(SocketAuthError::MissingMachineId);
    }
    let claims = ctx
        .auth()
        .verify_token(token)
        .ok_or(SocketAuthError::InvalidToken)?;

    Ok(SocketConnectionAuth {
        user_id: claims.user_id,
        token: token.to_string(),
        client_type,
        session_id,
        machine_id,
    })
}

async fn post_auth(
    State(ctx): State<AppContext>,
    Json(body): Json<ChallengeAuthBody>,
) -> Result<Json<TokenResponse>, ApiError> {
    let token = ctx
        .auth()
        .challenge_authenticate(&body.public_key, &body.challenge, &body.signature)
        .map_err(|error| match error {
            token::AuthError::InvalidPublicKey => ApiError::unauthorized("Invalid public key"),
            token::AuthError::InvalidSignature => ApiError::unauthorized("Invalid signature"),
            token::AuthError::RequestNotFound => ApiError::not_found("Request not found"),
        })?;

    Ok(Json(TokenResponse {
        success: true,
        token,
    }))
}

async fn post_terminal_auth_request(
    State(ctx): State<AppContext>,
    Json(body): Json<AuthRequestBody>,
) -> Result<Json<AuthRequestResponse>, ApiError> {
    let public_key = decode_box_public_key(&body.public_key)?;
    let record = ctx
        .db()
        .put_terminal_auth_request(&hex::encode(public_key), body.supports_v2.unwrap_or(false));
    if let (Some(response), Some(account_id)) = (record.response, record.response_account_id) {
        let token = ctx.auth().create_token(
            &account_id,
            Some(serde_json::json!({ "session": record.id })),
        );
        return Ok(Json(AuthRequestResponse::Authorized { token, response }));
    }
    Ok(Json(AuthRequestResponse::Requested))
}

async fn get_terminal_auth_request_status(
    State(ctx): State<AppContext>,
    Query(query): Query<AuthStatusQuery>,
) -> Result<Json<AuthRequestStatusResponse>, ApiError> {
    let Some(public_key) = decode_box_public_key_lossy(&query.public_key) else {
        return Ok(Json(AuthRequestStatusResponse {
            status: "not_found",
            supports_v2: false,
        }));
    };
    let Some(record) = ctx.db().get_terminal_auth_request(&hex::encode(public_key)) else {
        return Ok(Json(AuthRequestStatusResponse {
            status: "not_found",
            supports_v2: false,
        }));
    };

    if record.response.is_some() && record.response_account_id.is_some() {
        return Ok(Json(AuthRequestStatusResponse {
            status: "authorized",
            supports_v2: false,
        }));
    }

    Ok(Json(AuthRequestStatusResponse {
        status: "pending",
        supports_v2: record.supports_v2,
    }))
}

async fn post_terminal_auth_response(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<AuthResponseBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let public_key = decode_box_public_key(&body.public_key)?;
    let updated = ctx.db().authorize_terminal_auth_request(
        &hex::encode(public_key),
        &body.response,
        &user.user_id,
    );
    if !updated {
        return Err(ApiError::not_found("Request not found"));
    }
    Ok(Json(SuccessResponse { success: true }))
}

async fn post_account_auth_request(
    State(ctx): State<AppContext>,
    Json(body): Json<AccountAuthRequestBody>,
) -> Result<Json<AuthRequestResponse>, ApiError> {
    let public_key = decode_box_public_key(&body.public_key)?;
    let record = ctx.db().put_account_auth_request(&hex::encode(public_key));
    if let (Some(response), Some(account_id)) = (record.response, record.response_account_id) {
        let token = ctx.auth().create_token(&account_id, None);
        return Ok(Json(AuthRequestResponse::Authorized { token, response }));
    }
    Ok(Json(AuthRequestResponse::Requested))
}

async fn post_account_auth_response(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<AuthResponseBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let public_key = decode_box_public_key(&body.public_key)?;
    let updated = ctx.db().authorize_account_auth_request(
        &hex::encode(public_key),
        &body.response,
        &user.user_id,
    );
    if !updated {
        return Err(ApiError::not_found("Request not found"));
    }
    Ok(Json(SuccessResponse { success: true }))
}

fn decode_box_public_key(input: &str) -> Result<[u8; 32], ApiError> {
    decode_box_public_key_lossy(input).ok_or_else(|| ApiError::unauthorized("Invalid public key"))
}

fn decode_box_public_key_lossy(input: &str) -> Option<[u8; 32]> {
    let decoded = STANDARD.decode(input).ok()?;
    if decoded.len() != 32 {
        return None;
    }
    decoded.try_into().ok()
}
