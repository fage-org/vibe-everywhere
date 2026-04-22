//! Session API Handlers
//!
//! Session CRUD, message, control, and lifecycle endpoints.

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Extension, Json,
};
use serde::Deserialize;
use std::{sync::Arc, time::Duration};
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

    fn hook_slot() -> &'static Mutex<Option<CreateSessionRaceHook>> {
        CREATE_SESSION_RACE_HOOK.get_or_init(|| Mutex::new(None))
    }

    fn test_lock() -> &'static Mutex<()> {
        CREATE_SESSION_RACE_TEST_LOCK.get_or_init(|| Mutex::new(()))
    }

    pub struct CreateSessionRaceHookGuard {
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

    impl Drop for CreateSessionRaceHookGuard {
        fn drop(&mut self) {
            match hook_slot().lock() {
                Ok(mut hook) => *hook = None,
                Err(poisoned) => *poisoned.into_inner() = None,
            }
        }
    }
}

use ve_shared::{
    jwt::Claims,
    models::{ArchiveMetadata, ArchiveStatistics, CreateSessionRequest, Session, SessionMessage},
    proto::{DaemonMessage, SessionControlAction},
    types::Paginated,
};

use crate::authz::{
    authorize_session_create, require_client_device_id, require_host_access,
    require_session_access, ClientAccess, SessionAccess, SessionCollectionAccess,
};
use crate::hub::DaemonResponse;

use crate::db::idempotency::{IdempotencyKeyRecord, IdempotencyKeyStore};
use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::{self, extract_request_id, generate_request_id, parse_uuid};
use crate::validation::{validate_content, validate_title};

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
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            }),
            latest_summary: self.latest_summary.clone(),
            unread_event_count: self.unread_event_count as i32,
            pending_permission_count: self.pending_permission_count as i32,
            can_resume_cross_device: self.can_resume_cross_device != 0,
            claude_session_id: self.claude_session_id.clone(),
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
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
               last_activity_at, latest_summary, unread_event_count, pending_permission_count,
               can_resume_cross_device, claude_session_id, created_at, updated_at
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

#[allow(clippy::type_complexity)]
pub async fn list_sessions(
    access: SessionCollectionAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Session>>> {
    let rows: Vec<(
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
    )> = if let Some(host_id) = access.host_id {
        sqlx::query_as(
            r#"
                SELECT sessions.session_id, sessions.title, sessions.host_id, sessions.workspace_id,
                       sessions.agent_type, sessions.status, sessions.last_activity_at,
                       sessions.latest_summary, sessions.unread_event_count,
                       sessions.pending_permission_count, sessions.can_resume_cross_device,
                       sessions.claude_session_id, sessions.created_at, sessions.updated_at
                FROM sessions
                INNER JOIN device_session_access
                    ON device_session_access.session_id = sessions.session_id
                WHERE sessions.host_id = $1
                  AND device_session_access.device_id = $2
                  AND sessions.status != 'archived'
                ORDER BY sessions.updated_at DESC
                "#,
        )
        .bind(host_id.to_string())
        .bind(access.device_id.to_string())
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
                SELECT sessions.session_id, sessions.title, sessions.host_id, sessions.workspace_id,
                       sessions.agent_type, sessions.status, sessions.last_activity_at,
                       sessions.latest_summary, sessions.unread_event_count,
                       sessions.pending_permission_count, sessions.can_resume_cross_device,
                       sessions.claude_session_id, sessions.created_at, sessions.updated_at
                FROM sessions
                INNER JOIN device_session_access
                    ON device_session_access.session_id = sessions.session_id
                WHERE device_session_access.device_id = $1
                  AND sessions.status != 'archived'
                ORDER BY sessions.updated_at DESC
                "#,
        )
        .bind(access.device_id.to_string())
        .fetch_all(&state.db)
        .await?
    };

    let sessions: Result<Vec<Session>> = rows
        .into_iter()
        .map(
            |(
                session_id,
                title,
                host_id,
                workspace_id,
                agent_type,
                status,
                last_activity_at,
                latest_summary,
                unread_event_count,
                pending_permission_count,
                can_resume_cross_device,
                claude_session_id,
                created_at,
                updated_at,
            )| {
                SessionRecord {
                    session_id,
                    title,
                    host_id,
                    workspace_id,
                    agent_type,
                    status,
                    last_activity_at,
                    latest_summary,
                    unread_event_count,
                    pending_permission_count,
                    can_resume_cross_device,
                    claude_session_id,
                    created_at,
                    updated_at,
                }
                .to_model()
            },
        )
        .collect();

    Ok(Json(sessions?))
}

