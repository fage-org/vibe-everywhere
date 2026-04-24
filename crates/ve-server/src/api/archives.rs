//! Archive API Handlers
//!
//! Archived session query and management endpoints.

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::{ArchiveMetadata, SessionArchive};
use ve_shared::types::Paginated;

use crate::authz::{
    require_client_device_id, require_host_access, require_session_access, ArchiveAccess,
    ArchiveCollectionAccess,
};
use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::{self, parse_uuid};
use crate::validation::validate_batch_size;

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
    let offset = (page - 1) * limit;

    let rows: Vec<ArchiveRow> = match (query.host_id, query.workspace_id) {
        (Some(host_id), Some(workspace_id)) => {
            require_host_access(&state, device_id, host_id).await?;
            sqlx::query_as(
                r#"
                SELECT DISTINCT session_archives.archive_id, session_archives.session_id,
                       session_archives.title, CAST(session_archives.closed_at AS TEXT),
                       session_archives.close_reason, session_archives.host_id,
                       session_archives.workspace_id, CAST(session_archives.created_at AS TEXT),
                       CAST(session_archives.metadata_json AS TEXT)
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
                       CAST(session_archives.metadata_json AS TEXT)
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
                       CAST(session_archives.metadata_json AS TEXT)
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
                       CAST(session_archives.metadata_json AS TEXT)
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

    let total: (i64,) = match (query.host_id, query.workspace_id) {
        (Some(host_id), Some(workspace_id)) => {
            sqlx::query_as(
                r#"
                SELECT COUNT(DISTINCT session_archives.archive_id)
                FROM session_archives
                INNER JOIN device_session_access
                    ON device_session_access.session_id = session_archives.session_id
                INNER JOIN workspaces
                    ON workspaces.workspace_id = session_archives.workspace_id
                WHERE session_archives.host_id = $1
                  AND session_archives.workspace_id = $2
                  AND device_session_access.device_id = $3
                  AND workspaces.host_id = $1
                "#,
            )
            .bind(host_id.to_string())
            .bind(workspace_id.to_string())
            .bind(&device_id_str)
            .fetch_one(&state.db)
            .await?
        }
        (Some(host_id), None) => {
            sqlx::query_as(
                r#"
                SELECT COUNT(DISTINCT session_archives.archive_id)
                FROM session_archives
                INNER JOIN device_session_access
                    ON device_session_access.session_id = session_archives.session_id
                WHERE session_archives.host_id = $1 AND device_session_access.device_id = $2
                "#,
            )
            .bind(host_id.to_string())
            .bind(&device_id_str)
            .fetch_one(&state.db)
            .await?
        }
        (None, Some(workspace_id)) => {
            sqlx::query_as(
                r#"
                SELECT COUNT(DISTINCT session_archives.archive_id)
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
                "#,
            )
            .bind(workspace_id.to_string())
            .bind(&device_id_str)
            .fetch_one(&state.db)
            .await?
        }
        (None, None) => {
            sqlx::query_as(
                r#"
                SELECT COUNT(DISTINCT session_archives.archive_id)
                FROM session_archives
                INNER JOIN device_session_access
                    ON device_session_access.session_id = session_archives.session_id
                WHERE device_session_access.device_id = $1
                "#,
            )
            .bind(&device_id_str)
            .fetch_one(&state.db)
            .await?
        }
    };
    let total = total.0 as u64;

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
pub async fn batch_delete_archives_route(
    access: crate::authz::ClientAccess,
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
        if require_session_access(&state, device_id, session_id)
            .await
            .is_err()
        {
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

            sqlx::query(
                r#"
                DELETE FROM device_session_access WHERE session_id = $1 AND device_id = $2
                "#,
            )
            .bind(&session_id_raw)
            .bind(device_id.to_string())
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                DELETE FROM sessions WHERE session_id = $1 AND status = 'archived'
                "#,
            )
            .bind(&session_id_raw)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseBackend};
    use crate::db::{install_drivers, run_migrations, DbPool};
    use crate::hub::Hub;
    use crate::state::AppState;
    use ve_shared::jwt::Claims;

    fn test_config(database_url: String) -> Config {
        Config {
            listen_addr: "127.0.0.1:3000".parse().unwrap(),
            database_url,
            jwt_secret: "01234567890123456789012345678901".to_string(),
            jwt_expiration_secs: 3600,
            pair_code_ttl_secs: 300,
            heartbeat_interval_secs: 30,
            connection_timeout_secs: 60,
            data_dir: std::path::PathBuf::from("/tmp"),
            cors_origins: Vec::new(),
            ack_timeout_ms: 10000,
            ack_max_retries: 2,
            ack_retry_delay_ms: 500,
            permission_ttl_secs: 1800,
            permission_expiry_check_secs: 60,
            idempotency_ttl_secs: 86400,
            idempotency_cleanup_secs: 3600,
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
        }
    }

    async fn setup_state() -> Arc<AppState> {
        install_drivers();
        let temp_db =
            std::env::temp_dir().join(format!("ve-archives-api-test-{}.db", Uuid::new_v4()));
        let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
        let pool = DbPool::connect(&database_url).await.unwrap();
        run_migrations(&pool, DatabaseBackend::Sqlite)
            .await
            .unwrap();

        let config = test_config(database_url);
        let jwt_manager = Arc::new(ve_shared::jwt::JwtManager::new(
            &config.jwt_secret,
            config.jwt_expiration(),
        ));
        Arc::new(AppState::new(pool, Hub::new(), config, jwt_manager))
    }

    async fn insert_archive_fixture(
        state: &Arc<AppState>,
        device_id: Uuid,
        host_id: Uuid,
        workspace_id: Uuid,
        session_id: Uuid,
        archive_id: Uuid,
    ) {
        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp/project")
        .bind("project")
        .execute(&state.db)
        .await
        .unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(archive_id.to_string())
        .bind(session_id.to_string())
        .bind("archived")
        .bind(&now)
        .bind("user_closed")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind(Option::<String>::None)
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();
    }

    async fn insert_archive_for_existing_scope(
        state: &Arc<AppState>,
        device_id: Uuid,
        host_id: Uuid,
        workspace_id: Uuid,
        session_id: Uuid,
        archive_id: Uuid,
        title: &str,
    ) {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind(title)
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(archive_id.to_string())
        .bind(session_id.to_string())
        .bind(title)
        .bind(&now)
        .bind("user_closed")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind(Option::<String>::None)
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn list_archives_excludes_sessions_without_session_access() {
        let state = setup_state().await;
        let visible_device_id = Uuid::new_v4();
        let hidden_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let visible_session_id = Uuid::new_v4();
        let visible_archive_id = Uuid::new_v4();
        let hidden_session_id = Uuid::new_v4();
        let hidden_archive_id = Uuid::new_v4();

        insert_archive_fixture(
            &state,
            visible_device_id,
            host_id,
            workspace_id,
            visible_session_id,
            visible_archive_id,
        )
        .await;

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(hidden_device_id.to_string())
        .bind("other")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(hidden_device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(hidden_session_id.to_string())
        .bind("hidden")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(hidden_archive_id.to_string())
        .bind(hidden_session_id.to_string())
        .bind("hidden")
        .bind(&now)
        .bind("user_closed")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind(Option::<String>::None)
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();

        let response = list_archives(
            State(state.clone()),
            Extension(Claims::for_client(
                visible_device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            Query(ArchiveListQuery {
                host_id: Some(host_id),
                workspace_id: None,
                page: None,
                limit: None,
            }),
        )
        .await
        .unwrap()
        .0;

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].archive_id, visible_archive_id);
    }

    #[tokio::test]
    async fn list_archives_intersects_host_id_and_workspace_id_filters() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let other_host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let other_workspace_id = Uuid::new_v4();
        let matching_session_id = Uuid::new_v4();
        let host_only_session_id = Uuid::new_v4();
        let workspace_only_session_id = Uuid::new_v4();
        let matching_archive_id = Uuid::new_v4();
        let host_only_archive_id = Uuid::new_v4();
        let workspace_only_archive_id = Uuid::new_v4();

        insert_archive_fixture(
            &state,
            device_id,
            host_id,
            workspace_id,
            matching_session_id,
            matching_archive_id,
        )
        .await;

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(other_host_id.to_string())
            .bind("other-host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(other_workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp/other-project")
        .bind("other-project")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(other_host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        insert_archive_for_existing_scope(
            &state,
            device_id,
            host_id,
            other_workspace_id,
            host_only_session_id,
            host_only_archive_id,
            "host-only",
        )
        .await;

        insert_archive_for_existing_scope(
            &state,
            device_id,
            other_host_id,
            workspace_id,
            workspace_only_session_id,
            workspace_only_archive_id,
            "workspace-only",
        )
        .await;

        let response = list_archives(
            State(state.clone()),
            Extension(Claims::for_client(
                device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            Query(ArchiveListQuery {
                host_id: Some(host_id),
                workspace_id: Some(workspace_id),
                page: None,
                limit: None,
            }),
        )
        .await
        .unwrap()
        .0;

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.total, 1);
        assert_eq!(response.items[0].archive_id, matching_archive_id);
    }

    #[tokio::test]
    async fn get_archive_rejects_devices_without_session_access() {
        let state = setup_state().await;
        let owner_device_id = Uuid::new_v4();
        let other_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let archive_id = Uuid::new_v4();

        insert_archive_fixture(
            &state,
            owner_device_id,
            host_id,
            workspace_id,
            session_id,
            archive_id,
        )
        .await;

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(other_device_id.to_string())
        .bind("other")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(other_device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let error = get_archive(
            State(state.clone()),
            Extension(Claims::for_client(
                other_device_id,
                "other",
                chrono::Duration::hours(1),
            )),
            Path(archive_id),
        )
        .await
        .unwrap_err();

        assert!(
            matches!(error, ServerError::NotFound(message) if message == format!("Archive {}", archive_id))
        );
    }

    #[tokio::test]
    async fn batch_delete_archives_removes_archived_session_invariant_together() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let archive_id = Uuid::new_v4();

        insert_archive_fixture(
            &state,
            device_id,
            host_id,
            workspace_id,
            session_id,
            archive_id,
        )
        .await;

        let response = batch_delete_archives(
            State(state.clone()),
            Extension(Claims::for_client(
                device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            Json(BatchDeleteRequest {
                archive_ids: vec![archive_id],
            }),
        )
        .await
        .unwrap()
        .0;

        assert_eq!(response.deleted_count, 1);
        assert!(response.failed_ids.is_empty());

        let archive_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE archive_id = $1")
                .bind(archive_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(archive_count.0, 0);

        let session_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(session_count.0, 0);

        let access_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM device_session_access WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(access_count.0, 0);
    }

    #[tokio::test]
    async fn list_archives_rejects_page_zero() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        let error = list_archives(
            State(state.clone()),
            Extension(Claims::for_client(
                device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            Query(ArchiveListQuery {
                host_id: None,
                workspace_id: None,
                page: Some(0),
                limit: None,
            }),
        )
        .await
        .unwrap_err();

        assert!(
            matches!(error, ServerError::BadRequest(message) if message == "page must be greater than 0")
        );
    }
}
