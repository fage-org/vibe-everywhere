//! Settings API Handlers
//!
//! Notification preferences and settings endpoints.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::NotificationPreference;

use crate::error::Result;
use crate::state::AppState;

/// GET /api/settings/notifications
///
/// Get notification preferences for a device.
pub async fn get_notification_preferences(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<Uuid>,
) -> Result<Json<NotificationPreference>> {
    let device_id_str = device_id.to_string();

    let row = sqlx::query!(
        r#"
        SELECT device_id, enabled, permission_request_enabled, task_completed_enabled,
               task_failed_enabled, session_error_enabled
        FROM notification_preferences
        WHERE device_id = ?
        "#,
        device_id_str,
    )
    .fetch_optional(&state.db)
    .await?;

    let prefs = if let Some(row) = row {
        NotificationPreference {
            device_id,
            enabled: row.enabled != 0,
            permission_request_enabled: row.permission_request_enabled != 0,
            task_completed_enabled: row.task_completed_enabled != 0,
            task_failed_enabled: row.task_failed_enabled != 0,
            session_error_enabled: row.session_error_enabled != 0,
        }
    } else {
        // Return defaults if no preferences set
        NotificationPreference {
            device_id,
            ..Default::default()
        }
    };

    Ok(Json(prefs))
}

/// Update notification preferences request
#[derive(Debug, Deserialize)]
pub struct UpdateNotificationPreferencesRequest {
    pub enabled: Option<bool>,
    pub permission_request_enabled: Option<bool>,
    pub task_completed_enabled: Option<bool>,
    pub task_failed_enabled: Option<bool>,
    pub session_error_enabled: Option<bool>,
}

/// POST /api/settings/notifications
///
/// Update notification preferences for a device.
pub async fn update_notification_preferences(
    State(state): State<Arc<AppState>>,
    Path(device_id): Path<Uuid>,
    Json(req): Json<UpdateNotificationPreferencesRequest>,
) -> Result<Json<NotificationPreference>> {
    let device_id_str = device_id.to_string();

    // Check if record exists
    let existing = sqlx::query!(
        r#"
        SELECT device_id FROM notification_preferences WHERE device_id = ?
        "#,
        device_id_str,
    )
    .fetch_optional(&state.db)
    .await?;

    if existing.is_none() {
        // Insert new record
        let enabled_int = if req.enabled.unwrap_or(true) { 1 } else { 0 };
        let perm_int = if req.permission_request_enabled.unwrap_or(true) {
            1
        } else {
            0
        };
        let comp_int = if req.task_completed_enabled.unwrap_or(true) {
            1
        } else {
            0
        };
        let fail_int = if req.task_failed_enabled.unwrap_or(true) {
            1
        } else {
            0
        };
        let err_int = if req.session_error_enabled.unwrap_or(true) {
            1
        } else {
            0
        };

        sqlx::query!(
            r#"
            INSERT INTO notification_preferences (device_id, enabled, permission_request_enabled,
                task_completed_enabled, task_failed_enabled, session_error_enabled)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            device_id_str,
            enabled_int,
            perm_int,
            comp_int,
            fail_int,
            err_int,
        )
        .execute(&state.db)
        .await?;
    } else {
        // Update existing record
        let enabled_int = req.enabled.map(|b| if b { 1 } else { 0 });
        let perm_int = req
            .permission_request_enabled
            .map(|b| if b { 1 } else { 0 });
        let comp_int = req.task_completed_enabled.map(|b| if b { 1 } else { 0 });
        let fail_int = req.task_failed_enabled.map(|b| if b { 1 } else { 0 });
        let err_int = req.session_error_enabled.map(|b| if b { 1 } else { 0 });

        sqlx::query!(
            r#"
            UPDATE notification_preferences
            SET enabled = COALESCE(?, enabled),
                permission_request_enabled = COALESCE(?, permission_request_enabled),
                task_completed_enabled = COALESCE(?, task_completed_enabled),
                task_failed_enabled = COALESCE(?, task_failed_enabled),
                session_error_enabled = COALESCE(?, session_error_enabled)
            WHERE device_id = ?
            "#,
            enabled_int,
            perm_int,
            comp_int,
            fail_int,
            err_int,
            device_id_str,
        )
        .execute(&state.db)
        .await?;
    }

    tracing::info!(%device_id, "Notification preferences updated");

    // Return updated preferences
    get_notification_preferences(State(state), Path(device_id)).await
}
