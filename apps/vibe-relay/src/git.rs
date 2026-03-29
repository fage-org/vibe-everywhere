use super::*;
use tokio::time::{Duration, Instant, sleep};
use vibe_core::{
    ClaimGitOperationResponse, CompleteGitOperationRequest, DeviceCapability, GitInspectRequest,
    GitInspectResponse, GitOperationRequest, GitOperationResult, now_epoch_millis,
};

const GIT_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const GIT_REQUEST_POLL_INTERVAL: Duration = Duration::from_millis(150);
const GIT_REQUEST_TTL_MS: u64 = 5 * 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GitRequestStatus {
    Pending,
    Claimed,
    Completed,
}

#[derive(Debug, Clone)]
pub(super) struct GitRequestEntry {
    pub(super) request: GitOperationRequest,
    status: GitRequestStatus,
    result: Option<GitOperationResult>,
    created_at_epoch_ms: u64,
    updated_at_epoch_ms: u64,
}

pub(super) async fn inspect_git_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GitInspectRequest>,
) -> Result<Json<GitInspectResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;
    ensure_git_capability(&state, &actor, &payload.device_id).await?;

    let result = submit_git_request(
        &state,
        GitOperationRequest::Inspect {
            id: Uuid::new_v4().to_string(),
            device_id: payload.device_id,
            session_cwd: payload.session_cwd,
        },
    )
    .await?;

    match result {
        GitOperationResult::Inspect { response } => Ok(Json(response)),
        GitOperationResult::Error { message } => {
            Err(ApiError::bad_request("git_inspect_failed", message))
        }
    }
}

pub(super) async fn claim_next_git_request(
    Path(device_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ClaimGitOperationResponse>, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;
    ensure_authenticated_device_matches(&auth, &device_id)?;
    {
        let store = state.store.read().await;
        let Some(device) = store.devices.get(&device_id) else {
            return Err(ApiError::not_found("device_not_found", "Device not found"));
        };
        ensure_tenant_access(&auth.actor, &device.tenant_id)?;
    }

    cleanup_expired_git_requests(&state.git_requests).await;

    let mut requests = state.git_requests.write().await;
    let now = now_epoch_millis();
    let request = requests
        .values_mut()
        .filter(|entry| {
            entry.request.device_id() == device_id && entry.status == GitRequestStatus::Pending
        })
        .min_by_key(|entry| entry.created_at_epoch_ms)
        .map(|entry| {
            entry.status = GitRequestStatus::Claimed;
            entry.updated_at_epoch_ms = now;
            entry.request.clone()
        });

    Ok(Json(ClaimGitOperationResponse { request }))
}

pub(super) async fn complete_git_request(
    Path(request_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CompleteGitOperationRequest>,
) -> Result<StatusCode, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;
    if payload.device_id != auth.device_id {
        return Err(ApiError::forbidden(
            "device_forbidden",
            "The current device credential cannot complete another device Git request",
        ));
    }
    let mut requests = state.git_requests.write().await;
    let Some(entry) = requests.get_mut(&request_id) else {
        return Err(ApiError::not_found(
            "git_request_not_found",
            "Git request not found",
        ));
    };

    if entry.request.device_id() != payload.device_id {
        return Err(ApiError::bad_request(
            "git_device_mismatch",
            "Git request device does not match completion source",
        ));
    }

    entry.status = GitRequestStatus::Completed;
    entry.result = Some(payload.result);
    entry.updated_at_epoch_ms = now_epoch_millis();

    Ok(StatusCode::NO_CONTENT)
}

async fn ensure_git_capability(
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
        .any(|capability| matches!(capability, DeviceCapability::GitInspect))
    {
        return Ok(());
    }

    Err(ApiError::conflict(
        "git_inspect_unavailable",
        "Device does not advertise Git inspect capability",
    ))
}

async fn submit_git_request(
    state: &AppState,
    request: GitOperationRequest,
) -> Result<GitOperationResult, ApiError> {
    cleanup_expired_git_requests(&state.git_requests).await;

    let request_id = request.id().to_string();
    let now = now_epoch_millis();
    {
        let mut requests = state.git_requests.write().await;
        requests.insert(
            request_id.clone(),
            GitRequestEntry {
                request,
                status: GitRequestStatus::Pending,
                result: None,
                created_at_epoch_ms: now,
                updated_at_epoch_ms: now,
            },
        );
    }

    let started_at = Instant::now();
    loop {
        {
            let requests = state.git_requests.read().await;
            if let Some(entry) = requests.get(&request_id) {
                if let Some(result) = entry.result.clone() {
                    drop(requests);
                    state.git_requests.write().await.remove(&request_id);
                    return Ok(result);
                }
            }
        }

        if started_at.elapsed() >= GIT_REQUEST_TIMEOUT {
            return Err(ApiError::conflict(
                "git_request_timeout",
                "Git request timed out waiting for the agent",
            ));
        }

        sleep(GIT_REQUEST_POLL_INTERVAL).await;
    }
}

async fn cleanup_expired_git_requests(requests: &Arc<RwLock<HashMap<String, GitRequestEntry>>>) {
    let now = now_epoch_millis();
    requests.write().await.retain(|_, entry| {
        if entry.status == GitRequestStatus::Completed {
            now.saturating_sub(entry.updated_at_epoch_ms) <= GIT_REQUEST_TTL_MS
        } else {
            now.saturating_sub(entry.created_at_epoch_ms) <= GIT_REQUEST_TTL_MS
        }
    });
}
