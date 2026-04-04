use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
};

use crate::{api::types::ApiError, context::AppContext};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub token: String,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppContext: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx = AppContext::from_ref(state);
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| ApiError::unauthorized("Missing authorization header"))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::unauthorized("Missing authorization header"))?
            .to_string();
        let claims = ctx
            .auth()
            .verify_token(&token)
            .ok_or_else(|| ApiError::unauthorized("Invalid token"))?;

        if claims.user_id.is_empty() {
            return Err(ApiError::from(StatusCode::UNAUTHORIZED));
        }

        Ok(Self {
            user_id: claims.user_id,
            token,
        })
    }
}

impl From<StatusCode> for ApiError {
    fn from(status: StatusCode) -> Self {
        ApiError::unauthorized(status.as_str())
    }
}