/// POST /api/sessions
///
/// Create a new session with strict idempotency protection.
pub async fn create_session(
    client: ClientAccess,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<Session>> {
    let device_id = client.device_id;
    let trace_id = extract_request_id(&headers);

    validate_title(&req.title)?;
    validate_content(&req.initial_message)?;
    authorize_session_create(&state, device_id, &req).await?;

    let idempotency_key = req.idempotency_key.clone();
    let request_hash = IdempotencyKeyStore::compute_hash(
        &serde_json::to_string(&serde_json::json!({
            "host_id": req.host_id,
            "workspace_id": req.workspace_id,
            "title": req.title,
            "initial_message": req.initial_message,
        }))
        .map_err(|error| {
            ServerError::Internal(format!("Failed to serialize request hash: {}", error))
        })?,
    );
    let store = IdempotencyKeyStore::new(state.db.clone(), state.config.idempotency_ttl_secs);

    if let Some(existing) = store.get(&idempotency_key).await? {
        return load_existing_idempotent_session(
            &state,
            &trace_id,
            &idempotency_key,
            &request_hash,
            existing,
        )
        .await;
    }

    #[cfg(debug_assertions)]
    test_support::wait_for_create_session_race(&idempotency_key).await;

    let session_id = Uuid::new_v4();
    let session_id_str = session_id.to_string();
    let host_id_str = req.host_id.to_string();
    let workspace_id_str = req.workspace_id.to_string();
    let workspace: (String,) = sqlx::query_as(
        r#"
        SELECT path FROM workspaces WHERE workspace_id = $1 AND host_id = $2
        "#,
    )
    .bind(&workspace_id_str)
    .bind(&host_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!(
        "Workspace {}",
        req.workspace_id
    )))?;

    let message_id = Uuid::new_v4();
    let message_id_str = message_id.to_string();
    let expires_at = (chrono::Utc::now()
        + chrono::Duration::seconds(state.config.idempotency_ttl_secs as i64))
    .to_rfc3339();

    let tx_result: Result<()> = async {
        let mut tx = state.db.begin().await?;

        if sqlx::query_scalar::<_, String>(
            r#"
            SELECT session_id
            FROM idempotency_keys
            WHERE key = $1
            "#,
        )
        .bind(&idempotency_key)
        .fetch_optional(&mut *tx)
        .await?
        .is_some()
        {
            tx.rollback().await?;
            return Err(ServerError::Conflict("idempotency-existing".to_string()));
        }

        let created_at = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO sessions (
                session_id, title, host_id, workspace_id, agent_type, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, 'claude_code', $5, $5)
            "#,
        )
        .bind(&session_id_str)
        .bind(&req.title)
        .bind(&host_id_str)
        .bind(&workspace_id_str)
        .bind(&created_at)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO session_messages (message_id, session_id, message_type, content)
            VALUES ($1, $2, 'user', $3)
            "#,
        )
        .bind(&message_id_str)
        .bind(&session_id_str)
        .bind(&req.initial_message)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO idempotency_keys (key, request_hash, session_id, result_type, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(&idempotency_key)
        .bind(&request_hash)
        .bind(&session_id_str)
        .bind("session")
        .bind(&expires_at)
        .execute(&mut *tx)
        .await
        .map_err(|error| {
            if idempotency_duplicate_error(&error) {
                ServerError::Conflict("idempotency-conflict".to_string())
            } else {
                error.into()
            }
        })?;

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
        .bind(&host_id_str)
        .bind(&session_id_str)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
    .await;

    let tx_result = match tx_result {
        Err(ServerError::Database(error)) if sqlite_busy_error(&error) => {
            if let Some(existing) = wait_for_existing_idempotent_session(
                &state,
                &trace_id,
                &idempotency_key,
                &request_hash,
                10,
                Duration::from_millis(10),
            )
            .await?
            {
                return Ok(existing);
            }
            Err(ServerError::Database(error))
        }
        other => other,
    };

    if let Err(ServerError::Conflict(message)) = &tx_result {
        if message == "idempotency-conflict" || message == "idempotency-existing" {
            if let Some(existing) = wait_for_existing_idempotent_session(
                &state,
                &trace_id,
                &idempotency_key,
                &request_hash,
                10,
                Duration::from_millis(10),
            )
            .await?
            {
                return Ok(existing);
            }
            return Err(ServerError::Internal(
                "Idempotency key disappeared after conflict".into(),
            ));
        }
    }
    tx_result?;

    let request_id = generate_request_id();
    let command = DaemonMessage::CreateSession {
        request_id: request_id.clone(),
        session_id,
        workspace_path: workspace.0,
        agent_type: "claude_code".to_string(),
        initial_message: req.initial_message,
    };

    match send_daemon_command_and_wait(&state, req.host_id, request_id, command).await {
        Ok(response) => {
            if let Err(error) = ensure_command_acked(response) {
                mark_session_error(
                    &state,
                    &session_id_str,
                    "Failed to queue create_session request to daemon",
                )
                .await?;
                return Err(error);
            }
        }
        Err(error) => {
            mark_session_error(
                &state,
                &session_id_str,
                "Failed to queue create_session request to daemon",
            )
            .await?;
            return Err(error);
        }
    }

    tracing::info!(trace_id = %trace_id, session_id = %session_id, host_id = %req.host_id, "Session created");

    load_session_for_idempotency(&state, session_id).await
}

/// GET /api/sessions/:id
///
/// Get session details.
pub async fn get_session_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Session>> {
    get_session_by_id(state, access.session_id).await
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Session>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    get_session_by_id(state, id).await
}

async fn get_session_by_id(state: Arc<AppState>, id: Uuid) -> Result<Json<Session>> {
    // Type alias to avoid clippy type_complexity warning
    type SessionRow = (
        String,         // session_id
        String,         // title
        String,         // host_id
        String,         // workspace_id
        String,         // agent_type
        String,         // status
        Option<String>, // last_activity_at
        Option<String>, // latest_summary
        i64,            // unread_event_count
        i64,            // pending_permission_count
        i64,            // can_resume_cross_device
        Option<String>, // claude_session_id
        String,         // created_at
        String,         // updated_at
    );

    let session_id_str = id.to_string();

    let row: SessionRow = sqlx::query_as(
        r#"
        SELECT session_id, title, host_id, workspace_id, agent_type, status,
               last_activity_at, latest_summary, unread_event_count, pending_permission_count,
               can_resume_cross_device, claude_session_id, created_at, updated_at
        FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    Ok(Json(Session {
        session_id: id,
        title: row.1,
        host_id: parse_uuid(&row.2, "host_id")?,
        workspace_id: parse_uuid(&row.3, "workspace_id")?,
        agent_type: row.4,
        status: utils::parse_session_status(&row.5),
        last_activity_at: row.6.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        }),
        latest_summary: row.7,
        unread_event_count: row.8 as i32,
        pending_permission_count: row.9 as i32,
        can_resume_cross_device: row.10 != 0,
        claude_session_id: row.11,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.12)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.13)
            .map_err(|e| ServerError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&chrono::Utc),
    }))
}

/// Send message request
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

/// POST /api/sessions/:id/messages
///
/// Send a message to the session.
pub async fn send_message_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>> {
    send_message_for_session(state, headers, access.session_id, req).await
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    send_message_for_session(state, headers, id, req).await
}

async fn send_message_for_session(
    state: Arc<AppState>,
    headers: HeaderMap,
    id: Uuid,
    req: SendMessageRequest,
) -> Result<Json<serde_json::Value>> {
    // Extract trace_id for correlation
    let trace_id = extract_request_id(&headers);

    // Validate content
    validate_content(&req.content)?;

    let session_id_str = id.to_string();

    // Check session is not archived
    let session: (String, String) = sqlx::query_as(
        r#"
        SELECT status, host_id FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    if session.0 == "archived" {
        return Err(ServerError::SessionArchived);
    }

    let host_id = parse_uuid(&session.1, "host_id")?;

    let message_id = Uuid::new_v4();
    let message_id_str = message_id.to_string();

    let request_id = generate_request_id();
    let request = DaemonMessage::SendMessage {
        request_id: request_id.clone(),
        session_id: id,
        content: req.content.clone(),
    };

    ensure_command_acked(
        send_daemon_command_and_wait(&state, host_id, request_id, request).await?,
    )?;

    sqlx::query(
        r#"
        INSERT INTO session_messages (message_id, session_id, message_type, content)
        VALUES ($1, $2, 'user', $3)
        "#,
    )
    .bind(&message_id_str)
    .bind(&session_id_str)
    .bind(&req.content)
    .execute(&state.db)
    .await?;

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE sessions SET last_activity_at = $2, updated_at = $2
        WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .bind(&now)
    .execute(&state.db)
    .await?;

    tracing::debug!(trace_id = %trace_id, session_id = %id, message_id = %message_id, "Message sent");

    Ok(Json(
        serde_json::json!({ "success": true, "message_id": message_id }),
    ))
}

/// Message list query parameters
#[derive(Debug, Deserialize)]
pub struct MessageListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// GET /api/sessions/:id/messages
///
/// List session messages.
pub async fn list_messages_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
    Query(query): Query<MessageListQuery>,
) -> Result<Json<Paginated<SessionMessage>>> {
    list_messages_for_session(state, access.session_id, query).await
}

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Query(query): Query<MessageListQuery>,
) -> Result<Json<Paginated<SessionMessage>>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    list_messages_for_session(state, id, query).await
}

