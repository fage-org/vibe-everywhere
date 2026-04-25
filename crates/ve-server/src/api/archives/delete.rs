//! Archive batch delete endpoint

use axum::{extract::State, Extension, Json};
use std::sync::Arc;
use uuid::Uuid;

use super::{BatchDeleteRequest, BatchDeleteResponse, Result};
use crate::authz::require_client_device_id;
use crate::error::ServerError;
use crate::state::AppState;
use crate::utils::parse_uuid;
use crate::validation::validate_batch_size;

/// POST /api/archives/batch-delete
///
/// Delete multiple archives.
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

    let mut deleted_count = 0;
    let mut failed_ids = Vec::new();

    for archive_id in req.archive_ids {
        let archive_id_str = archive_id.to_string();
        let session_id = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT session_id FROM session_archives WHERE archive_id = $1
            "#,
        )
        .bind(&archive_id_str)
        .fetch_optional(&state.db)
        .await?;

        let Some((session_id_raw,)) = session_id else {
            failed_ids.push(archive_id);
            continue;
        };

        let session_id = parse_uuid(&session_id_raw, "session_id")?;
        if crate::authz::require_session_access(&state, device_id, session_id)
            .await
            .is_err()
        {
            tracing::warn!(%archive_id, %device_id, %session_id, "Archive delete skipped: no session access");
            failed_ids.push(archive_id);
            continue;
        }

        let result = async {
            let mut tx = state.db.begin().await?;

            let delete_archive = sqlx::query(
                r#"
                DELETE FROM session_archives WHERE archive_id = $1
                "#,
            )
            .bind(&archive_id_str)
            .execute(&mut *tx)
            .await?;

            if delete_archive.rows_affected() == 0 {
                tx.rollback().await?;
                return Ok(false);
            }

            // Only remove this device's access; do not delete the session itself
            // (other devices may still have active access).
            sqlx::query(
                r#"
                DELETE FROM device_session_access WHERE session_id = $1 AND device_id = $2
                "#,
            )
            .bind(&session_id_raw)
            .bind(device_id.to_string())
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            Ok::<bool, ServerError>(true)
        }
        .await;

        match result {
            Ok(true) => {
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
