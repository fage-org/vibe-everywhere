//! Archive API Handlers
//!
//! Archived session query and management endpoints.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::{ArchiveMetadata, SessionArchive};
use ve_shared::types::Paginated;

use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::{self, parse_uuid};

/// Archive row type alias (includes metadata_json)
type ArchiveRow = (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    Option<String>,
);

/// Archive list query parameters
#[derive(Debug, Deserialize)]
pub struct ArchiveListQuery {
    pub host_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

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
        let metadata = self.metadata_json.as_ref()
            .and_then(|json| serde_json::from_str::<ArchiveMetadata>(json).ok());

        Ok(SessionArchive {
            archive_id: parse_uuid(&self.archive_id, "archive_id")?,
            session_id: parse_uuid(&self.session_id, "session_id")?,
            title: self.title.clone(),
            closed_at: chrono::DateTime::parse_from_rfc3339(&self.closed_at)
                .map_err(|e| ServerError::Internal(format!("Invalid closed_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            close_reason: utils::parse_close_reason(&self.close_reason),
            host_id: parse_uuid(&self.host_id, "host_id")?,
            workspace_id: parse_uuid(&self.workspace_id, "workspace_id")?,
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            metadata,
        })
    }
}

/// GET /api/archives
///
/// List archived sessions.
pub async fn list_archives(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ArchiveListQuery>,
) -> Result<Json<Paginated<SessionArchive>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;

    let rows: Vec<ArchiveRow> = if let Some(host_id) = query.host_id {
        let host_id_str = host_id.to_string();
        sqlx::query_as(
                r#"
                SELECT archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, created_at, metadata_json
                FROM session_archives
                WHERE host_id = ?
                ORDER BY closed_at DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(host_id_str)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&state.db)
            .await?
    } else if let Some(workspace_id) = query.workspace_id {
        let workspace_id_str = workspace_id.to_string();
        sqlx::query_as(
                r#"
                SELECT archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, created_at, metadata_json
                FROM session_archives
                WHERE workspace_id = ?
                ORDER BY closed_at DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(workspace_id_str)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query_as(
                r#"
                SELECT archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, created_at, metadata_json
                FROM session_archives
                ORDER BY closed_at DESC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&state.db)
            .await?
    };

    let total = sqlx::query!(r#"SELECT COUNT(*) as count FROM session_archives"#)
        .fetch_one(&state.db)
        .await?
        .count as u64;

    let archives: Result<Vec<SessionArchive>> = rows
        .into_iter()
        .map(
            |(
                archive_id,
                session_id,
                title,
                closed_at,
                close_reason,
                host_id,
                workspace_id,
                created_at,
                metadata_json,
            )| {
                ArchiveRecord {
                    archive_id,
                    session_id,
                    title,
                    closed_at,
                    close_reason,
                    host_id,
                    workspace_id,
                    created_at,
                    metadata_json,
                }
                .to_model()
            },
        )
        .collect();

    Ok(Json(Paginated::new(archives?, total, page, limit)))
}

/// GET /api/archives/:id
///
/// Get archive details.
pub async fn get_archive(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<SessionArchive>> {
    let archive_id_str = id.to_string();

    let row = sqlx::query_as::<_, ArchiveRow>(
        r#"
        SELECT archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, created_at, metadata_json
        FROM session_archives
        WHERE archive_id = ?
        "#
    )
    .bind(archive_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Archive {}", id)))?;

    let record = ArchiveRecord {
        archive_id: row.0,
        session_id: row.1,
        title: row.2,
        closed_at: row.3,
        close_reason: row.4,
        host_id: row.5,
        workspace_id: row.6,
        created_at: row.7,
        metadata_json: row.8,
    };

    Ok(Json(record.to_model()?))
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

/// POST /api/archives/batch-delete
///
/// Delete multiple archives.
pub async fn batch_delete_archives(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<Json<BatchDeleteResponse>> {
    let mut deleted_count = 0;
    let mut failed_ids = Vec::new();

    for archive_id in req.archive_ids {
        let archive_id_str = archive_id.to_string();
        let result = sqlx::query!(
            r#"
            DELETE FROM session_archives WHERE archive_id = ?
            "#,
            archive_id_str,
        )
        .execute(&state.db)
        .await;

        match result {
            Ok(res) if res.rows_affected() > 0 => {
                deleted_count += 1;
            }
            _ => {
                failed_ids.push(archive_id);
            }
        }
    }

    tracing::info!(count = deleted_count, "Archives deleted");

    Ok(Json(BatchDeleteResponse {
        deleted_count,
        failed_ids,
    }))
}
