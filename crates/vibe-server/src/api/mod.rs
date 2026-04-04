pub mod account;
pub mod artifacts;
pub mod connect;
pub mod feed;
pub mod social;
pub mod socket;
pub mod types;
pub mod utility;

use axum::{
    Router,
    body::Body,
    extract::{MatchedPath, Path, Request, State},
    http::{StatusCode, header},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
    routing::get,
};
use tower_http::cors::CorsLayer;

use crate::{
    auth, context::AppContext, machines, monitoring, sessions, storage::files::FileStorageError,
};

pub fn build_router(ctx: AppContext) -> Router {
    let socket_layer = socket::build_layer(ctx.clone());

    Router::new()
        .route("/", get(root))
        .route("/files/{*path}", get(get_file))
        .merge(auth::routes())
        .merge(sessions::http::routes())
        .merge(machines::http::routes())
        .merge(account::routes())
        .merge(utility::routes())
        .merge(artifacts::routes())
        .merge(connect::routes())
        .merge(social::routes())
        .merge(feed::routes())
        .merge(monitoring::routes())
        .with_state(ctx.clone())
        .layer(from_fn_with_state(ctx, record_http_metrics))
        .layer(CorsLayer::permissive())
        .layer(socket_layer)
}

async fn root() -> &'static str {
    "Welcome to Vibe Server!"
}

async fn get_file(
    State(ctx): State<AppContext>,
    Path(path): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let bytes = match ctx.files().get(&path).await {
        Ok(bytes) => bytes,
        Err(FileStorageError::NotFound) => return Err(StatusCode::NOT_FOUND),
        Err(FileStorageError::InvalidPath) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let mime_type = match path.rsplit('.').next() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };
    Ok(([(header::CONTENT_TYPE, mime_type)], bytes).into_response())
}

