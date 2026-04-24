//! File API Handlers
//!
//! REST API endpoints for file tree and file content access.

use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};
use uuid::Uuid;

use ve_shared::jwt::Claims;
use ve_shared::models::{FileContent, FileTreeNode};
use ve_shared::proto::{DaemonMessage, ErrorPayload};

use crate::authz::{require_client_device_id, require_host_access, HostAccess};
use crate::error::{ApiResponse, Result, ServerError};
use crate::hub::DaemonResponse;
use crate::state::AppState;
use crate::utils::generate_request_id;

/// Query parameters for file tree request
#[derive(Debug, Deserialize)]
pub struct FileTreeQuery {
    /// Authorized workspace identity
    pub workspace_id: Uuid,
    /// Optional path within the workspace
    pub path: Option<String>,
}

/// File tree response
#[derive(Debug, Serialize)]
pub struct FileTreeResponse {
    pub tree: FileTreeNode,
}

/// Query parameters for file content request
#[derive(Debug, Deserialize)]
pub struct FileContentQuery {
    /// Authorized workspace identity
    pub workspace_id: Uuid,
    /// File path within workspace
    pub path: String,
}

struct AuthorizedWorkspace {
    path: String,
}

/// Get file tree for a host's workspace
///
/// GET /api/hosts/:host_id/files/tree
pub async fn get_file_tree_route(
    access: HostAccess,
    State(state): State<Arc<AppState>>,
    Query(query): Query<FileTreeQuery>,
) -> Result<Json<ApiResponse<FileTreeResponse>>> {
    get_file_tree_for_host(state, access.host_id, query).await
}

pub async fn get_file_tree(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(host_id): Path<Uuid>,
    Query(query): Query<FileTreeQuery>,
) -> Result<Json<ApiResponse<FileTreeResponse>>> {
    let device_id = require_client_device_id(&claims)?;
    require_host_access(&state, device_id, host_id).await?;
    get_file_tree_for_host(state, host_id, query).await
}

async fn get_file_tree_for_host(
    state: Arc<AppState>,
    host_id: Uuid,
    query: FileTreeQuery,
) -> Result<Json<ApiResponse<FileTreeResponse>>> {
    debug!(%host_id, workspace_id = %query.workspace_id, ?query.path, "File tree request");

    if !state.hub.is_daemon_connected(&host_id).await {
        return Err(ServerError::HostNotFound);
    }

    let workspace = load_authorized_workspace(&state, host_id, query.workspace_id).await?;
    let request_id = generate_request_id();
    let request = DaemonMessage::FileTreeRequest {
        request_id: request_id.clone(),
        session_id: Uuid::nil(),
        workspace_path: workspace.path,
        relative_path: query.path.filter(|path| !path.is_empty()),
    };

    let response = state
        .hub
        .send_and_wait(&host_id, request, request_id, Duration::from_secs(30))
        .await
        .map_err(|error| sanitize_file_transport_error("file tree", error.as_ref()))?;

    match response {
        DaemonResponse::Message(ve_shared::proto::DaemonToServer::FileTreeResponse {
            tree,
            ..
        }) => {
            let node: FileTreeNode = serde_json::from_value(tree)
                .map_err(|e| ServerError::Internal(format!("Failed to parse file tree: {e}")))?;
            Ok(Json(ApiResponse::success(FileTreeResponse { tree: node })))
        }
        DaemonResponse::Message(ve_shared::proto::DaemonToServer::Error {
            error_code,
            error_message,
            ..
        }) => Err(sanitize_file_operation_error(
            "file tree",
            Some(&error_code),
            &error_message,
        )),
        DaemonResponse::Error(error) => Err(sanitize_file_operation_error_from_payload(
            "file tree",
            &error,
        )),
        DaemonResponse::Ack(_) => Err(ServerError::Internal("Unexpected ACK response type".into())),
        _ => Err(ServerError::Internal("Unexpected response type".into())),
    }
}

/// Get file content
///
/// GET /api/hosts/:host_id/files/content
pub async fn get_file_content_route(
    access: HostAccess,
    State(state): State<Arc<AppState>>,
    Query(query): Query<FileContentQuery>,
) -> Result<Json<ApiResponse<FileContent>>> {
    get_file_content_for_host(state, access.host_id, query).await
}

