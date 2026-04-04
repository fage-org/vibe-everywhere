use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::Redirect,
    routing::{delete, get, post},
};
use hmac::{Hmac, Mac};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde_json::Value;
use sha2::Sha256;

use crate::{
    api::types::{ApiError, SuccessResponse, VendorPath},
    auth::AuthenticatedUser,
    context::AppContext,
    storage::db::{GithubTokenRecord, VendorTokenRecord, now_ms},
};

const SUPPORTED_VENDORS: &[&str] = &["openai", "anthropic", "gemini"];

pub fn routes() -> Router<AppContext> {
    Router::new()
        .route("/v1/connect/tokens", get(list_tokens))
        .route("/v1/connect/github/params", get(github_params))
        .route("/v1/connect/github/callback", get(github_callback))
        .route("/v1/connect/github/webhook", post(github_webhook))
        .route("/v1/connect/github", delete(disconnect_github))
        .route("/v1/connect/{vendor}/register", post(register_vendor))
        .route("/v1/connect/{vendor}/token", get(get_vendor_token))
        .route("/v1/connect/{vendor}", delete(delete_vendor))
}

async fn github_params(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let client_id = std::env::var("GITHUB_CLIENT_ID")
        .ok()
        .filter(|value| !value.is_empty());
    let redirect_uri = std::env::var("GITHUB_REDIRECT_URL")
        .ok()
        .filter(|value| !value.is_empty());
    match (client_id, redirect_uri) {
        (Some(client_id), Some(redirect_uri)) => {
            let state = ctx.auth().create_github_state_token(&user.user_id);
            let url = format!(
                "https://github.com/login/oauth/authorize?client_id={client_id}&redirect_uri={redirect_uri}&scope=read:user,user:email,read:org,codespace&state={state}"
            );
            Ok(Json(serde_json::json!({ "url": url })))
        }
        _ => Err(ApiError::bad_request("GitHub OAuth not configured")),
    }
}

