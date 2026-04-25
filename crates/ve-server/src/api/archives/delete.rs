//! Archive batch delete endpoint

use axum::{extract::State, Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use super::{BatchDeleteRequest, BatchDeleteResponse, Result};
use crate::authz::require_client_device_id;
use crate::state::AppState;
use crate::validation::validate_batch_size;

/// POST /api/archives/batch-delete
///
/// Delete multiple archives in a single transaction.
pub async fn batch_delete_archives_route(
    access: crate::authz::ArchiveCollectionAccess,
    State(state): State<Arc<AppState>>,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<Json<BatchDeleteResponse>> {
    batch_delete_archives_for_device(state, access.device_id, req).await
}

pub async fn batch_delete_archives(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<ve_shared::jwt::Claims>,
    Json(req): Json<BatchDeleteRequest>,
) -> Result<Json<BatchDeleteResponse>> {
    let device_id = require_client_device_id(&claims)?;
    batch_delete_archives_for_device(state, device_id, req).await
}

async fn batch_delete_archives_for_device(
    state: Arc<AppState>,
    device_id: Uuid,
    req: BatchDeleteRequest,
) -> Result<Json<BatchDeleteResponse>> {
    validate_batch_size(req.archive_ids.len(), "archive_ids")?;

    let device_id_str = device_id.to_string();

    // Single transaction for all operations: lookup, authorize, delete.
    // Eliminates N+1 transactions and TOCTOU race between auth check and deletion.
    let mut tx = state.db.begin().await?;

    let mut deleted_count = 0;
    let mut failed_ids = Vec::new();
    let mut deleted_session_ids: Vec<String> = Vec::new();

    for archive_id in &req.archive_ids {
        let archive_id_str = archive_id.to_string();

        // Look up the session for this archive
        let session: Option<(String,)> = sqlx::query_as(
            r#"SELECT session_id FROM session_archives WHERE archive_id = $1"#,
        )
        .bind(&archive_id_str)
        .fetch_optional(&mut *tx)
        .await
        .ok()
        .flatten();

        let Some((session_id,)) = session else {
            tracing::warn!(%archive_id, %device_id, "Archive delete skipped: not found");
            failed_ids.push(*archive_id);
            continue;
        };

        // Authorize: check device has session access (within same transaction)
        let has_access: Option<(i64,)> = sqlx::query_as(
            r#"SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2"#,
        )
        .bind(&device_id_str)
        .bind(&session_id)
        .fetch_optional(&mut *tx)
        .await
        .ok()
        .flatten();

        if has_access.is_none_or(|(c,)| c <= 0) {
            tracing::warn!(%archive_id, %device_id, session_id = %session_id, "Archive delete skipped: no session access");
            failed_ids.push(*archive_id);
            continue;
        }

        // Delete the archive
        let result = sqlx::query(
            r#"DELETE FROM session_archives WHERE archive_id = $1"#,
        )
        .bind(&archive_id_str)
        .execute(&mut *tx)
        .await;

        match result {
            Ok(r) if r.rows_affected() > 0 => {
                deleted_count += 1;
                deleted_session_ids.push(session_id);
            }
            _ => {
                failed_ids.push(*archive_id);
            }
        }
    }

    // Remove this device's access rows for successfully deleted sessions
    for session_id in &deleted_session_ids {
        let _ = sqlx::query(
            r#"DELETE FROM device_session_access WHERE device_id = $1 AND session_id = $2"#,
        )
        .bind(&device_id_str)
        .bind(session_id)
        .execute(&mut *tx)
        .await;
    }

    tx.commit().await?;

    tracing::info!(count = deleted_count, "Archives deleted");

    Ok(Json(BatchDeleteResponse {
        deleted_count,
        failed_ids,
    }))
}
