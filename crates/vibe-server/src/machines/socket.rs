use std::{collections::HashMap, sync::Arc, time::Duration};

use parking_lot::RwLock;
use serde_json::json;
use socketioxide::extract::{AckSender, SocketRef, State, TryData};

use crate::{
    api::types::{
        MachineAlivePayload, MachineUpdateMetadataPayload, MachineUpdateStatePayload,
        RpcCallPayload, RpcRegisterPayload,
    },
    auth::SocketConnectionAuth,
    context::AppContext,
    storage::db::CompareAndSwap,
};

use super::service::MachinesService;

fn has_rpc_method(method: &str) -> bool {
    !method.is_empty()
}

#[derive(Clone, Default)]
pub struct RpcRegistry {
    inner: Arc<RwLock<HashMap<String, HashMap<String, SocketRef>>>>,
}

impl RpcRegistry {
    pub fn register(&self, user_id: &str, method: &str, socket: SocketRef) {
        self.inner
            .write()
            .entry(user_id.to_string())
            .or_default()
            .insert(method.to_string(), socket);
    }

    pub fn unregister(&self, user_id: &str, method: &str, socket_id: &str) {
        let mut guard = self.inner.write();
        if let Some(methods) = guard.get_mut(user_id) {
            if methods
                .get(method)
                .is_some_and(|socket| socket.id.to_string() == socket_id)
            {
                methods.remove(method);
            }
            if methods.is_empty() {
                guard.remove(user_id);
            }
        }
    }

    pub fn get(&self, user_id: &str, method: &str) -> Option<SocketRef> {
        self.inner
            .read()
            .get(user_id)
            .and_then(|methods| methods.get(method).cloned())
    }

    pub fn cleanup_socket(&self, user_id: &str, socket_id: &str) {
        let mut guard = self.inner.write();
        if let Some(methods) = guard.get_mut(user_id) {
            methods.retain(|_, socket| socket.id.to_string() != socket_id);
            if methods.is_empty() {
                guard.remove(user_id);
            }
        }
    }
}

