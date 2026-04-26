//! Authorization helpers for device-scoped host/session access.

use std::sync::Arc;

use axum::{
    extract::{Extension, FromRef, FromRequestParts, Path, Query},
    http::request::Parts,
    RequestPartsExt,
};
use serde::Deserialize;
use uuid::Uuid;
use ve_shared::{
    jwt::{Claims, JwtManager, TokenType},
    models::{CreateSessionRequest, CreateWorkspaceRequest},
};

use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::parse_uuid;

#[derive(Debug, Clone, Copy)]
pub struct ClientAccess {
    pub device_id: Uuid,
}

#[derive(Debug, Clone, Copy)]
pub struct HostCollectionAccess {
    pub device_id: Uuid,
}

#[derive(Debug, Clone, Copy)]
pub struct WorkspaceCollectionAccess {
    pub device_id: Uuid,
    pub host_id: Option<Uuid>,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionCollectionAccess {
    pub device_id: Uuid,
    pub host_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy)]
pub struct PermissionCollectionAccess {
    pub device_id: Uuid,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy)]
pub struct ArchiveCollectionAccess {
    pub device_id: Uuid,
    pub host_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy)]
pub struct ArchiveAccess {
    pub device_id: Uuid,
    pub archive_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct HostScopedQuery {
    host_id: Option<Uuid>,
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

#[derive(Debug, Deserialize)]
struct SessionScopedQuery {
    session_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy)]
pub struct HostAccess {
    pub device_id: Uuid,
    pub host_id: Uuid,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionAccess {
    pub device_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct WorkspaceAccess {
    pub device_id: Uuid,
    pub workspace_id: Uuid,
    pub host_id: Uuid,
    pub path: String,
    pub display_name: String,
    pub is_favorited: bool,
    pub last_used_at: Option<String>,
    pub exists_on_host: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct PermissionAccess {
    pub device_id: Uuid,
    pub permission_id: Uuid,
    pub session_id: Uuid,
    pub risk_type: String,
    pub summary: String,
    pub target: Option<String>,
    pub status: String,
    pub created_at: String,
    pub responded_at: Option<String>,
}

pub fn require_client_device_id(claims: &Claims) -> Result<Uuid> {
    if claims.r#type != TokenType::Client {
        return Err(ServerError::Unauthorized);
    }

    claims.subject_uuid().map_err(|_| ServerError::InvalidToken)
}

pub fn require_bootstrap_device_id(claims: &Claims) -> Result<Uuid> {
    if claims.r#type != TokenType::ClientBootstrap {
        return Err(ServerError::Unauthorized);
    }

    claims.subject_uuid().map_err(|_| ServerError::InvalidToken)
}

pub fn require_daemon_host_id(claims: &Claims) -> Result<Uuid> {
    if claims.r#type != TokenType::Daemon {
        return Err(ServerError::Unauthorized);
    }

    claims.subject_uuid().map_err(|_| ServerError::InvalidToken)
}

pub async fn decode_ws_claims(jwt_manager: &JwtManager, token: &str, db: &crate::db::DbPool) -> Result<Claims> {
    let claims = jwt_manager
        .decode(token)
        .map_err(|_| ServerError::InvalidToken)?;

    if claims.is_expired() {
        return Err(ServerError::TokenExpired);
    }

    // Check token revocation for Client-type tokens
    if claims.r#type == TokenType::Client {
        if let Ok(device_id) = claims.subject_uuid() {
            let matches = crate::token_revocation::jti_matches_device(db, device_id, &claims.jti)
                .await
                .unwrap_or(true);
            if !matches {
                return Err(ServerError::InvalidToken);
            }
            let revoked = crate::token_revocation::is_revoked(db, &claims.jti)
                .await
                .unwrap_or(false);
            if revoked {
                return Err(ServerError::InvalidToken);
            }
        }
    }

    Ok(claims)
}

pub async fn require_host_access(state: &AppState, device_id: Uuid, host_id: Uuid) -> Result<()> {
    let access = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT 1
        FROM device_host_access
        WHERE device_id = $1 AND host_id = $2
        "#,
    )
    .bind(device_id.to_string())
    .bind(host_id.to_string())
    .fetch_optional(&state.db)
    .await?;

    if access.is_some() {
        Ok(())
    } else {
        Err(ServerError::NotFound(format!("Host {}", host_id)))
    }
}

pub async fn require_session_access(
    state: &AppState,
    device_id: Uuid,
    session_id: Uuid,
) -> Result<()> {
    let access = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT 1
        FROM device_session_access
        WHERE device_id = $1 AND session_id = $2
        "#,
    )
    .bind(device_id.to_string())
    .bind(session_id.to_string())
    .fetch_optional(&state.db)
    .await?;

    if access.is_some() {
        Ok(())
    } else {
        Err(ServerError::NotFound(format!("Session {}", session_id)))
    }
}

