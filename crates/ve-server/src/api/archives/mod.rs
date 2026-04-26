//! Archive API Handlers
//!
//! Archived session query and management endpoints.

mod delete;
mod get;
mod list;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ve_shared::models::{ArchiveMetadata, SessionArchive};

use crate::error::{Result, ServerError};
use crate::utils::{self, parse_uuid};

/// Archive list query parameters
#[derive(Debug, Deserialize)]
pub struct ArchiveListQuery {
    pub host_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Batch delete request
#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    pub archive_ids: Vec<Uuid>,
}

/// Batch delete response
#[derive(Debug, Serialize)]
pub struct BatchDeleteResponse {
    pub deleted_count: usize,
    pub failed_ids: Vec<Uuid>,
}

/// Archive row type alias (includes metadata_json and total_count from window function)
type ArchiveRow = (
    String, String, String, String, String, String, String, String, Option<String>, i64,
);

/// Database record for archive
struct ArchiveRecord {
    archive_id: String,
    session_id: String,
    title: String,
    closed_at: String,
    close_reason: String,
    host_id: String,
    workspace_id: String,
    created_at: String,
    metadata_json: Option<String>,
}

impl ArchiveRecord {
    fn to_model(&self) -> Result<SessionArchive> {
        let metadata = self
            .metadata_json
            .as_ref()
            .and_then(|json| serde_json::from_str::<ArchiveMetadata>(json).ok());

        Ok(SessionArchive {
            archive_id: parse_uuid(&self.archive_id, "archive_id")?,
            session_id: parse_uuid(&self.session_id, "session_id")?,
            title: self.title.clone(),
            closed_at: utils::parse_sqlite_timestamp(&self.closed_at)
                .map_err(|e| ServerError::Internal(format!("Invalid closed_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            close_reason: utils::parse_close_reason(&self.close_reason),
            host_id: parse_uuid(&self.host_id, "host_id")?,
            workspace_id: parse_uuid(&self.workspace_id, "workspace_id")?,
            created_at: utils::parse_sqlite_timestamp(&self.created_at)
                .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            metadata,
        })
    }
}

// Re-exports for route registration in lib.rs
pub use delete::{batch_delete_archives, batch_delete_archives_route};
pub use get::{get_archive, get_archive_route};
pub use list::{list_archives, list_archives_route};
