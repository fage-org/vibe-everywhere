//! Permission API Handlers
//!
//! Permission request query and response endpoints.
//!
//! All routes use the `PermissionAccess` / `PermissionCollectionAccess` extractors
//! for authorization, which handle device and session access verification in a
//! single database query. The older `get_permission` / `respond_permission`
//! functions that manually called `require_client_device_id` + `require_session_access`
//! have been removed — the `_route` versions are the canonical implementations.

use axum::{extract::State, Json};
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::{
    models::{PermissionDecision, PermissionRequest, PermissionResponseRequest},
    proto::DaemonMessage,
};

use crate::authz::{PermissionAccess, PermissionCollectionAccess};
use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::{self, parse_uuid};

#[cfg(debug_assertions)]
pub mod test_support {
    use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

    use tokio::sync::Barrier;
    use uuid::Uuid;

    #[derive(Clone)]
    struct PermissionResponseRaceHook {
        permission_ids: Vec<Uuid>,
        barrier: Arc<Barrier>,
    }

    static PERMISSION_RESPONSE_RACE_HOOK: OnceLock<Mutex<Option<PermissionResponseRaceHook>>> =
        OnceLock::new();
    static PERMISSION_RESPONSE_RACE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn hook_slot() -> &'static Mutex<Option<PermissionResponseRaceHook>> {
        PERMISSION_RESPONSE_RACE_HOOK.get_or_init(|| Mutex::new(None))
    }

    fn test_lock() -> &'static Mutex<()> {
        PERMISSION_RESPONSE_RACE_TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    pub struct PermissionResponseRaceHookGuard {
        _test_lock: MutexGuard<'static, ()>,
    }

    pub fn install_permission_response_race_hook(
        permission_ids: impl Into<Vec<Uuid>>,
    ) -> PermissionResponseRaceHookGuard {
        let test_lock = match test_lock().lock() {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut hook = match hook_slot().lock() {
            Ok(hook) => hook,
            Err(poisoned) => poisoned.into_inner(),
        };

        assert!(
            hook.is_none(),
            "permission response race hook already installed"
        );

        *hook = Some(PermissionResponseRaceHook {
            permission_ids: permission_ids.into(),
            barrier: Arc::new(Barrier::new(2)),
        });

        PermissionResponseRaceHookGuard {
            _test_lock: test_lock,
        }
    }

    pub async fn wait_for_permission_response_race(permission_id: Uuid) {
        let barrier = {
            let hook = match hook_slot().lock() {
                Ok(hook) => hook,
                Err(poisoned) => poisoned.into_inner(),
            };

            hook.as_ref()
                .filter(|hook| hook.permission_ids.contains(&permission_id))
                .map(|hook| hook.barrier.clone())
        };

        if let Some(barrier) = barrier {
            barrier.wait().await;
        }
    }

    impl Drop for PermissionResponseRaceHookGuard {
        fn drop(&mut self) {
            match hook_slot().lock() {
                Ok(mut hook) => *hook = None,
                Err(poisoned) => *poisoned.into_inner() = None,
            }
        }
    }
}

/// Permission list query parameters
#[derive(Debug)]
pub struct PermissionListQuery {
    pub session_id: Option<Uuid>,
    #[allow(dead_code)]
    pub status: Option<String>,
}

/// Database record for permission request
struct PermissionRecord {
    permission_id: String,
    session_id: String,
    risk_type: String,
    summary: String,
    target: Option<String>,
    status: String,
    created_at: String,
    responded_at: Option<String>,
}

type PermissionRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    String,
    Option<String>,
);

