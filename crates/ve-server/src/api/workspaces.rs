//! Workspace API Handlers
//!
//! Workspace CRUD endpoints.

use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use ve_shared::{
    jwt::Claims,
    models::{CreateWorkspaceRequest, Workspace},
    proto::{DaemonMessage, ErrorPayload},
    types::Paginated,
};

use crate::authz::{
    authorize_workspace_create, require_client_device_id, require_host_access, ClientAccess,
    WorkspaceAccess, WorkspaceCollectionAccess,
};
use crate::error::{Result, ServerError};
use crate::hub::DaemonResponse;
use crate::state::AppState;
use crate::utils;
use crate::utils::{generate_request_id, parse_uuid};

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
) -> Result<Json<Paginated<Workspace>>> {
    let offset = (access.page.saturating_sub(1) * access.limit) as i64;
    let limit = access.limit as i64;

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
        i64,
    )> = if let Some(host_id) = access.host_id {
        sqlx::query_as(
            r#"
                SELECT workspaces.workspace_id, workspaces.host_id, workspaces.path, workspaces.display_name,
                       workspaces.is_favorited, workspaces.last_used_at, workspaces.exists_on_host,
                       workspaces.created_at, workspaces.updated_at,
                       COUNT(*) OVER() as total_count
                FROM workspaces
                INNER JOIN device_host_access ON device_host_access.host_id = workspaces.host_id
                WHERE workspaces.host_id = $1 AND device_host_access.device_id = $2
                ORDER BY workspaces.is_favorited DESC, workspaces.last_used_at DESC
                LIMIT $3 OFFSET $4
                "#,
        )
        .bind(host_id.to_string())
        .bind(access.device_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
                SELECT workspaces.workspace_id, workspaces.host_id, workspaces.path, workspaces.display_name,
                       workspaces.is_favorited, workspaces.last_used_at, workspaces.exists_on_host,
                       workspaces.created_at, workspaces.updated_at,
                       COUNT(*) OVER() as total_count
                FROM workspaces
                INNER JOIN device_host_access ON device_host_access.host_id = workspaces.host_id
                WHERE device_host_access.device_id = $1
                ORDER BY workspaces.is_favorited DESC, workspaces.last_used_at DESC
                LIMIT $2 OFFSET $3
                "#,
        )
        .bind(access.device_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?
    };

    let total = rows.first().map(|r| r.9 as u64).unwrap_or(0);

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
                _,
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

    Ok(Json(Paginated::new(
        workspaces?,
        total,
        access.page,
        access.limit,
    )))
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

    if !state.hub.is_daemon_connected(&req.host_id).await {
        return Err(ServerError::HostNotFound);
    }

    let request_id = generate_request_id();
    let prepare_request = DaemonMessage::EnsureWorkspace {
        request_id: request_id.clone(),
        workspace_path: req.path.clone(),
    };
    let prepare_response = state
        .hub
        .send_and_wait(
            &req.host_id,
            prepare_request,
            request_id,
            Duration::from_secs(30),
        )
        .await
        .map_err(|error| sanitize_workspace_transport_error(error.as_ref()))?;

    match prepare_response {
        DaemonResponse::Ack(ack) if ack.success => {}
        DaemonResponse::Ack(ack) => {
            return Err(ServerError::BadRequest(
                ack.error
                    .unwrap_or_else(|| "Workspace could not be prepared".to_string()),
            ));
        }
        DaemonResponse::Error(error) => {
            return Err(sanitize_workspace_operation_error_from_payload(&error));
        }
        DaemonResponse::Message(ve_shared::proto::DaemonToServer::Error {
            error_code,
            error_message,
            ..
        }) => {
            return Err(sanitize_workspace_operation_error(
                Some(&error_code),
                &error_message,
            ));
        }
        _ => {
            return Err(ServerError::Internal(
                "Unexpected response while preparing workspace".to_string(),
            ));
        }
    }

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

fn sanitize_workspace_transport_error(error: &dyn std::error::Error) -> ServerError {
    tracing::warn!(error = %error, "Sanitized workspace preparation transport failure");
    ServerError::BadRequest("Workspace could not be prepared".to_string())
}

fn sanitize_workspace_operation_error(
    error_code: Option<&str>,
    raw_error_message: &str,
) -> ServerError {
    tracing::warn!(
        error_code,
        raw_error_message,
        "Sanitized workspace preparation error response"
    );

    let safe_message = match error_code {
        Some("WORKSPACE_INVALID") => "Workspace path is not available on the host",
        Some("INVALID_INPUT") => "Workspace path is invalid",
        _ => "Workspace could not be prepared",
    };

    ServerError::BadRequest(safe_message.to_string())
}

fn sanitize_workspace_operation_error_from_payload(error: &ErrorPayload) -> ServerError {
    sanitize_workspace_operation_error(Some(&error.error_code), &error.error_message)
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
    sqlx::query(
        r#"
        UPDATE workspaces
        SET display_name = $1, is_favorited = $2, updated_at = CURRENT_TIMESTAMP
        WHERE workspace_id = $3
        "#,
    )
    .bind(&display_name)
    .bind(is_favorited_int)
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use axum::{extract::State, Json};
    use tokio::sync::mpsc;
    use ve_shared::{
        jwt::{Claims, TokenType},
        proto::{AckPayload, ErrorPayload, WsEnvelope},
    };

    use crate::{
        config::Config,
        db,
        hub::{Hub, WsSender},
    };

    fn test_config(database_url: String) -> Config {
        Config {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            database_url,
            jwt_secret: "super_secure_test_secret_key_32_chars!!".to_string(),
            jwt_expiration_secs: 3600,
            pair_code_ttl_secs: 300,
            heartbeat_interval_secs: 30,
            connection_timeout_secs: 60,
            data_dir: std::env::temp_dir(),
            cors_origins: vec![],
            ack_timeout_ms: 3_000,
            ack_max_retries: 0,
            ack_retry_delay_ms: 0,
            permission_ttl_secs: 1800,
            permission_expiry_check_secs: 60,
            idempotency_ttl_secs: 86_400,
            idempotency_cleanup_secs: 3600,
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
        }
    }

    async fn setup_state() -> Arc<AppState> {
        db::install_drivers();
        let db_name = format!("workspace_create_{}.db", Uuid::new_v4());
        let db_url = format!("sqlite:/tmp/{}?mode=rwc", db_name);
        let config = test_config(db_url);
        let pool = db::create_pool(&config).await.unwrap();
        db::run_migrations(&pool, config.database_backend())
            .await
            .unwrap();
        Arc::new(AppState::new(pool, Hub::new(), config))
    }

    async fn seed_device_and_host(state: &AppState) -> (Uuid, Uuid) {
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();

        sqlx::query(
            r#"INSERT INTO client_devices (device_id, device_name, device_type, server_url)
               VALUES ($1, 'device', 'desktop', 'http://localhost')"#,
        )
        .bind(device_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            r#"INSERT INTO hosts (host_id, host_name, platform, pair_status)
               VALUES ($1, 'host', 'linux', 'paired')"#,
        )
        .bind(host_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            r#"INSERT INTO device_host_access (device_id, host_id)
               VALUES ($1, $2)"#,
        )
        .bind(device_id.to_string())
        .bind(host_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

        (device_id, host_id)
    }

    fn client_claims(device_id: Uuid) -> Claims {
        Claims::for_client(device_id, "device", chrono::Duration::hours(1))
    }

    async fn register_fake_daemon(state: &AppState, host_id: Uuid) -> mpsc::Receiver<WsEnvelope> {
        let (tx, rx): (WsSender, mpsc::Receiver<WsEnvelope>) = mpsc::channel(8);
        state.hub.register_daemon(host_id, tx).await;
        rx
    }

    #[tokio::test]
    async fn create_workspace_waits_for_daemon_ack_before_persisting() {
        let state = setup_state().await;
        let (device_id, host_id) = seed_device_and_host(&state).await;
        let mut daemon_rx = register_fake_daemon(&state, host_id).await;

        let req_path = format!("/tmp/ws-{}", Uuid::new_v4());
        let state_for_request = state.clone();
        let req_path_for_request = req_path.clone();
        let request_task = tokio::spawn(async move {
            create_workspace(
                ClientAccess { device_id },
                State(state_for_request),
                Json(CreateWorkspaceRequest {
                    host_id,
                    path: req_path_for_request,
                    display_name: Some("ws".to_string()),
                }),
            )
            .await
        });

        let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
            .await
            .expect("timed out waiting for ensure_workspace command")
            .expect("daemon command");
        assert_eq!(outbound.r#type, "ensure_workspace");

        let row_count_before: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE host_id = $1 AND path = $2")
                .bind(host_id.to_string())
                .bind(&req_path)
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(row_count_before.0, 0);
        assert!(!request_task.is_finished());

        state
            .hub
            .complete_with_ack(AckPayload {
                request_id: outbound.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = tokio::time::timeout(Duration::from_secs(1), request_task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        assert!(response.0.exists_on_host);
        assert_eq!(response.0.path, req_path);

        let row_count_after: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE host_id = $1 AND path = $2")
                .bind(host_id.to_string())
                .bind(&response.0.path)
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(row_count_after.0, 1);
    }

    #[tokio::test]
    async fn create_workspace_returns_bad_request_when_daemon_rejects_path() {
        let state = setup_state().await;
        let (device_id, host_id) = seed_device_and_host(&state).await;
        let mut daemon_rx = register_fake_daemon(&state, host_id).await;

        let req_path = format!("/tmp/ws-{}", Uuid::new_v4());
        let state_for_request = state.clone();
        let req_path_for_request = req_path.clone();
        let request_task = tokio::spawn(async move {
            create_workspace(
                ClientAccess { device_id },
                State(state_for_request),
                Json(CreateWorkspaceRequest {
                    host_id,
                    path: req_path_for_request,
                    display_name: Some("ws".to_string()),
                }),
            )
            .await
        });

        let outbound = tokio::time::timeout(Duration::from_secs(1), daemon_rx.recv())
            .await
            .expect("timed out waiting for ensure_workspace command")
            .expect("daemon command");
        assert_eq!(outbound.r#type, "ensure_workspace");

        state
            .hub
            .complete_with_error(ErrorPayload {
                request_id: outbound.request_id.clone().unwrap(),
                error_code: "WORKSPACE_INVALID".to_string(),
                error_message: "path rejected".to_string(),
            })
            .await;

        let error = tokio::time::timeout(Duration::from_secs(1), request_task)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();

        assert!(matches!(error, ServerError::BadRequest(_)));

        let row_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE host_id = $1 AND path = $2")
                .bind(host_id.to_string())
                .bind(&req_path)
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(row_count.0, 0);
    }

    #[test]
    fn client_claims_are_formal_client_tokens_for_workspace_tests() {
        let claims = client_claims(Uuid::new_v4());
        assert_eq!(claims.r#type, TokenType::Client);
    }
}
