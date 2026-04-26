//! Workspace delete endpoint

use axum::extract::{Path, State};
use axum::{Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use super::Result;
use crate::authz::{require_client_device_id, require_host_access};
use crate::state::AppState;

/// DELETE /api/workspaces/:id
///
/// Delete a workspace.
pub async fn delete_workspace(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<ve_shared::jwt::Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let workspace_id_str = id.to_string();

    let workspace_host: (String,) = sqlx::query_as(
        r#"
        SELECT host_id FROM workspaces WHERE workspace_id = $1
        "#,
    )
    .bind(&workspace_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(crate::error::ServerError::NotFound(format!("Workspace {}", id)))?;

    let device_id = require_client_device_id(&claims)?;
    let host_id = crate::utils::parse_uuid(&workspace_host.0, "host_id")?;
    require_host_access(&state, device_id, host_id).await?;

    let workspace_id_str = id.to_string();

    sqlx::query(
        r#"
        DELETE FROM workspaces WHERE workspace_id = $1
        "#,
    )
    .bind(&workspace_id_str)
    .execute(&state.db)
    .await?;

    tracing::info!(%id, "Workspace deleted");

    Ok(Json(serde_json::json!({ "success": true })))
}
