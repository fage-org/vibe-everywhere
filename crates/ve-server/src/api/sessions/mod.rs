//! Session API Handlers
//!
//! Session CRUD, message, control, and lifecycle endpoints.

pub mod close;
pub mod commands;
pub mod control;
pub mod crud;
pub mod messages;
#[cfg(test)]
mod tests;

use axum::Json;
use std::time::Duration;
use uuid::Uuid;

#[cfg(debug_assertions)]
pub mod test_support {
    use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

    use tokio::sync::Barrier;

    #[derive(Clone)]
    struct CreateSessionRaceHook {
        idempotency_key: String,
        barrier: Arc<Barrier>,
    }

    static CREATE_SESSION_RACE_HOOK: OnceLock<Mutex<Option<CreateSessionRaceHook>>> =
        OnceLock::new();
    static CREATE_SESSION_RACE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[derive(Clone)]
    struct ArchivedRerunRaceHook {
        archived_session_id: String,
        barrier: Arc<Barrier>,
    }

    static ARCHIVED_RERUN_RACE_HOOK: OnceLock<Mutex<Option<ArchivedRerunRaceHook>>> =
        OnceLock::new();
    static ARCHIVED_RERUN_RACE_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn hook_slot() -> &'static Mutex<Option<CreateSessionRaceHook>> {
        CREATE_SESSION_RACE_HOOK.get_or_init(|| Mutex::new(None))
    }

    fn test_lock() -> &'static Mutex<()> {
        CREATE_SESSION_RACE_TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn archived_rerun_hook_slot() -> &'static Mutex<Option<ArchivedRerunRaceHook>> {
        ARCHIVED_RERUN_RACE_HOOK.get_or_init(|| Mutex::new(None))
    }

    fn archived_rerun_test_lock() -> &'static Mutex<()> {
        ARCHIVED_RERUN_RACE_TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    pub struct CreateSessionRaceHookGuard {
        _test_lock: MutexGuard<'static, ()>,
    }

    pub struct ArchivedRerunRaceHookGuard {
        _test_lock: MutexGuard<'static, ()>,
    }

    pub fn install_create_session_race_hook(
        idempotency_key: impl Into<String>,
    ) -> CreateSessionRaceHookGuard {
        let test_lock = match test_lock().lock() {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut hook = match hook_slot().lock() {
            Ok(hook) => hook,
            Err(poisoned) => poisoned.into_inner(),
        };

        assert!(hook.is_none(), "create session race hook already installed");

        *hook = Some(CreateSessionRaceHook {
            idempotency_key: idempotency_key.into(),
            barrier: Arc::new(Barrier::new(2)),
        });

        CreateSessionRaceHookGuard {
            _test_lock: test_lock,
        }
    }

    pub async fn wait_for_create_session_race(idempotency_key: &str) {
        let barrier = {
            let hook = match hook_slot().lock() {
                Ok(hook) => hook,
                Err(poisoned) => poisoned.into_inner(),
            };

            hook.as_ref()
                .filter(|hook| hook.idempotency_key == idempotency_key)
                .map(|hook| hook.barrier.clone())
        };

        if let Some(barrier) = barrier {
            barrier.wait().await;
        }
    }

    pub fn install_archived_rerun_race_hook(
        archived_session_id: impl Into<String>,
    ) -> ArchivedRerunRaceHookGuard {
        let test_lock = match archived_rerun_test_lock().lock() {
            Ok(lock) => lock,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut hook = match archived_rerun_hook_slot().lock() {
            Ok(hook) => hook,
            Err(poisoned) => poisoned.into_inner(),
        };

        assert!(hook.is_none(), "archived rerun race hook already installed");

        *hook = Some(ArchivedRerunRaceHook {
            archived_session_id: archived_session_id.into(),
            barrier: Arc::new(Barrier::new(2)),
        });

        ArchivedRerunRaceHookGuard {
            _test_lock: test_lock,
        }
    }

    pub async fn wait_for_archived_rerun_race(archived_session_id: &str) {
        let barrier = {
            let hook = match archived_rerun_hook_slot().lock() {
                Ok(hook) => hook,
                Err(poisoned) => poisoned.into_inner(),
            };

            hook.as_ref()
                .filter(|hook| hook.archived_session_id == archived_session_id)
                .map(|hook| hook.barrier.clone())
        };

        if let Some(barrier) = barrier {
            barrier.wait().await;
        }
    }

    impl Drop for CreateSessionRaceHookGuard {
        fn drop(&mut self) {
            match hook_slot().lock() {
                Ok(mut hook) => *hook = None,
                Err(poisoned) => *poisoned.into_inner() = None,
            }
        }
    }

    impl Drop for ArchivedRerunRaceHookGuard {
        fn drop(&mut self) {
            match archived_rerun_hook_slot().lock() {
                Ok(mut hook) => *hook = None,
                Err(poisoned) => *poisoned.into_inner() = None,
            }
        }
    }
}

use ve_shared::{
    models::{ArchiveMetadata, ArchiveStatistics, CreateSessionRequest, Session, SessionMessage},
    proto::{DaemonMessage, SessionControlAction},
};

use crate::authz::{
    authorize_session_create, require_host_access, ClientAccess, SessionAccess,
    SessionCollectionAccess,
};
use crate::hub::DaemonResponse;

use crate::db::idempotency::{IdempotencyKeyRecord, IdempotencyKeyStore};
use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::{self, extract_request_id, generate_request_id, parse_uuid};
use crate::validation::{validate_content, validate_idempotency_key, validate_title};

/// Database record for session
struct SessionRecord {
    session_id: String,
    title: String,
    host_id: String,
    workspace_id: String,
    agent_type: String,
    status: String,
    last_activity_at: Option<String>,
    latest_summary: Option<String>,
    unread_event_count: i64,
    pending_permission_count: i64,
    can_resume_cross_device: i64,
    claude_session_id: Option<String>,
    created_at: String,
    updated_at: String,
}

impl SessionRecord {
    fn to_model(&self) -> Result<Session> {
        Ok(Session {
            session_id: parse_uuid(&self.session_id, "session_id")?,
            title: self.title.clone(),
            host_id: parse_uuid(&self.host_id, "host_id")?,
            workspace_id: parse_uuid(&self.workspace_id, "workspace_id")?,
            agent_type: self.agent_type.clone(),
            status: utils::parse_session_status(&self.status),
            last_activity_at: self.last_activity_at.as_ref().and_then(|s| {
                utils::parse_sqlite_timestamp(s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            }),
            latest_summary: self.latest_summary.clone(),
            unread_event_count: self.unread_event_count as i32,
            pending_permission_count: self.pending_permission_count as i32,
            can_resume_cross_device: self.can_resume_cross_device != 0,
            claude_session_id: self.claude_session_id.clone(),
            created_at: utils::parse_sqlite_timestamp(&self.created_at)
                .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: utils::parse_sqlite_timestamp(&self.updated_at)
                .map_err(|e| ServerError::Internal(format!("Invalid updated_at: {}", e)))?
                .with_timezone(&chrono::Utc),
        })
    }
}

fn idempotency_duplicate_error(error: &sqlx::Error) -> bool {
    let message = error.to_string();
    message.contains("UNIQUE constraint")
        || message.contains("PRIMARY KEY")
        || message.contains("duplicate key")
        || message.contains("unique constraint")
}

fn sqlite_busy_error(error: &sqlx::Error) -> bool {
    error.to_string().contains("database is locked")
}

async fn load_session_for_idempotency(state: &AppState, session_id: Uuid) -> Result<Json<Session>> {
    let session_id_str = session_id.to_string();
    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            i64,
            i64,
            i64,
            Option<String>,
            String,
            String,
        ),
    >(
        r#"
        SELECT session_id, title, host_id, workspace_id, agent_type, status,
               CAST(last_activity_at AS TEXT), latest_summary, unread_event_count,
               pending_permission_count, CASE WHEN can_resume_cross_device THEN 1 ELSE 0 END,
               claude_session_id,
               CAST(created_at AS TEXT), CAST(updated_at AS TEXT)
        FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(session_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", session_id)))?;

    let record = SessionRecord {
        session_id: row.0,
        title: row.1,
        host_id: row.2,
        workspace_id: row.3,
        agent_type: row.4,
        status: row.5,
        last_activity_at: row.6,
        latest_summary: row.7,
        unread_event_count: row.8,
        pending_permission_count: row.9,
        can_resume_cross_device: row.10,
        claude_session_id: row.11,
        created_at: row.12,
        updated_at: row.13,
    };

    Ok(Json(record.to_model()?))
}

async fn load_existing_idempotent_session(
    state: &AppState,
    trace_id: &str,
    idempotency_key: &str,
    request_hash: &str,
    record: IdempotencyKeyRecord,
) -> Result<Json<Session>> {
    if !IdempotencyKeyStore::new(state.db.clone(), state.config.idempotency_ttl_secs)
        .verify_hash(&record, request_hash)
    {
        return Err(ServerError::Conflict(
            "Request body changed for existing idempotency key".to_string(),
        ));
    }

    let session_id = parse_uuid(&record.result_ref, "session_id")?;
    tracing::info!(
        trace_id = %trace_id,
        session_id = %session_id,
        key = %idempotency_key,
        "Returning existing session (idempotent)"
    );

    load_session_for_idempotency(state, session_id).await
}

async fn wait_for_existing_idempotent_session(
    state: &AppState,
    trace_id: &str,
    idempotency_key: &str,
    request_hash: &str,
    attempts: usize,
    delay: Duration,
) -> Result<Option<Json<Session>>> {
    let store = IdempotencyKeyStore::new(state.db.clone(), state.config.idempotency_ttl_secs);

    for attempt in 0..attempts {
        if let Some(existing) = store.get(idempotency_key).await? {
            let session = load_existing_idempotent_session(
                state,
                trace_id,
                idempotency_key,
                request_hash,
                existing,
            )
            .await?;
            return Ok(Some(session));
        }

        if attempt + 1 < attempts {
            tokio::time::sleep(delay).await;
        }
    }

    Ok(None)
}

// Re-exports for route registration in lib.rs
pub use close::{close_session, close_session_route};
pub(crate) use control::archive_session_with_metadata;
pub use control::{control_session, control_session_route, ControlRequest};
pub use crud::{create_session, get_session, get_session_route, list_sessions};
pub use messages::{list_messages, list_messages_route, send_message, send_message_route, SendMessageRequest, MessageListQuery};