pub fn register_handlers(socket: SocketRef, ctx: AppContext, auth: SocketConnectionAuth) {
    let machine_ctx = ctx.clone();
    let machine_auth = auth.clone();
    socket.on(
        "machine-alive",
        move |TryData(payload): TryData<MachineAlivePayload>| {
            let ctx = machine_ctx.clone();
            let auth = machine_auth.clone();
            async move {
                if let Ok(payload) = payload {
                    ctx.metrics().incr_socket_event("machine-alive");
                    ctx.presence()
                        .machine_alive(&auth.user_id, &payload.machine_id, payload.time)
                        .await;
                }
            }
        },
    );

    let metadata_ctx = ctx.clone();
    let metadata_auth = auth.clone();
    socket.on(
        "machine-update-metadata",
        move |ack: AckSender, TryData(payload): TryData<MachineUpdateMetadataPayload>| {
            let ctx = metadata_ctx.clone();
            let auth = metadata_auth.clone();
            async move {
                ctx.metrics().incr_socket_event("machine-update-metadata");
                let Ok(payload) = payload else {
                    let response = json!({"result": "error", "message": "Invalid parameters"});
                    let _ = ack.send(&response);
                    return;
                };
                let service = MachinesService::new(ctx);
                match service.update_metadata(
                    &auth.user_id,
                    &payload.machine_id,
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
                    Ok(CompareAndSwap::NotFound) => {
                        let response = json!({"result": "error", "message": "Machine not found"});
                        let _ = ack.send(&response);
                    }
                    Err(_) => {
                        let response = json!({"result": "error", "message": "Internal error"});
                        let _ = ack.send(&response);
                    }
                }
            }
        },
    );

    let state_ctx = ctx.clone();
    let state_auth = auth.clone();
    socket.on(
        "machine-update-state",
        move |ack: AckSender, TryData(payload): TryData<MachineUpdateStatePayload>| {
            let ctx = state_ctx.clone();
            let auth = state_auth.clone();
            async move {
                ctx.metrics().incr_socket_event("machine-update-state");
                let Ok(payload) = payload else {
                    let response = json!({"result": "error", "message": "Invalid parameters"});
                    let _ = ack.send(&response);
                    return;
                };
                let service = MachinesService::new(ctx);
                match service.update_daemon_state(
                    &auth.user_id,
                    &payload.machine_id,
                    payload.expected_version,
                    payload.daemon_state.clone(),
                ) {
                    Ok(CompareAndSwap::Success(value)) => {
                        let response = json!({
                            "result": "success",
                            "version": payload.expected_version + 1,
                            "daemonState": value,
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
                            "daemonState": current_value,
                        });
                        let _ = ack.send(&response);
                    }
                    Ok(CompareAndSwap::NotFound) => {
                        let response = json!({"result": "error", "message": "Machine not found"});
                        let _ = ack.send(&response);
                    }
                    Err(_) => {
                        let response = json!({"result": "error", "message": "Internal error"});
                        let _ = ack.send(&response);
                    }
                }
            }
        },
    );

    let rpc_register_auth = auth.clone();
    socket.on(
        "rpc-register",
        move |TryData(payload): TryData<RpcRegisterPayload>,
              State(ctx): State<AppContext>,
              socket: SocketRef| async move {
            ctx.metrics().incr_socket_event("rpc-register");
            let Ok(payload) = payload else {
                let response = json!({"type": "register", "error": "Invalid method name"});
                let _ = socket.emit("rpc-error", &response);
                return;
            };
            if !has_rpc_method(&payload.method) {
                let response = json!({"type": "register", "error": "Invalid method name"});
                let _ = socket.emit("rpc-error", &response);
                return;
            }
            ctx.rpc()
                .register(&rpc_register_auth.user_id, &payload.method, socket.clone());
            let response = json!({ "method": payload.method });
            let _ = socket.emit("rpc-registered", &response);
        },
    );

    let rpc_unregister_auth = auth.clone();
    socket.on(
        "rpc-unregister",
        move |TryData(payload): TryData<RpcRegisterPayload>,
              State(ctx): State<AppContext>,
              socket: SocketRef| async move {
            ctx.metrics().incr_socket_event("rpc-unregister");
            let Ok(payload) = payload else {
                let response = json!({"type": "unregister", "error": "Invalid method name"});
                let _ = socket.emit("rpc-error", &response);
                return;
            };
            if !has_rpc_method(&payload.method) {
                let response = json!({"type": "unregister", "error": "Invalid method name"});
                let _ = socket.emit("rpc-error", &response);
                return;
            }
            ctx.rpc().unregister(
                &rpc_unregister_auth.user_id,
                &payload.method,
                &socket.id.to_string(),
            );
            let response = json!({ "method": payload.method });
            let _ = socket.emit("rpc-unregistered", &response);
        },
    );

    let rpc_call_auth = auth.clone();
    socket.on(
        "rpc-call",
        move |ack: AckSender,
              TryData(payload): TryData<RpcCallPayload>,
              State(ctx): State<AppContext>,
              socket: SocketRef| async move {
            ctx.metrics().incr_socket_event("rpc-call");
            let Ok(payload) = payload else {
                let response =
                    json!({"ok": false, "error": "Invalid parameters: method is required"});
                let _ = ack.send(&response);
                return;
            };
            if !has_rpc_method(&payload.method) {
                let response =
                    json!({"ok": false, "error": "Invalid parameters: method is required"});
                let _ = ack.send(&response);
                return;
            }
            let Some(target) = ctx.rpc().get(&rpc_call_auth.user_id, &payload.method) else {
                let response = json!({"ok": false, "error": "RPC method not available"});
                let _ = ack.send(&response);
                return;
            };
            if target.id == socket.id {
                let response = json!({"ok": false, "error": "Cannot call RPC on the same socket"});
                let _ = ack.send(&response);
                return;
            }

            let payload_json = json!({
                "method": payload.method,
                "params": payload.params,
            });
            let result = match target
                .timeout(Duration::from_secs(30))
                .emit_with_ack::<_, serde_json::Value>("rpc-request", &payload_json)
            {
                Ok(future) => future.await,
                Err(_) => {
                    let response = json!({"ok": false, "error": "RPC method not available"});
                    let _ = ack.send(&response);
                    return;
                }
            };

            match result {
                Ok(result) => {
                    let response = json!({"ok": true, "result": result});
                    let _ = ack.send(&response);
                }
                Err(error) => {
                    let response = json!({"ok": false, "error": error.to_string()});
                    let _ = ack.send(&response);
                }
            }
        },
    );
}