async fn record_http_metrics(
    State(ctx): State<AppContext>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().to_string();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or_else(|| request.uri().path())
        .to_string();
    let response = next.run(request).await;
    ctx.metrics()
        .incr_http_request(&method, &route, response.status().as_u16());
    response
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::{config::Config, context::AppContext};

    use super::build_router;

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

    #[tokio::test]
    async fn auth_request_route_accepts_valid_public_key() {
        let ctx = AppContext::new(test_config());
        let app = build_router(ctx);
        let body = serde_json::json!({
            "publicKey": STANDARD.encode([7u8; 32]),
        });
        let response = app
            .oneshot(
                Request::post("/v1/auth/account/request")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["state"], "requested");
    }

    #[tokio::test]
    async fn terminal_auth_request_route_returns_authorized_token_with_session_extra() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let public_key = [7u8; 32];
        let public_key_hex = hex::encode(public_key);
        let request_record = ctx.db().put_terminal_auth_request(&public_key_hex, true);
        assert!(ctx.db().authorize_terminal_auth_request(
            &public_key_hex,
            "cipher-response",
            &account.id
        ));
        let app = build_router(ctx.clone());
        let body = serde_json::json!({
            "publicKey": STANDARD.encode(public_key),
            "supportsV2": true,
        });

        let response = app
            .oneshot(
                Request::post("/v1/auth/request")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["state"], "authorized");
        assert_eq!(json["response"], "cipher-response");

        let token = json["token"].as_str().unwrap();
        let claims = ctx.auth().verify_token(token).unwrap();
        assert_eq!(
            claims.extras.unwrap()["session"],
            Value::String(request_record.id)
        );
    }

    #[tokio::test]
    async fn terminal_auth_status_route_reports_pending_authorized_and_not_found() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let public_key = [8u8; 32];
        let public_key_hex = hex::encode(public_key);
        ctx.db().put_terminal_auth_request(&public_key_hex, true);
        let app = build_router(ctx.clone());

        let pending_response = app
            .clone()
            .oneshot(
                Request::get(format!(
                    "/v1/auth/request/status?publicKey={}",
                    STANDARD.encode(public_key)
                ))
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(pending_response.status(), StatusCode::OK);
        let body = to_bytes(pending_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "pending");
        assert_eq!(json["supportsV2"], true);

        assert!(ctx.db().authorize_terminal_auth_request(
            &public_key_hex,
            "cipher-response",
            &account.id
        ));

        let authorized_response = app
            .clone()
            .oneshot(
                Request::get(format!(
                    "/v1/auth/request/status?publicKey={}",
                    STANDARD.encode(public_key)
                ))
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(authorized_response.status(), StatusCode::OK);
        let body = to_bytes(authorized_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "authorized");
        assert_eq!(json["supportsV2"], false);

        let not_found_response = app
            .oneshot(
                Request::get("/v1/auth/request/status?publicKey=bad-key")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(not_found_response.status(), StatusCode::OK);
        let body = to_bytes(not_found_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "not_found");
        assert_eq!(json["supportsV2"], false);
    }

    #[tokio::test]
    async fn terminal_auth_response_route_authorizes_request_and_is_idempotent() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let public_key = [10u8; 32];
        let public_key_hex = hex::encode(public_key);
        ctx.db().put_terminal_auth_request(&public_key_hex, false);
        let app = build_router(ctx.clone());

        let response = app
            .clone()
            .oneshot(
                Request::post("/v1/auth/response")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        serde_json::json!({
                            "publicKey": STANDARD.encode(public_key),
                            "response": "cipher-response",
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let stored = ctx.db().get_terminal_auth_request(&public_key_hex).unwrap();
        assert_eq!(stored.response.as_deref(), Some("cipher-response"));
        assert_eq!(
            stored.response_account_id.as_deref(),
            Some(account.id.as_str())
        );

        let second_response = app
            .oneshot(
                Request::post("/v1/auth/response")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        serde_json::json!({
                            "publicKey": STANDARD.encode(public_key),
                            "response": "ignored-second-response",
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second_response.status(), StatusCode::OK);

        let stored = ctx.db().get_terminal_auth_request(&public_key_hex).unwrap();
        assert_eq!(stored.response.as_deref(), Some("cipher-response"));
    }

    #[tokio::test]
    async fn account_auth_response_route_authorizes_request_and_is_idempotent() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let public_key = [11u8; 32];
        let public_key_hex = hex::encode(public_key);
        ctx.db().put_account_auth_request(&public_key_hex);
        let app = build_router(ctx.clone());

        let response = app
            .clone()
            .oneshot(
                Request::post("/v1/auth/account/response")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        serde_json::json!({
                            "publicKey": STANDARD.encode(public_key),
                            "response": "cipher-response",
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let stored = ctx.db().get_account_auth_request(&public_key_hex).unwrap();
        assert_eq!(stored.response.as_deref(), Some("cipher-response"));
        assert_eq!(
            stored.response_account_id.as_deref(),
            Some(account.id.as_str())
        );

        let second_response = app
            .oneshot(
                Request::post("/v1/auth/account/response")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        serde_json::json!({
                            "publicKey": STANDARD.encode(public_key),
                            "response": "ignored-second-response",
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second_response.status(), StatusCode::OK);

        let stored = ctx.db().get_account_auth_request(&public_key_hex).unwrap();
        assert_eq!(stored.response.as_deref(), Some("cipher-response"));
    }

    #[tokio::test]
    async fn challenge_auth_route_accepts_happy_camel_case_payload() {
        let ctx = AppContext::new(test_config());
        let app = build_router(ctx);
        let signing_key = SigningKey::from_bytes(&[5u8; 32]);
        let challenge = [9u8; 32];
        let signature = signing_key.sign(&challenge);
        let body = serde_json::json!({
            "publicKey": STANDARD.encode(signing_key.verifying_key().as_bytes()),
            "challenge": STANDARD.encode(challenge),
            "signature": STANDARD.encode(signature.to_bytes()),
        });

        let response = app
            .oneshot(
                Request::post("/v1/auth")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);
        assert!(json["token"].as_str().is_some());
    }

    #[tokio::test]
    async fn protected_session_route_requires_bearer_token() {
        let ctx = AppContext::new(test_config());
        let app = build_router(ctx);
        let response = app
            .oneshot(Request::get("/v1/sessions").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "Missing authorization header");
    }

    #[tokio::test]
    async fn protected_session_route_rejects_invalid_token_with_happy_error_shape() {
        let ctx = AppContext::new(test_config());
        let app = build_router(ctx);
        let response = app
            .oneshot(
                Request::get("/v1/sessions")
                    .header("authorization", "Bearer invalid-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "Invalid token");
    }

    #[tokio::test]
    async fn version_route_accepts_happy_snake_case_app_id_body() {
        let ctx = AppContext::new(test_config());
        let app = build_router(ctx);

        let response = app
            .oneshot(
                Request::post("/v1/version")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "platform": "ios",
                            "version": "1.0.0",
                            "app_id": "happy",
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["updateUrl"], "ios-store");
    }

    #[tokio::test]
    async fn session_routes_create_and_list_records() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let app = build_router(ctx.clone());

        let create_body = serde_json::json!({
            "tag": "tag-1",
            "metadata": "ciphertext",
            "agentState": "ignored-state",
            "dataEncryptionKey": "key",
        });
        let create_response = app
            .clone()
            .oneshot(
                Request::post("/v1/sessions")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);
        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["session"]["metadataVersion"], 0);
        assert!(json["session"]["agentState"].is_null());
        assert_eq!(json["session"]["agentStateVersion"], 0);
        assert!(json["session"]["lastMessage"].is_null());

        let list_response = app
            .oneshot(
                Request::get("/v1/sessions")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);
        let body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["sessions"].as_array().unwrap().len(), 1);
        assert_eq!(json["sessions"][0]["metadataVersion"], 0);
        assert!(json["sessions"][0]["agentState"].is_null());
    }

    #[tokio::test]
    async fn machine_routes_create_and_fetch_detail() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let app = build_router(ctx);

        let create_body = serde_json::json!({
            "id": "machine-1",
            "metadata": "ciphertext",
            "daemonState": "daemon",
            "dataEncryptionKey": "key",
        });
        let create_response = app
            .clone()
            .oneshot(
                Request::post("/v1/machines")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(create_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create_response.status(), StatusCode::OK);

        let detail_response = app
            .clone()
            .oneshot(
                Request::get("/v1/machines/machine-1")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(detail_response.status(), StatusCode::OK);
        let body = to_bytes(detail_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["machine"]["id"], "machine-1");

        let list_response = app
            .oneshot(
                Request::get("/v1/machines")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);
        let body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json.as_array().unwrap().len(), 1);
        assert_eq!(json[0]["id"], "machine-1");
    }

    #[tokio::test]
    async fn machine_routes_isolate_same_machine_id_across_accounts() {
        let ctx = AppContext::new(test_config());
        let account_a = ctx.db().upsert_account_by_public_key("pk-a");
        let account_b = ctx.db().upsert_account_by_public_key("pk-b");
        let token_a = ctx.auth().create_token(&account_a.id, None);
        let token_b = ctx.auth().create_token(&account_b.id, None);
        let app = build_router(ctx);

        let create_a = serde_json::json!({
            "id": "machine-1",
            "metadata": "ciphertext-a",
        });
        let create_b = serde_json::json!({
            "id": "machine-1",
            "metadata": "ciphertext-b",
        });

        let response_a = app
            .clone()
            .oneshot(
                Request::post("/v1/machines")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token_a}"))
                    .body(Body::from(create_a.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response_a.status(), StatusCode::OK);

        let response_b = app
            .clone()
            .oneshot(
                Request::post("/v1/machines")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token_b}"))
                    .body(Body::from(create_b.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response_b.status(), StatusCode::OK);

        let detail_a = app
            .clone()
            .oneshot(
                Request::get("/v1/machines/machine-1")
                    .header("authorization", format!("Bearer {token_a}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let detail_b = app
            .clone()
            .oneshot(
                Request::get("/v1/machines/machine-1")
                    .header("authorization", format!("Bearer {token_b}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body_a = to_bytes(detail_a.into_body(), usize::MAX).await.unwrap();
        let body_b = to_bytes(detail_b.into_body(), usize::MAX).await.unwrap();
        let json_a: Value = serde_json::from_slice(&body_a).unwrap();
        let json_b: Value = serde_json::from_slice(&body_b).unwrap();

        assert_eq!(json_a["machine"]["metadata"], "ciphertext-a");
        assert_eq!(json_b["machine"]["metadata"], "ciphertext-b");
    }

    #[tokio::test]
    async fn session_history_route_returns_http_message_shape() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let _ = ctx.db().append_message(
            &account.id,
            &session.id,
            vibe_wire::SessionMessageContent::new("ciphertext"),
            Some("local-1".into()),
        );
        let app = build_router(ctx);

        let response = app
            .oneshot(
                Request::get(format!("/v1/sessions/{}/messages", session.id))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let message = &json["messages"][0];
        assert_eq!(message["localId"], "local-1");
        assert!(message.get("sessionId").is_none());
        assert!(message.get("session_id").is_none());
        assert!(message["createdAt"].is_number());
        assert!(message["updatedAt"].is_number());
    }

    #[tokio::test]
    async fn session_message_writes_refresh_v1_ordering_and_v2_changed_since() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (first, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta-1", None);
        let (second, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-2", "meta-2", None);

        ctx.db().write(|state| {
            let first = state.sessions.get_mut(&first.id).unwrap();
            first.updated_at = 10;
            let second = state.sessions.get_mut(&second.id).unwrap();
            second.updated_at = 20;
        });

        let _ = ctx.db().append_message(
            &account.id,
            &first.id,
            vibe_wire::SessionMessageContent::new("ciphertext"),
            Some("local-1".into()),
        );

        let app = build_router(ctx);

        let list_response = app
            .clone()
            .oneshot(
                Request::get("/v1/sessions")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_response.status(), StatusCode::OK);
        let body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let list_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(list_json["sessions"][0]["id"], first.id);

        let changed_response = app
            .oneshot(
                Request::get("/v2/sessions?changedSince=20")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(changed_response.status(), StatusCode::OK);
        let body = to_bytes(changed_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let changed_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(changed_json["sessions"].as_array().unwrap().len(), 1);
        assert_eq!(changed_json["sessions"][0]["id"], first.id);
    }

    #[tokio::test]
    async fn v3_message_post_rejects_empty_local_id() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let app = build_router(ctx);

        let response = app
            .oneshot(
                Request::post(format!("/v3/sessions/{}/messages", session.id))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::from(
                        serde_json::json!({
                            "messages": [
                                { "content": "ciphertext", "localId": "" }
                            ]
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn v2_session_routes_cover_active_cursor_and_invalid_cursor_paths() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (first, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta-1", None);
        let (second, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-2", "meta-2", None);
        let recent_active_at = crate::storage::db::now_ms();
        let stale_active_at = recent_active_at.saturating_sub(901_000);
        ctx.db().write(|state| {
            let second = state.sessions.get_mut(&second.id).unwrap();
            second.active = true;
            second.active_at = recent_active_at;
            let first = state.sessions.get_mut(&first.id).unwrap();
            first.active = true;
            first.active_at = stale_active_at;
        });
        let app = build_router(ctx);

        let active_response = app
            .clone()
            .oneshot(
                Request::get("/v2/sessions/active?limit=1")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(active_response.status(), StatusCode::OK);
        let body = to_bytes(active_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["sessions"].as_array().unwrap().len(), 1);
        assert_eq!(json["sessions"][0]["id"], second.id);

        let page_one = app
            .clone()
            .oneshot(
                Request::get("/v2/sessions?limit=1")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(page_one.status(), StatusCode::OK);
        let body = to_bytes(page_one.into_body(), usize::MAX).await.unwrap();
        let page_one_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(page_one_json["sessions"].as_array().unwrap().len(), 1);
        assert_eq!(page_one_json["hasNext"], true);
        let first_page_id = page_one_json["sessions"][0]["id"].as_str().unwrap();
        let expected_first = if first.id.as_str() > second.id.as_str() {
            first.id.as_str()
        } else {
            second.id.as_str()
        };
        let expected_second = if expected_first == first.id.as_str() {
            second.id.as_str()
        } else {
            first.id.as_str()
        };
        assert_eq!(first_page_id, expected_first);

        let next_cursor = page_one_json["nextCursor"].as_str().unwrap();
        let page_two = app
            .clone()
            .oneshot(
                Request::get(format!("/v2/sessions?limit=1&cursor={next_cursor}"))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(page_two.status(), StatusCode::OK);
        let body = to_bytes(page_two.into_body(), usize::MAX).await.unwrap();
        let page_two_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(page_two_json["sessions"].as_array().unwrap().len(), 1);
        assert_eq!(page_two_json["sessions"][0]["id"], expected_second);
        assert_eq!(page_two_json["hasNext"], false);
        assert!(page_two_json["nextCursor"].is_null());

        let invalid_cursor = app
            .clone()
            .oneshot(
                Request::get("/v2/sessions?cursor=bad-cursor")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_cursor.status(), StatusCode::BAD_REQUEST);

        let invalid_active_limit = app
            .clone()
            .oneshot(
                Request::get("/v2/sessions/active?limit=0")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_active_limit.status(), StatusCode::BAD_REQUEST);

        let invalid_page_limit = app
            .clone()
            .oneshot(
                Request::get("/v2/sessions?limit=201")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_page_limit.status(), StatusCode::BAD_REQUEST);

        let invalid_changed_since = app
            .oneshot(
                Request::get("/v2/sessions?changedSince=0")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid_changed_since.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn delete_session_route_removes_owned_session() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let app = build_router(ctx);

        let delete_response = app
            .clone()
            .oneshot(
                Request::delete(format!("/v1/sessions/{}", session.id))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delete_response.status(), StatusCode::OK);
        let body = to_bytes(delete_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["success"], true);

        let history_response = app
            .oneshot(
                Request::get(format!("/v1/sessions/{}/messages", session.id))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(history_response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn v3_message_get_pages_forward() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let _ = ctx.db().append_message(
            &account.id,
            &session.id,
            vibe_wire::SessionMessageContent::new("ciphertext-1"),
            Some("local-1".into()),
        );
        let _ = ctx.db().append_message(
            &account.id,
            &session.id,
            vibe_wire::SessionMessageContent::new("ciphertext-2"),
            Some("local-2".into()),
        );
        let _ = ctx.db().append_message(
            &account.id,
            &session.id,
            vibe_wire::SessionMessageContent::new("ciphertext-3"),
            Some("local-3".into()),
        );
        let app = build_router(ctx);

        let first_page = app
            .clone()
            .oneshot(
                Request::get(format!(
                    "/v3/sessions/{}/messages?after_seq=1&limit=1",
                    session.id
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(first_page.status(), StatusCode::OK);
        let body = to_bytes(first_page.into_body(), usize::MAX).await.unwrap();
        let first_page_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(first_page_json["messages"].as_array().unwrap().len(), 1);
        assert_eq!(first_page_json["messages"][0]["seq"], 2);
        assert_eq!(first_page_json["messages"][0]["localId"], "local-2");
        assert_eq!(first_page_json["hasMore"], true);

        let second_page = app
            .oneshot(
                Request::get(format!(
                    "/v3/sessions/{}/messages?after_seq=2&limit=2",
                    session.id
                ))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second_page.status(), StatusCode::OK);
        let body = to_bytes(second_page.into_body(), usize::MAX).await.unwrap();
        let second_page_json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(second_page_json["messages"].as_array().unwrap().len(), 1);
        assert_eq!(second_page_json["messages"][0]["seq"], 3);
        assert_eq!(second_page_json["hasMore"], false);
    }

    #[tokio::test]
    async fn v3_message_get_rejects_invalid_limits() {
        let ctx = AppContext::new(test_config());
        let account = ctx.db().upsert_account_by_public_key("pk");
        let token = ctx.auth().create_token(&account.id, None);
        let (session, _) = ctx
            .db()
            .create_or_load_session(&account.id, "tag-1", "meta", None);
        let app = build_router(ctx);

        let zero_limit = app
            .clone()
            .oneshot(
                Request::get(format!("/v3/sessions/{}/messages?limit=0", session.id))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(zero_limit.status(), StatusCode::BAD_REQUEST);

        let over_limit = app
            .oneshot(
                Request::get(format!("/v3/sessions/{}/messages?limit=501", session.id))
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(over_limit.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn file_routes_serve_public_assets_and_reject_traversal() {
        let ctx = AppContext::new(test_config());
        ctx.files().put("public/test.txt", b"hello").await.unwrap();
        let app = build_router(ctx);

        let response = app
            .clone()
            .oneshot(
                Request::get("/files/public/test.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&body[..], b"hello");

        let response = app
            .oneshot(
                Request::get("/files/../../etc/passwd")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
