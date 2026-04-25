//! Archive get endpoint

use axum::extract::{Path, State};
use axum::{Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::SessionArchive;

use super::{ArchiveRecord, ArchiveRow, Result};
use crate::authz::{require_client_device_id, ArchiveAccess};
use crate::state::AppState;

/// GET /api/archives/:id
///
/// Get archive details.
pub async fn get_archive_route(
    access: ArchiveAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<SessionArchive>> {
    get_archive_for_device(state, access.device_id, access.archive_id).await
}

pub async fn get_archive(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<ve_shared::jwt::Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<SessionArchive>> {
    let device_id = require_client_device_id(&claims)?;
    get_archive_for_device(state, device_id, id).await
}

async fn get_archive_for_device(
    state: Arc<AppState>,
    device_id: Uuid,
    id: Uuid,
) -> Result<Json<SessionArchive>> {
    use crate::error::ServerError;
    let archive_id_str = id.to_string();

    let row = sqlx::query_as::<_, ArchiveRow>(
        r#"
        SELECT session_archives.archive_id, session_archives.session_id, session_archives.title,
               CAST(session_archives.closed_at AS TEXT), session_archives.close_reason,
               session_archives.host_id, session_archives.workspace_id,
               CAST(session_archives.created_at AS TEXT), CAST(session_archives.metadata_json AS TEXT)
        FROM session_archives
        INNER JOIN device_session_access
            ON device_session_access.session_id = session_archives.session_id
        WHERE session_archives.archive_id = $1 AND device_session_access.device_id = $2
        "#,
    )
    .bind(archive_id_str)
    .bind(device_id.to_string())
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