pub async fn require_workspace_for_host(
    state: &AppState,
    workspace_id: Uuid,
    host_id: Uuid,
) -> Result<()> {
    let workspace = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT 1
        FROM workspaces
        WHERE workspace_id = $1 AND host_id = $2
        "#,
    )
    .bind(workspace_id.to_string())
    .bind(host_id.to_string())
    .fetch_optional(&state.db)
    .await?;

    if workspace.is_some() {
        Ok(())
    } else {
        Err(ServerError::NotFound(format!("Workspace {}", workspace_id)))
    }
}

pub async fn authorize_workspace_create(
    state: &AppState,
    device_id: Uuid,
    request: &CreateWorkspaceRequest,
) -> Result<()> {
    require_host_access(state, device_id, request.host_id).await
}

pub async fn authorize_session_create(
    state: &AppState,
    device_id: Uuid,
    request: &CreateSessionRequest,
) -> Result<()> {
    require_host_access(state, device_id, request.host_id).await?;
    require_workspace_for_host(state, request.workspace_id, request.host_id).await
}

async fn extract_client_device_id<S>(parts: &mut Parts, state: &S) -> Result<Uuid>
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    let Extension(claims) = parts
        .extract::<Extension<Claims>>()
        .await
        .map_err(|_| ServerError::Unauthorized)?;
    let device_id = require_client_device_id(&claims)?;
    let app_state = Arc::<AppState>::from_ref(state);
    // Verify device still exists in DB (JWT may outlive device deletion)
    sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM client_devices WHERE device_id = $1",
    )
    .bind(device_id.to_string())
    .fetch_optional(&app_state.db)
    .await?
    .ok_or(ServerError::Unauthorized)?;
    Ok(device_id)
}

impl<S> FromRequestParts<S> for ClientAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        Ok(Self { device_id })
    }
}

impl<S> FromRequestParts<S> for HostCollectionAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        Ok(Self { device_id })
    }
}

impl<S> FromRequestParts<S> for WorkspaceCollectionAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        let Query(query) = parts
            .extract::<Query<HostScopedQuery>>()
            .await
            .map_err(|_| ServerError::BadRequest("Invalid workspace query".to_string()))?;

        if let Some(host_id) = query.host_id {
            let app_state = Arc::<AppState>::from_ref(state);
            require_host_access(&app_state, device_id, host_id).await?;
        }

        Ok(Self {
            device_id,
            host_id: query.host_id,
            page: query.page,
            limit: query.limit,
        })
    }
}

impl<S> FromRequestParts<S> for SessionCollectionAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        let Query(query) = parts
            .extract::<Query<HostScopedQuery>>()
            .await
            .map_err(|_| ServerError::BadRequest("Invalid session query".to_string()))?;

        if let Some(host_id) = query.host_id {
            let app_state = Arc::<AppState>::from_ref(state);
            require_host_access(&app_state, device_id, host_id).await?;
        }

        Ok(Self {
            device_id,
            host_id: query.host_id,
        })
    }
}

impl<S> FromRequestParts<S> for PermissionCollectionAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        let Query(query) = parts
            .extract::<Query<SessionScopedQuery>>()
            .await
            .map_err(|_| ServerError::BadRequest("Invalid permission query".to_string()))?;

        if let Some(session_id) = query.session_id {
            let app_state = Arc::<AppState>::from_ref(state);
            require_session_access(&app_state, device_id, session_id).await?;
        }

        Ok(Self {
            device_id,
            session_id: query.session_id,
        })
    }
}

impl<S> FromRequestParts<S> for ArchiveCollectionAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        let Query(query) = parts
            .extract::<Query<HostScopedQuery>>()
            .await
            .map_err(|_| ServerError::BadRequest("Invalid archive query".to_string()))?;

        if let Some(host_id) = query.host_id {
            let app_state = Arc::<AppState>::from_ref(state);
            require_host_access(&app_state, device_id, host_id).await?;
        }

        Ok(Self {
            device_id,
            host_id: query.host_id,
        })
    }
}

impl<S> FromRequestParts<S> for HostAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        let Path(host_id) = parts
            .extract::<Path<Uuid>>()
            .await
            .map_err(|_| ServerError::BadRequest("Invalid host id".to_string()))?;
        let app_state = Arc::<AppState>::from_ref(state);
        require_host_access(&app_state, device_id, host_id).await?;

        Ok(Self { device_id, host_id })
    }
}

impl<S> FromRequestParts<S> for ArchiveAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        let Path(archive_id) = parts
            .extract::<Path<Uuid>>()
            .await
            .map_err(|_| ServerError::BadRequest("Invalid archive id".to_string()))?;
        let app_state = Arc::<AppState>::from_ref(state);
        let archive_session_id: (String,) = sqlx::query_as(
            r#"
            SELECT session_archives.session_id
            FROM session_archives
            INNER JOIN device_session_access
                ON device_session_access.session_id = session_archives.session_id
            WHERE session_archives.archive_id = $1
              AND device_session_access.device_id = $2
            "#,
        )
        .bind(archive_id.to_string())
        .bind(device_id.to_string())
        .fetch_optional(&app_state.db)
        .await?
        .ok_or(ServerError::NotFound(format!("Archive {}", archive_id)))?;
        let session_id = parse_uuid(&archive_session_id.0, "session_id")?;

        Ok(Self {
            device_id,
            archive_id,
            session_id,
        })
    }
}

