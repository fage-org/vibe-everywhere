use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};

use crate::{
    api::types::{
        ApiError, CreateOrLoadMachineBody, CreateOrLoadMachineHttpRecord,
        CreateOrLoadMachineResponse, MachineDetailResponse, MachineHttpRecord, MachinePath,
    },
    auth::AuthenticatedUser,
    context::AppContext,
    machines::service::MachinesService,
};

pub fn routes() -> Router<AppContext> {
    Router::new()
        .route(
            "/v1/machines",
            get(list_machines).post(create_or_load_machine),
        )
        .route("/v1/machines/{id}", get(get_machine))
}

async fn create_or_load_machine(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<CreateOrLoadMachineBody>,
) -> Result<Json<CreateOrLoadMachineResponse>, ApiError> {
    let service = MachinesService::new(ctx);
    let machine = service.create_or_load(
        &user.user_id,
        &body.id,
        &body.metadata,
        body.daemon_state,
        body.data_encryption_key,
    )?;
    Ok(Json(CreateOrLoadMachineResponse {
        machine: CreateOrLoadMachineHttpRecord::from_record(&machine),
    }))
}

async fn list_machines(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<MachineHttpRecord>>, ApiError> {
    let service = MachinesService::new(ctx);
    Ok(Json(
        service
            .list(&user.user_id)
            .into_iter()
            .map(|machine| MachineHttpRecord::from_record(&machine))
            .collect(),
    ))
}

async fn get_machine(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<MachinePath>,
) -> Result<Json<MachineDetailResponse>, ApiError> {
    let service = MachinesService::new(ctx);
    let machine = service.detail(&user.user_id, &path.id)?;
    Ok(Json(MachineDetailResponse {
        machine: MachineHttpRecord::from_record(&machine),
    }))
}
