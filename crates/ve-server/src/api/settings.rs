//! Settings API Handlers
//!
//! Notification preferences and settings endpoints.

use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use std::sync::Arc;
use ve_shared::{jwt::Claims, models::NotificationPreference};

use crate::authz::require_client_device_id;
use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::validation::validate_device_id_format;

/// GET /api/settings/notifications
///
/// Get notification preferences for a device.
pub async fn get_notification_preferences(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<NotificationPreference>> {
    let device_id = require_client_device_id(&claims)?;
    // Validate device_id format
    let device_id_str = device_id.to_string();
    validate_device_id_format(&device_id_str)?;

    // Check if device exists
    let device_exists: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT device_id FROM client_devices WHERE device_id = $1
        "#,
    )
    .bind(&device_id_str)
    .fetch_optional(&state.db)
    .await?;

    if device_exists.is_none() {
        return Err(ServerError::NotFound(format!("Device {}", device_id)));
    }

    let row: Option<(String, i64, i64, i64, i64, i64)> = sqlx::query_as(
        r#"
        SELECT device_id,
               CASE WHEN enabled THEN 1 ELSE 0 END,
               CASE WHEN permission_request_enabled THEN 1 ELSE 0 END,
               CASE WHEN task_completed_enabled THEN 1 ELSE 0 END,
               CASE WHEN task_failed_enabled THEN 1 ELSE 0 END,
               CASE WHEN session_error_enabled THEN 1 ELSE 0 END
        FROM notification_preferences
        WHERE device_id = $1
        "#,
    )
    .bind(&device_id_str)
    .fetch_optional(&state.db)
    .await?;

    let prefs = if let Some(row) = row {
        NotificationPreference {
            device_id,
            enabled: row.1 != 0,
            permission_request_enabled: row.2 != 0,
            task_completed_enabled: row.3 != 0,
            task_failed_enabled: row.4 != 0,
            session_error_enabled: row.5 != 0,
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
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateNotificationPreferencesRequest>,
) -> Result<Json<NotificationPreference>> {
    let device_id = require_client_device_id(&claims)?;
    // Validate device_id format
    let device_id_str = device_id.to_string();
    validate_device_id_format(&device_id_str)?;

    // Check if device exists in client_devices table
    let device_exists: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT device_id FROM client_devices WHERE device_id = $1
        "#,
    )
    .bind(&device_id_str)
    .fetch_optional(&state.db)
    .await?;

    if device_exists.is_none() {
        return Err(ServerError::NotFound(format!("Device {}", device_id)));
    }

    sqlx::query(
        r#"
        INSERT INTO notification_preferences (device_id, enabled, permission_request_enabled,
            task_completed_enabled, task_failed_enabled, session_error_enabled)
        VALUES ($1, COALESCE($2, TRUE), COALESCE($3, TRUE), COALESCE($4, TRUE), COALESCE($5, TRUE), COALESCE($6, TRUE))
        ON CONFLICT(device_id) DO UPDATE SET
            enabled = COALESCE(EXCLUDED.enabled, notification_preferences.enabled),
            permission_request_enabled = COALESCE(EXCLUDED.permission_request_enabled, notification_preferences.permission_request_enabled),
            task_completed_enabled = COALESCE(EXCLUDED.task_completed_enabled, notification_preferences.task_completed_enabled),
            task_failed_enabled = COALESCE(EXCLUDED.task_failed_enabled, notification_preferences.task_failed_enabled),
            session_error_enabled = COALESCE(EXCLUDED.session_error_enabled, notification_preferences.session_error_enabled)
        "#,
    )
    .bind(&device_id_str)
    .bind(req.enabled)
    .bind(req.permission_request_enabled)
    .bind(req.task_completed_enabled)
    .bind(req.task_failed_enabled)
    .bind(req.session_error_enabled)
    .execute(&state.db)
    .await?;

    tracing::info!(%device_id, "Notification preferences updated");

    // Return updated preferences
    get_notification_preferences(Extension(claims), State(state)).await
}