pub async fn get_file_content(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(host_id): Path<Uuid>,
    Query(query): Query<FileContentQuery>,
) -> Result<Json<ApiResponse<FileContent>>> {
    let device_id = require_client_device_id(&claims)?;
    require_host_access(&state, device_id, host_id).await?;
    get_file_content_for_host(state, host_id, query).await
}

async fn get_file_content_for_host(
    state: Arc<AppState>,
    host_id: Uuid,
    query: FileContentQuery,
) -> Result<Json<ApiResponse<FileContent>>> {
    debug!(
        %host_id,
        workspace_id = %query.workspace_id,
        path = %query.path,
        "File content request"
    );

    if query.path.is_empty() {
        return Err(ServerError::BadRequest("path parameter is required".into()));
    }

    if !state.hub.is_daemon_connected(&host_id).await {
        return Err(ServerError::HostNotFound);
    }

    let workspace = load_authorized_workspace(&state, host_id, query.workspace_id).await?;
    let request_id = generate_request_id();
    let request = DaemonMessage::FileContentRequest {
        request_id: request_id.clone(),
        workspace_path: workspace.path,
        relative_path: query.path.clone(),
    };

    let response = state
        .hub
        .send_and_wait(&host_id, request, request_id, Duration::from_secs(30))
        .await
        .map_err(|error| sanitize_file_transport_error("file content", error.as_ref()))?;

    match response {
        DaemonResponse::Message(ve_shared::proto::DaemonToServer::FileContentResponse {
            file_path,
            content,
            file_type,
            truncated,
            total_size,
            ..
        }) => {
            let file_type = match file_type.as_str() {
                "text" => ve_shared::models::FileType::Text,
                "binary" => ve_shared::models::FileType::Binary,
                _ => ve_shared::models::FileType::Unknown,
            };
            Ok(Json(ApiResponse::success(FileContent {
                path: file_path,
                content,
                file_type,
                truncated,
                total_size,
            })))
        }
        DaemonResponse::Message(ve_shared::proto::DaemonToServer::Error {
            error_code,
            error_message,
            ..
        }) => Err(sanitize_file_operation_error(
            "file content",
            Some(&error_code),
            &error_message,
        )),
        DaemonResponse::Error(error) => Err(sanitize_file_operation_error_from_payload(
            "file content",
            &error,
        )),
        DaemonResponse::Ack(_) => Err(ServerError::Internal("Unexpected ACK response type".into())),
        _ => Err(ServerError::Internal("Unexpected response type".into())),
    }
}

fn sanitize_file_transport_error(
    operation: &'static str,
    error: &dyn std::error::Error,
) -> ServerError {
    warn!(operation, error = %error, "Sanitized daemon file transport failure");
    ServerError::BadRequest("File operation failed".to_string())
}

fn sanitize_file_operation_error(
    operation: &'static str,
    error_code: Option<&str>,
    raw_error_message: &str,
) -> ServerError {
    warn!(
        operation,
        error_code, raw_error_message, "Sanitized daemon file operation error response"
    );

    let safe_message = match error_code {
        Some("WORKSPACE_INVALID") => "Workspace is not available for this file operation",
        Some("FORBIDDEN") => "Access to the requested path is not allowed",
        Some("INVALID_INPUT") => "The requested file operation is invalid",
        _ => "File operation failed",
    };

    ServerError::BadRequest(safe_message.to_string())
}

fn sanitize_file_operation_error_from_payload(
    operation: &'static str,
    error: &ErrorPayload,
) -> ServerError {
    sanitize_file_operation_error(operation, Some(&error.error_code), &error.error_message)
}

async fn load_authorized_workspace(
    state: &AppState,
    host_id: Uuid,
    workspace_id: Uuid,
) -> Result<AuthorizedWorkspace> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT path
        FROM workspaces
        WHERE workspace_id = $1 AND host_id = $2
        "#,
    )
    .bind(workspace_id.to_string())
    .bind(host_id.to_string())
    .fetch_optional(&state.db)
    .await?;

    let (path,) = row.ok_or_else(|| {
        ServerError::NotFound(format!(
            "Workspace {} not found for host {}",
            workspace_id, host_id
        ))
    })?;

    Ok(AuthorizedWorkspace { path })
}
