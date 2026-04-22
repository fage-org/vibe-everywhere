//! Host API Handlers
//!
//! Host query and management endpoints.

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::authz::{
    require_client_device_id, require_host_access, HostAccess, HostCollectionAccess,
};
use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::{self, parse_uuid};
use crate::validation::{validate_host_can_be_deleted, HostDeletionStatus};
use ve_shared::{jwt::Claims, models::Host};

/// Host list response
#[derive(Debug, Serialize)]
pub struct HostListResponse {
    pub hosts: Vec<Host>,
}

/// GET /api/hosts
///
/// List all paired hosts.
pub async fn list_hosts(
    access: HostCollectionAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<HostListResponse>> {
    type HostRow = (
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
    );

    let rows: Vec<HostRow> = sqlx::query_as(
        r#"
        SELECT hosts.host_id, hosts.host_name, hosts.platform, hosts.online_status,
               hosts.daemon_status, hosts.last_active_at, hosts.pair_status,
               hosts.pair_code, hosts.qr_payload, hosts.created_at, hosts.updated_at
        FROM hosts
        INNER JOIN device_host_access
            ON device_host_access.host_id = hosts.host_id
        WHERE hosts.pair_status = 'paired' AND device_host_access.device_id = $1
        ORDER BY hosts.updated_at DESC
        "#,
    )
    .bind(access.device_id.to_string())
    .fetch_all(&state.db)
    .await?;

    let hosts: Result<Vec<Host>> = rows
        .into_iter()
        .map(|row| {
            let host_id = parse_uuid(&row.0, "host_id")?;
            Ok(Host {
                host_id,
                host_name: row.1,
                platform: utils::parse_platform(&row.2),
                online_status: utils::parse_online_status(&row.3),
                daemon_status: utils::parse_daemon_status(&row.4),
                last_active_at: row.5.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&chrono::Utc))
                }),
                pair_status: utils::parse_pair_status(&row.6),
                pair_code: row.7,
                qr_payload: row.8,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.9)
                    .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.10)
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
pub async fn get_host_route(
    access: HostAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Host>> {
    get_host_by_id(state, access.host_id).await
}

pub async fn get_host(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Host>> {
    let device_id = require_client_device_id(&claims)?;
    require_host_access(&state, device_id, id).await?;
    get_host_by_id(state, id).await
}

async fn get_host_by_id(state: Arc<AppState>, id: Uuid) -> Result<Json<Host>> {
    type HostRow = (
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
        String,
        String,
    );

    let host_id_str = id.to_string();

    let row: HostRow = sqlx::query_as(
        r#"
        SELECT host_id, host_name, platform, online_status, daemon_status,
               last_active_at, pair_status, pair_code, qr_payload, created_at, updated_at
        FROM hosts
        WHERE host_id = $1
        "#,
    )
    .bind(&host_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Host {}", id)))?;

    Ok(Json(Host {
        host_id: id,
        host_name: row.1,
        platform: utils::parse_platform(&row.2),
        online_status: utils::parse_online_status(&row.3),
        daemon_status: utils::parse_daemon_status(&row.4),
        last_active_at: row.5.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        }),
        pair_status: utils::parse_pair_status(&row.6),
        pair_code: row.7,
        qr_payload: row.8,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.9)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.10)
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
pub async fn unbind_host_route(
    access: HostAccess,
    State(state): State<Arc<AppState>>,
    Json(req): Json<UnbindRequest>,
) -> Result<Json<UnbindResponse>> {
    unbind_host_by_id(state, access.host_id, req).await
}

pub async fn unbind_host(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UnbindRequest>,
) -> Result<Json<UnbindResponse>> {
    let device_id = require_client_device_id(&claims)?;
    require_host_access(&state, device_id, id).await?;
    unbind_host_by_id(state, id, req).await
}

async fn unbind_host_by_id(
    state: Arc<AppState>,
    id: Uuid,
    req: UnbindRequest,
) -> Result<Json<UnbindResponse>> {
    if !req.confirm {
        return Err(ServerError::BadRequest("Confirmation required".to_string()));
    }

    let host_id_str = id.to_string();

    // Check for dependent resources
    let session_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM sessions WHERE host_id = $1 AND status != 'archived'
        "#,
    )
    .bind(&host_id_str)
    .fetch_one(&state.db)
    .await?;

    let archive_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM session_archives WHERE host_id = $1
        "#,
    )
    .bind(&host_id_str)
    .fetch_one(&state.db)
    .await?;

    let workspace_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM workspaces WHERE host_id = $1
        "#,
    )
    .bind(&host_id_str)
    .fetch_one(&state.db)
    .await?;

    let deletion_status = HostDeletionStatus {
        session_count: session_count.0 as usize,
        archive_count: archive_count.0 as usize,
        workspace_count: workspace_count.0 as usize,
    };

    if !validate_host_can_be_deleted(&deletion_status) {
        return Err(ServerError::Conflict(format!(
            "Cannot unbind host: {} active session(s), {} archive(s). Close or delete sessions first.",
            deletion_status.session_count,
            deletion_status.archive_count
        )));
    }

    sqlx::query(
        r#"
        DELETE FROM hosts WHERE host_id = $1
        "#,
    )
    .bind(&host_id_str)
    .execute(&state.db)
    .await?;

    tracing::info!(%id, sessions = deletion_status.session_count, archives = deletion_status.archive_count, workspaces = deletion_status.workspace_count, "Host unbound");

    Ok(Json(UnbindResponse { success: true }))
}
