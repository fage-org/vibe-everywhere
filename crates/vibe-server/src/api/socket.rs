use std::time::Duration;

use serde_json::json;
use socketioxide::{
    SocketIo, TransportType,
    extract::{AckSender, SocketRef, State, TryData},
};

use crate::{
    api::types::{
        AccessKeyGetPayload, ArtifactCreatePayload, ArtifactReadPayload, ArtifactUpdatePayload,
        SessionAlivePayload, SessionEndPayload, SocketMessagePayload, SocketUpdateMetadataPayload,
        SocketUpdateStatePayload, UsageReportPayload,
    },
    auth::{SocketAuthPayload, SocketClientType, SocketConnectionAuth, validate_socket_auth},
    context::AppContext,
    events::{ClientConnection, build_usage_update},
    machines::socket::register_handlers as register_machine_handlers,
    sessions::SessionsService,
    storage::db::CompareAndSwap,
};

pub fn build_layer(ctx: AppContext) -> socketioxide::layer::SocketIoLayer {
    let (layer, io) = SocketIo::builder()
        .with_state(ctx)
        .req_path("/v1/updates")
        .ping_timeout(Duration::from_millis(45_000))
        .ping_interval(Duration::from_millis(15_000))
        .connect_timeout(Duration::from_millis(20_000))
        .transports([TransportType::Websocket, TransportType::Polling])
        .build_layer();
    register_namespace(&io);
    layer
}

fn register_namespace(io: &SocketIo) {
    io.ns("/", on_connect);
}

async fn on_connect(
    socket: SocketRef,
    State(ctx): State<AppContext>,
    TryData(payload): TryData<SocketAuthPayload>,
) {
    let auth = match validate_socket_auth(payload, &ctx).await {
        Ok(auth) => auth,
        Err(error) => {
            reject_socket(&socket, &error.to_string());
            return;
        }
    };

    let socket_id = socket.id.to_string();
    ctx.metrics().incr_socket_event("connect");
    ctx.metrics()
        .connection_opened(socket_client_type_label(&auth.client_type));
    ctx.events().add_connection(ClientConnection {
        socket_id: socket_id.clone(),
        socket: socket.clone(),
        user_id: auth.user_id.clone(),
        auth: auth.clone(),
    });

    if matches!(auth.client_type, SocketClientType::MachineScoped)
        && let Some(machine_id) = auth.machine_id.as_deref()
    {
        ctx.presence()
            .machine_connected(&auth.user_id, machine_id, crate::storage::db::now_ms())
            .await;
    }

    register_session_handlers(socket.clone(), ctx.clone(), auth.clone());
    register_machine_handlers(socket.clone(), ctx.clone(), auth.clone());
    register_aux_handlers(socket.clone(), ctx.clone(), auth.clone());

    let disconnect_ctx = ctx.clone();
    let disconnect_auth = auth.clone();
    socket.on_disconnect(move || {
        let ctx = disconnect_ctx.clone();
        let auth = disconnect_auth.clone();
        let socket_id = socket_id.clone();
        async move {
            ctx.events().remove_connection(&auth.user_id, &socket_id);
            ctx.rpc().cleanup_socket(&auth.user_id, &socket_id);
            ctx.metrics().incr_socket_event("disconnect");
            ctx.metrics()
                .connection_closed(socket_client_type_label(&auth.client_type));
            if matches!(auth.client_type, SocketClientType::MachineScoped)
                && let Some(machine_id) = auth.machine_id.as_deref()
            {
                ctx.presence()
                    .machine_disconnected(&auth.user_id, machine_id, crate::storage::db::now_ms())
                    .await;
            }
        }
    });
}

fn reject_socket(socket: &SocketRef, message: &str) {
    let _ = socket.emit("error", &json!({ "message": message }));
    let _ = socket.clone().disconnect();
}

fn register_session_handlers(socket: SocketRef, ctx: AppContext, auth: SocketConnectionAuth) {
    let ping_ctx = ctx.clone();
    socket.on("ping", move |ack: AckSender| async move {
        ping_ctx.metrics().incr_socket_event("ping");
        let _ = ack.send(&json!({}));
    });

    let metadata_ctx = ctx.clone();
    let metadata_auth = auth.clone();
    socket.on(
        "update-metadata",
        move |ack: AckSender, TryData(payload): TryData<SocketUpdateMetadataPayload>| {
            let ctx = metadata_ctx.clone();
            let auth = metadata_auth.clone();
            async move {
                ctx.metrics().incr_socket_event("update-metadata");
                let Ok(payload) = payload else {
                    let response = json!({"result": "error"});
                    let _ = ack.send(&response);
                    return;
                };
                let service = SessionsService::new(ctx);
                match service.update_metadata(
                    &auth.user_id,
                    &payload.sid,
                    payload.expected_version,
                    payload.metadata.clone(),
                ) {
                    Ok(CompareAndSwap::Success(value)) => {
                        let response = json!({
                            "result": "success",
                            "version": payload.expected_version + 1,
                            "metadata": value,
                        });
                        let _ = ack.send(&response);
                    }
                    Ok(CompareAndSwap::VersionMismatch {
                        current_version,
                        current_value,
                    }) => {
                        let response = json!({
                            "result": "version-mismatch",
                            "version": current_version,
                            "metadata": current_value,
                        });
                        let _ = ack.send(&response);
                    }
                    Ok(CompareAndSwap::NotFound) | Err(_) => {
                        let response = json!({"result": "error"});
                        let _ = ack.send(&response);
                    }
                }
            }
        },
    );

    let state_ctx = ctx.clone();
    let state_auth = auth.clone();
    socket.on(
        "update-state",
        move |ack: AckSender, TryData(payload): TryData<SocketUpdateStatePayload>| {
            let ctx = state_ctx.clone();
            let auth = state_auth.clone();
            async move {
                ctx.metrics().incr_socket_event("update-state");
                let Ok(payload) = payload else {
                    let response = json!({"result": "error"});
                    let _ = ack.send(&response);
                    return;
                };
                let Some(agent_state) = payload.agent_state.clone() else {
                    let response = json!({"result": "error"});
                    let _ = ack.send(&response);
                    return;
                };
                let service = SessionsService::new(ctx);
                match service.update_agent_state(
                    &auth.user_id,
                    &payload.sid,
                    payload.expected_version,
                    agent_state,
                ) {
                    Ok(CompareAndSwap::Success(value)) => {
                        let response = json!({
                            "result": "success",
                            "version": payload.expected_version + 1,
                            "agentState": value,
                        });
                        let _ = ack.send(&response);
                    }
                    Ok(CompareAndSwap::VersionMismatch {
                        current_version,
                        current_value,
                    }) => {
                        let response = json!({
                            "result": "version-mismatch",
                            "version": current_version,
                            "agentState": current_value,
                        });
                        let _ = ack.send(&response);
                    }
                    Ok(CompareAndSwap::NotFound) | Err(_) => {
                        let response = json!({"result": "error"});
                        let _ = ack.send(&response);
                    }
                }
            }
        },
    );

    let alive_ctx = ctx.clone();
    let alive_auth = auth.clone();
    socket.on(
        "session-alive",
        move |TryData(payload): TryData<SessionAlivePayload>| {
            let ctx = alive_ctx.clone();
            let auth = alive_auth.clone();
            async move {
                if let Ok(payload) = payload {
                    ctx.metrics().incr_socket_event("session-alive");
                    ctx.presence()
                        .session_alive(
                            &auth.user_id,
                            &payload.sid,
                            payload.time,
                            payload.thinking.unwrap_or(false),
                        )
                        .await;
                }
            }
        },
    );

    let message_ctx = ctx.clone();
    let message_auth = auth.clone();
    socket.on(
        "message",
        move |TryData(payload): TryData<SocketMessagePayload>, socket: SocketRef| {
            let ctx = message_ctx.clone();
            let auth = message_auth.clone();
            async move {
                let Ok(payload) = payload else {
                    return;
                };
                ctx.metrics().incr_socket_event("message");
                let service = SessionsService::new(ctx);
                let _ = service.append_single_message(
                    &auth.user_id,
                    &payload.sid,
                    payload.message,
                    payload.local_id,
                    Some(&socket.id.to_string()),
                );
            }
        },
    );

    let end_ctx = ctx.clone();
    let end_auth = auth;
    socket.on(
        "session-end",
        move |TryData(payload): TryData<SessionEndPayload>| {
            let ctx = end_ctx.clone();
            let auth = end_auth.clone();
            async move {
                if let Ok(payload) = payload {
                    ctx.metrics().incr_socket_event("session-end");
                    ctx.presence()
                        .session_end(&auth.user_id, &payload.sid, payload.time)
                        .await;
                }
            }
        },
    );
}

