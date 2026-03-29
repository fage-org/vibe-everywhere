use super::*;
use tokio::time::{Duration, Instant, sleep};
use vibe_core::{
    ClaimWorkspaceOperationResponse, CompleteWorkspaceOperationRequest, DeviceCapability,
    WorkspaceBrowseRequest, WorkspaceBrowseResponse, WorkspaceFilePreviewRequest,
    WorkspaceFilePreviewResponse, WorkspaceOperationRequest, WorkspaceOperationResult,
    now_epoch_millis,
};

const WORKSPACE_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const WORKSPACE_REQUEST_POLL_INTERVAL: Duration = Duration::from_millis(150);
const WORKSPACE_REQUEST_TTL_MS: u64 = 5 * 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceRequestStatus {
    Pending,
    Claimed,
    Completed,
}

#[derive(Debug, Clone)]
pub(super) struct WorkspaceRequestEntry {
    pub(super) request: WorkspaceOperationRequest,
    status: WorkspaceRequestStatus,
    result: Option<WorkspaceOperationResult>,
    created_at_epoch_ms: u64,
    updated_at_epoch_ms: u64,
}

pub(super) async fn browse_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<WorkspaceBrowseRequest>,
) -> Result<Json<WorkspaceBrowseResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;
    ensure_workspace_capability(&state, &actor, &payload.device_id).await?;

    let result = submit_workspace_request(
        &state,
        WorkspaceOperationRequest::Browse {
            id: Uuid::new_v4().to_string(),
            device_id: payload.device_id,
            session_cwd: payload.session_cwd,
            path: payload.path,
        },
    )
    .await?;

    match result {
        WorkspaceOperationResult::Browse { response } => Ok(Json(response)),
        WorkspaceOperationResult::Error { message } => {
            Err(ApiError::bad_request("workspace_browse_failed", message))
        }
        WorkspaceOperationResult::Preview { .. } => Err(ApiError::internal(
            "workspace_response_invalid",
            "Unexpected workspace response type",
        )),
    }
}

pub(super) async fn preview_workspace_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<WorkspaceFilePreviewRequest>,
) -> Result<Json<WorkspaceFilePreviewResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;
    ensure_workspace_capability(&state, &actor, &payload.device_id).await?;

    let result = submit_workspace_request(
        &state,
        WorkspaceOperationRequest::Preview {
            id: Uuid::new_v4().to_string(),
            device_id: payload.device_id,
            session_cwd: payload.session_cwd,
            path: payload.path,
            line: payload.line,
            limit: payload.limit,
        },
    )
    .await?;

    match result {
        WorkspaceOperationResult::Preview { response } => Ok(Json(response)),
        WorkspaceOperationResult::Error { message } => {
            Err(ApiError::bad_request("workspace_preview_failed", message))
        }
        WorkspaceOperationResult::Browse { .. } => Err(ApiError::internal(
            "workspace_response_invalid",
            "Unexpected workspace response type",
        )),
    }
}

pub(super) async fn claim_next_workspace_request(
    Path(device_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ClaimWorkspaceOperationResponse>, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;
    ensure_authenticated_device_matches(&auth, &device_id)?;
    {
        let store = state.store.read().await;
        let Some(device) = store.devices.get(&device_id) else {
            return Err(ApiError::not_found("device_not_found", "Device not found"));
        };
        ensure_tenant_access(&auth.actor, &device.tenant_id)?;
    }

    cleanup_expired_workspace_requests(&state.workspace_requests).await;

    let mut requests = state.workspace_requests.write().await;
    let now = now_epoch_millis();
    let request = requests
        .values_mut()
        .filter(|entry| {
            entry.request.device_id() == device_id
                && entry.status == WorkspaceRequestStatus::Pending
        })
        .min_by_key(|entry| entry.created_at_epoch_ms)
        .map(|entry| {
            entry.status = WorkspaceRequestStatus::Claimed;
            entry.updated_at_epoch_ms = now;
            entry.request.clone()
        });

    Ok(Json(ClaimWorkspaceOperationResponse { request }))
}

pub(super) async fn complete_workspace_request(
    Path(request_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CompleteWorkspaceOperationRequest>,
) -> Result<StatusCode, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;
    if payload.device_id != auth.device_id {
        return Err(ApiError::forbidden(
            "device_forbidden",
            "The current device credential cannot complete another device workspace request",
        ));
    }
    let mut requests = state.workspace_requests.write().await;
    let Some(entry) = requests.get_mut(&request_id) else {
        return Err(ApiError::not_found(
            "workspace_request_not_found",
            "Workspace request not found",
        ));
    };

    if entry.request.device_id() != payload.device_id {
        return Err(ApiError::bad_request(
            "workspace_device_mismatch",
            "Workspace request device does not match completion source",
        ));
    }

    entry.status = WorkspaceRequestStatus::Completed;
    entry.result = Some(payload.result);
    entry.updated_at_epoch_ms = now_epoch_millis();

    Ok(StatusCode::NO_CONTENT)
}

async fn ensure_workspace_capability(
    state: &AppState,
    actor: &ActorIdentity,
    device_id: &str,
) -> Result<(), ApiError> {
    let store = state.store.read().await;
    let Some(device) = store.devices.get(device_id) else {
        return Err(ApiError::not_found("device_not_found", "Device not found"));
    };
    ensure_tenant_access(actor, &device.tenant_id)?;

    if device
        .capabilities
        .iter()
        .any(|capability| matches!(capability, DeviceCapability::WorkspaceBrowse))
    {
        return Ok(());
    }

    Err(ApiError::conflict(
        "workspace_browse_unavailable",
        "Device does not advertise workspace browse capability",
    ))
}

async fn submit_workspace_request(
    state: &AppState,
    request: WorkspaceOperationRequest,
) -> Result<WorkspaceOperationResult, ApiError> {
    cleanup_expired_workspace_requests(&state.workspace_requests).await;

    let request_id = request.id().to_string();
    let now = now_epoch_millis();
    {
        let mut requests = state.workspace_requests.write().await;
        requests.insert(
            request_id.clone(),
            WorkspaceRequestEntry {
                request,
                status: WorkspaceRequestStatus::Pending,
                result: None,
                created_at_epoch_ms: now,
                updated_at_epoch_ms: now,
            },
        );
    }

    let started_at = Instant::now();
    loop {
        {
            let requests = state.workspace_requests.read().await;
            if let Some(entry) = requests.get(&request_id) {
                if let Some(result) = entry.result.clone() {
                    drop(requests);
                    state.workspace_requests.write().await.remove(&request_id);
                    return Ok(result);
                }
            }
        }

        if started_at.elapsed() >= WORKSPACE_REQUEST_TIMEOUT {
            return Err(ApiError::conflict(
                "workspace_request_timeout",
                "Workspace request timed out waiting for the agent",
            ));
        }

        sleep(WORKSPACE_REQUEST_POLL_INTERVAL).await;
    }
}

async fn cleanup_expired_workspace_requests(
    requests: &Arc<RwLock<HashMap<String, WorkspaceRequestEntry>>>,
) {
    let now = now_epoch_millis();
    requests.write().await.retain(|_, entry| {
        if entry.status == WorkspaceRequestStatus::Completed {
            now.saturating_sub(entry.updated_at_epoch_ms) <= WORKSPACE_REQUEST_TTL_MS
        } else {
            now.saturating_sub(entry.created_at_epoch_ms) <= WORKSPACE_REQUEST_TTL_MS
        }
    });
}
