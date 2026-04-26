//! Workspace get endpoint

use axum::Json;

use ve_shared::models::Workspace;

use super::Result;
use crate::authz::WorkspaceAccess;
use crate::utils::parse_optional_sqlite_timestamp;

/// GET /api/workspaces/:id
///
/// Get a specific workspace.
pub async fn get_workspace(
    access: WorkspaceAccess,
) -> Result<Json<Workspace>> {
    Ok(Json(Workspace {
        workspace_id: access.workspace_id,
        host_id: access.host_id,
        path: access.path,
        display_name: access.display_name,
        is_favorited: access.is_favorited,
        last_used_at: access.last_used_at
            .as_deref()
            .and_then(parse_optional_sqlite_timestamp),
        exists_on_host: access.exists_on_host,
        created_at: crate::utils::parse_sqlite_timestamp(&access.created_at)
            .map(|dt| dt.to_utc())
            .unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: crate::utils::parse_sqlite_timestamp(&access.updated_at)
            .map(|dt| dt.to_utc())
            .unwrap_or_else(|_| chrono::Utc::now()),
    }))
}
