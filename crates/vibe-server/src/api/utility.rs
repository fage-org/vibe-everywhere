use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use base64::Engine as _;
use hmac::{Hmac, Mac};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use sha2::Sha256;
use vibe_wire::{VoiceTokenAllowed, VoiceTokenDenied, VoiceTokenDeniedReason, VoiceTokenResponse};

use crate::{
    api::types::{
        ApiError, KvBulkGetBody, KvBulkGetResponse, KvEntry, KvListQuery, KvListResponse,
        KvMutateBody, KvMutateConflict, KvMutateConflictResponse, KvMutateResult,
        KvMutateSuccessResponse, KvPath, PushTokenBody, PushTokenItem, PushTokenListResponse,
        SuccessResponse, UpdatePushTokenPath, VersionCheckBody, VoiceTokenBody,
    },
    auth::AuthenticatedUser,
    context::AppContext,
    storage::db::{KvRecord, now_ms},
    version::check_update_url,
};

const VOICE_FREE_LIMIT_SECONDS: u64 = 3_600;

pub fn routes() -> Router<AppContext> {
    Router::new()
        .route("/v1/kv/{key}", get(get_kv))
        .route("/v1/kv", get(list_kv).post(mutate_kv))
        .route("/v1/kv/bulk", post(bulk_get_kv))
        .route(
            "/v1/push-tokens",
            post(create_push_token).get(list_push_tokens),
        )
        .route("/v1/push-tokens/{token}", delete(delete_push_token))
        .route("/v1/version", post(version))
        .route("/v1/voice/token", post(voice_token))
}

async fn get_kv(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<KvPath>,
) -> Result<Json<KvEntry>, ApiError> {
    let key = (user.user_id, path.key);
    let record = ctx
        .db()
        .read(|state| state.kv.get(&key).cloned())
        .filter(|record| record.value.is_some())
        .ok_or_else(|| ApiError::not_found("Key not found"))?;
    Ok(Json(KvEntry::from_record(&record)))
}

