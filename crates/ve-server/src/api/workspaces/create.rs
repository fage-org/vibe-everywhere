//! Workspace create endpoint

use std::time::Duration;

use axum::Json;
use axum::extract::State;
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::{CreateWorkspaceRequest, Workspace};
use ve_shared::proto::{DaemonMessage, ErrorPayload};

use super::Result;
use crate::authz::{authorize_workspace_create, ClientAccess};
use crate::error::ServerError;
use crate::hub::DaemonResponse;
use crate::state::AppState;
use crate::utils::generate_request_id;
use crate::validation::{validate_workspace_display_name, validate_workspace_path};

/// POST /api/workspaces
///
/// Create a new workspace.
pub async fn create_workspace(
    client: ClientAccess,
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWorkspaceRequest>,
) -> Result<Json<Workspace>> {
    validate_workspace_path(&req.path)?;
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
    validate_workspace_display_name(&display_name)?;

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

pub fn sanitize_workspace_transport_error(error: &dyn std::error::Error) -> ServerError {
    tracing::warn!(error = %error, "Sanitized workspace preparation transport failure");
    ServerError::BadRequest("Workspace could not be prepared".to_string())
}

pub fn sanitize_workspace_operation_error(
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

pub fn sanitize_workspace_operation_error_from_payload(error: &ErrorPayload) -> ServerError {
    sanitize_workspace_operation_error(Some(&error.error_code), &error.error_message)
}
