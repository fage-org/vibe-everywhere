//! Workspace get endpoint

use axum::extract::{Path, State};
use axum::{Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::Workspace;

use super::Result;
use crate::authz::{require_client_device_id, require_host_access};
use crate::error::ServerError;
use crate::state::AppState;

/// GET /api/workspaces/:id
///
/// Get a specific workspace.
pub async fn get_workspace(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<ve_shared::jwt::Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Workspace>> {
    use super::{workspace_record_from_row, WorkspaceRow};
    use crate::utils::parse_uuid;
    let workspace_id_str = id.to_string();

    let row: WorkspaceRow = sqlx::query_as(
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
    let host_id = parse_uuid(&row.1, "host_id")?;
    require_host_access(&state, device_id, host_id).await?;

    Ok(Json(workspace_record_from_row(row).to_model()?))
}
