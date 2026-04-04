use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;
use vibe_server::{
    api::build_router,
    config::Config,
    context::AppContext,
    storage::db::{
        FeedPostRecord, GithubTokenRecord, UsageReportRecord, VendorTokenRecord, now_ms,
    },
};

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

fn auth_token(ctx: &AppContext, public_key: &str) -> (String, String) {
    let account = ctx.db().upsert_account_by_public_key(public_key);
    let token = ctx.auth().create_token(&account.id, None);
    (account.id, token)
}

async fn json_response(app: axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn account_utility_and_usage_routes_work() {
    let ctx = AppContext::new(test_config());
    let (account_id, token) = auth_token(&ctx, "wave4-user");
    ctx.db().write(|state| {
        let account = state.accounts.get_mut(&account_id).unwrap();
        account.username = Some("wave4".into());
        state.usage_reports.insert(
            "usage-1".into(),
            UsageReportRecord {
                id: "usage-1".into(),
                account_id: account_id.clone(),
                session_id: None,
                key: "claude".into(),
                tokens: [("total".into(), 42_u64)].into_iter().collect(),
                cost: [("total".into(), 1.5_f64)].into_iter().collect(),
                created_at: now_ms(),
                updated_at: now_ms(),
            },
        );
    });
    let app = build_router(ctx.clone());

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/account/settings")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                json!({"settings":"{\"theme\":\"light\"}","expectedVersion":0}).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert_eq!(json["version"], 1);

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/kv")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                json!({"mutations":[{"key":"pref/theme","value":"light","version":-1}]})
                    .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);

    let (status, json) = json_response(
        app.clone(),
        Request::get("/v1/kv/pref%2Ftheme")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["value"], "light");

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/push-tokens")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"token":"expo-1"}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/voice/token")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"agentId":"agent-1"}).to_string()))
            .unwrap(),
    )
    .await;
    if std::env::var("ELEVENLABS_API_KEY").is_ok() {
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["allowed"], true);
        assert_eq!(json["agentId"], "agent-1");
    } else {
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["error"], "Voice service not configured");
    }

    let (status, json) = json_response(
        app,
        Request::post("/v1/usage/query")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["groupBy"], "day");
    assert_eq!(json["totalReports"], 1);
    assert_eq!(json["usage"][0]["tokens"]["total"], 42);
    assert_eq!(json["usage"][0]["reportCount"], 1);

    let (status, json) = json_response(
        build_router(ctx.clone()),
        Request::get("/health").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["status"], "ok");

    let response = build_router(ctx)
        .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let metrics = String::from_utf8(body.to_vec()).unwrap();
    assert!(metrics.contains("http_requests_total"));
}

#[tokio::test]
async fn usage_query_respects_epoch_second_bounds_at_millisecond_precision() {
    let ctx = AppContext::new(test_config());
    let (account_id, token) = auth_token(&ctx, "wave4-usage-bounds");
    ctx.db().write(|state| {
        state.usage_reports.insert(
            "usage-boundary".into(),
            UsageReportRecord {
                id: "usage-boundary".into(),
                account_id,
                session_id: None,
                key: "claude".into(),
                tokens: [("total".into(), 5_u64)].into_iter().collect(),
                cost: [("total".into(), 0.2_f64)].into_iter().collect(),
                created_at: 1_500,
                updated_at: 1_500,
            },
        );
    });

    let (status, json) = json_response(
        build_router(ctx),
        Request::post("/v1/usage/query")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"endTime":1}).to_string()))
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["groupBy"], "day");
    assert_eq!(json["totalReports"], 0);
    assert_eq!(json["usage"], json!([]));
}

