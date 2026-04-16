//! Permission API Handlers
//!
//! Permission request query and response endpoints.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::{PermissionDecision, PermissionRequest, PermissionResponseRequest};
use ve_shared::proto::DaemonMessage;

use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::{self, parse_uuid};

/// Permission list query parameters
#[derive(Debug, Deserialize)]
pub struct PermissionListQuery {
    pub session_id: Option<Uuid>,
    #[allow(dead_code)]
    pub status: Option<String>,
}

/// Database record for permission request
struct PermissionRecord {
    permission_id: String,
    session_id: String,
    risk_type: String,
    summary: String,
    target: Option<String>,
    status: String,
    created_at: String,
    responded_at: Option<String>,
}

impl PermissionRecord {
    fn to_model(&self) -> Result<PermissionRequest> {
        Ok(PermissionRequest {
            permission_id: parse_uuid(&self.permission_id, "permission_id")?,
            session_id: parse_uuid(&self.session_id, "session_id")?,
            risk_type: utils::parse_risk_type(&self.risk_type),
            summary: self.summary.clone(),
            target: self.target.clone(),
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            status: utils::parse_permission_status(&self.status),
            responded_at: self.responded_at.as_ref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            }),
        })
    }
}

/// GET /api/permissions
///
/// List permission requests, optionally filtered.
pub async fn list_permissions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PermissionListQuery>,
) -> Result<Json<Vec<PermissionRequest>>> {
    let rows = if let Some(session_id) = query.session_id {
        let session_id_str = session_id.to_string();
        sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, Option<String>)>(
            r#"
            SELECT permission_id, session_id, risk_type, summary, target, status, created_at, responded_at
            FROM permission_requests
            WHERE session_id = ?
            ORDER BY created_at DESC
            "#
        )
        .bind(session_id_str)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, Option<String>)>(
            r#"
            SELECT permission_id, session_id, risk_type, summary, target, status, created_at, responded_at
            FROM permission_requests
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&state.db)
        .await?
    };

    let permissions: Result<Vec<PermissionRequest>> = rows
        .into_iter()
        .map(
            |(
                permission_id,
                session_id,
                risk_type,
                summary,
                target,
                status,
                created_at,
                responded_at,
            )| {
                PermissionRecord {
                    permission_id,
                    session_id,
                    risk_type,
                    summary,
                    target,
                    status,
                    created_at,
                    responded_at,
                }
                .to_model()
            },
        )
        .collect();

    Ok(Json(permissions?))
}

/// GET /api/permissions/:id
///
/// Get a specific permission request.
pub async fn get_permission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PermissionRequest>> {
    let permission_id_str = id.to_string();

    let row = sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, Option<String>)>(
        r#"
        SELECT permission_id, session_id, risk_type, summary, target, status, created_at, responded_at
        FROM permission_requests
        WHERE permission_id = ?
        "#
    )
    .bind(permission_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Permission {}", id)))?;

    let record = PermissionRecord {
        permission_id: row.0,
        session_id: row.1,
        risk_type: row.2,
        summary: row.3,
        target: row.4,
        status: row.5,
        created_at: row.6,
        responded_at: row.7,
    };

    Ok(Json(record.to_model()?))
}

/// POST /api/permissions/:id/respond
///
/// Respond to a permission request (approve/deny).
pub async fn respond_permission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<PermissionResponseRequest>,
) -> Result<Json<PermissionRequest>> {
    let permission_id_str = id.to_string();

    // Fetch existing permission
    let row = sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, Option<String>)>(
        r#"
        SELECT permission_id, session_id, risk_type, summary, target, status, created_at, responded_at
        FROM permission_requests
        WHERE permission_id = ?
        "#
    )
    .bind(&permission_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Permission {}", id)))?;

    let existing = PermissionRecord {
        permission_id: row.0,
        session_id: row.1,
        risk_type: row.2,
        summary: row.3,
        target: row.4,
        status: row.5,
        created_at: row.6,
        responded_at: row.7,
    };

    // Check if already responded (idempotent - return current state)
    let current_status = utils::parse_permission_status(&existing.status);
    if current_status.is_responded() {
        tracing::info!(%id, "Permission already responded (idempotent)");
        return Ok(Json(existing.to_model()?));
    }

    // Get session info for host_id
    let session = sqlx::query!(
        r#"
        SELECT host_id FROM sessions WHERE session_id = ?
        "#,
        existing.session_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!(
        "Session {}",
        existing.session_id
    )))?;

    let host_id = parse_uuid(&session.host_id, "host_id")?;
    let session_id = parse_uuid(&existing.session_id, "session_id")?;
    let session_id_str = existing.session_id.clone();

    // Update status
    let new_status = match req.decision {
        PermissionDecision::ApproveOnce => "approved_once",
        PermissionDecision::DenyOnce => "denied_once",
        PermissionDecision::ApproveSession => "approved_session",
    };

    sqlx::query!(
        r#"
        UPDATE permission_requests
        SET status = ?, responded_at = datetime('now')
        WHERE permission_id = ?
        "#,
        new_status,
        permission_id_str,
    )
    .execute(&state.db)
    .await?;

    // Decrement pending permission count in session
    sqlx::query!(
        r#"
        UPDATE sessions
        SET pending_permission_count = MAX(0, pending_permission_count - 1), updated_at = datetime('now')
        WHERE session_id = ?
        "#,
        session_id_str,
    )
    .execute(&state.db)
    .await?;

    // Send response to daemon
    state.hub.send_to_daemon(
        &host_id,
        DaemonMessage::PermissionResponse {
            permission_id: id,
            session_id,
            decision: req.decision,
        },
    );

    tracing::info!(%id, ?req.decision, "Permission responded");

    Ok(Json(PermissionRequest {
        permission_id: id,
        session_id,
        risk_type: utils::parse_risk_type(&existing.risk_type),
        summary: existing.summary,
        target: existing.target,
        created_at: chrono::DateTime::parse_from_rfc3339(&existing.created_at)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        status: utils::parse_permission_status(new_status),
        responded_at: Some(chrono::Utc::now()),
    }))
}