#[derive(serde::Deserialize)]
struct GithubCallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct GithubAccessTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct GithubProfile {
    id: u64,
    login: String,
    #[serde(default)]
    avatar_url: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    bio: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

async fn github_callback(
    State(ctx): State<AppContext>,
    Query(query): Query<GithubCallbackQuery>,
) -> Redirect {
    let webapp_url = ctx.config().webapp_url.trim_end_matches('/').to_string();
    let Some(code) = query.code else {
        return redirect_to_webapp(&webapp_url, "error=missing_code");
    };
    let Some(state) = query.state else {
        return redirect_to_webapp(&webapp_url, "error=invalid_state");
    };
    let Some(user_id) = ctx.auth().verify_github_state_token(&state) else {
        return redirect_to_webapp(&webapp_url, "error=invalid_state");
    };

    let Some(client_id) = std::env::var("GITHUB_CLIENT_ID")
        .ok()
        .filter(|value| !value.is_empty())
    else {
        return redirect_to_webapp(&webapp_url, "error=server_config");
    };
    let Some(client_secret) = std::env::var("GITHUB_CLIENT_SECRET")
        .ok()
        .filter(|value| !value.is_empty())
    else {
        return redirect_to_webapp(&webapp_url, "error=server_config");
    };

    match complete_github_callback(&ctx, &user_id, &client_id, &client_secret, &code).await {
        Ok(login) => redirect_to_webapp(
            &webapp_url,
            &format!("github=connected&user={}", urlencoding::encode(&login)),
        ),
        Err(GithubCallbackError::Oauth(error)) => redirect_to_webapp(
            &webapp_url,
            &format!("error={}", urlencoding::encode(&error)),
        ),
        Err(GithubCallbackError::GithubUserFetchFailed) => {
            redirect_to_webapp(&webapp_url, "error=github_user_fetch_failed")
        }
        Err(GithubCallbackError::ServerError) => {
            redirect_to_webapp(&webapp_url, "error=server_error")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GithubCallbackError {
    GithubUserFetchFailed,
    ServerError,
    Oauth(String),
}

async fn complete_github_callback(
    ctx: &AppContext,
    user_id: &str,
    client_id: &str,
    client_secret: &str,
    code: &str,
) -> Result<String, GithubCallbackError> {
    let client = reqwest::Client::new();
    let token_response = client
        .post("https://github.com/login/oauth/access_token")
        .header(ACCEPT, "application/json")
        .header(CONTENT_TYPE, "application/json")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code,
        }))
        .send()
        .await
        .map_err(|_| GithubCallbackError::ServerError)?;
    let token_response: GithubAccessTokenResponse = token_response
        .json()
        .await
        .map_err(|_| GithubCallbackError::ServerError)?;
    if let Some(error) = token_response.error {
        return Err(GithubCallbackError::Oauth(error));
    }
    let Some(access_token) = token_response.access_token else {
        return Err(GithubCallbackError::ServerError);
    };

    let profile_response = client
        .get("https://api.github.com/user")
        .header(AUTHORIZATION, format!("Bearer {access_token}"))
        .header(ACCEPT, "application/vnd.github.v3+json")
        .header(USER_AGENT, "vibe-server")
        .send()
        .await
        .map_err(|_| GithubCallbackError::ServerError)?;
    if !profile_response.status().is_success() {
        return Err(GithubCallbackError::GithubUserFetchFailed);
    }
    let profile = profile_response
        .json::<GithubProfile>()
        .await
        .map_err(|_| GithubCallbackError::ServerError)?;
    let avatar = if profile.avatar_url.is_empty() {
        None
    } else {
        let avatar_response = client
            .get(&profile.avatar_url)
            .header(USER_AGENT, "vibe-server")
            .send()
            .await
            .map_err(|_| GithubCallbackError::ServerError)?;
        let avatar_bytes = avatar_response
            .bytes()
            .await
            .map_err(|_| GithubCallbackError::ServerError)?;
        let image_ref = ctx
            .files()
            .store_image(
                user_id,
                "avatars",
                "github",
                Some(&format!("image-url:{}", profile.avatar_url)),
                &avatar_bytes,
            )
            .await
            .map_err(|_| GithubCallbackError::ServerError)?;
        Some(serde_json::to_value(image_ref).map_err(|_| GithubCallbackError::ServerError)?)
    };
    let (first_name, last_name) = separate_name(profile.name.as_deref());
    let mut profile_json =
        serde_json::to_value(&profile).map_err(|_| GithubCallbackError::ServerError)?;
    if let Value::Object(ref mut object) = profile_json {
        for (key, value) in profile.extra {
            object.entry(key).or_insert(value);
        }
    }

    let profile_id = profile.id;
    let login = profile.login.clone();
    let avatar_for_update = avatar
        .as_ref()
        .and_then(|value| ctx.files().resolve_image_json(Some(value)));

    let displaced_accounts = ctx.db().write(|state| {
        let connected_elsewhere = state
            .accounts
            .values()
            .filter(|account| {
                account.id != user_id
                    && account
                        .github_profile
                        .as_ref()
                        .and_then(|value| value.get("id"))
                        .and_then(Value::as_u64)
                        == Some(profile_id)
            })
            .map(|account| account.id.clone())
            .collect::<Vec<_>>();
        for account_id in &connected_elsewhere {
            if let Some(account) = state.accounts.get_mut(account_id.as_str()) {
                account.github_profile = None;
                account.username = None;
            }
            state.github_tokens.remove(account_id);
            state
                .vendor_tokens
                .remove(&(account_id.clone(), "github".into()));
        }

        state.github_tokens.insert(
            user_id.to_string(),
            GithubTokenRecord {
                id: format!("ght_{}", uuid::Uuid::now_v7()),
                account_id: user_id.to_string(),
                github_user_id: profile_id.to_string(),
                token: access_token,
                created_at: now_ms(),
                updated_at: now_ms(),
            },
        );

        if let Some(account) = state.accounts.get_mut(user_id) {
            account.github_profile = Some(profile_json.clone());
            account.username = Some(login.clone());
            account.first_name = first_name.clone();
            account.last_name = last_name.clone();
            account.avatar = avatar.clone();
            account.updated_at = now_ms();
        }
        connected_elsewhere
    });
    for displaced_account_id in &displaced_accounts {
        ctx.events()
            .publish_account_profile_update(
                ctx.db(),
                displaced_account_id,
                Some(None),
                Some(None),
                None,
                None,
                None,
            )
            .map_err(|_| GithubCallbackError::ServerError)?;
    }
    ctx.events()
        .publish_account_profile_update(
            ctx.db(),
            user_id,
            Some(Some(profile_json)),
            Some(Some(login.clone())),
            Some(first_name),
            Some(last_name),
            Some(avatar_for_update),
        )
        .map_err(|_| GithubCallbackError::ServerError)?;

    Ok(login)
}

async fn github_webhook(headers: HeaderMap, body: Bytes) -> Result<Json<Value>, ApiError> {
    let secret = std::env::var("GITHUB_WEBHOOK_SECRET")
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApiError::internal("Webhooks not configured"))?;
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::internal("Internal server error"))?;
    verify_github_webhook(&secret, signature, &body)?;
    Ok(Json(serde_json::json!({ "received": true })))
}