async fn list_messages_for_session(
    state: Arc<AppState>,
    id: Uuid,
    query: MessageListQuery,
) -> Result<Json<Paginated<SessionMessage>>> {
    let page = query.page.unwrap_or(1);
    if page == 0 {
        return Err(ServerError::BadRequest(
            "page must be greater than 0".to_string(),
        ));
    }
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;
    let session_id_str = id.to_string();
    let limit_i32 = limit as i32;
    let offset_i32 = offset as i32;

    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT message_id, session_id, message_type, content, created_at
        FROM session_messages
        WHERE session_id = $1
        ORDER BY created_at ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&session_id_str)
    .bind(limit_i32)
    .bind(offset_i32)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count FROM session_messages WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_one(&state.db)
    .await?;
    let total = total.0 as u64;

    let messages: Result<Vec<SessionMessage>> = rows
        .into_iter()
        .map(|row| {
            Ok(SessionMessage {
                message_id: parse_uuid(&row.0, "message_id")?,
                session_id: id,
                message_type: utils::parse_message_type(&row.2),
                content: row.3,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.4)
                    .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                    .with_timezone(&chrono::Utc),
            })
        })
        .collect();

    Ok(Json(Paginated::new(messages?, total, page, limit)))
}

/// Control request
#[derive(Debug, Deserialize)]
pub struct ControlRequest {
    pub action: String,
}

/// Validate whether a control action is allowed for the current persisted session status.
fn validate_live_control_action(status: &str, action: SessionControlAction) -> Result<()> {
    if status == "archived" {
        return Err(ServerError::SessionArchived);
    }

    if action == SessionControlAction::Rerun {
        return Err(ServerError::Conflict(
            "rerun is only supported for archived sessions; use restart for live sessions"
                .to_string(),
        ));
    }

    Ok(())
}

async fn send_daemon_command_and_wait(
    state: &AppState,
    host_id: Uuid,
    request_id: String,
    request: DaemonMessage,
) -> Result<DaemonResponse> {
    state
        .hub
        .send_and_wait(
            &host_id,
            request,
            request_id,
            std::time::Duration::from_millis(state.config.ack_timeout_ms),
        )
        .await
        .map_err(|error| sanitize_session_command_transport_error(error.as_ref()))
}

fn sanitize_session_command_transport_error(error: &dyn std::error::Error) -> ServerError {
    tracing::warn!(error = %error, "Sanitized daemon session command transport failure");
    ServerError::Conflict("Daemon command failed".to_string())
}

fn sanitize_session_command_response_error() -> ServerError {
    ServerError::Conflict("Daemon command failed".to_string())
}

async fn mark_session_error(state: &AppState, session_id: &str, message: &str) -> Result<()> {
    let failed_at = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'error', latest_summary = $2, updated_at = $3
        WHERE session_id = $1
        "#,
    )
    .bind(session_id)
    .bind(message)
    .bind(&failed_at)
    .execute(&state.db)
    .await?;

    Ok(())
}

fn ensure_command_acked(response: DaemonResponse) -> Result<()> {
    match response {
        DaemonResponse::Ack(ack) if ack.success => Ok(()),
        DaemonResponse::Ack(ack) => {
            tracing::warn!(
                has_error = ack.error.is_some(),
                "Sanitized daemon session command ack failure"
            );
            Err(sanitize_session_command_response_error())
        }
        DaemonResponse::Error(error) => {
            tracing::warn!(error_code = %error.error_code, "Sanitized daemon session command error response");
            Err(sanitize_session_command_response_error())
        }
        DaemonResponse::Message(_) => Err(ServerError::Conflict(
            "Unexpected daemon response type".to_string(),
        )),
    }
}

async fn persist_control_success(
    state: &AppState,
    session_id: Uuid,
    action: SessionControlAction,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let session_id_str = session_id.to_string();

    match action {
        SessionControlAction::Pause => {
            sqlx::query(
                r#"
                UPDATE sessions SET status = 'paused', updated_at = $2
                WHERE session_id = $1
                "#,
            )
            .bind(&session_id_str)
            .bind(&now)
            .execute(&state.db)
            .await?;
        }
        SessionControlAction::Terminate => {}
        SessionControlAction::Restart => {
            sqlx::query(
                r#"
                UPDATE sessions SET status = 'running', updated_at = $2
                WHERE session_id = $1
                "#,
            )
            .bind(&session_id_str)
            .bind(&now)
            .execute(&state.db)
            .await?;
        }
        SessionControlAction::Interrupt | SessionControlAction::Rerun => {}
    }

    Ok(())
}

/// POST /api/sessions/:id/control
///
/// Send a control command (pause, terminate, interrupt, restart, or archived rerun).
pub async fn control_session_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ControlRequest>,
) -> Result<Json<serde_json::Value>> {
    control_session_for_id(state, headers, access.device_id, access.session_id, req).await
}

pub async fn control_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<ControlRequest>,
) -> Result<Json<serde_json::Value>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    control_session_for_id(state, headers, device_id, id, req).await
}

