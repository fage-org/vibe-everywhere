use std::collections::BTreeMap;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde_json::json;

use crate::{
    api::types::{
        AccountProfileResponse, AccountSettingsResponse, ApiError, UpdateAccountSettingsBody,
        UpdateAccountSettingsConflict, UpdateAccountSettingsSuccess, UsageBucket, UsageQueryBody,
        UsageQueryResponse,
    },
    auth::AuthenticatedUser,
    context::AppContext,
    storage::db::now_ms,
};

pub fn routes() -> Router<AppContext> {
    Router::new()
        .route("/v1/account/profile", get(profile))
        .route(
            "/v1/account/settings",
            get(get_settings).post(update_settings),
        )
        .route("/v1/usage/query", post(query_usage))
}

async fn profile(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<AccountProfileResponse>, ApiError> {
    let account = ctx
        .db()
        .get_account(&user.user_id)
        .ok_or_else(|| ApiError::not_found("Account not found"))?;
    let connected_services = ctx.db().read(|state| {
        let mut vendors = state
            .vendor_tokens
            .values()
            .filter(|token| token.account_id == user.user_id)
            .filter(|token| matches!(token.vendor.as_str(), "openai" | "anthropic" | "gemini"))
            .map(|token| token.vendor.clone())
            .collect::<Vec<_>>();
        vendors.sort();
        vendors.dedup();
        vendors
    });
    Ok(Json(AccountProfileResponse {
        id: account.id,
        timestamp: now_ms(),
        first_name: account.first_name,
        last_name: account.last_name,
        username: account.username,
        avatar: ctx.files().resolve_image_json(account.avatar.as_ref()),
        github: account.github_profile,
        connected_services,
    }))
}

async fn get_settings(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<AccountSettingsResponse>, ApiError> {
    let account = ctx
        .db()
        .get_account(&user.user_id)
        .ok_or_else(|| ApiError::internal("Failed to get account settings"))?;
    Ok(Json(AccountSettingsResponse {
        settings: account.settings,
        settings_version: account.settings_version,
    }))
}

async fn update_settings(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<UpdateAccountSettingsBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let response = ctx.db().write(|state| {
        let Some(account) = state.accounts.get_mut(&user.user_id) else {
            return Err(ApiError::internal("Failed to update account settings"));
        };
        if account.settings_version != body.expected_version {
            return Ok(json!(UpdateAccountSettingsConflict {
                success: false,
                error: "version-mismatch".into(),
                current_version: account.settings_version,
                current_settings: account.settings.clone(),
            }));
        }
        account.settings = body.settings.clone();
        account.settings_version += 1;
        account.updated_at = now_ms();
        Ok(json!(UpdateAccountSettingsSuccess {
            success: true,
            version: account.settings_version,
        }))
    })?;
    if response["success"] == true {
        let version = response["version"].as_u64().unwrap_or_default();
        ctx.events()
            .publish_account_settings_update(ctx.db(), &user.user_id, version, body.settings)
            .map_err(|_| ApiError::internal("Failed to update account settings"))?;
    }
    Ok(Json(response))
}

async fn query_usage(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<UsageQueryBody>,
) -> Result<Json<UsageQueryResponse>, ApiError> {
    let group_by = body.group_by.unwrap_or_else(|| "day".into());
    if group_by != "day" && group_by != "hour" {
        return Err(ApiError::bad_request("groupBy must be 'hour' or 'day'"));
    }
    if let Some(session_id) = body.session_id.as_deref()
        && ctx
            .db()
            .get_session_for_account(&user.user_id, session_id)
            .is_none()
    {
        return Err(ApiError::not_found("Session not found"));
    }

    let start_time_ms = body.start_time.map(|value| value.saturating_mul(1000));
    let end_time_ms = body.end_time.map(|value| value.saturating_mul(1000));
    let mut reports = ctx.db().read(|state| {
        let mut reports = state
            .usage_reports
            .values()
            .filter(|report| report.account_id == user.user_id)
            .filter(|report| {
                body.session_id
                    .as_deref()
                    .is_none_or(|session_id| report.session_id.as_deref() == Some(session_id))
            })
            .filter(|report| start_time_ms.is_none_or(|start| report.created_at >= start))
            .filter(|report| end_time_ms.is_none_or(|end| report.created_at <= end))
            .cloned()
            .collect::<Vec<_>>();
        reports.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        reports
    });
    let total_reports = reports.len() as u64;
    let mut usage = {
        let mut aggregated = BTreeMap::<u64, UsageBucket>::new();
        for report in reports.drain(..) {
            let created_at_seconds = report.created_at / 1000;
            let bucket_key = if group_by == "hour" {
                (created_at_seconds / 3600) * 3600
            } else {
                (created_at_seconds / 86_400) * 86_400
            };
            let bucket = aggregated.entry(bucket_key).or_insert_with(|| UsageBucket {
                timestamp: bucket_key,
                ..UsageBucket::default()
            });
            for (key, value) in &report.tokens {
                *bucket.tokens.entry(key.clone()).or_default() += value;
            }
            for (key, value) in &report.cost {
                *bucket.cost.entry(key.clone()).or_default() += value;
            }
            bucket.report_count += 1;
        }
        aggregated.into_values().collect::<Vec<_>>()
    };
    usage.sort_by_key(|bucket| bucket.timestamp);
    Ok(Json(UsageQueryResponse {
        usage,
        group_by,
        total_reports,
    }))
}
