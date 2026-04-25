//! Workspace update endpoint

use axum::extract::{Path, State};
use axum::{Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::Workspace;

use super::{workspace_record_from_row, Result, WorkspaceRow};
use crate::authz::{require_client_device_id, require_host_access, WorkspaceAccess};
use crate::error::ServerError;
use crate::state::AppState;
use crate::utils::parse_uuid;
use crate::validation::validate_workspace_display_name;

/// Update workspace request
#[derive(Debug, serde::Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub display_name: Option<String>,
    pub is_favorited: Option<bool>,
}

/// POST /api/workspaces/:id
///
/// Update workspace details.
pub async fn update_workspace_route(
    access: WorkspaceAccess,
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>> {
    update_workspace_with_existing(
        state,
        access.workspace_id,
        req,
        (
            access.workspace_id.to_string(),
            access.host_id.to_string(),
            access.path,
            access.display_name,
            if access.is_favorited { 1 } else { 0 },
            access.last_used_at,
            if access.exists_on_host { 1 } else { 0 },
            access.created_at,
            access.updated_at,
        ),
    )
    .await
}

pub async fn update_workspace(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<ve_shared::jwt::Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>> {
    let workspace_id_str = id.to_string();

    let existing: WorkspaceRow = sqlx::query_as(
        r#"
        SELECT workspace_id, host_id, path, display_name,
               CASE WHEN is_favorited THEN 1 ELSE 0 END,
               CAST(last_used_at AS TEXT),
               CASE WHEN exists_on_host THEN 1 ELSE 0 END,
               CAST(created_at AS TEXT), CAST(updated_at AS TEXT)
        FROM workspaces
        WHERE workspace_id = $1
        "#,
    )
    .bind(&workspace_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Workspace {}", id)))?;

    let device_id = require_client_device_id(&claims)?;
    let host_id = parse_uuid(&existing.1, "host_id")?;
    require_host_access(&state, device_id, host_id).await?;

    update_workspace_with_existing(state, id, req, existing).await
}

async fn update_workspace_with_existing(
    state: Arc<AppState>,
    id: Uuid,
    req: UpdateWorkspaceRequest,
    existing: WorkspaceRow,
) -> Result<Json<Workspace>> {
    let workspace_id_str = id.to_string();
    let display_name = req.display_name.unwrap_or(existing.3.clone());
    validate_workspace_display_name(&display_name)?;
    let is_favorited = req.is_favorited.unwrap_or(existing.4 != 0);
    sqlx::query(
        r#"
        UPDATE workspaces
        SET display_name = $1, is_favorited = $2, updated_at = CURRENT_TIMESTAMP
        WHERE workspace_id = $3
        "#,
    )
    .bind(&display_name)
    .bind(is_favorited)
    .bind(&workspace_id_str)
    .execute(&state.db)
    .await?;

    let mut record = workspace_record_from_row(existing);
    record.display_name = display_name;
    record.is_favorited = if is_favorited { 1 } else { 0 };
    record.updated_at = chrono::Utc::now().to_rfc3339();
    Ok(Json(record.to_model()?))
}