async fn control_session_for_id(
    state: Arc<AppState>,
    headers: HeaderMap,
    device_id: Uuid,
    id: Uuid,
    req: ControlRequest,
) -> Result<Json<serde_json::Value>> {
    // Extract trace_id for correlation
    let trace_id = extract_request_id(&headers);

    let session_id_str = id.to_string();

    let session: (String, String, String, String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT status, host_id, workspace_id, title, agent_type, claude_session_id
        FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    let host_id = parse_uuid(&session.1, "host_id")?;
    let action = utils::parse_control_action(&req.action)?;

    if session.0 == "archived" {
        require_host_access(&state, device_id, host_id).await?;
        return handle_archived_rerun(&state, &trace_id, device_id, id, &req.action, session).await;
    }

    validate_live_control_action(&session.0, action)?;

    let request_id = generate_request_id();
    let request = DaemonMessage::SessionControl {
        request_id: request_id.clone(),
        session_id: id,
        action,
    };

    ensure_command_acked(
        send_daemon_command_and_wait(&state, host_id, request_id, request).await?,
    )?;
    persist_control_success(&state, id, action).await?;

    tracing::debug!(trace_id = %trace_id, session_id = %id, action = ?action, "Control command sent");

    Ok(Json(serde_json::json!({ "success": true })))
}

async fn handle_archived_rerun(
    state: &AppState,
    trace_id: &str,
    requester_device_id: Uuid,
    archived_session_id: Uuid,
    action_raw: &str,
    session: (String, String, String, String, String, Option<String>),
) -> Result<Json<serde_json::Value>> {
    if action_raw != "rerun" {
        return Err(ServerError::SessionArchived);
    }

    let (_status, host_id_raw, workspace_id_raw, title, agent_type, live_claude_session_id) =
        session;

    let host_id = parse_uuid(&host_id_raw, "host_id")?;
    let host_id_str = host_id.to_string();
    let workspace_id = parse_uuid(&workspace_id_raw, "workspace_id")?;
    let workspace_id_str = workspace_id.to_string();
    let archived_session_id_str = archived_session_id.to_string();
    let requester_device_id_str = requester_device_id.to_string();

    let workspace: (String,) = sqlx::query_as(
        r#"
        SELECT path FROM workspaces WHERE workspace_id = $1 AND host_id = $2
        "#,
    )
    .bind(&workspace_id_str)
    .bind(&host_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Workspace {}", workspace_id)))?;

    let archived_claude_session_id = if let Some(claude_session_id) = live_claude_session_id {
        claude_session_id
    } else {
        let metadata_json = sqlx::query_as::<_, (Option<String>,)>(
            r#"
            SELECT metadata_json FROM session_archives WHERE session_id = $1
            ORDER BY closed_at DESC LIMIT 1
            "#,
        )
        .bind(&archived_session_id_str)
        .fetch_optional(&state.db)
        .await?;

        let archive_metadata = metadata_json
            .and_then(|(json,)| json)
            .map(|json| {
                serde_json::from_str::<ve_shared::models::ArchiveMetadata>(&json).map_err(|error| {
                    ServerError::Internal(format!(
                        "Failed to parse archive metadata for session {}: {}",
                        archived_session_id, error
                    ))
                })
            })
            .transpose()?;

        archive_metadata
            .and_then(|metadata| metadata.claude_session_id)
            .ok_or_else(|| {
                ServerError::Conflict(
                    "Archived session has no Claude session ID to resume".to_string(),
                )
            })?
    };

    if let Some(existing_session_id) =
        find_reusable_rerun_session_id(state, &archived_session_id_str).await?
    {
        ensure_device_session_access(state, &requester_device_id_str, &existing_session_id).await?;

        tracing::info!(
            trace_id = %trace_id,
            archived_session_id = %archived_session_id,
            existing_session_id = %existing_session_id,
            "Archived session rerun reused existing live session"
        );

        return Ok(Json(serde_json::json!({
            "success": true,
            "session_id": existing_session_id,
            "resumed_from_session_id": archived_session_id,
        })));
    }

    if has_dispatching_rerun(state, &archived_session_id_str).await? {
        return Err(ServerError::Conflict(
            "Archived session rerun is still dispatching; retry shortly".to_string(),
        ));
    }

    let new_session_id = Uuid::new_v4();
    let new_session_id_str = new_session_id.to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let mut tx = state.db.begin().await?;

    let insert_result = sqlx::query(
        r#"
        INSERT INTO sessions (
            session_id, title, host_id, workspace_id, agent_type, status, claude_session_id,
            rerun_from_session_id, can_resume_cross_device, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, 'dispatching', $6, $7, 1, $8, $8)
        "#,
    )
    .bind(&new_session_id_str)
    .bind(&title)
    .bind(&host_id_str)
    .bind(&workspace_id_str)
    .bind(&agent_type)
    .bind(&archived_claude_session_id)
    .bind(&archived_session_id_str)
    .bind(&now)
    .execute(&mut *tx)
    .await;

    match insert_result {
        Ok(_) => {
            sqlx::query(
                r#"
                INSERT INTO device_session_access (device_id, session_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(&requester_device_id_str)
            .bind(&new_session_id_str)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
        }
        Err(error)
            if error.to_string().contains("UNIQUE constraint")
                || error.to_string().contains("duplicate key") =>
        {
            tx.rollback().await?;

            if let Some(existing_session_id) =
                find_reusable_rerun_session_id(state, &archived_session_id_str).await?
            {
                ensure_device_session_access(state, &requester_device_id_str, &existing_session_id)
                    .await?;

                tracing::info!(
                    trace_id = %trace_id,
                    archived_session_id = %archived_session_id,
                    existing_session_id = %existing_session_id,
                    "Archived session rerun reused concurrent live session"
                );

                return Ok(Json(serde_json::json!({
                    "success": true,
                    "session_id": existing_session_id,
                    "resumed_from_session_id": archived_session_id,
                })));
            }

            if has_dispatching_rerun(state, &archived_session_id_str).await? {
                return Err(ServerError::Conflict(
                    "Archived session rerun is still dispatching; retry shortly".to_string(),
                ));
            }

            return Err(ServerError::Internal(
                "Active rerun disappeared after uniqueness conflict".to_string(),
            ));
        }
        Err(error) => return Err(error.into()),
    }

    let request_id = generate_request_id();
    match send_daemon_command_and_wait(
        state,
        host_id,
        request_id.clone(),
        DaemonMessage::RerunSession {
            request_id,
            session_id: new_session_id,
            workspace_path: workspace.0,
            agent_type: agent_type.clone(),
            claude_session_id: archived_claude_session_id.clone(),
        },
    )
    .await
    {
        Ok(response) => {
            if let Err(error) = ensure_command_acked(response) {
                mark_session_error(
                    state,
                    &new_session_id_str,
                    "Failed to queue rerun request to daemon",
                )
                .await?;
                return Err(error);
            }
        }
        Err(error) => {
            mark_session_error(
                state,
                &new_session_id_str,
                "Failed to queue rerun request to daemon",
            )
            .await?;

            tracing::warn!(
                trace_id = %trace_id,
                archived_session_id = %archived_session_id,
                new_session_id = %new_session_id,
                error = %error,
                "Archived session rerun failed before daemon ack"
            );

            return Err(ServerError::Conflict(
                "Failed to queue rerun request to daemon".to_string(),
            ));
        }
    }

    let dispatched_at = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'pending', updated_at = $2
        WHERE session_id = $1 AND status = 'dispatching'
        "#,
    )
    .bind(&new_session_id_str)
    .bind(&dispatched_at)
    .execute(&state.db)
    .await?;

    tracing::info!(
        trace_id = %trace_id,
        archived_session_id = %archived_session_id,
        new_session_id = %new_session_id,
        claude_session_id = %archived_claude_session_id,
        "Archived session rerun requested"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "session_id": new_session_id,
        "resumed_from_session_id": archived_session_id,
    })))
}

async fn find_reusable_rerun_session_id(
    state: &AppState,
    archived_session_id: &str,
) -> Result<Option<Uuid>> {
    let session_id = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT session_id
        FROM sessions
        WHERE rerun_from_session_id = $1
          AND status NOT IN ('dispatching', 'archived', 'error')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(archived_session_id)
    .fetch_optional(&state.db)
    .await?
    .map(|(session_id,)| parse_uuid(&session_id, "session_id"))
    .transpose()?;

    Ok(session_id)
}

async fn has_dispatching_rerun(state: &AppState, archived_session_id: &str) -> Result<bool> {
    let row = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*)
        FROM sessions
        WHERE rerun_from_session_id = $1
          AND status = 'dispatching'
        "#,
    )
    .bind(archived_session_id)
    .fetch_one(&state.db)
    .await?;

    Ok(row.0 > 0)
}

async fn ensure_device_session_access(
    state: &AppState,
    device_id: &str,
    session_id: &Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO device_session_access (device_id, session_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(device_id)
    .bind(session_id.to_string())
    .execute(&state.db)
    .await?;

    Ok(())
}

fn archive_closed_by(close_reason: &str) -> &'static str {
    match close_reason {
        "terminated" => "daemon",
        _ => "server",
    }
}