impl<S> FromRequestParts<S> for SessionAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        let Path(session_id) = parts
            .extract::<Path<Uuid>>()
            .await
            .map_err(|_| ServerError::BadRequest("Invalid session id".to_string()))?;
        let app_state = Arc::<AppState>::from_ref(state);
        require_session_access(&app_state, device_id, session_id).await?;

        Ok(Self {
            device_id,
            session_id,
        })
    }
}

impl<S> FromRequestParts<S> for WorkspaceAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        let Path(workspace_id) = parts
            .extract::<Path<Uuid>>()
            .await
            .map_err(|_| ServerError::BadRequest("Invalid workspace id".to_string()))?;
        let app_state = Arc::<AppState>::from_ref(state);
        let workspace_row: (
            String,
            String,
            String,
            String,
            i64,
            Option<String>,
            i64,
            String,
            String,
        ) = sqlx::query_as(
            r#"
            SELECT workspaces.workspace_id, workspaces.host_id, workspaces.path, workspaces.display_name,
                   CASE WHEN workspaces.is_favorited THEN 1 ELSE 0 END,
                   CAST(workspaces.last_used_at AS TEXT),
                   CASE WHEN workspaces.exists_on_host THEN 1 ELSE 0 END,
                   CAST(workspaces.created_at AS TEXT),
                   CAST(workspaces.updated_at AS TEXT)
            FROM workspaces
            INNER JOIN device_host_access
                ON device_host_access.host_id = workspaces.host_id
            WHERE workspaces.workspace_id = $1
              AND device_host_access.device_id = $2
            "#,
        )
        .bind(workspace_id.to_string())
        .bind(device_id.to_string())
        .fetch_optional(&app_state.db)
        .await?
        .ok_or(ServerError::NotFound(format!("Workspace {}", workspace_id)))?;
        let host_id = parse_uuid(&workspace_row.1, "host_id")?;

        Ok(Self {
            device_id,
            workspace_id,
            host_id,
            path: workspace_row.2,
            display_name: workspace_row.3,
            is_favorited: workspace_row.4 != 0,
            last_used_at: workspace_row.5,
            exists_on_host: workspace_row.6 != 0,
            created_at: workspace_row.7,
            updated_at: workspace_row.8,
        })
    }
}

impl<S> FromRequestParts<S> for PermissionAccess
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let device_id = extract_client_device_id(parts, state).await?;
        let Path(permission_id) = parts
            .extract::<Path<Uuid>>()
            .await
            .map_err(|_| ServerError::BadRequest("Invalid permission id".to_string()))?;
        let app_state = Arc::<AppState>::from_ref(state);
        let permission_row: (
            String,
            String,
            String,
            String,
            Option<String>,
            String,
            String,
            Option<String>,
        ) = sqlx::query_as(
            r#"
            SELECT permission_requests.permission_id,
                   permission_requests.session_id,
                   permission_requests.risk_type,
                   permission_requests.summary,
                   permission_requests.target,
                   permission_requests.status,
                   CAST(permission_requests.created_at AS TEXT),
                   CAST(permission_requests.responded_at AS TEXT)
            FROM permission_requests
            INNER JOIN device_session_access
                ON device_session_access.session_id = permission_requests.session_id
            WHERE permission_requests.permission_id = $1
              AND device_session_access.device_id = $2
            "#,
        )
        .bind(permission_id.to_string())
        .bind(device_id.to_string())
        .fetch_optional(&app_state.db)
        .await?
        .ok_or(ServerError::NotFound(format!(
            "Permission {}",
            permission_id
        )))?;
        let session_id = parse_uuid(&permission_row.1, "session_id")?;

        Ok(Self {
            device_id,
            permission_id,
            session_id,
            risk_type: permission_row.2,
            summary: permission_row.3,
            target: permission_row.4,
            status: permission_row.5,
            created_at: permission_row.6,
            responded_at: permission_row.7,
        })
    }
}

pub async fn grant_session_access_to_host_devices(
    state: &AppState,
    host_id: Uuid,
    session_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO device_session_access (device_id, session_id)
        SELECT device_host_access.device_id, $2
        FROM device_host_access
        WHERE device_host_access.host_id = $1
          AND NOT EXISTS (
              SELECT 1
              FROM device_session_access
              WHERE device_id = device_host_access.device_id AND session_id = $2
          )
        "#,
    )
    .bind(host_id.to_string())
    .bind(session_id.to_string())
    .execute(&state.db)
    .await?;

    Ok(())
}
