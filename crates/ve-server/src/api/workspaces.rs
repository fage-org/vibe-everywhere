//! Workspace API Handlers
//!
//! Workspace CRUD endpoints.

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::{
    jwt::Claims,
    models::{CreateWorkspaceRequest, Workspace},
};

use crate::authz::{
    authorize_workspace_create, require_client_device_id, require_host_access, ClientAccess,
    WorkspaceAccess, WorkspaceCollectionAccess,
};
use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils;
use crate::utils::parse_uuid;

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
                utils::parse_sqlite_timestamp(s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            }),
            exists_on_host: self.exists_on_host != 0,
            created_at: utils::parse_sqlite_timestamp(&self.created_at)
                .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: utils::parse_sqlite_timestamp(&self.updated_at)
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
    access: WorkspaceCollectionAccess,
    State(state): State<Arc<AppState>>,
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
    )> = if let Some(host_id) = access.host_id {
        sqlx::query_as(
            r#"
                SELECT workspaces.workspace_id, workspaces.host_id, workspaces.path, workspaces.display_name,
                       workspaces.is_favorited, workspaces.last_used_at, workspaces.exists_on_host,
                       workspaces.created_at, workspaces.updated_at
                FROM workspaces
                INNER JOIN device_host_access ON device_host_access.host_id = workspaces.host_id
                WHERE workspaces.host_id = $1 AND device_host_access.device_id = $2
                ORDER BY workspaces.is_favorited DESC, workspaces.last_used_at DESC
                "#,
        )
        .bind(host_id.to_string())
        .bind(access.device_id.to_string())
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
                SELECT workspaces.workspace_id, workspaces.host_id, workspaces.path, workspaces.display_name,
                       workspaces.is_favorited, workspaces.last_used_at, workspaces.exists_on_host,
                       workspaces.created_at, workspaces.updated_at
                FROM workspaces
                INNER JOIN device_host_access ON device_host_access.host_id = workspaces.host_id
                WHERE device_host_access.device_id = $1
                ORDER BY workspaces.is_favorited DESC, workspaces.last_used_at DESC
                "#,
        )
        .bind(access.device_id.to_string())
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
    client: ClientAccess,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>> {
    authorize_workspace_create(&state, client.device_id, &req).await?;
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

    sqlx::query(
        r#"
        INSERT INTO workspaces (workspace_id, host_id, path, display_name)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&workspace_id_str)
    .bind(&host_id_str)
    .bind(&req.path)
    .bind(&display_name)
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
pub async fn get_workspace_route(
    access: WorkspaceAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Workspace>> {
    get_workspace_by_id(state, access.workspace_id).await
}

pub async fn get_workspace(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Workspace>> {
    type WorkspaceRow = (
        String,
        String,
        String,
        String,
        i64,
        Option<String>,
        i64,
        String,
        String,
    );

    let workspace_id_str = id.to_string();

    let row: WorkspaceRow = sqlx::query_as(
        r#"
        SELECT workspace_id, host_id, path, display_name, is_favorited,
               last_used_at, exists_on_host, created_at, updated_at
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

    get_workspace_by_id(state, id).await
}

async fn get_workspace_by_id(state: Arc<AppState>, id: Uuid) -> Result<Json<Workspace>> {
    type WorkspaceRow = (
        String,
        String,
        String,
        String,
        i64,
        Option<String>,
        i64,
        String,
        String,
    );

    let workspace_id_str = id.to_string();

    let row: WorkspaceRow = sqlx::query_as(
        r#"
        SELECT workspace_id, host_id, path, display_name, is_favorited,
               last_used_at, exists_on_host, created_at, updated_at
        FROM workspaces
        WHERE workspace_id = $1
        "#,
    )
    .bind(&workspace_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Workspace {}", id)))?;

    Ok(Json(Workspace {
        workspace_id: id,
        host_id: parse_uuid(&row.1, "host_id")?,
        path: row.2,
        display_name: row.3,
        is_favorited: row.4 != 0,
        last_used_at: row.5.and_then(|s| {
            utils::parse_sqlite_timestamp(&s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        }),
        exists_on_host: row.6 != 0,
        created_at: utils::parse_sqlite_timestamp(&row.7)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        updated_at: utils::parse_sqlite_timestamp(&row.8)
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
pub async fn update_workspace_route(
    access: WorkspaceAccess,
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>> {
    update_workspace_by_id(state, access.workspace_id, req).await
}

pub async fn update_workspace(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> Result<Json<Workspace>> {
    type WorkspaceRow = (
        String,
        String,
        String,
        String,
        i64,
        Option<String>,
        i64,
        String,
        String,
    );

    let workspace_id_str = id.to_string();

    // Fetch existing
    let existing: WorkspaceRow = sqlx::query_as(
        r#"
        SELECT workspace_id, host_id, path, display_name, is_favorited,
               last_used_at, exists_on_host, created_at, updated_at
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

    update_workspace_by_id(state, id, req).await
}

async fn update_workspace_by_id(
    state: Arc<AppState>,
    id: Uuid,
    req: UpdateWorkspaceRequest,
) -> Result<Json<Workspace>> {
    type WorkspaceRow = (
        String,
        String,
        String,
        String,
        i64,
        Option<String>,
        i64,
        String,
        String,
    );

    let workspace_id_str = id.to_string();

    // Fetch existing
    let existing: WorkspaceRow = sqlx::query_as(
        r#"
        SELECT workspace_id, host_id, path, display_name, is_favorited,
               last_used_at, exists_on_host, created_at, updated_at
        FROM workspaces
        WHERE workspace_id = $1
        "#,
    )
    .bind(&workspace_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Workspace {}", id)))?;

    let display_name = req.display_name.unwrap_or(existing.3.clone());
    let is_favorited = req.is_favorited.unwrap_or(existing.4 != 0);
    let is_favorited_int = if is_favorited { 1 } else { 0 };
    let updated_at = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        UPDATE workspaces
        SET display_name = $1, is_favorited = $2, updated_at = $3
        WHERE workspace_id = $4
        "#,
    )
    .bind(&display_name)
    .bind(is_favorited_int)
    .bind(&updated_at)
    .bind(&workspace_id_str)
    .execute(&state.db)
    .await?;

    Ok(Json(Workspace {
        workspace_id: id,
        host_id: parse_uuid(&existing.1, "host_id")?,
        path: existing.2,
        display_name,
        is_favorited,
        last_used_at: existing.5.and_then(|s| {
            utils::parse_sqlite_timestamp(&s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        }),
        exists_on_host: existing.6 != 0,
        created_at: utils::parse_sqlite_timestamp(&existing.7)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::Utc::now(),
    }))
}

/// DELETE /api/workspaces/:id
///
/// Delete a workspace.
pub async fn delete_workspace_route(
    access: WorkspaceAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>> {
    delete_workspace_by_id(state, access.workspace_id).await
}

pub async fn delete_workspace(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
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
    .ok_or(ServerError::NotFound(format!("Workspace {}", id)))?;

    let device_id = require_client_device_id(&claims)?;
    let host_id = parse_uuid(&workspace_host.0, "host_id")?;
    require_host_access(&state, device_id, host_id).await?;

    delete_workspace_by_id(state, id).await
}

async fn delete_workspace_by_id(state: Arc<AppState>, id: Uuid) -> Result<Json<serde_json::Value>> {
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