async fn list_kv(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Query(query): Query<KvListQuery>,
) -> Result<Json<KvListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(100);
    if !(1..=1000).contains(&limit) {
        return Err(ApiError::bad_request("limit must be between 1 and 1000"));
    }
    let mut items = ctx.db().read(|state| {
        state
            .kv
            .values()
            .filter(|record| record.account_id == user.user_id)
            .filter(|record| record.value.is_some())
            .filter(|record| {
                query
                    .prefix
                    .as_deref()
                    .is_none_or(|prefix| record.key.starts_with(prefix))
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    items.sort_by(|a, b| a.key.cmp(&b.key));
    items.truncate(limit);
    Ok(Json(KvListResponse {
        items: items.iter().map(KvEntry::from_record).collect(),
    }))
}

async fn bulk_get_kv(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<KvBulkGetBody>,
) -> Result<Json<KvBulkGetResponse>, ApiError> {
    if body.keys.is_empty() || body.keys.len() > 100 {
        return Err(ApiError::bad_request("keys must contain 1..100 items"));
    }
    let values = ctx.db().read(|state| {
        body.keys
            .iter()
            .filter_map(|key| state.kv.get(&(user.user_id.clone(), key.clone())))
            .filter(|record| record.value.is_some())
            .map(KvEntry::from_record)
            .collect::<Vec<_>>()
    });
    Ok(Json(KvBulkGetResponse { values }))
}

async fn mutate_kv(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<KvMutateBody>,
) -> Result<impl IntoResponse, ApiError> {
    if body.mutations.is_empty() || body.mutations.len() > 100 {
        return Err(ApiError::bad_request("mutations must contain 1..100 items"));
    }
    let mut conflicts = Vec::new();
    let mut results = Vec::new();
    let mut changes = Vec::new();
    ctx.db().write(|state| {
        for mutation in &body.mutations {
            let key = (user.user_id.clone(), mutation.key.clone());
            let existing = state.kv.get(&key);
            let current_version = existing.map(|record| record.version as i64).unwrap_or(-1);
            if mutation.version != current_version {
                conflicts.push(KvMutateConflict {
                    key: mutation.key.clone(),
                    error: "version-mismatch".into(),
                    version: existing.map(|record| record.version).unwrap_or(0),
                    value: existing.and_then(|record| record.value.clone()),
                });
            }
        }
        if !conflicts.is_empty() {
            return;
        }
        for mutation in body.mutations {
            let key = (user.user_id.clone(), mutation.key.clone());
            match state.kv.get_mut(&key) {
                Some(current) => {
                    current.value = mutation.value;
                    current.version += 1;
                    current.updated_at = now_ms();
                    changes.push(crate::events::socket_updates::KvBatchChange {
                        key: current.key.clone(),
                        value: current.value.clone(),
                        version: current.version,
                    });
                    results.push(KvMutateResult {
                        key: current.key.clone(),
                        version: current.version,
                    });
                }
                None => {
                    let record = KvRecord {
                        account_id: user.user_id.clone(),
                        key: mutation.key.clone(),
                        value: mutation.value,
                        version: 0,
                        created_at: now_ms(),
                        updated_at: now_ms(),
                    };
                    changes.push(crate::events::socket_updates::KvBatchChange {
                        key: record.key.clone(),
                        value: record.value.clone(),
                        version: record.version,
                    });
                    state.kv.insert(key, record);
                    results.push(KvMutateResult {
                        key: mutation.key,
                        version: 0,
                    });
                }
            }
        }
    });
    if !conflicts.is_empty() {
        return Ok((
            StatusCode::CONFLICT,
            Json(
                serde_json::to_value(KvMutateConflictResponse {
                    success: false,
                    errors: conflicts,
                })
                .unwrap(),
            ),
        ));
    }
    ctx.events()
        .publish_kv_batch_update(ctx.db(), &user.user_id, changes)
        .map_err(|_| ApiError::internal("Failed to mutate values"))?;
    Ok((
        StatusCode::OK,
        Json(
            serde_json::to_value(KvMutateSuccessResponse {
                success: true,
                results,
            })
            .unwrap(),
        ),
    ))
}

async fn create_push_token(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<PushTokenBody>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let user_id = user.user_id;
    let token = body.token;
    ctx.db().write(|state| {
        let key = (user_id.clone(), token.clone());
        if let Some(existing) = state.push_tokens.get_mut(&key) {
            existing.updated_at = now_ms();
            return;
        }
        state.push_tokens.insert(
            key,
            crate::storage::db::PushTokenRecord {
                id: format!("pt_{}", uuid::Uuid::now_v7()),
                account_id: user_id,
                token,
                created_at: now_ms(),
                updated_at: now_ms(),
            },
        );
    });
    Ok(Json(SuccessResponse { success: true }))
}

async fn list_push_tokens(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<PushTokenListResponse>, ApiError> {
    let mut tokens = ctx.db().read(|state| {
        state
            .push_tokens
            .values()
            .filter(|record| record.account_id == user.user_id)
            .cloned()
            .collect::<Vec<_>>()
    });
    tokens.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(Json(PushTokenListResponse {
        tokens: tokens.iter().map(PushTokenItem::from_record).collect(),
    }))
}

async fn delete_push_token(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<UpdatePushTokenPath>,
) -> Result<Json<SuccessResponse>, ApiError> {
    ctx.db()
        .write(|state| state.push_tokens.remove(&(user.user_id, path.token)));
    Ok(Json(SuccessResponse { success: true }))
}

async fn version(
    State(ctx): State<AppContext>,
    Json(body): Json<VersionCheckBody>,
) -> Json<crate::version::VersionCheckResponse> {
    Json(check_update_url(
        ctx.config(),
        &body.platform,
        &body.version,
    ))
}

async fn voice_token(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<VoiceTokenBody>,
) -> Result<Json<VoiceTokenResponse>, ApiError> {
    let elevenlabs_api_key = std::env::var("ELEVENLABS_API_KEY")
        .map_err(|_| ApiError::internal("Voice service not configured"))?;
    if elevenlabs_api_key.is_empty() {
        return Err(ApiError::internal("Voice service not configured"));
    }
    let eleven_user_id = derive_eleven_user_id(&ctx, &user.user_id)?;
    let used_seconds = get_used_voice_seconds(&elevenlabs_api_key, &eleven_user_id).await?;

    if used_seconds >= VOICE_FREE_LIMIT_SECONDS && !has_active_subscription(&user.user_id).await {
        return Ok(Json(VoiceTokenResponse::Denied(VoiceTokenDenied {
            reason: VoiceTokenDeniedReason::VoiceLimitReached,
            used_seconds,
            limit_seconds: VOICE_FREE_LIMIT_SECONDS,
            agent_id: body.agent_id,
        })));
    }

    let token = fetch_voice_token(&elevenlabs_api_key, &body.agent_id).await?;
    Ok(Json(VoiceTokenResponse::Allowed(VoiceTokenAllowed {
        agent_id: body.agent_id,
        token,
        eleven_user_id,
        used_seconds,
        limit_seconds: VOICE_FREE_LIMIT_SECONDS,
    })))
}

fn derive_eleven_user_id(ctx: &AppContext, user_id: &str) -> Result<String, ApiError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(ctx.config().master_secret.as_bytes())
        .map_err(|_| ApiError::internal("Failed to get voice token"))?;
    mac.update(user_id.as_bytes());
    let digest = mac.finalize().into_bytes();
    Ok(format!(
        "u_{}",
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest)
    ))
}

async fn get_used_voice_seconds(api_key: &str, eleven_user_id: &str) -> Result<u64, ApiError> {
    #[derive(serde::Deserialize)]
    struct Conversation {
        call_duration_secs: Option<u64>,
    }

    #[derive(serde::Deserialize)]
    struct ConversationsResponse {
        conversations: Option<Vec<Conversation>>,
    }

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://api.elevenlabs.io/v1/convai/conversations?user_id={eleven_user_id}&page_size=100"
        ))
        .header("xi-api-key", api_key)
        .header(ACCEPT, "application/json")
        .send()
        .await
        .map_err(|_| ApiError::internal("Failed to check voice usage"))?;
    if !response.status().is_success() {
        return Err(ApiError::internal("Failed to check voice usage"));
    }
    let body: ConversationsResponse = response
        .json::<ConversationsResponse>()
        .await
        .map_err(|_| ApiError::internal("Failed to check voice usage"))?;
    Ok(body
        .conversations
        .unwrap_or_default()
        .into_iter()
        .map(|conversation| conversation.call_duration_secs.unwrap_or(0))
        .sum())
}