pub(crate) async fn archive_session_with_metadata(
    state: &AppState,
    session_id: Uuid,
    close_reason: &str,
    summary_override: Option<String>,
) -> Result<Uuid> {
    let session_id_str = session_id.to_string();
    let session: (
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
    ) = sqlx::query_as(
        r#"
        SELECT session_id, title, host_id, workspace_id, status, agent_type,
               latest_summary, claude_session_id, created_at
        FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", session_id)))?;

    if session.4 == "archived" {
        let archive: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT archive_id FROM session_archives WHERE session_id = $1
            "#,
        )
        .bind(&session_id_str)
        .fetch_optional(&state.db)
        .await?;

        return archive
            .map(|(archive_id,)| parse_uuid(&archive_id, "archive_id"))
            .transpose()?
            .ok_or_else(|| {
                ServerError::Conflict(format!(
                    "Session {} is archived without an archive record",
                    session_id
                ))
            });
    }

    let host_id = parse_uuid(&session.2, "host_id")?;
    let workspace_id = parse_uuid(&session.3, "workspace_id")?;
    let host_id_str = host_id.to_string();
    let workspace_id_str = workspace_id.to_string();

    let workspace_row: Option<(String, String)> = sqlx::query_as(
        r#"
        SELECT path, display_name FROM workspaces WHERE workspace_id = $1
        "#,
    )
    .bind(&workspace_id_str)
    .fetch_optional(&state.db)
    .await?;

    let message_count_row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count FROM session_messages WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_one(&state.db)
    .await?;
    let message_count = message_count_row.0 as u32;

    let permission_count_row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count FROM permission_requests WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_one(&state.db)
    .await?;
    let permission_count = permission_count_row.0 as u32;

    let created_at = chrono::DateTime::parse_from_rfc3339(&session.8)
        .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?;
    let duration_seconds = (chrono::Utc::now() - created_at.with_timezone(&chrono::Utc))
        .num_seconds()
        .max(0) as u32;

    let metadata = ArchiveMetadata {
        workspace_path: workspace_row
            .as_ref()
            .map(|w| w.0.clone())
            .unwrap_or_default(),
        workspace_display_name: workspace_row.as_ref().map(|w| w.1.clone()),
        agent_type: session.5.clone(),
        closed_by: archive_closed_by(close_reason).to_string(),
        final_summary: summary_override.clone().or(session.6.clone()),
        claude_session_id: session.7.clone(),
        statistics: Some(ArchiveStatistics {
            message_count,
            event_count: 0,
            permission_count,
            duration_seconds,
        }),
        last_commit_sha: None,
        last_commit_message: None,
    };

    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| ServerError::Internal(format!("Failed to serialize metadata: {}", e)))?;

    let archive_id = Uuid::new_v4();
    let archive_id_str = archive_id.to_string();
    let closed_at = chrono::Utc::now().to_rfc3339();
    let archive_created_at = chrono::Utc::now().to_rfc3339();
    let updated_at = chrono::Utc::now().to_rfc3339();
    let mut tx = state.db.begin().await?;

    let update_result = sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'archived', latest_summary = COALESCE($3, latest_summary), updated_at = $2
        WHERE session_id = $1 AND status != 'archived'
        "#,
    )
    .bind(&session_id_str)
    .bind(&updated_at)
    .bind(&summary_override)
    .execute(&mut *tx)
    .await?;

    if update_result.rows_affected() == 0 {
        let archive: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT archive_id FROM session_archives WHERE session_id = $1
            "#,
        )
        .bind(&session_id_str)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;

        return archive
            .map(|(existing_archive_id,)| parse_uuid(&existing_archive_id, "archive_id"))
            .transpose()?
            .ok_or_else(|| {
                ServerError::Conflict(format!(
                    "Session {} is archived without an archive record",
                    session_id
                ))
            });
    }

    sqlx::query(
        r#"
        INSERT INTO session_archives (
            archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id,
            metadata_json, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(&archive_id_str)
    .bind(&session_id_str)
    .bind(&session.1)
    .bind(&closed_at)
    .bind(close_reason)
    .bind(&host_id_str)
    .bind(&workspace_id_str)
    .bind(&metadata_json)
    .bind(&archive_created_at)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(archive_id)
}

/// POST /api/sessions/:id/close
///
/// Request the daemon to close a session; archival is finalized from daemon status updates.
#[allow(clippy::type_complexity)]
pub async fn close_session_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>> {
    close_session_for_id(state, headers, access.session_id).await
}

#[allow(clippy::type_complexity)]
pub async fn close_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    close_session_for_id(state, headers, id).await
}