fn github_webhook_signature(secret: &str, body: &[u8]) -> Result<String, ApiError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|_| ApiError::internal("Webhooks not configured"))?;
    mac.update(body);
    Ok(format!(
        "sha256={}",
        hex::encode(mac.finalize().into_bytes())
    ))
}

fn verify_github_webhook(secret: &str, signature: &str, body: &[u8]) -> Result<(), ApiError> {
    let expected = github_webhook_signature(secret, body)?;
    if signature != expected {
        return Err(ApiError::internal("Internal server error"));
    }
    Ok(())
}

async fn disconnect_github(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<SuccessResponse>, ApiError> {
    ctx.db().write(|state| {
        state.github_tokens.remove(&user.user_id);
        state
            .vendor_tokens
            .remove(&(user.user_id.clone(), "github".into()));
        if let Some(account) = state.accounts.get_mut(&user.user_id) {
            account.github_profile = None;
            account.username = None;
            account.updated_at = now_ms();
        }
    });
    ctx.events()
        .publish_account_profile_update(
            ctx.db(),
            &user.user_id,
            Some(None),
            Some(None),
            None,
            None,
            None,
        )
        .map_err(|_| ApiError::internal("Failed to disconnect GitHub account"))?;
    Ok(Json(SuccessResponse { success: true }))
}

async fn register_vendor(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<VendorPath>,
    Json(body): Json<Value>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let vendor = validate_vendor(&path.vendor)?;
    let token = body
        .get("token")
        .and_then(|value| value.as_str())
        .ok_or_else(|| ApiError::bad_request("token is required"))?
        .to_string();
    ctx.db().write(|state| {
        state.vendor_tokens.insert(
            (user.user_id.clone(), vendor.to_string()),
            VendorTokenRecord {
                id: format!("vt_{}", uuid::Uuid::now_v7()),
                account_id: user.user_id.clone(),
                vendor: vendor.to_string(),
                token,
                created_at: now_ms(),
                updated_at: now_ms(),
            },
        );
    });
    Ok(Json(SuccessResponse { success: true }))
}

async fn get_vendor_token(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<VendorPath>,
) -> Result<Json<Value>, ApiError> {
    let vendor = validate_vendor(&path.vendor)?;
    let token = ctx.db().read(|state| {
        state
            .vendor_tokens
            .get(&(user.user_id, vendor.to_string()))
            .map(|record| record.token.clone())
    });
    Ok(Json(serde_json::json!({ "token": token })))
}

async fn delete_vendor(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<VendorPath>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let vendor = validate_vendor(&path.vendor)?;
    ctx.db().write(|state| {
        state
            .vendor_tokens
            .remove(&(user.user_id, vendor.to_string()))
    });
    Ok(Json(SuccessResponse { success: true }))
}

async fn list_tokens(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<Value>, ApiError> {
    let tokens = ctx.db().read(|state| {
        state
            .vendor_tokens
            .values()
            .filter(|record| record.account_id == user.user_id)
            .filter(|record| validate_vendor(&record.vendor).is_ok())
            .map(|record| {
                serde_json::json!({
                    "vendor": record.vendor,
                    "token": record.token,
                })
            })
            .collect::<Vec<_>>()
    });
    Ok(Json(serde_json::json!({ "tokens": tokens })))
}

fn validate_vendor(vendor: &str) -> Result<&'static str, ApiError> {
    SUPPORTED_VENDORS
        .iter()
        .copied()
        .find(|candidate| *candidate == vendor)
        .ok_or_else(|| ApiError::bad_request("Unsupported vendor"))
}

fn separate_name(name: Option<&str>) -> (Option<String>, Option<String>) {
    let Some(name) = name.map(str::trim).filter(|value| !value.is_empty()) else {
        return (None, None);
    };
    let mut parts = name.split_whitespace();
    let first = parts.next().map(ToOwned::to_owned);
    let rest = parts.collect::<Vec<_>>();
    let last = (!rest.is_empty()).then(|| rest.join(" "));
    (first, last)
}

fn redirect_to_webapp(base_url: &str, query: &str) -> Redirect {
    Redirect::to(&format!("{}?{}", base_url.trim_end_matches('/'), query))
}

#[cfg(test)]
mod tests {
    use axum::{
        body::to_bytes,
        extract::{Query, State},
        response::IntoResponse,
    };
    use serde_json::Value;

    use crate::{config::Config, context::AppContext};

    use super::{GithubCallbackQuery, github_callback, redirect_to_webapp, verify_github_webhook};

    fn test_config() -> Config {
        Config {
            host: "127.0.0.1".parse().unwrap(),
            port: 3005,
            master_secret: "secret".into(),
            ios_up_to_date: ">=1.4.1".into(),
            android_up_to_date: ">=1.4.1".into(),
            ios_store_url: "ios-store".into(),
            android_store_url: "android-store".into(),
            webapp_url: "https://app.vibe.example/".into(),
        }
    }

    #[test]
    fn github_webhook_signature_mismatch_uses_happy_500_error_shape() {
        let response = verify_github_webhook("secret", "sha256=bad", br#"{"ping":true}"#)
            .unwrap_err()
            .into_response();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn github_webhook_signature_accepts_matching_digest() {
        let body = br#"{"ping":true}"#;
        let signature = super::github_webhook_signature("secret", body).unwrap();
        assert!(verify_github_webhook("secret", &signature, body).is_ok());
    }

    #[tokio::test]
    async fn github_webhook_error_body_stays_generic() {
        let response = verify_github_webhook("secret", "sha256=bad", br#"{"ping":true}"#)
            .unwrap_err()
            .into_response();
        let (_, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"], "Internal server error");
    }

    #[test]
    fn redirect_to_webapp_trims_trailing_slash() {
        let response =
            redirect_to_webapp("https://app.vibe.example/", "error=invalid_state").into_response();
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .and_then(|value| value.to_str().ok());
        assert_eq!(
            location,
            Some("https://app.vibe.example?error=invalid_state")
        );
    }

    #[tokio::test]
    async fn github_callback_uses_configured_vibe_webapp_url_for_missing_code() {
        let ctx = AppContext::new(test_config());
        let response = github_callback(
            State(ctx),
            Query(GithubCallbackQuery {
                code: None,
                state: Some("state".into()),
            }),
        )
        .await
        .into_response();
        let location = response
            .headers()
            .get(axum::http::header::LOCATION)
            .and_then(|value| value.to_str().ok());
        assert_eq!(
            location,
            Some("https://app.vibe.example?error=missing_code")
        );
    }
}