fn register_aux_handlers(socket: SocketRef, ctx: AppContext, auth: SocketConnectionAuth) {
    let usage_ctx = ctx.clone();
    let usage_auth = auth.clone();
    socket.on(
        "usage-report",
        move |ack: AckSender, TryData(payload): TryData<UsageReportPayload>| {
            let ctx = usage_ctx.clone();
            let auth = usage_auth.clone();
            async move {
                ctx.metrics().incr_socket_event("usage-report");
                let Ok(payload) = payload else {
                    let _ = ack.send(&json!({"success": false, "error": "Invalid parameters"}));
                    return;
                };
                if payload.key.is_empty() {
                    let _ = ack.send(&json!({"success": false, "error": "Invalid key"}));
                    return;
                }
                if payload.tokens.get("total").is_none() {
                    let _ = ack.send(&json!({"success": false, "error": "Invalid tokens object - must include total"}));
                    return;
                }
                if payload.cost.get("total").is_none() {
                    let _ = ack.send(&json!({"success": false, "error": "Invalid cost object - must include total"}));
                    return;
                }
                if let Some(session_id) = payload.session_id.as_deref()
                    && ctx
                        .db()
                        .get_session_for_account(&auth.user_id, session_id)
                        .is_none()
                {
                    let _ = ack.send(&json!({"success": false, "error": "Session not found"}));
                    return;
                }
                let report = ctx.db().write(|state| {
                    let existing = state
                        .usage_reports
                        .values()
                        .find(|report| {
                            report.account_id == auth.user_id
                                && report.session_id == payload.session_id
                                && report.key == payload.key
                        })
                        .cloned();
                    let now = crate::storage::db::now_ms();
                    let report = crate::storage::db::UsageReportRecord {
                        id: existing
                            .as_ref()
                            .map(|report| report.id.clone())
                            .unwrap_or_else(|| format!("usage_{}", uuid::Uuid::now_v7())),
                        account_id: auth.user_id.clone(),
                        session_id: payload.session_id.clone(),
                        key: payload.key.clone(),
                        tokens: payload.tokens.clone(),
                        cost: payload.cost.clone(),
                        created_at: existing
                            .as_ref()
                            .map(|report| report.created_at)
                            .unwrap_or(now),
                        updated_at: now,
                    };
                    state.usage_reports.insert(report.id.clone(), report.clone());
                    report
                });
                if let Some(session_id) = payload.session_id {
                    ctx.events().emit_ephemeral(
                        &auth.user_id,
                        build_usage_update(
                            &session_id,
                            &payload.key,
                            payload.tokens,
                            payload.cost,
                        ),
                        crate::events::RecipientFilter::UserScopedOnly,
                        None,
                    );
                }
                let _ = ack.send(&json!({
                    "success": true,
                    "reportId": report.id,
                    "createdAt": report.created_at,
                    "updatedAt": report.updated_at,
                }));
            }
        },
    );

    let access_ctx = ctx.clone();
    let access_auth = auth.clone();
    socket.on(
        "access-key-get",
        move |ack: AckSender, TryData(payload): TryData<AccessKeyGetPayload>| {
            let ctx = access_ctx.clone();
            let auth = access_auth.clone();
            async move {
                ctx.metrics().incr_socket_event("access-key-get");
                let Ok(payload) = payload else {
                    let _ = ack.send(&json!({"ok": false, "error": "Invalid parameters: sessionId and machineId are required"}));
                    return;
                };
                if ctx
                    .db()
                    .get_session_for_account(&auth.user_id, &payload.session_id)
                    .is_none()
                    || ctx
                        .db()
                        .get_machine_for_account(&auth.user_id, &payload.machine_id)
                        .is_none()
                {
                    let _ = ack.send(&json!({"ok": false, "error": "Session or machine not found"}));
                    return;
                }
                let access_key = ctx.db().read(|state| {
                    state.access_keys.get(&(
                        auth.user_id.clone(),
                        payload.session_id.clone(),
                        payload.machine_id.clone(),
                    )).cloned()
                });
                let response = match access_key {
                    Some(record) => json!({
                        "ok": true,
                        "accessKey": {
                            "data": record.data,
                            "dataVersion": record.data_version,
                            "createdAt": record.created_at,
                            "updatedAt": record.updated_at
                        }
                    }),
                    None => json!({"ok": true, "accessKey": null}),
                };
                let _ = ack.send(&response);
            }
        },
    );

    let artifact_read_ctx = ctx.clone();
    let artifact_read_auth = auth.clone();
    socket.on(
        "artifact-read",
        move |ack: AckSender, TryData(payload): TryData<ArtifactReadPayload>| {
            let ctx = artifact_read_ctx.clone();
            let auth = artifact_read_auth.clone();
            async move {
                ctx.metrics().incr_socket_event("artifact-read");
                let Ok(payload) = payload else {
                    let _ = ack.send(&json!({"result": "error", "message": "Invalid parameters"}));
                    return;
                };
                let artifact = ctx
                    .db()
                    .read(|state| state.artifacts.get(&payload.artifact_id).cloned());
                let Some(artifact) =
                    artifact.filter(|artifact| artifact.account_id == auth.user_id)
                else {
                    let _ = ack.send(&json!({"result": "error", "message": "Artifact not found"}));
                    return;
                };
                let _ = ack.send(&json!({
                    "result": "success",
                    "artifact": {
                        "id": artifact.id,
                        "header": artifact.header,
                        "headerVersion": artifact.header_version,
                        "body": artifact.body,
                        "bodyVersion": artifact.body_version,
                        "seq": artifact.seq,
                        "createdAt": artifact.created_at,
                        "updatedAt": artifact.updated_at
                    }
                }));
            }
        },
    );

    let artifact_create_ctx = ctx.clone();
    let artifact_create_auth = auth.clone();
    socket.on(
        "artifact-create",
        move |ack: AckSender, TryData(payload): TryData<ArtifactCreatePayload>| {
            let ctx = artifact_create_ctx.clone();
            let auth = artifact_create_auth.clone();
            async move {
                ctx.metrics().incr_socket_event("artifact-create");
                let Ok(payload) = payload else {
                    let _ = ack.send(&json!({"result": "error", "message": "Invalid parameters"}));
                    return;
                };
                if uuid::Uuid::parse_str(&payload.id).is_err() {
                    let _ = ack.send(&json!({"result": "error", "message": "Invalid parameters"}));
                    return;
                }
                match ctx.db().create_artifact(
                    &auth.user_id,
                    payload.id,
                    payload.header,
                    payload.body,
                    payload.data_encryption_key,
                ) {
                    crate::storage::db::ArtifactCreateOutcome::ExistsOtherAccount => {
                        let _ = ack.send(&json!({
                            "result": "error",
                            "message": "Artifact with this ID already exists for another account"
                        }));
                    }
                    crate::storage::db::ArtifactCreateOutcome::Created(artifact) => {
                        let _ =
                            ctx.events()
                                .publish_new_artifact(ctx.db(), &auth.user_id, &artifact);
                        let _ = ack.send(&json!({
                            "result": "success",
                            "artifact": {
                                "id": artifact.id,
                                "header": artifact.header,
                                "headerVersion": artifact.header_version,
                                "body": artifact.body,
                                "bodyVersion": artifact.body_version,
                                "seq": artifact.seq,
                                "createdAt": artifact.created_at,
                                "updatedAt": artifact.updated_at
                            }
                        }));
                    }
                    crate::storage::db::ArtifactCreateOutcome::ExistingOwned(existing) => {
                        let _ = ack.send(&json!({
                            "result": "success",
                            "artifact": {
                                "id": existing.id,
                                "header": existing.header,
                                "headerVersion": existing.header_version,
                                "body": existing.body,
                                "bodyVersion": existing.body_version,
                                "seq": existing.seq,
                                "createdAt": existing.created_at,
                                "updatedAt": existing.updated_at
                            }
                        }));
                    }
                }
            }
        },
    );

    let artifact_update_ctx = ctx.clone();
    let artifact_update_auth = auth.clone();
    socket.on(
        "artifact-update",
        move |ack: AckSender, TryData(payload): TryData<ArtifactUpdatePayload>| {
            let ctx = artifact_update_ctx.clone();
            let auth = artifact_update_auth.clone();
            async move {
                ctx.metrics().incr_socket_event("artifact-update");
                let Ok(payload) = payload else {
                    let _ = ack.send(&json!({"result": "error", "message": "Invalid parameters"}));
                    return;
                };
                let result = ctx.db().write(|state| {
                    let Some(artifact) = state.artifacts.get_mut(&payload.artifact_id) else {
                        return Err(json!({"result": "error", "message": "Artifact not found"}));
                    };
                    if artifact.account_id != auth.user_id {
                        return Err(json!({"result": "error", "message": "Artifact not found"}));
                    }
                    if payload.header.is_none() && payload.body.is_none() {
                        return Err(json!({"result": "error", "message": "No updates provided"}));
                    }
                    let header_mismatch = payload
                        .header
                        .as_ref()
                        .is_some_and(|header| artifact.header_version != header.expected_version);
                    let body_mismatch = payload
                        .body
                        .as_ref()
                        .is_some_and(|body| artifact.body_version != body.expected_version);
                    if header_mismatch || body_mismatch {
                        return Err(json!({
                            "result": "version-mismatch",
                            "header": payload.header.as_ref().filter(|_| header_mismatch).map(|_| json!({
                                "currentVersion": artifact.header_version,
                                "currentData": artifact.header,
                            })),
                            "body": payload.body.as_ref().filter(|_| body_mismatch).map(|_| json!({
                                "currentVersion": artifact.body_version,
                                "currentData": artifact.body,
                            })),
                        }));
                    }
                    let mut header_resp = None;
                    let mut body_resp = None;
                    let mut header_update = None;
                    let mut body_update = None;
                    if let Some(header) = payload.header {
                        artifact.header = header.data.clone();
                        artifact.header_version += 1;
                        header_resp = Some(json!({"version": artifact.header_version, "data": artifact.header}));
                        header_update = Some(crate::events::socket_updates::LateVersionedValue {
                            value: header.data,
                            version: artifact.header_version,
                        });
                    }
                    if let Some(body) = payload.body {
                        artifact.body = body.data.clone();
                        artifact.body_version += 1;
                        body_resp = Some(json!({"version": artifact.body_version, "data": artifact.body}));
                        body_update = Some(crate::events::socket_updates::LateVersionedValue {
                            value: body.data,
                            version: artifact.body_version,
                        });
                    }
                    artifact.seq += 1;
                    artifact.updated_at = crate::storage::db::now_ms();
                    Ok((json!({"result": "success", "header": header_resp, "body": body_resp}), header_update, body_update))
                });
                match result {
                    Ok((response, header, body)) => {
                        let _ = ctx.events().publish_update_artifact(
                            ctx.db(),
                            &auth.user_id,
                            &payload.artifact_id,
                            header,
                            body,
                        );
                        let _ = ack.send(&response);
                    }
                    Err(response) => {
                        let _ = ack.send(&response);
                    }
                }
            }
        },
    );

    socket.on(
        "artifact-delete",
        move |ack: AckSender,
              TryData(payload): TryData<ArtifactReadPayload>,
              State(ctx): State<AppContext>| async move {
            ctx.metrics().incr_socket_event("artifact-delete");
            let Ok(payload) = payload else {
                let _ = ack.send(&json!({"result": "error", "message": "Invalid parameters"}));
                return;
            };
            let deleted = ctx.db().write(|state| {
                let can_delete = state
                    .artifacts
                    .get(&payload.artifact_id)
                    .is_some_and(|artifact| artifact.account_id == auth.user_id);
                if !can_delete {
                    return None;
                }
                state.artifacts.remove(&payload.artifact_id)
            });
            if deleted.is_none() {
                let _ = ack.send(&json!({"result": "error", "message": "Artifact not found"}));
                return;
            }
            let _ =
                ctx.events()
                    .publish_delete_artifact(ctx.db(), &auth.user_id, &payload.artifact_id);
            let _ = ack.send(&json!({"result": "success"}));
        },
    );
}

