//! Workspace list endpoint

use axum::extract::State;
use axum::Json;
use std::sync::Arc;

use ve_shared::types::Paginated;

use super::{Result, WorkspaceListRow};
use ve_shared::models::Workspace;
use crate::authz::WorkspaceCollectionAccess;
use crate::state::AppState;

/// GET /api/workspaces
///
/// List workspaces, optionally filtered by host.
pub async fn list_workspaces(
    access: WorkspaceCollectionAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Paginated<Workspace>>> {
    let offset = (access.page.saturating_sub(1) * access.limit) as i64;
    let limit = access.limit as i64;

    let rows: Vec<WorkspaceListRow> = if let Some(host_id) = access.host_id {
        sqlx::query_as(
            r#"
                SELECT workspaces.workspace_id, workspaces.host_id, workspaces.path, workspaces.display_name,
                       CASE WHEN workspaces.is_favorited THEN 1 ELSE 0 END AS is_favorited,
                       CAST(workspaces.last_used_at AS TEXT) AS last_used_at,
                       CASE WHEN workspaces.exists_on_host THEN 1 ELSE 0 END AS exists_on_host,
                       CAST(workspaces.created_at AS TEXT) AS created_at,
                       CAST(workspaces.updated_at AS TEXT) AS updated_at,
                       COUNT(*) OVER() as total_count
                FROM workspaces
                INNER JOIN device_host_access ON device_host_access.host_id = workspaces.host_id
                WHERE workspaces.host_id = $1 AND device_host_access.device_id = $2
                ORDER BY workspaces.is_favorited DESC, workspaces.last_used_at DESC
                LIMIT $3 OFFSET $4
                "#,
        )
        .bind(host_id.to_string())
        .bind(access.device_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
                SELECT workspaces.workspace_id, workspaces.host_id, workspaces.path, workspaces.display_name,
                       CASE WHEN workspaces.is_favorited THEN 1 ELSE 0 END AS is_favorited,
                       CAST(workspaces.last_used_at AS TEXT) AS last_used_at,
                       CASE WHEN workspaces.exists_on_host THEN 1 ELSE 0 END AS exists_on_host,
                       CAST(workspaces.created_at AS TEXT) AS created_at,
                       CAST(workspaces.updated_at AS TEXT) AS updated_at,
                       COUNT(*) OVER() as total_count
                FROM workspaces
                INNER JOIN device_host_access ON device_host_access.host_id = workspaces.host_id
                WHERE device_host_access.device_id = $1
                ORDER BY workspaces.is_favorited DESC, workspaces.last_used_at DESC
                LIMIT $2 OFFSET $3
                "#,
        )
        .bind(access.device_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?
    };

    let total = rows.first().map(|r| r.total_count as u64).unwrap_or(0);

    let workspaces: Result<Vec<Workspace>> = rows
        .into_iter()
        .map(|row| row.to_model())
        .collect();

    Ok(Json(Paginated::new(
        workspaces?,
        total,
        access.page,
        access.limit,
    )))
}
