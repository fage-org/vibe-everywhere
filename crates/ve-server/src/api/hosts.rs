//! Host API Handlers
//!
//! Host query and management endpoints.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::{self, parse_uuid};
use ve_shared::models::Host;

/// Host list response
#[derive(Debug, Serialize)]
pub struct HostListResponse {
    pub hosts: Vec<Host>,
}

/// GET /api/hosts
///
/// List all paired hosts.
pub async fn list_hosts(State(state): State<Arc<AppState>>) -> Result<Json<HostListResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT host_id, host_name, platform, online_status, daemon_status,
               last_active_at, pair_status, pair_code, qr_payload, created_at, updated_at
        FROM hosts
        WHERE pair_status = 'paired'
        ORDER BY updated_at DESC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let hosts: Result<Vec<Host>> = rows
        .into_iter()
        .map(|row| {
            let host_id = parse_uuid(&row.host_id, "host_id")?;
            Ok(Host {
                host_id,
                host_name: row.host_name,
                platform: utils::parse_platform(&row.platform),
                online_status: utils::parse_online_status(&row.online_status),
                daemon_status: utils::parse_daemon_status(&row.daemon_status),
                last_active_at: row.last_active_at.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&chrono::Utc))
                }),
                pair_status: utils::parse_pair_status(&row.pair_status),
                pair_code: row.pair_code,
                qr_payload: row.qr_payload,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                    .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
                    .map_err(|e| ServerError::Internal(format!("Invalid updated_at: {}", e)))?
                    .with_timezone(&chrono::Utc),
            })
        })
        .collect();

    Ok(Json(HostListResponse { hosts: hosts? }))
}

/// GET /api/hosts/:id
///
/// Get a specific host by ID.
pub async fn get_host(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Host>> {
    let host_id_str = id.to_string();

    let row = sqlx::query!(
        r#"
        SELECT host_id, host_name, platform, online_status, daemon_status,
               last_active_at, pair_status, pair_code, qr_payload, created_at, updated_at
        FROM hosts
        WHERE host_id = ?
        "#,
        host_id_str,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Host {}", id)))?;

    Ok(Json(Host {
        host_id: id,
        host_name: row.host_name,
        platform: utils::parse_platform(&row.platform),
        online_status: utils::parse_online_status(&row.online_status),
        daemon_status: utils::parse_daemon_status(&row.daemon_status),
        last_active_at: row.last_active_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        }),
        pair_status: utils::parse_pair_status(&row.pair_status),
        pair_code: row.pair_code,
        qr_payload: row.qr_payload,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
            .map_err(|e| ServerError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&chrono::Utc),
    }))
}

/// Unbind request
#[derive(Debug, Deserialize)]
pub struct UnbindRequest {
    pub confirm: bool,
}

/// Unbind response
#[derive(Debug, Serialize)]
pub struct UnbindResponse {
    pub success: bool,
}

/// POST /api/hosts/:id
///
/// Unbind (delete) a host.
pub async fn unbind_host(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UnbindRequest>,
) -> Result<Json<UnbindResponse>> {
    if !req.confirm {
        return Err(ServerError::BadRequest("Confirmation required".to_string()));
    }

    let host_id_str = id.to_string();

    sqlx::query!(
        r#"
        DELETE FROM hosts WHERE host_id = ?
        "#,
        host_id_str,
    )
    .execute(&state.db)
    .await?;

    tracing::info!(%id, "Host unbound");

    Ok(Json(UnbindResponse { success: true }))
}