async fn has_active_subscription(user_id: &str) -> bool {
    #[derive(serde::Deserialize)]
    struct RevenueCatResponse {
        subscriber: Option<RevenueCatSubscriber>,
    }

    #[derive(serde::Deserialize)]
    struct RevenueCatSubscriber {
        entitlements: Option<RevenueCatEntitlements>,
    }

    #[derive(serde::Deserialize)]
    struct RevenueCatEntitlements {
        active: Option<serde_json::Map<String, serde_json::Value>>,
    }

    let Some(api_key) = std::env::var("REVENUECAT_API_KEY")
        .ok()
        .filter(|value| !value.is_empty())
    else {
        return false;
    };
    let client = reqwest::Client::new();
    let Ok(response) = client
        .get(format!(
            "https://api.revenuecat.com/v1/subscribers/{user_id}"
        ))
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
    else {
        return false;
    };
    if !response.status().is_success() {
        return false;
    }
    let Ok(body) = response.json::<RevenueCatResponse>().await else {
        return false;
    };
    body.subscriber
        .and_then(|subscriber| subscriber.entitlements)
        .and_then(|entitlements| entitlements.active)
        .is_some_and(|active| active.contains_key("pro"))
}

async fn fetch_voice_token(api_key: &str, agent_id: &str) -> Result<String, ApiError> {
    #[derive(serde::Deserialize)]
    struct TokenResponse {
        token: String,
    }

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://api.elevenlabs.io/v1/convai/conversation/token?agent_id={agent_id}"
        ))
        .header("xi-api-key", api_key)
        .header(ACCEPT, "application/json")
        .send()
        .await
        .map_err(|_| ApiError::internal("Failed to get voice token"))?;
    if !response.status().is_success() {
        return Err(ApiError::internal("Failed to get voice token"));
    }
    let body: TokenResponse = response
        .json()
        .await
        .map_err(|_| ApiError::internal("Failed to get voice token"))?;
    Ok(body.token)
}
