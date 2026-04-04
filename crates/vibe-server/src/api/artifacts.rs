use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};

use crate::{
    api::types::{
        AccessKeyPath, AccessKeyValue, ApiError, Artifact, ArtifactInfo, ArtifactPath,
        CreateAccessKeyBody, CreateAccessKeyResponse, CreateArtifactBody, GetAccessKeyResponse,
        SuccessResponse, UpdateAccessKeyBody, UpdateAccessKeyResponse, UpdateArtifactBody,
        UpdateArtifactResponse,
    },
    auth::AuthenticatedUser,
    context::AppContext,
    storage::db::{AccessKeyRecord, ArtifactCreateOutcome, now_ms},
};

pub fn routes() -> Router<AppContext> {
    Router::new()
        .route("/v1/artifacts", get(list_artifacts).post(create_artifact))
        .route(
            "/v1/artifacts/{id}",
            get(get_artifact)
                .post(update_artifact)
                .delete(delete_artifact),
        )
        .route(
            "/v1/access-keys/{session_id}/{machine_id}",
            get(get_access_key)
                .post(create_access_key)
                .put(update_access_key),
        )
}

async fn list_artifacts(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<ArtifactInfo>>, ApiError> {
    let mut artifacts = ctx.db().read(|state| {
        state
            .artifacts
            .values()
            .filter(|artifact| artifact.account_id == user.user_id)
            .cloned()
            .collect::<Vec<_>>()
    });
    artifacts.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(Json(
        artifacts.iter().map(ArtifactInfo::from_record).collect(),
    ))
}

async fn get_artifact(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<ArtifactPath>,
) -> Result<Json<Artifact>, ApiError> {
    let artifact = ctx
        .db()
        .read(|state| state.artifacts.get(&path.id).cloned())
        .filter(|artifact| artifact.account_id == user.user_id)
        .ok_or_else(|| ApiError::not_found("Artifact not found"))?;
    Ok(Json(Artifact::from_record(&artifact)))
}

async fn create_artifact(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<CreateArtifactBody>,
) -> Result<impl IntoResponse, ApiError> {
    uuid::Uuid::parse_str(&body.id).map_err(|_| ApiError::bad_request("id must be a UUID"))?;
    let user_id = user.user_id;
    let outcome = ctx.db().create_artifact(
        &user_id,
        body.id,
        body.header,
        body.body,
        body.data_encryption_key,
    );
    match outcome {
        ArtifactCreateOutcome::ExistingOwned(existing) => {
            return Ok((
                StatusCode::OK,
                Json(serde_json::to_value(Artifact::from_record(&existing)).unwrap()),
            ));
        }
        ArtifactCreateOutcome::ExistsOtherAccount => {
            return Ok((
                StatusCode::CONFLICT,
                Json(
                    serde_json::json!({ "error": "Artifact with this ID already exists for another account" }),
                ),
            ));
        }
        ArtifactCreateOutcome::Created(record) => {
            ctx.events()
                .publish_new_artifact(ctx.db(), &user_id, &record)
                .map_err(|_| ApiError::internal("Failed to create artifact"))?;
            Ok((
                StatusCode::OK,
                Json(serde_json::to_value(Artifact::from_record(&record)).unwrap()),
            ))
        }
    }
}

async fn update_artifact(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<ArtifactPath>,
    Json(body): Json<UpdateArtifactBody>,
) -> Result<Json<UpdateArtifactResponse>, ApiError> {
    let mut header_update = None;
    let mut body_update = None;
    let response = ctx.db().write(|state| {
        let Some(artifact) = state.artifacts.get_mut(&path.id) else {
            return Err(ApiError::not_found("Artifact not found"));
        };
        if artifact.account_id != user.user_id {
            return Err(ApiError::not_found("Artifact not found"));
        }

        let mut response = UpdateArtifactResponse {
            success: true,
            header_version: None,
            body_version: None,
            error: None,
            current_header_version: None,
            current_header: None,
            current_body_version: None,
            current_body: None,
        };
        let mut next_header = artifact.header.clone();
        let mut next_body = artifact.body.clone();
        let mut next_header_version = artifact.header_version;
        let mut next_body_version = artifact.body_version;
        let mut changed = false;

        if let (Some(header), Some(expected_version)) = (&body.header, body.expected_header_version)
        {
            if artifact.header_version != expected_version {
                response.success = false;
                response.error = Some("version-mismatch".into());
                response.current_header_version = Some(artifact.header_version);
                response.current_header = Some(artifact.header.clone());
            } else {
                next_header = header.clone();
                next_header_version += 1;
            }
        }

        if let (Some(body_value), Some(expected_version)) = (&body.body, body.expected_body_version)
        {
            if artifact.body_version != expected_version {
                response.success = false;
                response.error = Some("version-mismatch".into());
                response.current_body_version = Some(artifact.body_version);
                response.current_body = Some(artifact.body.clone());
            } else {
                next_body = body_value.clone();
                next_body_version += 1;
            }
        }

        if response.success {
            if next_header_version != artifact.header_version {
                artifact.header = next_header.clone();
                artifact.header_version = next_header_version;
                response.header_version = Some(next_header_version);
                header_update = Some(crate::events::socket_updates::LateVersionedValue {
                    value: next_header,
                    version: next_header_version,
                });
                changed = true;
            }
            if next_body_version != artifact.body_version {
                artifact.body = next_body.clone();
                artifact.body_version = next_body_version;
                response.body_version = Some(next_body_version);
                body_update = Some(crate::events::socket_updates::LateVersionedValue {
                    value: next_body,
                    version: next_body_version,
                });
                changed = true;
            }
        }

        if changed {
            artifact.seq += 1;
            artifact.updated_at = now_ms();
        }
        Ok(response)
    })?;
    if response.success {
        if header_update.is_some() || body_update.is_some() {
            ctx.events()
                .publish_update_artifact(
                    ctx.db(),
                    &user.user_id,
                    &path.id,
                    header_update,
                    body_update,
                )
                .map_err(|_| ApiError::internal("Failed to update artifact"))?;
        }
    }
    Ok(Json(response))
}

async fn delete_artifact(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<ArtifactPath>,
) -> Result<Json<SuccessResponse>, ApiError> {
    let deleted = ctx.db().write(|state| {
        state
            .artifacts
            .get(&path.id)
            .filter(|artifact| artifact.account_id == user.user_id)
            .cloned()
            .map(|_| state.artifacts.remove(&path.id))
    });
    if deleted.flatten().is_none() {
        return Err(ApiError::not_found("Artifact not found"));
    }
    ctx.events()
        .publish_delete_artifact(ctx.db(), &user.user_id, &path.id)
        .map_err(|_| ApiError::internal("Failed to delete artifact"))?;
    Ok(Json(SuccessResponse { success: true }))
}

async fn get_access_key(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<AccessKeyPath>,
) -> Result<Json<GetAccessKeyResponse>, ApiError> {
    verify_access_key_parents(&ctx, &user.user_id, &path)?;
    let access_key = ctx.db().read(|state| {
        state
            .access_keys
            .get(&(
                user.user_id.clone(),
                path.session_id.clone(),
                path.machine_id.clone(),
            ))
            .cloned()
    });
    Ok(Json(GetAccessKeyResponse {
        access_key: access_key.as_ref().map(AccessKeyValue::from_record),
    }))
}

async fn create_access_key(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<AccessKeyPath>,
    Json(body): Json<CreateAccessKeyBody>,
) -> Result<impl IntoResponse, ApiError> {
    verify_access_key_parents(&ctx, &user.user_id, &path)?;
    let key = (
        user.user_id.clone(),
        path.session_id.clone(),
        path.machine_id.clone(),
    );
    if ctx.db().read(|state| state.access_keys.contains_key(&key)) {
        return Ok((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "Access key already exists" })),
        ));
    }
    let record = AccessKeyRecord {
        account_id: user.user_id,
        session_id: path.session_id,
        machine_id: path.machine_id,
        data: body.data,
        data_version: 1,
        created_at: now_ms(),
        updated_at: now_ms(),
    };
    ctx.db()
        .write(|state| state.access_keys.insert(key, record.clone()));
    Ok((
        StatusCode::OK,
        Json(
            serde_json::to_value(CreateAccessKeyResponse {
                success: true,
                access_key: AccessKeyValue::from_record(&record),
            })
            .unwrap(),
        ),
    ))
}

