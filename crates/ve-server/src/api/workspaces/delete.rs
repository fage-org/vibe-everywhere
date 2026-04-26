//! Workspace delete endpoint

use axum::extract::State;
use axum::Json;
use std::sync::Arc;

use super::Result;
use crate::authz::WorkspaceAccess;
use crate::state::AppState;

/// DELETE /api/workspaces/:id
///
/// Delete a workspace.
pub async fn delete_workspace(
    State(state): State<Arc<AppState>>,
    access: WorkspaceAccess,
) -> Result<Json<serde_json::Value>> {
    let workspace_id_str = access.workspace_id.to_string();

    sqlx::query(
        r#"
        DELETE FROM workspaces WHERE workspace_id = $1
        "#,
    )
    .bind(&workspace_id_str)
    .execute(&state.db)
    .await?;

    tracing::info!(%access.workspace_id, "Workspace deleted");

    Ok(Json(serde_json::json!({ "success": true })))
}