#[tokio::test]
async fn artifact_and_access_key_routes_work() {
    let ctx = AppContext::new(test_config());
    let (account_id, token) = auth_token(&ctx, "wave4-artifact");
    let (session, _) = ctx
        .db()
        .create_or_load_session(&account_id, "tag", "metadata", None);
    let (machine, _) =
        ctx.db()
            .create_or_load_machine(&account_id, "machine-1", "metadata", None, None);
    let app = build_router(ctx.clone());
    let artifact_id = uuid::Uuid::now_v7().to_string();

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/artifacts")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                json!({
                    "id": artifact_id,
                    "header": "h1",
                    "body": "b1",
                    "dataEncryptionKey": "dek"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["headerVersion"], 1);

    let (status, json) = json_response(
        app.clone(),
        Request::post(format!("/v1/artifacts/{artifact_id}"))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                json!({
                    "header":"h2",
                    "expectedHeaderVersion":1,
                    "body":"b2",
                    "expectedBodyVersion":1
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert_eq!(json["headerVersion"], 2);

    let (status, json) = json_response(
        app.clone(),
        Request::post(format!("/v1/access-keys/{}/{}", session.id, machine.id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"data":"secret"}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert_eq!(json["accessKey"]["dataVersion"], 1);

    let (status, json) = json_response(
        app,
        Request::put(format!("/v1/access-keys/{}/{}", session.id, machine.id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                json!({"data":"secret-2","expectedVersion":1}).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert_eq!(json["version"], 2);
}

#[tokio::test]
async fn connect_social_and_feed_routes_work() {
    let ctx = AppContext::new(test_config());
    let (account_id, token) = auth_token(&ctx, "wave4-social");
    let friend = ctx.db().upsert_account_by_public_key("wave4-friend");
    let friend_token = ctx.auth().create_token(&friend.id, None);
    ctx.db().write(|state| {
        state.accounts.get_mut(&account_id).unwrap().username = Some("owner".into());
        state.accounts.get_mut(&friend.id).unwrap().username = Some("friend".into());
        state.accounts.get_mut(&account_id).unwrap().feed_seq = 1;
        state.feed_posts.insert(
            "feed-1".into(),
            FeedPostRecord {
                id: "feed-1".into(),
                account_id: account_id.clone(),
                repeat_key: None,
                body: json!({"kind":"text","text":"hello feed"}),
                cursor: "0-1".into(),
                created_at: now_ms(),
                updated_at: now_ms(),
            },
        );
    });
    let app = build_router(ctx.clone());

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/connect/anthropic/register")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"token":"service-token"}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);

    let (status, json) = json_response(
        app.clone(),
        Request::get("/v1/account/profile")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        json["connectedServices"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "anthropic")
    );

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/friends/add")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"uid":friend.id}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user"]["username"], "friend");
    assert_eq!(json["user"]["status"], "requested");

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/friends/add")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {friend_token}"))
            .body(Body::from(json!({"uid":account_id}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user"]["status"], "friend");

    let (status, json) = json_response(
        app.clone(),
        Request::get("/v1/friends")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["friends"][0]["username"], "friend");

    let (status, json) = json_response(
        app,
        Request::get("/v1/feed")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["items"][0]["body"]["kind"], "friend_accepted");
    assert_eq!(
        json["items"][0]["repeatKey"],
        format!("friend_accepted_{}", friend.id)
    );
    assert_eq!(json["items"][1]["body"]["text"], "hello feed");
    assert_eq!(json["hasMore"], false);
}

#[tokio::test]
async fn github_tokens_stay_out_of_generic_connect_routes_and_connected_services() {
    let ctx = AppContext::new(test_config());
    let (account_id, token) = auth_token(&ctx, "wave4-github-hidden");
    ctx.db().write(|state| {
        state.accounts.get_mut(&account_id).unwrap().github_profile =
            Some(json!({ "id": 7, "login": "octocat" }));
        state.vendor_tokens.insert(
            (account_id.clone(), "anthropic".into()),
            VendorTokenRecord {
                id: "vt_anthropic".into(),
                account_id: account_id.clone(),
                vendor: "anthropic".into(),
                token: "service-token".into(),
                created_at: now_ms(),
                updated_at: now_ms(),
            },
        );
        state.github_tokens.insert(
            account_id.clone(),
            GithubTokenRecord {
                id: "ght_github".into(),
                account_id: account_id.clone(),
                github_user_id: "7".into(),
                token: "github-secret".into(),
                created_at: now_ms(),
                updated_at: now_ms(),
            },
        );
    });
    let app = build_router(ctx);

    let (status, json) = json_response(
        app.clone(),
        Request::get("/v1/account/profile")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["connectedServices"], json!(["anthropic"]));
    assert_eq!(json["github"]["login"], "octocat");

    let (status, json) = json_response(
        app.clone(),
        Request::get("/v1/connect/tokens")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        json,
        json!({"tokens":[{"vendor":"anthropic","token":"service-token"}]})
    );

    let (status, json) = json_response(
        app,
        Request::get("/v1/connect/github/token")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["error"], "Unsupported vendor");
}

#[tokio::test]
async fn friend_routes_return_null_for_self_and_missing_users() {
    let ctx = AppContext::new(test_config());
    let (_, token) = auth_token(&ctx, "wave4-social-null");
    let self_uid = ctx.auth().verify_token(&token).unwrap().user_id;
    let app = build_router(ctx);

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/friends/add")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"uid":"acct_ghost"}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user"], Value::Null);

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/friends/add")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"uid":self_uid}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user"], Value::Null);

    let (status, json) = json_response(
        app,
        Request::post("/v1/friends/remove")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"uid":"acct_ghost"}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user"], Value::Null);
}

#[tokio::test]
async fn friend_request_notifications_respect_happy_repeat_key_and_cooldown() {
    let ctx = AppContext::new(test_config());
    let (account_id, token) = auth_token(&ctx, "wave4-social-repeat-owner");
    let friend = ctx
        .db()
        .upsert_account_by_public_key("wave4-social-repeat-friend");
    let friend_token = ctx.auth().create_token(&friend.id, None);
    ctx.db().write(|state| {
        state.accounts.get_mut(&account_id).unwrap().username = Some("owner".into());
        state.accounts.get_mut(&friend.id).unwrap().username = Some("friend".into());
    });
    let app = build_router(ctx.clone());

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/friends/add")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"uid":friend.id}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user"]["status"], "requested");

    let (status, json) = json_response(
        app.clone(),
        Request::get("/v1/feed")
            .header("authorization", format!("Bearer {friend_token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["items"].as_array().unwrap().len(), 1);
    assert_eq!(json["items"][0]["body"]["kind"], "friend_request");
    assert_eq!(
        json["items"][0]["repeatKey"],
        format!("friend_request_{}", account_id)
    );

    let (status, _) = json_response(
        app.clone(),
        Request::post("/v1/friends/remove")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"uid":friend.id}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/friends/add")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(json!({"uid":friend.id}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["user"]["status"], "requested");

    let (status, json) = json_response(
        app,
        Request::get("/v1/feed")
            .header("authorization", format!("Bearer {friend_token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        json["items"][0]["repeatKey"],
        format!("friend_request_{}", account_id)
    );
}

#[tokio::test]
async fn artifact_update_mismatch_does_not_partially_apply() {
    let ctx = AppContext::new(test_config());
    let (_, token) = auth_token(&ctx, "wave4-artifact-atomic");
    let app = build_router(ctx.clone());
    let artifact_id = uuid::Uuid::now_v7().to_string();

    let _ = json_response(
        app.clone(),
        Request::post("/v1/artifacts")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                json!({
                    "id": artifact_id,
                    "header": "h1",
                    "body": "b1",
                    "dataEncryptionKey": "dek"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;

    let (status, json) = json_response(
        app.clone(),
        Request::post(format!("/v1/artifacts/{artifact_id}"))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                json!({
                    "header":"h2",
                    "expectedHeaderVersion":1,
                    "body":"b2",
                    "expectedBodyVersion":99
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], false);
    assert_eq!(json["error"], "version-mismatch");

    let (status, json) = json_response(
        app,
        Request::get(format!("/v1/artifacts/{artifact_id}"))
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["header"], "h1");
    assert_eq!(json["headerVersion"], 1);
    assert_eq!(json["body"], "b1");
    assert_eq!(json["bodyVersion"], 1);
}

#[tokio::test]
async fn kv_mutation_conflict_remains_all_or_nothing() {
    let ctx = AppContext::new(test_config());
    let (_, token) = auth_token(&ctx, "wave4-kv-atomic");
    let app = build_router(ctx.clone());

    let _ = json_response(
        app.clone(),
        Request::post("/v1/kv")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                json!({"mutations":[{"key":"a","value":"one","version":-1},{"key":"b","value":"two","version":-1}]}).to_string(),
            ))
            .unwrap(),
    )
    .await;

    let (status, json) = json_response(
        app.clone(),
        Request::post("/v1/kv")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::from(
                json!({"mutations":[{"key":"a","value":"updated","version":0},{"key":"missing","value":"x","version":0}]}).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(json["success"], false);

    let (status, json) = json_response(
        app,
        Request::get("/v1/kv/a")
            .header("authorization", format!("Bearer {token}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["value"], "one");
    assert_eq!(json["version"], 0);
}