async fn update_access_key(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<AccessKeyPath>,
    Json(body): Json<UpdateAccessKeyBody>,
) -> Result<Json<UpdateAccessKeyResponse>, ApiError> {
    let key = (
        user.user_id.clone(),
        path.session_id.clone(),
        path.machine_id.clone(),
    );
    let response = ctx.db().write(|state| {
        let Some(record) = state.access_keys.get_mut(&key) else {
            return Err(ApiError::not_found("Access key not found"));
        };
        if record.data_version != body.expected_version {
            return Ok(UpdateAccessKeyResponse {
                success: false,
                version: None,
                error: Some("version-mismatch".into()),
                current_version: Some(record.data_version),
                current_data: Some(record.data.clone()),
            });
        }
        record.data = body.data;
        record.data_version += 1;
        record.updated_at = now_ms();
        Ok(UpdateAccessKeyResponse {
            success: true,
            version: Some(record.data_version),
            error: None,
            current_version: None,
            current_data: None,
        })
    })?;
    Ok(Json(response))
}

fn verify_access_key_parents(
    ctx: &AppContext,
    user_id: &str,
    path: &AccessKeyPath,
) -> Result<(), ApiError> {
    if ctx
        .db()
        .get_session_for_account(user_id, &path.session_id)
        .is_none()
        || ctx
            .db()
            .get_machine_for_account(user_id, &path.machine_id)
            .is_none()
    {
        return Err(ApiError::not_found("Session or machine not found"));
    }
    Ok(())
}