fn socket_client_type_label(client_type: &SocketClientType) -> &'static str {
    match client_type {
        SocketClientType::UserScoped => "user-scoped",
        SocketClientType::SessionScoped => "session-scoped",
        SocketClientType::MachineScoped => "machine-scoped",
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use engineioxide::Packet as EioPacket;
    use serde::de::DeserializeOwned;
    use serde_json::{Value as JsonValue, json};
    use socketioxide::SocketIo;
    use socketioxide_core::{
        Value,
        packet::{Packet, PacketData},
        parser::{Parse, ParserState},
    };
    use socketioxide_parser_common::CommonParser;
    use tower::ServiceExt;

    use crate::{api::build_router, config::Config, context::AppContext};

    use super::register_namespace;

    fn test_config() -> Config {
        Config {
            host: "127.0.0.1".parse().unwrap(),
            port: 3005,
            master_secret: "secret".into(),
            ios_up_to_date: ">=1.4.1".into(),
            android_up_to_date: ">=1.4.1".into(),
            ios_store_url: "ios-store".into(),
            android_store_url: "android-store".into(),
        }
    }

    fn build_test_io(ctx: AppContext) -> SocketIo {
        let (_svc, io) = SocketIo::builder()
            .with_state(ctx)
            .req_path("/v1/updates")
            .ping_timeout(Duration::from_millis(45_000))
            .ping_interval(Duration::from_millis(15_000))
            .connect_timeout(Duration::from_millis(20_000))
            .transports([
                socketioxide::TransportType::Websocket,
                socketioxide::TransportType::Polling,
            ])
            .build_svc();
        register_namespace(&io);
        io
    }

    fn encode_event(event: &str, data: impl serde::Serialize, ack_id: Option<i64>) -> EioPacket {
        let mut packet = Packet::event("/", CommonParser.encode_value(&data, Some(event)).unwrap());
        if let Some(ack_id) = ack_id {
            packet.inner.set_ack_id(ack_id);
        }
        match CommonParser.encode(packet) {
            Value::Str(msg, _) => EioPacket::Message(msg),
            Value::Bytes(_) => unreachable!(),
        }
    }

    async fn recv_message(rx: &mut tokio::sync::mpsc::Receiver<EioPacket>) -> String {
        let packet = tokio::time::timeout(Duration::from_millis(50), rx.recv())
            .await
            .unwrap()
            .unwrap();
        match packet {
            EioPacket::Message(msg) => msg.to_string(),
            other => panic!("unexpected packet: {other:?}"),
        }
    }

    fn decode_message(message: String) -> Packet {
        CommonParser
            .decode_str(&ParserState::default(), message.into())
            .unwrap()
    }

    async fn recv_packet(rx: &mut tokio::sync::mpsc::Receiver<EioPacket>) -> Packet {
        decode_message(recv_message(rx).await)
    }

    async fn recv_packet_timeout(
        rx: &mut tokio::sync::mpsc::Receiver<EioPacket>,
        timeout: Duration,
    ) -> Option<Packet> {
        let packet = tokio::time::timeout(timeout, rx.recv()).await.ok()??;
        match packet {
            EioPacket::Message(message) => Some(decode_message(message.to_string())),
            other => panic!("unexpected packet: {other:?}"),
        }
    }

    async fn recv_until_ack(
        rx: &mut tokio::sync::mpsc::Receiver<EioPacket>,
        ack_id: i64,
    ) -> Packet {
        for _ in 0..4 {
            let packet = recv_packet(rx).await;
            if matches!(packet.inner, PacketData::EventAck(_, id) if id == ack_id) {
                return packet;
            }
        }
        panic!("timed out waiting for ack packet {ack_id}");
    }

    async fn recv_until_event(
        rx: &mut tokio::sync::mpsc::Receiver<EioPacket>,
        event: &str,
    ) -> Packet {
        for _ in 0..4 {
            let mut packet = recv_packet(rx).await;
            if let PacketData::Event(value, _) = &mut packet.inner
                && CommonParser.read_event(value).unwrap() == event
            {
                return packet;
            }
        }
        panic!("timed out waiting for event {event}");
    }

    fn encode_ack(data: impl serde::Serialize, ack_id: i64) -> EioPacket {
        let packet = Packet::ack("/", CommonParser.encode_value(&data, None).unwrap(), ack_id);
        match CommonParser.encode(packet) {
            Value::Str(msg, _) => EioPacket::Message(msg),
            Value::Bytes(_) => unreachable!(),
        }
    }

    fn decode_event_payload<T: DeserializeOwned>(packet: &mut Packet, with_event: bool) -> T {
        match &mut packet.inner {
            PacketData::Event(value, _) | PacketData::EventAck(value, _) => {
                CommonParser.decode_value(value, with_event).unwrap()
            }
            other => panic!("unexpected packet data: {other:?}"),
        }
    }

    #[tokio::test]
    async fn invalid_auth_emits_error_event_then_disconnects() {
        let io = build_test_io(AppContext::new(test_config()));
        let (_tx, mut rx) = io.new_dummy_sock("/", serde_json::json!({})).await;

        let _ = recv_message(&mut rx).await;

        let mut error_packet = decode_message(recv_message(&mut rx).await);
        match &mut error_packet.inner {
            PacketData::Event(value, _) => {
                assert_eq!(CommonParser.read_event(value).unwrap(), "error");
            }
            other => panic!("expected error event packet, got {other:?}"),
        }
        let payload: JsonValue = decode_event_payload(&mut error_packet, true);
        assert_eq!(payload["message"], "Missing authentication token");

        let disconnect_packet = decode_message(recv_message(&mut rx).await);
        assert!(matches!(disconnect_packet.inner, PacketData::Disconnect));
    }

    #[tokio::test]
    async fn session_scoped_connect_rejects_empty_session_id() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let io = build_test_io(ctx);
        let (_tx, mut rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "session-scoped",
                    "sessionId": "",
                }),
            )
            .await;

        let _ = recv_message(&mut rx).await;

        let mut error_packet = decode_message(recv_message(&mut rx).await);
        match &mut error_packet.inner {
            PacketData::Event(value, _) => {
                assert_eq!(CommonParser.read_event(value).unwrap(), "error");
            }
            other => panic!("expected error event packet, got {other:?}"),
        }
        let payload: JsonValue = decode_event_payload(&mut error_packet, true);
        assert_eq!(
            payload["message"],
            "Session ID required for session-scoped clients"
        );

        let disconnect_packet = decode_message(recv_message(&mut rx).await);
        assert!(matches!(disconnect_packet.inner, PacketData::Disconnect));
    }

    #[tokio::test]
    async fn machine_scoped_connect_rejects_empty_machine_id() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let io = build_test_io(ctx);
        let (_tx, mut rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "machine-scoped",
                    "machineId": "",
                }),
            )
            .await;

        let _ = recv_message(&mut rx).await;

        let mut error_packet = decode_message(recv_message(&mut rx).await);
        match &mut error_packet.inner {
            PacketData::Event(value, _) => {
                assert_eq!(CommonParser.read_event(value).unwrap(), "error");
            }
            other => panic!("expected error event packet, got {other:?}"),
        }
        let payload: JsonValue = decode_event_payload(&mut error_packet, true);
        assert_eq!(
            payload["message"],
            "Machine ID required for machine-scoped clients"
        );

        let disconnect_packet = decode_message(recv_message(&mut rx).await);
        assert!(matches!(disconnect_packet.inner, PacketData::Disconnect));
    }

    #[tokio::test]
    async fn ping_ack_returns_empty_object() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let io = build_test_io(ctx);
        let (tx, mut rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event("ping", (), Some(7))).await.unwrap();

        let mut ack_packet = decode_message(recv_message(&mut rx).await);
        match ack_packet.inner {
            PacketData::EventAck(_, ack_id) => assert_eq!(ack_id, 7),
            ref other => panic!("expected ack packet, got {other:?}"),
        }
        let payload: JsonValue = decode_event_payload(&mut ack_packet, false);
        assert_eq!(payload, json!({}));
    }

    #[tokio::test]
    async fn message_event_persists_and_broadcasts_without_echoing_sender() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let io = build_test_io(ctx.clone());
        let (sender_tx, mut sender_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token.clone() }))
            .await;
        let (_receiver_tx, mut receiver_rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "session-scoped",
                    "sessionId": session.id,
                }),
            )
            .await;

        let _ = recv_message(&mut sender_rx).await;
        let _ = recv_message(&mut receiver_rx).await;

        sender_tx
            .send(encode_event(
                "message",
                json!({
                    "sid": session.id,
                    "message": "ciphertext",
                    "localId": "local-1",
                }),
                None,
            ))
            .await
            .unwrap();

        let mut update_packet = recv_until_event(&mut receiver_rx, "update").await;
        let update_payload: JsonValue = decode_event_payload(&mut update_packet, true);
        assert_eq!(update_payload["body"]["t"], "new-message");
        assert_eq!(update_payload["body"]["sid"], session.id);
        assert_eq!(update_payload["body"]["message"]["localId"], "local-1");
        assert_eq!(
            update_payload["body"]["message"]["content"]["c"],
            "ciphertext"
        );

        assert!(
            recv_packet_timeout(&mut sender_rx, Duration::from_millis(30))
                .await
                .is_none()
        );

        let stored = ctx.db().list_session_messages_desc(&session.id, 10);
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].local_id.as_deref(), Some("local-1"));
        assert_eq!(stored[0].content.ciphertext, "ciphertext");
    }

    #[tokio::test]
    async fn session_alive_event_emits_ephemeral_and_flushes_presence() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let alive_at = crate::storage::db::now_ms().saturating_sub(20_000);
        ctx.db().write(|state| {
            let session = state.sessions.get_mut(&session.id).unwrap();
            session.active = false;
            session.active_at = alive_at.saturating_sub(40_000);
        });
        let io = build_test_io(ctx.clone());
        let (tx, mut rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event(
            "session-alive",
            json!({
                "sid": session.id,
                "time": alive_at,
                "thinking": true,
            }),
            None,
        ))
        .await
        .unwrap();

        let mut event_packet = recv_until_event(&mut rx, "ephemeral").await;
        let event_payload: JsonValue = decode_event_payload(&mut event_packet, true);
        assert_eq!(event_payload["type"], "activity");
        assert_eq!(event_payload["id"], session.id);
        assert_eq!(event_payload["active"], true);
        assert_eq!(event_payload["activeAt"], alive_at);
        assert_eq!(event_payload["thinking"], true);

        ctx.presence().flush_pending().await;

        let updated = ctx
            .db()
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert!(updated.active);
        assert_eq!(updated.active_at, alive_at);
    }

    #[tokio::test]
    async fn session_end_event_marks_session_inactive_and_emits_ephemeral() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let end_at = crate::storage::db::now_ms().saturating_sub(5_000);
        ctx.db().write(|state| {
            let session = state.sessions.get_mut(&session.id).unwrap();
            session.active = true;
            session.active_at = end_at.saturating_sub(2_000);
        });
        let io = build_test_io(ctx.clone());
        let (tx, mut rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event(
            "session-end",
            json!({
                "sid": session.id,
                "time": end_at,
            }),
            None,
        ))
        .await
        .unwrap();

        let mut event_packet = recv_until_event(&mut rx, "ephemeral").await;
        let event_payload: JsonValue = decode_event_payload(&mut event_packet, true);
        assert_eq!(event_payload["type"], "activity");
        assert_eq!(event_payload["id"], session.id);
        assert_eq!(event_payload["active"], false);
        assert_eq!(event_payload["activeAt"], end_at);

        let updated = ctx
            .db()
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert!(!updated.active);
        assert_eq!(updated.active_at, end_at);
    }

    #[tokio::test]
    async fn session_update_metadata_broadcasts_and_rejects_stale_versions() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let (other_session, _) =
            ctx.db()
                .create_or_load_session(&account.id, "tag-2", "other", None);
        let io = build_test_io(ctx);
        let (sender_tx, mut sender_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;
        let (_matching_tx, mut matching_rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "session-scoped",
                    "sessionId": session.id,
                }),
            )
            .await;
        let (_other_tx, mut other_rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "session-scoped",
                    "sessionId": other_session.id,
                }),
            )
            .await;

        let _ = recv_message(&mut sender_rx).await;
        let _ = recv_message(&mut matching_rx).await;
        let _ = recv_message(&mut other_rx).await;

        sender_tx
            .send(encode_event(
                "update-metadata",
                json!({
                    "sid": session.id,
                    "metadata": "meta-next",
                    "expectedVersion": 0,
                }),
                Some(7),
            ))
            .await
            .unwrap();

        let mut ack_packet = recv_until_ack(&mut sender_rx, 7).await;
        let ack_payload: JsonValue = decode_event_payload(&mut ack_packet, false);
        assert_eq!(ack_payload["result"], "success");
        assert_eq!(ack_payload["version"], 1);
        assert_eq!(ack_payload["metadata"], "meta-next");

        let mut update_packet = recv_until_event(&mut matching_rx, "update").await;
        let update_payload: JsonValue = decode_event_payload(&mut update_packet, true);
        assert_eq!(update_payload["body"]["t"], "update-session");
        assert_eq!(update_payload["body"]["id"], session.id);
        assert_eq!(update_payload["body"]["metadata"]["version"], 1);
        assert_eq!(update_payload["body"]["metadata"]["value"], "meta-next");

        assert!(
            recv_packet_timeout(&mut other_rx, Duration::from_millis(30))
                .await
                .is_none()
        );

        sender_tx
            .send(encode_event(
                "update-metadata",
                json!({
                    "sid": session.id,
                    "metadata": "stale",
                    "expectedVersion": 0,
                }),
                Some(8),
            ))
            .await
            .unwrap();

        let mut mismatch_packet = recv_until_ack(&mut sender_rx, 8).await;
        let mismatch_payload: JsonValue = decode_event_payload(&mut mismatch_packet, false);
        assert_eq!(mismatch_payload["result"], "version-mismatch");
        assert_eq!(mismatch_payload["version"], 1);
        assert_eq!(mismatch_payload["metadata"], "meta-next");
    }

    #[tokio::test]
    async fn session_update_state_returns_success_then_version_mismatch() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let io = build_test_io(ctx);
        let (tx, mut rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "session-scoped",
                    "sessionId": session.id,
                }),
            )
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event(
            "update-state",
            json!({
                "sid": session.id,
                "agentState": "ready",
                "expectedVersion": 0,
            }),
            Some(9),
        ))
        .await
        .unwrap();

        let mut success_packet = recv_until_ack(&mut rx, 9).await;
        let success_payload: JsonValue = decode_event_payload(&mut success_packet, false);
        assert_eq!(success_payload["result"], "success");
        assert_eq!(success_payload["version"], 1);
        assert_eq!(success_payload["agentState"], "ready");

        tx.send(encode_event(
            "update-state",
            json!({
                "sid": session.id,
                "agentState": "stale",
                "expectedVersion": 0,
            }),
            Some(10),
        ))
        .await
        .unwrap();

        let mut mismatch_packet = recv_until_ack(&mut rx, 10).await;
        let mismatch_payload: JsonValue = decode_event_payload(&mut mismatch_packet, false);
        assert_eq!(mismatch_payload["result"], "version-mismatch");
        assert_eq!(mismatch_payload["version"], 1);
        assert_eq!(mismatch_payload["agentState"], "ready");
    }

    #[tokio::test]
    async fn session_update_state_rejects_missing_agent_state_but_accepts_null() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let io = build_test_io(ctx.clone());
        let (tx, mut rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "session-scoped",
                    "sessionId": session.id,
                }),
            )
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event(
            "update-state",
            json!({
                "sid": session.id,
                "expectedVersion": 0,
            }),
            Some(17),
        ))
        .await
        .unwrap();

        let mut missing_packet = recv_until_ack(&mut rx, 17).await;
        let missing_payload: JsonValue = decode_event_payload(&mut missing_packet, false);
        assert_eq!(missing_payload["result"], "error");
        let unchanged = ctx
            .db()
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert_eq!(unchanged.agent_state_version, 0);
        assert!(unchanged.agent_state.is_none());

        tx.send(encode_event(
            "update-state",
            json!({
                "sid": session.id,
                "agentState": null,
                "expectedVersion": 0,
            }),
            Some(18),
        ))
        .await
        .unwrap();

        let mut null_packet = recv_until_ack(&mut rx, 18).await;
        let null_payload: JsonValue = decode_event_payload(&mut null_packet, false);
        assert_eq!(null_payload["result"], "success");
        assert_eq!(null_payload["version"], 1);
        assert!(null_payload["agentState"].is_null());

        let updated = ctx
            .db()
            .get_session_for_account(&account.id, &session.id)
            .expect("session should exist");
        assert_eq!(updated.agent_state_version, 1);
        assert!(updated.agent_state.is_none());
    }

    #[tokio::test]
    async fn machine_update_state_broadcasts_and_rejects_stale_versions() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let machine = ctx
            .db()
            .create_or_load_machine(&account.id, "machine-1", "meta", None, None)
            .0;
        let other_machine = ctx
            .db()
            .create_or_load_machine(&account.id, "machine-2", "meta", None, None)
            .0;
        let io = build_test_io(ctx);
        let (sender_tx, mut sender_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;
        let (_matching_tx, mut matching_rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "machine-scoped",
                    "machineId": machine.id,
                }),
            )
            .await;
        let (_other_tx, mut other_rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "machine-scoped",
                    "machineId": other_machine.id,
                }),
            )
            .await;

        let _ = recv_message(&mut sender_rx).await;
        let _ = recv_message(&mut matching_rx).await;
        let _ = recv_message(&mut other_rx).await;
        let _ = recv_until_event(&mut sender_rx, "ephemeral").await;
        let _ = recv_until_event(&mut sender_rx, "ephemeral").await;

        sender_tx
            .send(encode_event(
                "machine-update-state",
                json!({
                    "machineId": machine.id,
                    "daemonState": "online",
                    "expectedVersion": 0,
                }),
                Some(11),
            ))
            .await
            .unwrap();

        let mut ack_packet = recv_until_ack(&mut sender_rx, 11).await;
        let ack_payload: JsonValue = decode_event_payload(&mut ack_packet, false);
        assert_eq!(ack_payload["result"], "success");
        assert_eq!(ack_payload["version"], 1);
        assert_eq!(ack_payload["daemonState"], "online");

        let mut update_packet = recv_until_event(&mut matching_rx, "update").await;
        let update_payload: JsonValue = decode_event_payload(&mut update_packet, true);
        assert_eq!(update_payload["body"]["t"], "update-machine");
        assert_eq!(update_payload["body"]["machineId"], machine.id);
        assert_eq!(update_payload["body"]["daemonState"]["version"], 1);
        assert_eq!(update_payload["body"]["daemonState"]["value"], "online");

        assert!(
            recv_packet_timeout(&mut other_rx, Duration::from_millis(30))
                .await
                .is_none()
        );

        sender_tx
            .send(encode_event(
                "machine-update-state",
                json!({
                    "machineId": machine.id,
                    "daemonState": "stale",
                    "expectedVersion": 0,
                }),
                Some(12),
            ))
            .await
            .unwrap();

        let mut mismatch_packet = recv_until_ack(&mut sender_rx, 12).await;
        let mismatch_payload: JsonValue = decode_event_payload(&mut mismatch_packet, false);
        assert_eq!(mismatch_payload["result"], "version-mismatch");
        assert_eq!(mismatch_payload["version"], 1);
        assert_eq!(mismatch_payload["daemonState"], "online");
    }

    #[tokio::test]
    async fn machine_update_metadata_broadcasts_and_rejects_stale_versions() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let machine = ctx
            .db()
            .create_or_load_machine(&account.id, "machine-1", "meta", None, None)
            .0;
        let other_machine = ctx
            .db()
            .create_or_load_machine(&account.id, "machine-2", "meta", None, None)
            .0;
        let io = build_test_io(ctx);
        let (sender_tx, mut sender_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;
        let (_matching_tx, mut matching_rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "machine-scoped",
                    "machineId": machine.id,
                }),
            )
            .await;
        let (_other_tx, mut other_rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "machine-scoped",
                    "machineId": other_machine.id,
                }),
            )
            .await;

        let _ = recv_message(&mut sender_rx).await;
        let _ = recv_message(&mut matching_rx).await;
        let _ = recv_message(&mut other_rx).await;
        let _ = recv_until_event(&mut sender_rx, "ephemeral").await;
        let _ = recv_until_event(&mut sender_rx, "ephemeral").await;

        sender_tx
            .send(encode_event(
                "machine-update-metadata",
                json!({
                    "machineId": machine.id,
                    "metadata": "meta-next",
                    "expectedVersion": 1,
                }),
                Some(15),
            ))
            .await
            .unwrap();

        let mut ack_packet = recv_until_ack(&mut sender_rx, 15).await;
        let ack_payload: JsonValue = decode_event_payload(&mut ack_packet, false);
        assert_eq!(ack_payload["result"], "success");
        assert_eq!(ack_payload["version"], 2);
        assert_eq!(ack_payload["metadata"], "meta-next");

        let mut update_packet = recv_until_event(&mut matching_rx, "update").await;
        let update_payload: JsonValue = decode_event_payload(&mut update_packet, true);
        assert_eq!(update_payload["body"]["t"], "update-machine");
        assert_eq!(update_payload["body"]["machineId"], machine.id);
        assert_eq!(update_payload["body"]["metadata"]["version"], 2);
        assert_eq!(update_payload["body"]["metadata"]["value"], "meta-next");

        assert!(
            recv_packet_timeout(&mut other_rx, Duration::from_millis(30))
                .await
                .is_none()
        );

        sender_tx
            .send(encode_event(
                "machine-update-metadata",
                json!({
                    "machineId": machine.id,
                    "metadata": "stale",
                    "expectedVersion": 1,
                }),
                Some(16),
            ))
            .await
            .unwrap();

        let mut mismatch_packet = recv_until_ack(&mut sender_rx, 16).await;
        let mismatch_payload: JsonValue = decode_event_payload(&mut mismatch_packet, false);
        assert_eq!(mismatch_payload["result"], "version-mismatch");
        assert_eq!(mismatch_payload["version"], 2);
        assert_eq!(mismatch_payload["metadata"], "meta-next");
    }

    #[tokio::test]
    async fn machine_scoped_connect_persists_presence_before_emitting_ephemeral() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let machine = ctx
            .db()
            .create_or_load_machine(&account.id, "machine-1", "meta", None, None)
            .0;
        let io = build_test_io(ctx.clone());
        let (_user_tx, mut user_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token.clone() }))
            .await;
        let (_machine_tx, mut machine_rx) = io
            .new_dummy_sock(
                "/",
                serde_json::json!({
                    "token": token,
                    "clientType": "machine-scoped",
                    "machineId": machine.id,
                }),
            )
            .await;

        let _ = recv_message(&mut user_rx).await;
        let _ = recv_message(&mut machine_rx).await;

        let mut event_packet = recv_until_event(&mut user_rx, "ephemeral").await;
        let event_payload: JsonValue = decode_event_payload(&mut event_packet, true);
        assert_eq!(event_payload["type"], "machine-activity");
        assert_eq!(event_payload["id"], machine.id);
        assert_eq!(event_payload["active"], true);

        let updated = ctx
            .db()
            .get_machine_for_account(&account.id, &machine.id)
            .expect("machine should exist");
        assert!(updated.active);
        assert_eq!(event_payload["activeAt"].as_u64(), Some(updated.active_at));
    }

    #[tokio::test]
    async fn machine_alive_event_emits_ephemeral_and_flushes_presence() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let machine = ctx
            .db()
            .create_or_load_machine(&account.id, "machine-1", "meta", None, None)
            .0;
        let alive_at = crate::storage::db::now_ms().saturating_sub(20_000);
        ctx.db().write(|state| {
            let machine = state
                .machines
                .get_mut(&(account.id.clone(), machine.id.clone()))
                .expect("machine should exist");
            machine.active = false;
            machine.active_at = alive_at.saturating_sub(40_000);
        });
        let io = build_test_io(ctx.clone());
        let (tx, mut rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event(
            "machine-alive",
            json!({
                "machineId": machine.id,
                "time": alive_at,
            }),
            None,
        ))
        .await
        .unwrap();

        let mut event_packet = recv_until_event(&mut rx, "ephemeral").await;
        let event_payload: JsonValue = decode_event_payload(&mut event_packet, true);
        assert_eq!(event_payload["type"], "machine-activity");
        assert_eq!(event_payload["id"], machine.id);
        assert_eq!(event_payload["active"], true);
        assert_eq!(event_payload["activeAt"], alive_at);

        ctx.presence().flush_pending().await;

        let updated = ctx
            .db()
            .get_machine_for_account(&account.id, &machine.id)
            .expect("machine should exist");
        assert!(updated.active);
        assert_eq!(updated.active_at, alive_at);
    }

    #[tokio::test]
    async fn rpc_register_call_and_unregister_round_trip() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let io = build_test_io(ctx);
        let (caller_tx, mut caller_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;
        let (callee_tx, mut callee_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;

        let _ = recv_message(&mut caller_rx).await;
        let _ = recv_message(&mut callee_rx).await;

        callee_tx
            .send(encode_event(
                "rpc-register",
                json!({ "method": "machine:call" }),
                None,
            ))
            .await
            .unwrap();

        let mut registered_packet = recv_until_event(&mut callee_rx, "rpc-registered").await;
        let registered_payload: JsonValue = decode_event_payload(&mut registered_packet, true);
        assert_eq!(registered_payload["method"], "machine:call");

        caller_tx
            .send(encode_event(
                "rpc-call",
                json!({
                    "method": "machine:call",
                    "params": { "value": 7 },
                }),
                Some(13),
            ))
            .await
            .unwrap();

        let mut rpc_request = recv_until_event(&mut callee_rx, "rpc-request").await;
        let ack_id = match rpc_request.inner {
            PacketData::Event(_, ack_id) => ack_id.expect("rpc request should carry ack id"),
            ref other => panic!("expected rpc-request event, got {other:?}"),
        };
        let rpc_request_payload: JsonValue = decode_event_payload(&mut rpc_request, true);
        assert_eq!(rpc_request_payload["method"], "machine:call");
        assert_eq!(rpc_request_payload["params"]["value"], 7);
        callee_tx
            .send(encode_ack(json!({ "echo": 7 }), ack_id))
            .await
            .unwrap();

        let mut call_ack = recv_until_ack(&mut caller_rx, 13).await;
        let call_payload: JsonValue = decode_event_payload(&mut call_ack, false);
        assert_eq!(call_payload["ok"], true);
        assert_eq!(call_payload["result"]["echo"], 7);

        callee_tx
            .send(encode_event(
                "rpc-unregister",
                json!({ "method": "machine:call" }),
                None,
            ))
            .await
            .unwrap();

        let mut unregistered_packet = recv_until_event(&mut callee_rx, "rpc-unregistered").await;
        let unregistered_payload: JsonValue = decode_event_payload(&mut unregistered_packet, true);
        assert_eq!(unregistered_payload["method"], "machine:call");

        caller_tx
            .send(encode_event(
                "rpc-call",
                json!({
                    "method": "machine:call",
                    "params": { "value": 8 },
                }),
                Some(14),
            ))
            .await
            .unwrap();

        let mut unavailable_ack = recv_until_ack(&mut caller_rx, 14).await;
        let unavailable_payload: JsonValue = decode_event_payload(&mut unavailable_ack, false);
        assert_eq!(unavailable_payload["ok"], false);
        assert_eq!(unavailable_payload["error"], "RPC method not available");
    }

    #[tokio::test]
    async fn rpc_rejects_empty_method_names() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let io = build_test_io(ctx);
        let (tx, mut rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event("rpc-register", json!({ "method": "" }), None))
            .await
            .unwrap();

        let mut register_error = recv_until_event(&mut rx, "rpc-error").await;
        let register_payload: JsonValue = decode_event_payload(&mut register_error, true);
        assert_eq!(register_payload["type"], "register");
        assert_eq!(register_payload["error"], "Invalid method name");

        tx.send(encode_event(
            "rpc-unregister",
            json!({ "method": "" }),
            None,
        ))
        .await
        .unwrap();

        let mut unregister_error = recv_until_event(&mut rx, "rpc-error").await;
        let unregister_payload: JsonValue = decode_event_payload(&mut unregister_error, true);
        assert_eq!(unregister_payload["type"], "unregister");
        assert_eq!(unregister_payload["error"], "Invalid method name");

        tx.send(encode_event(
            "rpc-call",
            json!({
                "method": "",
                "params": {},
            }),
            Some(19),
        ))
        .await
        .unwrap();

        let mut call_ack = recv_until_ack(&mut rx, 19).await;
        let call_payload: JsonValue = decode_event_payload(&mut call_ack, false);
        assert_eq!(call_payload["ok"], false);
        assert_eq!(
            call_payload["error"],
            "Invalid parameters: method is required"
        );
    }

    #[tokio::test]
    async fn artifact_delete_rejects_cross_account_access() {
        let ctx = AppContext::new(test_config());
        let owner = ctx.db().upsert_account_by_public_key("artifact-owner");
        let caller = ctx.db().upsert_account_by_public_key("artifact-caller");
        let token = ctx.auth().create_token(&caller.id, None);
        ctx.db().write(|state| {
            state.artifacts.insert(
                "artifact-1".into(),
                crate::storage::db::ArtifactRecord {
                    id: "artifact-1".into(),
                    account_id: owner.id.clone(),
                    header: "header".into(),
                    header_version: 1,
                    body: "body".into(),
                    body_version: 1,
                    data_encryption_key: "dek".into(),
                    seq: 0,
                    created_at: crate::storage::db::now_ms(),
                    updated_at: crate::storage::db::now_ms(),
                },
            );
        });
        let io = build_test_io(ctx.clone());
        let (tx, mut rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event(
            "artifact-delete",
            json!({ "artifactId": "artifact-1" }),
            Some(21),
        ))
        .await
        .unwrap();

        let mut ack_packet = recv_until_ack(&mut rx, 21).await;
        let payload: JsonValue = decode_event_payload(&mut ack_packet, false);
        assert_eq!(payload["result"], "error");
        assert_eq!(payload["message"], "Artifact not found");
        assert!(
            ctx.db()
                .read(|state| state.artifacts.contains_key("artifact-1"))
        );
    }

    #[tokio::test]
    async fn artifact_create_is_idempotent_for_same_account_and_conflicts_cross_account() {
        let ctx = AppContext::new(test_config());
        let owner = ctx.db().upsert_account_by_public_key("artifact-owner");
        let other = ctx.db().upsert_account_by_public_key("artifact-other");
        let owner_token = ctx.auth().create_token(&owner.id, None);
        let other_token = ctx.auth().create_token(&other.id, None);
        let artifact_id = uuid::Uuid::now_v7().to_string();
        ctx.db().write(|state| {
            state.artifacts.insert(
                artifact_id.clone(),
                crate::storage::db::ArtifactRecord {
                    id: artifact_id.clone(),
                    account_id: owner.id.clone(),
                    header: "header".into(),
                    header_version: 1,
                    body: "body".into(),
                    body_version: 1,
                    data_encryption_key: "dek".into(),
                    seq: 0,
                    created_at: crate::storage::db::now_ms(),
                    updated_at: crate::storage::db::now_ms(),
                },
            );
        });
        let io = build_test_io(ctx.clone());
        let (owner_tx, mut owner_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": owner_token }))
            .await;
        let (other_tx, mut other_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": other_token }))
            .await;

        let _ = recv_message(&mut owner_rx).await;
        let _ = recv_message(&mut other_rx).await;

        owner_tx
            .send(encode_event(
                "artifact-create",
                json!({
                    "id": artifact_id,
                    "header": "ignored",
                    "body": "ignored",
                    "dataEncryptionKey": "ignored"
                }),
                Some(25),
            ))
            .await
            .unwrap();

        let mut same_ack = recv_until_ack(&mut owner_rx, 25).await;
        let same_payload: JsonValue = decode_event_payload(&mut same_ack, false);
        assert_eq!(same_payload["result"], "success");
        assert_eq!(same_payload["artifact"]["id"], artifact_id);
        assert_eq!(same_payload["artifact"]["header"], "header");

        other_tx
            .send(encode_event(
                "artifact-create",
                json!({
                    "id": artifact_id,
                    "header": "ignored",
                    "body": "ignored",
                    "dataEncryptionKey": "ignored"
                }),
                Some(26),
            ))
            .await
            .unwrap();

        let mut other_ack = recv_until_ack(&mut other_rx, 26).await;
        let other_payload: JsonValue = decode_event_payload(&mut other_ack, false);
        assert_eq!(other_payload["result"], "error");
        assert_eq!(
            other_payload["message"],
            "Artifact with this ID already exists for another account"
        );
    }

    #[tokio::test]
    async fn usage_report_validates_totals_and_upserts_latest_record() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("usage-user");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "usage", "meta", None);
        let io = build_test_io(ctx.clone());
        let (tx, mut rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event(
            "usage-report",
            json!({
                "key": "claude",
                "sessionId": session.id,
                "tokens": { "input": 10 },
                "cost": { "total": 1.5 }
            }),
            Some(22),
        ))
        .await
        .unwrap();

        let mut invalid_ack = recv_until_ack(&mut rx, 22).await;
        let invalid_payload: JsonValue = decode_event_payload(&mut invalid_ack, false);
        assert_eq!(invalid_payload["success"], false);
        assert_eq!(
            invalid_payload["error"],
            "Invalid tokens object - must include total"
        );

        tx.send(encode_event(
            "usage-report",
            json!({
                "key": "claude",
                "sessionId": session.id,
                "tokens": { "input": 10, "total": 10 },
                "cost": { "total": 1.5 }
            }),
            Some(23),
        ))
        .await
        .unwrap();

        let mut first_ack = recv_until_ack(&mut rx, 23).await;
        let first_payload: JsonValue = decode_event_payload(&mut first_ack, false);
        assert_eq!(first_payload["success"], true);
        let first_created_at = first_payload["createdAt"].as_u64().unwrap();

        tx.send(encode_event(
            "usage-report",
            json!({
                "key": "claude",
                "sessionId": session.id,
                "tokens": { "input": 15, "total": 15 },
                "cost": { "total": 2.0 }
            }),
            Some(24),
        ))
        .await
        .unwrap();

        let mut second_ack = recv_until_ack(&mut rx, 24).await;
        let second_payload: JsonValue = decode_event_payload(&mut second_ack, false);
        assert_eq!(second_payload["success"], true);
        assert_eq!(ctx.db().read(|state| state.usage_reports.len()), 1);
        let stored = ctx
            .db()
            .read(|state| state.usage_reports.values().next().cloned())
            .unwrap();
        assert_eq!(stored.tokens["total"], 15);
        assert_eq!(stored.cost["total"], 2.0);
        assert_eq!(stored.created_at, first_created_at);
        assert_eq!(
            second_payload["createdAt"].as_u64().unwrap(),
            first_created_at
        );
    }

    #[tokio::test]
    async fn usage_report_ephemeral_uses_fresh_timestamp_when_upserting() {
        let ctx = AppContext::new(test_config());
        let account = ctx
            .db()
            .upsert_account_by_public_key("usage-user-ephemeral");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "usage", "meta", None);
        ctx.db().write(|state| {
            state.usage_reports.insert(
                "usage-existing".into(),
                crate::storage::db::UsageReportRecord {
                    id: "usage-existing".into(),
                    account_id: account.id.clone(),
                    session_id: Some(session.id.clone()),
                    key: "claude".into(),
                    tokens: [("total".into(), 1_u64)].into_iter().collect(),
                    cost: [("total".into(), 0.1_f64)].into_iter().collect(),
                    created_at: 1,
                    updated_at: 1,
                },
            );
        });
        let io = build_test_io(ctx.clone());
        let (tx, mut rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token }))
            .await;

        let _ = recv_message(&mut rx).await;

        tx.send(encode_event(
            "usage-report",
            json!({
                "key": "claude",
                "sessionId": session.id,
                "tokens": { "input": 15, "total": 15 },
                "cost": { "total": 2.0 }
            }),
            Some(32),
        ))
        .await
        .unwrap();

        let mut event_packet = recv_until_event(&mut rx, "ephemeral").await;
        let event_payload: JsonValue = decode_event_payload(&mut event_packet, true);
        assert_eq!(event_payload["type"], "usage");
        assert_eq!(event_payload["id"], session.id);
        assert_eq!(event_payload["key"], "claude");
        assert!(event_payload["timestamp"].as_u64().unwrap() > 1);

        let mut ack_packet = recv_until_ack(&mut rx, 32).await;
        let ack_payload: JsonValue = decode_event_payload(&mut ack_packet, false);
        assert_eq!(ack_payload["success"], true);
        assert_eq!(ack_payload["createdAt"], 1);
    }

    #[tokio::test]
    async fn github_disconnect_emits_update_without_avatar_change() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("github-user");
        let token = ctx.auth().create_token(&account.id, None);
        ctx.db().write(|state| {
            let account = state.accounts.get_mut(&account.id).unwrap();
            account.github_profile = Some(json!({ "id": 7, "login": "octocat" }));
            account.username = Some("octocat".into());
            account.avatar = Some(json!({
                "path": "public/users/acct/avatars/github.png",
                "width": 1,
                "height": 1,
                "thumbhash": "abc"
            }));
            state.github_tokens.insert(
                account.id.clone(),
                crate::storage::db::GithubTokenRecord {
                    id: "ght_github".into(),
                    account_id: account.id.clone(),
                    github_user_id: "7".into(),
                    token: "token".into(),
                    created_at: crate::storage::db::now_ms(),
                    updated_at: crate::storage::db::now_ms(),
                },
            );
        });
        let io = build_test_io(ctx.clone());
        let (_socket_tx, mut socket_rx) = io
            .new_dummy_sock("/", serde_json::json!({ "token": token.clone() }))
            .await;
        let _ = recv_message(&mut socket_rx).await;

        let response = build_router(ctx.clone())
            .oneshot(
                Request::delete("/v1/connect/github")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: JsonValue = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let mut update_packet = recv_until_event(&mut socket_rx, "update").await;
        let update_payload: JsonValue = decode_event_payload(&mut update_packet, true);
        assert_eq!(update_payload["body"]["t"], "update-account");
        assert_eq!(update_payload["body"]["github"], JsonValue::Null);
        assert_eq!(update_payload["body"]["username"], JsonValue::Null);
        assert!(
            !update_payload["body"]
                .as_object()
                .unwrap()
                .contains_key("avatar")
        );

        let stored = ctx.db().get_account(&account.id).unwrap();
        assert!(stored.github_profile.is_none());
        assert!(stored.username.is_none());
        assert!(stored.avatar.is_some());
        assert!(
            ctx.db()
                .read(|state| !state.github_tokens.contains_key(&account.id))
        );
    }
}