async fn close_session_for_id(
    state: Arc<AppState>,
    headers: HeaderMap,
    id: Uuid,
) -> Result<Json<serde_json::Value>> {
    let trace_id = extract_request_id(&headers);

    let session_id_str = id.to_string();

    let session_row: Option<(String, String)> = sqlx::query_as(
        r#"
        SELECT status, host_id
        FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?;

    let session = session_row.ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    if session.0 == "archived" {
        let archive: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT archive_id FROM session_archives WHERE session_id = $1
            "#,
        )
        .bind(&session_id_str)
        .fetch_optional(&state.db)
        .await?;

        return match archive {
            Some((archive_id,)) => Ok(Json(serde_json::json!({
                "success": true,
                "already_archived": true,
                "archive_id": archive_id,
            }))),
            None => Err(ServerError::Conflict(format!(
                "Session {} is archived without an archive record",
                id
            ))),
        };
    }

    let host_id = parse_uuid(&session.1, "host_id")?;
    let request_id = generate_request_id();
    let request = DaemonMessage::CloseSession {
        request_id: request_id.clone(),
        session_id: id,
    };

    ensure_command_acked(
        send_daemon_command_and_wait(&state, host_id, request_id, request).await?,
    )?;

    tracing::info!(trace_id = %trace_id, session_id = %id, "Session close acknowledged by daemon");

    Ok(Json(serde_json::json!({
        "success": true,
        "close_requested": true,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseBackend};
    use crate::db::{install_drivers, run_migrations, DbPool};
    use crate::hub::Hub;
    use crate::state::AppState;
    use axum::response::IntoResponse;
    use ve_shared::models::{ArchiveMetadata, ArchiveStatistics};

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
            std::env::temp_dir().join(format!("ve-sessions-api-test-{}.db", Uuid::new_v4()));
        let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
        let pool = DbPool::connect(&database_url).await.unwrap();
        run_migrations(&pool, DatabaseBackend::Sqlite)
            .await
            .unwrap();

        Arc::new(AppState::new(pool, Hub::new(), test_config(database_url)))
    }

    async fn insert_host_workspace_and_archive(
        state: &Arc<AppState>,
        archived_session_id: Uuid,
        host_id: Uuid,
        workspace_id: Uuid,
        metadata_json: Option<String>,
    ) {
        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, metadata_json, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind("user_closed")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind(metadata_json)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();
    }

    async fn insert_archived_rerun_fixture(
        state: &Arc<AppState>,
        archived_session_id: Uuid,
        host_id: Uuid,
        workspace_id: Uuid,
        device_ids: &[Uuid],
    ) {
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;

        for device_id in device_ids {
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

            sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
                .bind(device_id.to_string())
                .bind(host_id.to_string())
                .execute(&state.db)
                .await
                .unwrap();
        }

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        for device_id in device_ids {
            sqlx::query(
                "INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)",
            )
            .bind(device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    async fn archive_session_with_metadata_returns_existing_archive_for_duplicate_calls() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

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
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let first_archive_id =
            archive_session_with_metadata(&state, session_id, "user_closed", None)
                .await
                .unwrap();
        let second_archive_id = archive_session_with_metadata(
            &state,
            session_id,
            "terminated",
            Some("ignored".to_string()),
        )
        .await
        .unwrap();

        assert_eq!(second_archive_id, first_archive_id);

        let archive_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(archive_count.0, 1);

        let archive: (String, String) = sqlx::query_as(
            "SELECT close_reason, metadata_json FROM session_archives WHERE session_id = $1",
        )
        .bind(session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(archive.0, "user_closed");

        let metadata: ArchiveMetadata = serde_json::from_str(&archive.1).unwrap();
        assert_eq!(metadata.final_summary, None);

        let latest_summary: (Option<String>,) =
            sqlx::query_as("SELECT latest_summary FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(latest_summary.0, None);
    }

    #[test]
    fn live_control_rejects_rerun() {
        let error =
            validate_live_control_action("running", SessionControlAction::Rerun).unwrap_err();
        assert!(matches!(
            error,
            ServerError::Conflict(message)
                if message == "rerun is only supported for archived sessions; use restart for live sessions"
        ));
    }

    #[test]
    fn live_control_allows_restart() {
        assert!(validate_live_control_action("running", SessionControlAction::Restart).is_ok());
    }

    #[tokio::test]
    async fn archived_rerun_returns_internal_error_when_archive_metadata_is_invalid() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some("{not-json".to_string()),
        )
        .await;

        let session = (
            "archived".to_string(),
            host_id.to_string(),
            workspace_id.to_string(),
            "archived".to_string(),
            "claude_code".to_string(),
            None,
        );

        let before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();

        let error = handle_archived_rerun(
            &state,
            "tr-test",
            Uuid::new_v4(),
            archived_session_id,
            "rerun",
            session,
        )
        .await
        .unwrap_err();

        let response = error.into_response();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );

        let after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(before.0, after.0);
    }

    #[tokio::test]
    async fn archived_rerun_returns_conflict_when_archive_metadata_has_no_claude_session_id() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: None,
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;

        let session = (
            "archived".to_string(),
            host_id.to_string(),
            workspace_id.to_string(),
            "archived".to_string(),
            "claude_code".to_string(),
            None,
        );

        let before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();

        let error = handle_archived_rerun(
            &state,
            "tr-test",
            Uuid::new_v4(),
            archived_session_id,
            "rerun",
            session,
        )
        .await
        .unwrap_err();

        let response = error.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::CONFLICT);

        let after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(before.0, after.0);
    }

    async fn insert_device_and_accessible_sessions(
        state: &Arc<AppState>,
        device_id: Uuid,
        host_id: Uuid,
        visible_session_id: Uuid,
        hidden_session_id: Uuid,
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

        let workspace_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        for (session_id, title) in [
            (visible_session_id, "visible"),
            (hidden_session_id, "hidden"),
        ] {
            sqlx::query(
                r#"
                INSERT INTO sessions (
                    session_id, title, host_id, workspace_id, agent_type, status,
                    unread_event_count, pending_permission_count, can_resume_cross_device,
                    created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, 'running', 0, 0, 1, $6, $6)
                "#,
            )
            .bind(session_id.to_string())
            .bind(title)
            .bind(host_id.to_string())
            .bind(workspace_id.to_string())
            .bind("claude_code")
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&state.db)
            .await
            .unwrap();
        }

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(visible_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn create_session_rejects_workspace_from_other_host() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let allowed_host_id = Uuid::new_v4();
        let other_host_id = Uuid::new_v4();
        let foreign_workspace_id = Uuid::new_v4();

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

        for host_id in [allowed_host_id, other_host_id] {
            sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
                .bind(host_id.to_string())
                .bind("host")
                .bind("linux")
                .execute(&state.db)
                .await
                .unwrap();
        }

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(foreign_workspace_id.to_string())
        .bind(other_host_id.to_string())
        .bind("/tmp/foreign")
        .bind("foreign")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(allowed_host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let error = create_session(
            ClientAccess { device_id },
            State(state.clone()),
            HeaderMap::new(),
            Json(CreateSessionRequest {
                idempotency_key: Uuid::new_v4().to_string(),
                host_id: allowed_host_id,
                workspace_id: foreign_workspace_id,
                title: "test".to_string(),
                initial_message: "hello".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(
            matches!(error, ServerError::NotFound(message) if message == format!("Workspace {}", foreign_workspace_id))
        );

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(count.0, 0);
    }

    #[cfg(debug_assertions)]
    #[tokio::test]
    async fn create_session_is_strictly_idempotent_under_concurrency() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let idempotency_key = Uuid::new_v4().to_string();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(4);

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
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_daemon(host_id, daemon_tx).await;
        let _race_hook = test_support::install_create_session_race_hook(idempotency_key.clone());

        let spawn_request = || {
            let state = state.clone();
            let idempotency_key = idempotency_key.clone();
            tokio::spawn(async move {
                create_session(
                    ClientAccess { device_id },
                    State(state),
                    HeaderMap::new(),
                    Json(CreateSessionRequest {
                        idempotency_key,
                        host_id,
                        workspace_id,
                        title: "test".to_string(),
                        initial_message: "hello".to_string(),
                    }),
                )
                .await
            })
        };

        let first_task = spawn_request();
        let second_task = spawn_request();

        let outbound = daemon_rx.recv().await.unwrap();
        assert_eq!(outbound.r#type, "create_session");
        let session_id = outbound.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: outbound.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let first = first_task.await.unwrap().unwrap().0;
        let second = second_task.await.unwrap().unwrap().0;
        assert_eq!(first.session_id, second.session_id);
        assert_eq!(first.session_id, session_id);

        let session_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(session_count.0, 1);

        let message_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_messages WHERE session_id = $1")
                .bind(first.session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(message_count.0, 1);

        let idempotency_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM idempotency_keys WHERE key = $1")
                .bind(&idempotency_key)
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(idempotency_count.0, 1);

        let access_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(device_id.to_string())
        .bind(first.session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(access_count.0, 1);

        assert!(daemon_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn create_session_waits_for_daemon_ack_before_returning_success() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

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
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            create_session(
                ClientAccess { device_id },
                State(state_for_request),
                HeaderMap::new(),
                Json(CreateSessionRequest {
                    idempotency_key: Uuid::new_v4().to_string(),
                    host_id,
                    workspace_id,
                    title: "test".to_string(),
                    initial_message: "hello".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "create_session");
        let session_id = queued.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        let status_before: (String,) =
            sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(status_before.0, "running");
        assert!(!request_task.is_finished());

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;
        assert_eq!(response.session_id, session_id);
    }

    #[tokio::test]
    async fn create_session_marks_session_error_when_daemon_rejects_create() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

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
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            create_session(
                ClientAccess { device_id },
                State(state_for_request),
                HeaderMap::new(),
                Json(CreateSessionRequest {
                    idempotency_key: Uuid::new_v4().to_string(),
                    host_id,
                    workspace_id,
                    title: "test".to_string(),
                    initial_message: "hello".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "create_session");
        let session_id = queued.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_error(ve_shared::proto::ErrorPayload {
                request_id: queued.request_id.clone().unwrap(),
                error_code: "session_create_failed".to_string(),
                error_message: "daemon rejected create".to_string(),
            })
            .await;

        let error = request_task.await.unwrap().unwrap_err();
        assert!(
            matches!(error, ServerError::Conflict(message) if message == "Daemon command failed")
        );

        let row: (String, Option<String>) =
            sqlx::query_as("SELECT status, latest_summary FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(row.0, "error");
        assert_eq!(
            row.1.as_deref(),
            Some("Failed to queue create_session request to daemon")
        );
    }

    #[tokio::test]
    async fn archived_rerun_marks_new_session_error_when_daemon_rejects_rerun() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let rerun_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[rerun_device_id],
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            control_session(
                State(state_for_request),
                Extension(Claims::for_client(
                    rerun_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: false,
                error: Some("daemon rejected rerun".to_string()),
            })
            .await;

        let error = request_task.await.unwrap().unwrap_err();
        assert!(
            matches!(error, ServerError::Conflict(message) if message == "Daemon command failed")
        );

        let row: (String, Option<String>) =
            sqlx::query_as("SELECT status, latest_summary FROM sessions WHERE session_id = $1")
                .bind(new_session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(row.0, "error");
        assert_eq!(
            row.1.as_deref(),
            Some("Failed to queue rerun request to daemon")
        );
    }

    #[tokio::test]
    async fn archived_rerun_grants_new_session_to_requester() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let new_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(new_device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(new_device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let session = (
            "archived".to_string(),
            host_id.to_string(),
            workspace_id.to_string(),
            "archived".to_string(),
            "claude_code".to_string(),
            None,
        );

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            handle_archived_rerun(
                &state_for_request,
                "tr-test",
                new_device_id,
                archived_session_id,
                "rerun",
                session,
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        let status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(new_session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(status.0, "dispatching");

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;
        let response_session_id = response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let access_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(new_device_id.to_string())
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(access_count.0, 1);
    }

    #[tokio::test]
    async fn archived_rerun_does_not_grant_new_session_to_host_only_devices() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let inherited_device_id = Uuid::new_v4();
        let host_only_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        for device_id in [inherited_device_id, host_only_device_id] {
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

            sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
                .bind(device_id.to_string())
                .bind(host_id.to_string())
                .execute(&state.db)
                .await
                .unwrap();
        }

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(inherited_device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let session = (
            "archived".to_string(),
            host_id.to_string(),
            workspace_id.to_string(),
            "archived".to_string(),
            "claude_code".to_string(),
            None,
        );

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            handle_archived_rerun(
                &state_for_request,
                "tr-test",
                inherited_device_id,
                archived_session_id,
                "rerun",
                session,
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;
        let response_session_id = response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let inherited_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(inherited_device_id.to_string())
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(inherited_count.0, 1);

        let host_only_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(host_only_device_id.to_string())
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(host_only_count.0, 0);
    }

    #[tokio::test]
    async fn archived_rerun_via_control_requires_current_host_access_even_with_archived_session_acl(
    ) {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let inherited_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (_daemon_tx, mut daemon_rx) =
            tokio::sync::mpsc::channel::<ve_shared::proto::WsEnvelope>(1);
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(inherited_device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(inherited_device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let before_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();

        let error = control_session(
            State(state.clone()),
            Extension(Claims::for_client(
                inherited_device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            HeaderMap::new(),
            Path(archived_session_id),
            Json(ControlRequest {
                action: "rerun".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(
            matches!(error, ServerError::NotFound(message) if message == format!("Host {}", host_id))
        );

        let after_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(after_count.0, before_count.0);

        assert!(daemon_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn archived_rerun_via_control_grants_new_session_only_to_rerun_requester() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let archived_only_device_id = Uuid::new_v4();
        let rerun_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        for device_id in [archived_only_device_id, rerun_device_id] {
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

            sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
                .bind(device_id.to_string())
                .bind(host_id.to_string())
                .execute(&state.db)
                .await
                .unwrap();
        }

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        for device_id in [archived_only_device_id, rerun_device_id] {
            sqlx::query(
                "INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)",
            )
            .bind(device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();
        }

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            control_session(
                State(state_for_request),
                Extension(Claims::for_client(
                    rerun_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let rerun_response = request_task.await.unwrap().unwrap().0;
        let response_session_id = rerun_response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let rerun_device_sessions = list_sessions(
            SessionCollectionAccess {
                device_id: rerun_device_id,
                host_id: Some(host_id),
            },
            State(state.clone()),
        )
        .await
        .unwrap()
        .0;
        assert!(rerun_device_sessions
            .iter()
            .any(|session| session.session_id == new_session_id));

        let archived_only_sessions = list_sessions(
            SessionCollectionAccess {
                device_id: archived_only_device_id,
                host_id: Some(host_id),
            },
            State(state.clone()),
        )
        .await
        .unwrap()
        .0;
        assert!(!archived_only_sessions
            .iter()
            .any(|session| session.session_id == new_session_id));
    }

    #[tokio::test]
    async fn archived_rerun_persists_origin_link_on_first_live_rerun() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let rerun_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[rerun_device_id],
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            control_session(
                State(state_for_request),
                Extension(Claims::for_client(
                    rerun_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        let rerun_row_before_ack: (String, String, String) = sqlx::query_as(
            r#"
            SELECT status, claude_session_id, rerun_from_session_id
            FROM sessions
            WHERE session_id = $1
            "#,
        )
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(rerun_row_before_ack.0, "dispatching");
        assert_eq!(rerun_row_before_ack.1, "claude-session-1");
        assert_eq!(rerun_row_before_ack.2, archived_session_id.to_string());

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let rerun_response = request_task.await.unwrap().unwrap().0;
        let response_session_id = rerun_response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let rerun_row_after_ack: (String, String, String) = sqlx::query_as(
            r#"
            SELECT status, claude_session_id, rerun_from_session_id
            FROM sessions
            WHERE session_id = $1
            "#,
        )
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(rerun_row_after_ack.0, "pending");
        assert_eq!(rerun_row_after_ack.1, "claude-session-1");
        assert_eq!(rerun_row_after_ack.2, archived_session_id.to_string());

        let active_rerun_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sessions WHERE rerun_from_session_id = $1 AND status NOT IN ('dispatching', 'archived', 'error')",
        )
        .bind(archived_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(active_rerun_count.0, 1);

        let access_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(rerun_device_id.to_string())
        .bind(new_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(access_count.0, 1);
    }

    #[tokio::test]
    async fn archived_rerun_waits_for_daemon_ack_before_marking_pending() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let rerun_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[rerun_device_id],
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let request_task = tokio::spawn(async move {
            control_session(
                State(state_for_request),
                Extension(Claims::for_client(
                    rerun_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let new_session_id = queued.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        let status_before: (String,) =
            sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
                .bind(new_session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(status_before.0, "dispatching");
        assert!(!request_task.is_finished());

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;
        let response_session_id = response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(response_session_id, new_session_id);

        let status_after: (String,) =
            sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
                .bind(new_session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(status_after.0, "pending");
    }

    #[tokio::test]
    async fn archived_rerun_returns_existing_active_rerun_on_repeat_request() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let first_device_id = Uuid::new_v4();
        let second_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(2);

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[first_device_id, second_device_id],
        )
        .await;
        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_first = state.clone();
        let first_task = tokio::spawn(async move {
            control_session(
                State(state_for_first),
                Extension(Claims::for_client(
                    first_device_id,
                    "device",
                    chrono::Duration::hours(1),
                )),
                HeaderMap::new(),
                Path(archived_session_id),
                Json(ControlRequest {
                    action: "rerun".to_string(),
                }),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "rerun_session");
        let first_session_id = queued.payload["payload"]["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let first_response = first_task.await.unwrap().unwrap().0;
        let first_response_session_id = first_response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        assert_eq!(first_response_session_id, first_session_id);

        let second_response = control_session(
            State(state.clone()),
            Extension(Claims::for_client(
                second_device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            HeaderMap::new(),
            Path(archived_session_id),
            Json(ControlRequest {
                action: "rerun".to_string(),
            }),
        )
        .await
        .unwrap()
        .0;
        let second_session_id = second_response["session_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();

        assert_eq!(second_session_id, first_session_id);

        let active_rerun_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sessions WHERE rerun_from_session_id = $1 AND status NOT IN ('dispatching', 'archived', 'error')",
        )
        .bind(archived_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(active_rerun_count.0, 1);

        let second_device_access: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(second_device_id.to_string())
        .bind(first_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(second_device_access.0, 1);
    }

    #[tokio::test]
    async fn archived_rerun_returns_conflict_while_existing_rerun_is_dispatching() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let first_device_id = Uuid::new_v4();
        let second_device_id = Uuid::new_v4();
        let dispatching_session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

        insert_archived_rerun_fixture(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            &[first_device_id, second_device_id],
        )
        .await;

        sqlx::query(
            r#"
            INSERT INTO sessions (
                session_id, title, host_id, workspace_id, agent_type, status,
                claude_session_id, rerun_from_session_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, 'dispatching', $6, $7, $8, $8)
            "#,
        )
        .bind(dispatching_session_id.to_string())
        .bind("dispatching rerun")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind("claude-session-1")
        .bind(archived_session_id.to_string())
        .bind(&now)
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(first_device_id.to_string())
            .bind(dispatching_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let error = control_session(
            State(state.clone()),
            Extension(Claims::for_client(
                second_device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            HeaderMap::new(),
            Path(archived_session_id),
            Json(ControlRequest {
                action: "rerun".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ServerError::Conflict(message)
                if message == "Archived session rerun is still dispatching; retry shortly"
        ));

        let second_device_access: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM device_session_access WHERE device_id = $1 AND session_id = $2",
        )
        .bind(second_device_id.to_string())
        .bind(dispatching_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(second_device_access.0, 0);
    }

    #[tokio::test]
    async fn archived_rerun_marks_new_session_error_when_daemon_queue_fails() {
        let state = setup_state().await;
        let archived_session_id = Uuid::new_v4();
        let inherited_device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let metadata = ArchiveMetadata {
            workspace_path: "/tmp".to_string(),
            workspace_display_name: Some("tmp".to_string()),
            agent_type: "claude_code".to_string(),
            closed_by: "server".to_string(),
            final_summary: None,
            claude_session_id: Some("claude-session-1".to_string()),
            statistics: Some(ArchiveStatistics {
                message_count: 0,
                event_count: 0,
                permission_count: 0,
                duration_seconds: 0,
            }),
            last_commit_sha: None,
            last_commit_message: None,
        };

        insert_host_workspace_and_archive(
            &state,
            archived_session_id,
            host_id,
            workspace_id,
            Some(serde_json::to_string(&metadata).unwrap()),
        )
        .await;

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(inherited_device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
            .bind(inherited_device_id.to_string())
            .bind(host_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(archived_session_id.to_string())
        .bind("archived")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(inherited_device_id.to_string())
            .bind(archived_session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let before_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();

        let error = control_session(
            State(state.clone()),
            Extension(Claims::for_client(
                inherited_device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            HeaderMap::new(),
            Path(archived_session_id),
            Json(ControlRequest {
                action: "rerun".to_string(),
            }),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ServerError::Conflict(message)
                if message == "Failed to queue rerun request to daemon"
        ));

        let after_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(after_count.0, before_count.0 + 1);

        let rerun_row: (String, Option<String>) = sqlx::query_as(
            r#"
            SELECT status, latest_summary
            FROM sessions
            WHERE host_id = $1 AND workspace_id = $2 AND session_id != $3
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind(archived_session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(rerun_row.0, "error");
        assert_eq!(
            rerun_row.1.as_deref(),
            Some("Failed to queue rerun request to daemon")
        );
    }

    #[tokio::test]
    async fn close_session_waits_for_daemon_ack_before_creating_archive() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::channel(1);

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
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_daemon(host_id, daemon_tx).await;

        let state_for_request = state.clone();
        let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
        let request_task = tokio::spawn(async move {
            close_session(
                State(state_for_request),
                Extension(claims),
                HeaderMap::new(),
                Path(session_id),
            )
            .await
        });

        let queued = daemon_rx.recv().await.unwrap();
        assert_eq!(queued.r#type, "close_session");
        assert_eq!(
            queued.request_id.as_deref().map(|s| s.is_empty()),
            Some(false)
        );
        assert_eq!(
            queued.payload["payload"]["session_id"],
            serde_json::json!(session_id.to_string())
        );

        let status_before: (String,) =
            sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(status_before.0, "running");

        let archive_count_before: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(archive_count_before.0, 0);
        assert!(!request_task.is_finished());

        state
            .hub
            .complete_with_ack(ve_shared::proto::AckPayload {
                request_id: queued.request_id.clone().unwrap(),
                success: true,
                error: None,
            })
            .await;

        let response = request_task.await.unwrap().unwrap().0;

        assert_eq!(response["success"], serde_json::json!(true));
        assert_eq!(response["close_requested"], serde_json::json!(true));
        assert!(response.get("archive_id").is_none());

        let status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(status.0, "running");

        let archive_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(archive_count.0, 0);
    }

    #[tokio::test]
    async fn close_session_returns_conflict_when_archive_row_is_missing_for_archived_session() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

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
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        let claims = Claims::for_client(device_id, "device", chrono::Duration::hours(1));
        let error = close_session(
            State(state.clone()),
            Extension(claims),
            HeaderMap::new(),
            Path(session_id),
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            ServerError::Conflict(message)
                if message == format!("Session {} is archived without an archive record", session_id)
        ));
    }

    #[tokio::test]
    async fn list_messages_rejects_page_zero() {
        let state = setup_state().await;
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

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
        .bind("/tmp")
        .bind("tmp")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
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

        let error = list_messages(
            State(state.clone()),
            Extension(Claims::for_client(
                device_id,
                "device",
                chrono::Duration::hours(1),
            )),
            Path(session_id),
            Query(MessageListQuery {
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
