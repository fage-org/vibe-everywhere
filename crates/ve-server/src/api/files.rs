//! File API Handlers
//!
//! REST API endpoints for file tree and file content access.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;
use uuid::Uuid;

use ve_shared::models::{FileContent, FileTreeNode};
use ve_shared::proto::DaemonMessage;

use crate::error::{ApiResponse, Result, ServerError};
use crate::state::AppState;
use crate::utils::generate_request_id;

/// Query parameters for file tree request
#[derive(Debug, Deserialize)]
pub struct FileTreeQuery {
    /// Path within workspace (optional, defaults to workspace root)
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
    /// File path within workspace
    pub path: String,
}

/// Get file tree for a host's workspace
///
/// GET /api/hosts/:host_id/files/tree
pub async fn get_file_tree(
    State(state): State<Arc<AppState>>,
    Path(host_id): Path<Uuid>,
    Query(query): Query<FileTreeQuery>,
) -> Result<Json<ApiResponse<FileTreeResponse>>> {
    debug!(%host_id, ?query, "File tree request");

    // Check if daemon is connected
    let daemon_sender = state
        .hub
        .get_daemon_sender(&host_id)
        .await
        .ok_or(ServerError::HostNotFound)?;

    let workspace_path = query.path.unwrap_or_default();

    // Generate request ID
    let request_id = generate_request_id();

    // Send request to daemon via WS
    let request = DaemonMessage::FileTreeRequest {
        request_id: request_id.clone(),
        session_id: Uuid::nil(), // Will use any available session
        workspace_path,
    };

    // Send and wait for response
    let response = state
        .hub
        .send_and_wait(&host_id, daemon_sender, request, request_id.clone(), Duration::from_secs(30))
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get response from daemon: {}", e)))?;

    // Parse response
    match response {
        ve_shared::proto::DaemonToServer::FileTreeResponse { tree, .. } => {
            let node: FileTreeNode = serde_json::from_value(tree)
                .map_err(|e| ServerError::Internal(format!("Failed to parse file tree: {}", e)))?;
            Ok(Json(ApiResponse::success(FileTreeResponse { tree: node })))
        }
        _ => Err(ServerError::Internal("Unexpected response type".into())),
    }
}

/// Get file content
///
/// GET /api/hosts/:host_id/files/content
pub async fn get_file_content(
    State(state): State<Arc<AppState>>,
    Path(host_id): Path<Uuid>,
    Query(query): Query<FileContentQuery>,
) -> Result<Json<ApiResponse<FileContent>>> {
    debug!(%host_id, path = %query.path, "File content request");

    if query.path.is_empty() {
        return Err(ServerError::BadRequest("path parameter is required".into()));
    }

    // Check if daemon is connected
    let daemon_sender = state
        .hub
        .get_daemon_sender(&host_id)
        .await
        .ok_or(ServerError::HostNotFound)?;

    // Generate request ID
    let request_id = generate_request_id();

    // Send request to daemon via WS
    let request = DaemonMessage::FileContentRequest {
        request_id: request_id.clone(),
        file_path: query.path.clone(),
    };

    // Send and wait for response
    let response = state
        .hub
        .send_and_wait(&host_id, daemon_sender, request, request_id.clone(), Duration::from_secs(30))
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get response from daemon: {}", e)))?;

    // Parse response
    match response {
        ve_shared::proto::DaemonToServer::FileContentResponse {
            request_id: _,
            file_path,
            content,
            file_type,
        } => {
            let ft = match file_type.as_str() {
                "text" => ve_shared::models::FileType::Text,
                "binary" => ve_shared::models::FileType::Binary,
                _ => ve_shared::models::FileType::Unknown,
            };
            let total_size = content.len() as u64;
            Ok(Json(ApiResponse::success(FileContent {
                path: file_path,
                content,
                file_type: ft,
                truncated: false,
                total_size,
            })))
        }
        _ => Err(ServerError::Internal("Unexpected response type".into())),
    }
}
