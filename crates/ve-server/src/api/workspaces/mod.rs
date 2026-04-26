//! Workspace API Handlers
//!
//! Workspace CRUD endpoints.

mod create;
mod delete;
mod get;
mod list;
mod update;
#[cfg(test)]
mod tests;

use serde::Deserialize;

use ve_shared::models::Workspace;

use crate::error::Result;
use crate::utils;
use crate::utils::parse_uuid;

/// Workspace list query parameters (handled via WorkspaceCollectionAccess extractor)
/// Empty struct kept for future query param extensions.
#[derive(Debug, Deserialize)]
pub struct WorkspaceListQuery {}

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

/// Workspace list row with window function total count
#[derive(sqlx::FromRow)]
struct WorkspaceListRow {
    workspace_id: String,
    host_id: String,
    path: String,
    display_name: String,
    is_favorited: i64,
    last_used_at: Option<String>,
    exists_on_host: i64,
    created_at: String,
    updated_at: String,
    total_count: i64,
}

impl WorkspaceListRow {
    fn to_model(&self) -> Result<Workspace> {
        workspace_record_from_row((
            self.workspace_id.clone(),
            self.host_id.clone(),
            self.path.clone(),
            self.display_name.clone(),
            self.is_favorited,
            self.last_used_at.clone(),
            self.exists_on_host,
            self.created_at.clone(),
            self.updated_at.clone(),
        ))
        .to_model()
    }
}

type WorkspaceRow = (
    String, String, String, String, i64, Option<String>, i64, String, String,
);

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
                .map_err(|e| crate::error::ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: utils::parse_sqlite_timestamp(&self.updated_at)
                .map_err(|e| crate::error::ServerError::Internal(format!("Invalid updated_at: {}", e)))?
                .with_timezone(&chrono::Utc),
        })
    }
}

fn workspace_record_from_row(row: WorkspaceRow) -> WorkspaceRecord {
    let (
        workspace_id,
        host_id,
        path,
        display_name,
        is_favorited,
        last_used_at,
        exists_on_host,
        created_at,
        updated_at,
    ) = row;

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
}

// Re-exports for route registration in lib.rs
pub use create::create_workspace;
pub use delete::delete_workspace;
pub use get::get_workspace;
pub use list::list_workspaces;
pub use update::{update_workspace, UpdateWorkspaceRequest};
