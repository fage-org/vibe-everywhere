//! Archive list endpoints

use axum::extract::{Query, State};
use axum::{Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::types::Paginated;

use super::{ArchiveListQuery, ArchiveRecord, ArchiveRow, Result};
use ve_shared::models::SessionArchive;
use crate::authz::{require_host_access, ArchiveCollectionAccess};
use crate::error::ServerError;
use crate::state::AppState;

/// GET /api/archives
///
/// List archived sessions.
pub async fn list_archives_route(
    access: ArchiveCollectionAccess,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ArchiveListQuery>,
) -> Result<Json<Paginated<SessionArchive>>> {
    list_archives_for_device(state, access.device_id, query).await
}

pub async fn list_archives(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<ve_shared::jwt::Claims>,
    Query(query): Query<ArchiveListQuery>,
) -> Result<Json<Paginated<SessionArchive>>> {
    use crate::authz::require_client_device_id;
    let device_id = require_client_device_id(&claims)?;
    list_archives_for_device(state, device_id, query).await
}

async fn list_archives_for_device(
    state: Arc<AppState>,
    device_id: Uuid,
    query: ArchiveListQuery,
) -> Result<Json<Paginated<SessionArchive>>> {
    let device_id_str = device_id.to_string();
    let page = query.page.unwrap_or(1);
    if page == 0 {
        return Err(ServerError::BadRequest(
            "page must be greater than 0".to_string(),
        ));
    }
    let limit = query.limit.unwrap_or(20).min(100);
    if limit == 0 {
        return Err(ServerError::BadRequest(
            "limit must be greater than 0".to_string(),
        ));
    }
    let offset = (page as u64 - 1) * limit as u64;

    let rows: Vec<ArchiveRow> = match (query.host_id, query.workspace_id) {
        (Some(host_id), Some(workspace_id)) => {
            require_host_access(&state, device_id, host_id).await?;
            sqlx::query_as(
                r#"
                SELECT DISTINCT session_archives.archive_id, session_archives.session_id,
                       session_archives.title, CAST(session_archives.closed_at AS TEXT),
                       session_archives.close_reason, session_archives.host_id,
                       session_archives.workspace_id, CAST(session_archives.created_at AS TEXT),
                       CAST(session_archives.metadata_json AS TEXT),
                       COUNT(*) OVER() AS total_count
                FROM session_archives
                INNER JOIN device_session_access
                    ON device_session_access.session_id = session_archives.session_id
                INNER JOIN workspaces
                    ON workspaces.workspace_id = session_archives.workspace_id
                WHERE session_archives.host_id = $1
                  AND session_archives.workspace_id = $2
                  AND device_session_access.device_id = $3
                  AND workspaces.host_id = $1
                ORDER BY CAST(session_archives.closed_at AS TEXT) DESC
                LIMIT $4 OFFSET $5
                "#,
            )
            .bind(host_id.to_string())
            .bind(workspace_id.to_string())
            .bind(&device_id_str)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&state.db)
            .await?
        }
        (Some(host_id), None) => {
            require_host_access(&state, device_id, host_id).await?;
            sqlx::query_as(
                r#"
                SELECT DISTINCT session_archives.archive_id, session_archives.session_id,
                       session_archives.title, CAST(session_archives.closed_at AS TEXT),
                       session_archives.close_reason, session_archives.host_id,
                       session_archives.workspace_id, CAST(session_archives.created_at AS TEXT),
                       CAST(session_archives.metadata_json AS TEXT),
                       COUNT(*) OVER() AS total_count
                FROM session_archives
                INNER JOIN device_session_access
                    ON device_session_access.session_id = session_archives.session_id
                WHERE session_archives.host_id = $1 AND device_session_access.device_id = $2
                ORDER BY CAST(session_archives.closed_at AS TEXT) DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(host_id.to_string())
            .bind(&device_id_str)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&state.db)
            .await?
        }
        (None, Some(workspace_id)) => {
            sqlx::query_as(
                r#"
                SELECT DISTINCT session_archives.archive_id, session_archives.session_id,
                       session_archives.title, CAST(session_archives.closed_at AS TEXT),
                       session_archives.close_reason, session_archives.host_id,
                       session_archives.workspace_id, CAST(session_archives.created_at AS TEXT),
                       CAST(session_archives.metadata_json AS TEXT),
                       COUNT(*) OVER() AS total_count
                FROM session_archives
                INNER JOIN device_session_access
                    ON device_session_access.session_id = session_archives.session_id
                INNER JOIN workspaces
                    ON workspaces.workspace_id = session_archives.workspace_id
                WHERE session_archives.workspace_id = $1
                  AND device_session_access.device_id = $2
                  AND workspaces.host_id IN (
                      SELECT host_id FROM device_host_access WHERE device_id = $2
                  )
                ORDER BY CAST(session_archives.closed_at AS TEXT) DESC
                LIMIT $3 OFFSET $4
                "#,
            )
            .bind(workspace_id.to_string())
            .bind(&device_id_str)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&state.db)
            .await?
        }
        (None, None) => {
            sqlx::query_as(
                r#"
                SELECT DISTINCT session_archives.archive_id, session_archives.session_id,
                       session_archives.title, CAST(session_archives.closed_at AS TEXT),
                       session_archives.close_reason, session_archives.host_id,
                       session_archives.workspace_id, CAST(session_archives.created_at AS TEXT),
                       CAST(session_archives.metadata_json AS TEXT),
                       COUNT(*) OVER() AS total_count
                FROM session_archives
                INNER JOIN device_session_access
                    ON device_session_access.session_id = session_archives.session_id
                WHERE device_session_access.device_id = $1
                ORDER BY CAST(session_archives.closed_at AS TEXT) DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(&device_id_str)
            .bind(limit as i32)
            .bind(offset as i32)
            .fetch_all(&state.db)
            .await?
        }
    };

    let total = rows.first().map(|r| r.9 as u64).unwrap_or(0);

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
                _total_count,
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