impl PermissionRecord {
    fn to_model(&self) -> Result<PermissionRequest> {
        Ok(PermissionRequest {
            permission_id: parse_uuid(&self.permission_id, "permission_id")?,
            session_id: parse_uuid(&self.session_id, "session_id")?,
            risk_type: utils::parse_risk_type(&self.risk_type),
            summary: self.summary.clone(),
            target: self.target.clone(),
            created_at: utils::parse_sqlite_timestamp(&self.created_at)
                .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            status: utils::parse_permission_status(&self.status),
            responded_at: self.responded_at.as_ref().and_then(|s| {
                utils::parse_sqlite_timestamp(s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            }),
        })
    }
}

fn permission_record_from_row(row: PermissionRow) -> PermissionRecord {
    let (permission_id, session_id, risk_type, summary, target, status, created_at, responded_at) =
        row;
    PermissionRecord {
        permission_id,
        session_id,
        risk_type,
        summary,
        target,
        status,
        created_at,
        responded_at,
    }
}

/// GET /api/permissions
///
/// List permission requests, optionally filtered.
pub async fn list_permissions(
    access: PermissionCollectionAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PermissionRequest>>> {
    let rows = if let Some(session_id) = access.session_id {
        let session_id_str = session_id.to_string();
        sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, Option<String>)>(
            r#"
            SELECT permission_id, session_id, risk_type, summary, target, status, CAST(created_at AS TEXT), CAST(responded_at AS TEXT)
            FROM permission_requests
            WHERE session_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(session_id_str)
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as::<_, (String, String, String, String, Option<String>, String, String, Option<String>)>(
            r#"
            SELECT permission_requests.permission_id, permission_requests.session_id, permission_requests.risk_type,
                   permission_requests.summary, permission_requests.target, permission_requests.status,
                   CAST(permission_requests.created_at AS TEXT), CAST(permission_requests.responded_at AS TEXT)
            FROM permission_requests
            INNER JOIN device_session_access
                ON device_session_access.session_id = permission_requests.session_id
            WHERE device_session_access.device_id = $1
            ORDER BY permission_requests.created_at DESC
            "#
        )
        .bind(access.device_id.to_string())
        .fetch_all(&state.db)
        .await?
    };

    let permissions: Result<Vec<PermissionRequest>> = rows
        .into_iter()
        .map(
            |(
                permission_id,
                session_id,
                risk_type,
                summary,
                target,
                status,
                created_at,
                responded_at,
            )| {
                PermissionRecord {
                    permission_id,
                    session_id,
                    risk_type,
                    summary,
                    target,
                    status,
                    created_at,
                    responded_at,
                }
                .to_model()
            },
        )
        .collect();

    Ok(Json(permissions?))
}

/// GET /api/permissions/:id
///
/// Get a specific permission request.
pub async fn get_permission_route(
    access: PermissionAccess,
    State(_state): State<Arc<AppState>>,
) -> Result<Json<PermissionRequest>> {
    Ok(Json(
        PermissionRecord {
            permission_id: access.permission_id.to_string(),
            session_id: access.session_id.to_string(),
            risk_type: access.risk_type,
            summary: access.summary,
            target: access.target,
            status: access.status,
            created_at: access.created_at,
            responded_at: access.responded_at,
        }
        .to_model()?,
    ))
}

/// Fetch a permission by ID without authorization check (internal use only).
#[allow(dead_code)]
async fn get_permission_by_id(state: Arc<AppState>, id: Uuid) -> Result<Json<PermissionRequest>> {
    let permission_id_str = id.to_string();

    let row: PermissionRow = sqlx::query_as(
        r#"
        SELECT permission_id, session_id, risk_type, summary, target, status, CAST(created_at AS TEXT), CAST(responded_at AS TEXT)
        FROM permission_requests
        WHERE permission_id = $1
        "#
    )
    .bind(permission_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Permission {}", id)))?;

    let record = permission_record_from_row(row);

    Ok(Json(record.to_model()?))
}

/// POST /api/permissions/:id/respond
///
/// Respond to a permission request (approve/deny).
pub async fn respond_permission_route(
    access: PermissionAccess,
    State(state): State<Arc<AppState>>,
    Json(req): Json<PermissionResponseRequest>,
) -> Result<Json<PermissionRequest>> {
    respond_permission_existing(
        state,
        access.permission_id,
        req,
        PermissionRecord {
            permission_id: access.permission_id.to_string(),
            session_id: access.session_id.to_string(),
            risk_type: access.risk_type,
            summary: access.summary,
            target: access.target,
            status: access.status,
            created_at: access.created_at,
            responded_at: access.responded_at,
        },
    )
    .await
}

async fn respond_permission_existing(
    state: Arc<AppState>,
    id: Uuid,
    req: PermissionResponseRequest,
    existing: PermissionRecord,
) -> Result<Json<PermissionRequest>> {
    let permission_id_str = id.to_string();

    // Check if already responded (idempotent - return current state)
    let current_status = utils::parse_permission_status(&existing.status);
    if current_status.is_responded() {
        tracing::info!(%id, "Permission already responded (idempotent)");
        return Ok(Json(existing.to_model()?));
    }

    let session_id = parse_uuid(&existing.session_id, "session_id")?;
    let session: (String,) = sqlx::query_as(
        r#"
        SELECT host_id FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&existing.session_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!(
        "Session {}",
        existing.session_id
    )))?;

    let host_id = parse_uuid(&session.0, "host_id")?;
    let session_id_str = existing.session_id.clone();
    let new_status = match req.decision {
        PermissionDecision::ApproveOnce => "approved_once",
        PermissionDecision::DenyOnce => "denied_once",
        PermissionDecision::ApproveSession => "approved_session",
    };

    let mut tx = state.db.begin().await?;

    #[cfg(debug_assertions)]
    test_support::wait_for_permission_response_race(id).await;

    let permission_update = sqlx::query(
        r#"
        UPDATE permission_requests
        SET status = $1, responded_at = CURRENT_TIMESTAMP
        WHERE permission_id = $2 AND status = 'pending'
        "#,
    )
    .bind(new_status)
    .bind(&permission_id_str)
    .execute(&mut *tx)
    .await?;

    if permission_update.rows_affected() == 0 {
        tx.rollback().await?;
        tracing::info!(%id, "Permission already responded during concurrent update");
        return get_permission_by_id(state, id).await;
    }

    sqlx::query(
        r#"
        UPDATE sessions
        SET pending_permission_count = CASE
                WHEN pending_permission_count > 0 THEN pending_permission_count - 1
                ELSE 0
            END,
            updated_at = CURRENT_TIMESTAMP
        WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .execute(&mut *tx)
    .await?;

    let daemon_message = DaemonMessage::PermissionResponse {
        permission_id: id,
        session_id,
        decision: req.decision,
    };

    if !state.hub.send_to_daemon(&host_id, daemon_message).await {
        tx.rollback().await?;
        return Err(ServerError::Conflict(
            "Failed to deliver permission response".to_string(),
        ));
    }

    tx.commit().await?;

    // Broadcast permission response to subscribed clients
    state
        .hub
        .broadcast_to_session(
            &state.db,
            &session_id,
            ve_shared::proto::ClientMessage::PermissionResponse {
                permission_id: id,
                session_id,
                decision: req.decision,
            },
        )
        .await;

    tracing::info!(%id, ?req.decision, "Permission responded");

    Ok(Json(PermissionRequest {
        permission_id: id,
        session_id,
        risk_type: utils::parse_risk_type(&existing.risk_type),
        summary: existing.summary,
        target: existing.target,
        created_at: utils::parse_sqlite_timestamp(&existing.created_at)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        status: utils::parse_permission_status(new_status),
        responded_at: Some(chrono::Utc::now()),
    }))
}
