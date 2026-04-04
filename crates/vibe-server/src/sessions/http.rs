use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get},
};

use crate::{
    api::types::{
        ActiveSessionListResponse, ActiveSessionsQuery, ApiError, CreateOrLoadSessionBody,
        CreateOrLoadSessionResponse, CursorPagedSessionListResponse, CursorPagedSessionsQuery,
        SessionHistoryResponse, SessionHttpRecord, SessionListResponse, SessionMessageHttpRecord,
        SessionMessageSendRecord, SessionPath, V3MessagesQuery, V3MessagesResponse,
        V3SendMessagesBody, V3SendMessagesResponse,
    },
    auth::AuthenticatedUser,
    context::AppContext,
    sessions::service::SessionsService,
};

pub fn routes() -> Router<AppContext> {
    Router::new()
        .route(
            "/v1/sessions",
            get(list_sessions).post(create_or_load_session),
        )
        .route("/v2/sessions/active", get(list_active_sessions))
        .route("/v2/sessions", get(list_sessions_v2))
        .route(
            "/v1/sessions/{session_id}/messages",
            get(get_session_messages),
        )
        .route("/v1/sessions/{session_id}", delete(delete_session))
        .route(
            "/v3/sessions/{session_id}/messages",
            get(get_session_messages_v3).post(post_session_messages_v3),
        )
}

fn validate_limit(limit: Option<usize>, default: usize, max: usize) -> Result<usize, ApiError> {
    let limit = limit.unwrap_or(default);
    if !(1..=max).contains(&limit) {
        return Err(ApiError::bad_request(format!(
            "limit must be between 1 and {max}",
        )));
    }
    Ok(limit)
}

fn validate_changed_since(changed_since: Option<u64>) -> Result<Option<u64>, ApiError> {
    match changed_since {
        Some(0) => Err(ApiError::bad_request(
            "changedSince must be a positive integer",
        )),
        other => Ok(other),
    }
}

async fn list_sessions(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<SessionListResponse>, ApiError> {
    let service = SessionsService::new(ctx);
    let sessions = service
        .list_v1(&user.user_id)
        .into_iter()
        .map(|session| SessionHttpRecord::from_record(&session, true))
        .collect();
    Ok(Json(SessionListResponse { sessions }))
}

async fn list_active_sessions(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Query(query): Query<ActiveSessionsQuery>,
) -> Result<Json<ActiveSessionListResponse>, ApiError> {
    let service = SessionsService::new(ctx);
    let limit = validate_limit(query.limit, 150, 500)?;
    let sessions = service
        .list_active(&user.user_id, limit)
        .into_iter()
        .map(|session| SessionHttpRecord::from_record(&session, false))
        .collect();
    Ok(Json(ActiveSessionListResponse { sessions }))
}

async fn list_sessions_v2(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Query(query): Query<CursorPagedSessionsQuery>,
) -> Result<Json<CursorPagedSessionListResponse>, ApiError> {
    let service = SessionsService::new(ctx);
    let limit = validate_limit(query.limit, 50, 200)?;
    let changed_since = validate_changed_since(query.changed_since)?;
    let (sessions, next_cursor, has_next) =
        service.list_v2(&user.user_id, query.cursor.as_deref(), limit, changed_since)?;
    Ok(Json(CursorPagedSessionListResponse {
        sessions: sessions
            .into_iter()
            .map(|session| SessionHttpRecord::from_record(&session, false))
            .collect(),
        next_cursor,
        has_next,
    }))
}

async fn create_or_load_session(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<CreateOrLoadSessionBody>,
) -> Result<Json<CreateOrLoadSessionResponse>, ApiError> {
    let service = SessionsService::new(ctx);
    let session = service.create_or_load(
        &user.user_id,
        &body.tag,
        &body.metadata,
        body.data_encryption_key,
    )?;
    Ok(Json(CreateOrLoadSessionResponse {
        session: SessionHttpRecord::from_record(&session, true),
    }))
}

async fn get_session_messages(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<SessionPath>,
) -> Result<Json<SessionHistoryResponse>, ApiError> {
    let service = SessionsService::new(ctx);
    let messages = service.history(&user.user_id, &path.session_id)?;
    Ok(Json(SessionHistoryResponse {
        messages: messages
            .into_iter()
            .map(|message| SessionMessageHttpRecord::from_record(&message))
            .collect(),
    }))
}

async fn delete_session(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<SessionPath>,
) -> Result<Json<crate::api::types::SuccessResponse>, ApiError> {
    let service = SessionsService::new(ctx);
    service.delete(&user.user_id, &path.session_id)?;
    Ok(Json(crate::api::types::SuccessResponse { success: true }))
}

async fn get_session_messages_v3(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<SessionPath>,
    Query(query): Query<V3MessagesQuery>,
) -> Result<Json<V3MessagesResponse>, ApiError> {
    let service = SessionsService::new(ctx);
    let after_seq = query.after_seq.unwrap_or(0);
    let limit = validate_limit(query.limit, 100, 500)?;
    let (messages, has_more) =
        service.page_messages(&user.user_id, &path.session_id, after_seq, limit)?;
    Ok(Json(V3MessagesResponse {
        messages: messages
            .into_iter()
            .map(|message| SessionMessageHttpRecord::from_record(&message))
            .collect(),
        has_more,
    }))
}

async fn post_session_messages_v3(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<SessionPath>,
    Json(body): Json<V3SendMessagesBody>,
) -> Result<Json<V3SendMessagesResponse>, ApiError> {
    if body.messages.is_empty() || body.messages.len() > 100 {
        return Err(ApiError::bad_request("messages must contain 1..100 items"));
    }
    if body
        .messages
        .iter()
        .any(|message| message.local_id.is_empty())
    {
        return Err(ApiError::bad_request(
            "messages[*].localId must be non-empty",
        ));
    }
    let service = SessionsService::new(ctx.clone());
    let (messages, created) = service.append_bulk_messages(
        &user.user_id,
        &path.session_id,
        body.messages
            .into_iter()
            .map(|message| (message.content, message.local_id))
            .collect(),
    )?;

    for message in &created {
        service.emit_new_message(&user.user_id, &path.session_id, message, None)?;
    }

    Ok(Json(V3SendMessagesResponse {
        messages: messages
            .into_iter()
            .map(|message| SessionMessageSendRecord::from_record(&message))
            .collect(),
    }))
}
