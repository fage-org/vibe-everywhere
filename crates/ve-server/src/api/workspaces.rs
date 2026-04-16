//! Workspace API Handlers
//!
//! Workspace CRUD endpoints.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::{CreateWorkspaceRequest, Workspace};

use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::parse_uuid;

/// Workspace list query parameters
#[derive(Debug, Deserialize)]
pub struct WorkspaceListQuery {
    pub host_id: Option<Uuid>,
}

/// Database record for workspace
struct WorkspaceRecord {
    workspace_id: String,
    host_id: String,
    path: String,
    display_name: String,
    is_favorited: i64,
    last_used_at: Option<String>,
    exists_on_host: i64,
    created_at: String,
    updated_at: String,
}

impl WorkspaceRecord {
    fn to_model(&self) -> Result<Workspace> {
        Ok(Workspace {
            workspace_id: parse_uuid(&self.workspace_id, "workspace_id")?,
            host_id: parse_uuid(&self.host_id, "host_id")?,
            path: self.path.clone(),
            display_name: self.display_name.clone(),
            is_favorited: self.is_favorited != 0,
            last_used_at: self.last_used_at.as_ref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            }),
            exists_on_host: self.exists_on_host != 0,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .map_err(|e| ServerError::Internal(format!("Invalid updated_at: {}", e)))?
                .with_timezone(&chrono::Utc),
        })
    }
}

/// GET /api/workspaces
///
/// List workspaces, optionally filtered by host.
#[allow(clippy::type_complexity)]
pub async fn list_workspaces(
    State(state): State<Arc<AppState>>,
    Query(query): Query<WorkspaceListQuery>,
) -> Result<Json<Vec<Workspace>>> {
    let rows: Vec<(
        String,
        String,
        String,
        String,
        i64,
        Option<String>,
        i64,
        String,
        String,
    )> = if let Some(host_id) = query.host_id {
        let host_id_str = host_id.to_string();
        sqlx::query_as(
            r#"
                SELECT workspace_id, host_id, path, display_name, is_favorited,
                       last_used_at, exists_on_host, created_at, updated_at
                FROM workspaces
                WHERE host_id = ?
                ORDER BY is_favorited DESC, last_used_at DESC
                "#,
        )
        .bind(host_id_str)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
                SELECT workspace_id, host_id, path, display_name, is_favorited,
                       last_used_at, exists_on_host, created_at, updated_at
                FROM workspaces
                ORDER BY is_favorited DESC, last_used_at DESC
                "#,
        )
        .fetch_all(&state.db)
        .await?
    };

    let workspaces: Result<Vec<Workspace>> = rows
        .into_iter()
        .map(
            |(
                workspace_id,
                host_id,
                path,
                display_name,
                is_favorited,
                last_used_at,
                exists_on_host,
                created_at,
                updated_at,
            )| {
                WorkspaceRecord {
                    workspace_id,
                    host_id,
                    path,
                    display_name,
                    is_favorited,
                    last_used_at,
                    exists_on_host,
                    created_at,
                    updated_at,
                }
                .to_model()
            },
        )
        .collect();

    Ok(Json(workspaces?))
}

/// POST /api/workspaces
///
/// Create a new workspace.
pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>> {
    let workspace_id = Uuid::new_v4();
    let workspace_id_str = workspace_id.to_string();
    let host_id_str = req.host_id.to_string();

    let display_name = req.display_name.clone().unwrap_or_else(|| {
        std::path::Path::new(&req.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Workspace")
            .to_string()
    });

    sqlx::query!(
        r#"
        INSERT INTO workspaces (workspace_id, host_id, path, display_name)
        VALUES (?, ?, ?, ?)
        "#,
        workspace_id_str,
        host_id_str,
        req.path,
        display_name,
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE constraint") {
            ServerError::Conflict("Workspace path already exists for this host".to_string())
        } else {
            ServerError::Database(e)
        }
    })?;

    tracing::info!(%workspace_id, %req.host_id, "Workspace created");

    Ok(Json(Workspace {
        workspace_id,
        host_id: req.host_id,
        path: req.path,
        display_name,
        is_favorited: false,
        last_used_at: None,
        exists_on_host: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }))
}

/// GET /api/workspaces/:id
///
/// Get a specific workspace.
pub async fn get_workspace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Workspace>> {
    let workspace_id_str = id.to_string();

    let row = sqlx::query!(
        r#"
        SELECT workspace_id, host_id, path, display_name, is_favorited,
               last_used_at, exists_on_host, created_at, updated_at
        FROM workspaces
        WHERE workspace_id = ?
        "#,
        workspace_id_str,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Workspace {}", id)))?;

    Ok(Json(Workspace {
        workspace_id: id,
        host_id: parse_uuid(&row.host_id, "host_id")?,
        path: row.path,
        display_name: row.display_name,
        is_favorited: row.is_favorited != 0,
        last_used_at: row.last_used_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        }),
        exists_on_host: row.exists_on_host != 0,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
            .map_err(|e| ServerError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&chrono::Utc),
    }))
}

/// Update workspace request
#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub display_name: Option<String>,
    pub is_favorited: Option<bool>,
}

/// POST /api/workspaces/:id
///
/// Update workspace details.
pub async fn update_workspace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>> {
    let workspace_id_str = id.to_string();

    // Fetch existing
    let existing = sqlx::query!(
        r#"
        SELECT workspace_id, host_id, path, display_name, is_favorited,
               last_used_at, exists_on_host, created_at, updated_at
        FROM workspaces
        WHERE workspace_id = ?
        "#,
        workspace_id_str,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Workspace {}", id)))?;

    let display_name = req.display_name.unwrap_or(existing.display_name);
    let is_favorited = req.is_favorited.unwrap_or(existing.is_favorited != 0);
    let is_favorited_int = if is_favorited { 1 } else { 0 };

    sqlx::query!(
        r#"
        UPDATE workspaces
        SET display_name = ?, is_favorited = ?, updated_at = datetime('now')
        WHERE workspace_id = ?
        "#,
        display_name,
        is_favorited_int,
        workspace_id_str,
    )
    .execute(&state.db)
    .await?;

    Ok(Json(Workspace {
        workspace_id: id,
        host_id: parse_uuid(&existing.host_id, "host_id")?,
        path: existing.path,
        display_name,
        is_favorited,
        last_used_at: existing.last_used_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        }),
        exists_on_host: existing.exists_on_host != 0,
        created_at: chrono::DateTime::parse_from_rfc3339(&existing.created_at)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::Utc::now(),
    }))
}

/// DELETE /api/workspaces/:id
///
/// Delete a workspace.
pub async fn delete_workspace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let workspace_id_str = id.to_string();

    sqlx::query!(
        r#"
        DELETE FROM workspaces WHERE workspace_id = ?
        "#,
        workspace_id_str,
    )
    .execute(&state.db)
    .await?;

    tracing::info!(%id, "Workspace deleted");

    Ok(Json(serde_json::json!({ "success": true })))
}
