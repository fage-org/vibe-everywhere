//! Workspace update endpoint

use axum::extract::State;
use axum::Json;
use std::sync::Arc;

use ve_shared::models::Workspace;

use super::Result;
use crate::authz::WorkspaceAccess;
use crate::state::AppState;
use crate::utils::parse_optional_sqlite_timestamp;
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
pub async fn update_workspace(
    State(state): State<Arc<AppState>>,
    access: WorkspaceAccess,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>> {
    let workspace_id_str = access.workspace_id.to_string();
    let display_name = req.display_name.unwrap_or(access.display_name.clone());
    validate_workspace_display_name(&display_name)?;
    let is_favorited = req.is_favorited.unwrap_or(access.is_favorited);

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

    Ok(Json(Workspace {
        workspace_id: access.workspace_id,
        host_id: access.host_id,
        path: access.path,
        display_name,
        is_favorited,
        last_used_at: access.last_used_at
            .as_deref()
            .and_then(parse_optional_sqlite_timestamp),
        exists_on_host: access.exists_on_host,
        created_at: crate::utils::parse_sqlite_timestamp(&access.created_at)
            .map(|dt| dt.to_utc())
            .unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: chrono::Utc::now(),
    }))
}
