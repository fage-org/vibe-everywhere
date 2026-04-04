use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};

use crate::{
    api::types::{
        ApiError, FriendMutationBody, FriendsListResponse, UserPath, UserProfile,
        UserProfileResponse, UserSearchQuery, UserSearchResponse,
    },
    auth::AuthenticatedUser,
    context::AppContext,
    storage::db::{FeedPostRecord, RelationshipRecord, now_ms},
};

const NOTIFICATION_COOLDOWN_MS: u64 = 24 * 60 * 60 * 1000;

pub fn routes() -> Router<AppContext> {
    Router::new()
        .route("/v1/user/{id}", get(get_user))
        .route("/v1/user/search", get(search_users))
        .route("/v1/friends", get(list_friends))
        .route("/v1/friends/add", post(add_friend))
        .route("/v1/friends/remove", post(remove_friend))
}

async fn get_user(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Path(path): Path<UserPath>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let account = ctx
        .db()
        .get_account(&path.id)
        .ok_or_else(|| ApiError::not_found("User not found"))?;
    let status = relationship_status(&ctx, &user.user_id, &path.id);
    Ok(Json(UserProfileResponse {
        user: Some(build_user_profile(&ctx, account, &status)),
    }))
}

async fn search_users(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<UserSearchResponse>, ApiError> {
    let term = query
        .query
        .ok_or_else(|| ApiError::bad_request("query is required"))?
        .to_lowercase();
    let mut users = ctx.db().read(|state| {
        state
            .accounts
            .values()
            .filter(|account| {
                account
                    .username
                    .as_deref()
                    .unwrap_or_default()
                    .to_lowercase()
                    .starts_with(&term)
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    users.sort_by(|a, b| a.username.cmp(&b.username));
    users.truncate(10);
    Ok(Json(UserSearchResponse {
        users: users
            .into_iter()
            .map(|account| {
                let status = relationship_status(&ctx, &user.user_id, &account.id);
                build_user_profile(&ctx, account, &status)
            })
            .collect(),
    }))
}

async fn list_friends(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
) -> Result<Json<FriendsListResponse>, ApiError> {
    let mut friends = ctx.db().read(|state| {
        state
            .relationships
            .values()
            .filter(|relationship| {
                relationship.from_account_id == user.user_id && relationship.status == "friend"
            })
            .filter_map(|relationship| state.accounts.get(&relationship.to_account_id))
            .cloned()
            .collect::<Vec<_>>()
    });
    friends.sort_by(|a, b| a.username.cmp(&b.username));
    Ok(Json(FriendsListResponse {
        friends: friends
            .into_iter()
            .map(|account| build_user_profile(&ctx, account, "friend"))
            .collect(),
    }))
}

async fn add_friend(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<FriendMutationBody>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    if body.uid == user.user_id {
        return Ok(Json(UserProfileResponse { user: None }));
    }
    let Some(friend) = ctx.db().get_account(&body.uid) else {
        return Ok(Json(UserProfileResponse { user: None }));
    };
    let friend_id = friend.id.clone();

    let mut updates = Vec::new();
    let mut feed_updates = Vec::new();
    let response_status = ctx.db().write(|state| {
        let current_status = state
            .relationships
            .get(&(user.user_id.clone(), friend_id.clone()))
            .map(|relationship| relationship.status.clone())
            .unwrap_or_else(|| "none".into());
        let reverse_status = state
            .relationships
            .get(&(friend_id.clone(), user.user_id.clone()))
            .map(|relationship| relationship.status.clone())
            .unwrap_or_else(|| "none".into());

        if reverse_status == "requested" {
            set_relationship(state, &friend_id, &user.user_id, "friend");
            set_relationship(state, &user.user_id, &friend_id, "friend");
            updates.push((
                user.user_id.clone(),
                friend_id.clone(),
                "friend".to_string(),
            ));
            updates.push((
                friend_id.clone(),
                user.user_id.clone(),
                "friend".to_string(),
            ));
            if should_send_notification(state, &user.user_id, &friend_id, "friend") {
                feed_updates.push(create_feed_post(
                    state,
                    &user.user_id,
                    serde_json::json!({ "kind": "friend_accepted", "uid": friend_id }),
                    Some(format!("friend_accepted_{}", friend_id)),
                ));
                mark_relationship_notified(state, &user.user_id, &friend_id);
            }
            if should_send_notification(state, &friend_id, &user.user_id, "friend") {
                feed_updates.push(create_feed_post(
                    state,
                    &friend_id,
                    serde_json::json!({ "kind": "friend_accepted", "uid": user.user_id }),
                    Some(format!("friend_accepted_{}", user.user_id)),
                ));
                mark_relationship_notified(state, &friend_id, &user.user_id);
            }
            return "friend".to_string();
        }

        if matches!(current_status.as_str(), "none" | "rejected") {
            set_relationship(state, &user.user_id, &friend_id, "requested");
            if reverse_status == "none" {
                set_relationship(state, &friend_id, &user.user_id, "pending");
            }
            updates.push((
                user.user_id.clone(),
                friend_id.clone(),
                "requested".to_string(),
            ));
            updates.push((
                friend_id.clone(),
                user.user_id.clone(),
                if reverse_status == "none" {
                    "pending"
                } else {
                    reverse_status.as_str()
                }
                .to_string(),
            ));
            if should_send_notification(
                state,
                &friend_id,
                &user.user_id,
                state
                    .relationships
                    .get(&(friend_id.clone(), user.user_id.clone()))
                    .map(|relationship| relationship.status.as_str())
                    .unwrap_or("none"),
            ) {
                feed_updates.push(create_feed_post(
                    state,
                    &friend_id,
                    serde_json::json!({ "kind": "friend_request", "uid": user.user_id }),
                    Some(format!("friend_request_{}", user.user_id)),
                ));
                mark_relationship_notified(state, &friend_id, &user.user_id);
            }
            return "requested".to_string();
        }

        current_status
    });

    for (account_id, target_id, status) in updates {
        ctx.events()
            .publish_relationship_update(ctx.db(), &account_id, &target_id, &status)
            .map_err(|_| ApiError::internal("Failed to add friend"))?;
    }
    for post in feed_updates {
        ctx.events()
            .publish_new_feed_post(ctx.db(), &post.account_id, &post)
            .map_err(|_| ApiError::internal("Failed to add friend"))?;
    }
    Ok(Json(UserProfileResponse {
        user: Some(build_user_profile(&ctx, friend, &response_status)),
    }))
}

async fn remove_friend(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Json(body): Json<FriendMutationBody>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let Some(friend) = ctx.db().get_account(&body.uid) else {
        return Ok(Json(UserProfileResponse { user: None }));
    };
    let friend_id = friend.id.clone();

    let mut updates = Vec::new();
    let response_status = ctx.db().write(|state| {
        let current_status = state
            .relationships
            .get(&(user.user_id.clone(), friend_id.clone()))
            .map(|relationship| relationship.status.clone())
            .unwrap_or_else(|| "none".into());
        let reverse_status = state
            .relationships
            .get(&(friend_id.clone(), user.user_id.clone()))
            .map(|relationship| relationship.status.clone())
            .unwrap_or_else(|| "none".into());

        match current_status.as_str() {
            "requested" => {
                set_relationship(state, &user.user_id, &friend_id, "rejected");
                updates.push((
                    user.user_id.clone(),
                    friend_id.clone(),
                    "rejected".to_string(),
                ));
                "rejected".to_string()
            }
            "friend" => {
                set_relationship(state, &friend_id, &user.user_id, "requested");
                set_relationship(state, &user.user_id, &friend_id, "pending");
                updates.push((
                    user.user_id.clone(),
                    friend_id.clone(),
                    "pending".to_string(),
                ));
                updates.push((
                    friend_id.clone(),
                    user.user_id.clone(),
                    "requested".to_string(),
                ));
                "requested".to_string()
            }
            "pending" => {
                set_relationship(state, &user.user_id, &friend_id, "none");
                updates.push((user.user_id.clone(), friend_id.clone(), "none".to_string()));
                if reverse_status != "rejected" {
                    set_relationship(state, &friend_id, &user.user_id, "none");
                    updates.push((friend_id.clone(), user.user_id.clone(), "none".to_string()));
                }
                "none".to_string()
            }
            _ => current_status,
        }
    });

    for (account_id, target_id, status) in updates {
        ctx.events()
            .publish_relationship_update(ctx.db(), &account_id, &target_id, &status)
            .map_err(|_| ApiError::internal("Failed to remove friend"))?;
    }
    Ok(Json(UserProfileResponse {
        user: Some(build_user_profile(&ctx, friend, &response_status)),
    }))
}

fn build_user_profile(
    ctx: &AppContext,
    account: crate::storage::db::AccountRecord,
    status: &str,
) -> UserProfile {
    let bio = account
        .github_profile
        .as_ref()
        .and_then(|value| value.get("bio"))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    let username = account.username.clone().or_else(|| {
        account
            .github_profile
            .as_ref()
            .and_then(|value| value.get("login"))
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned)
    });

    UserProfile {
        id: account.id,
        first_name: account.first_name.unwrap_or_default(),
        last_name: account.last_name,
        username: username.unwrap_or_default(),
        avatar: ctx.files().resolve_image_json(account.avatar.as_ref()),
        bio,
        status: status.to_string(),
    }
}

fn relationship_status(ctx: &AppContext, from_user_id: &str, to_user_id: &str) -> String {
    ctx.db()
        .read(|state| {
            state
                .relationships
                .get(&(from_user_id.to_string(), to_user_id.to_string()))
                .map(|relationship| relationship.status.clone())
        })
        .unwrap_or_else(|| "none".into())
}

fn set_relationship(
    state: &mut crate::storage::db::DatabaseState,
    from_user_id: &str,
    to_user_id: &str,
    status: &str,
) {
    let key = (from_user_id.to_string(), to_user_id.to_string());
    let now = now_ms();
    match state.relationships.get_mut(&key) {
        Some(existing) => {
            existing.status = status.to_string();
            existing.updated_at = now;
        }
        None => {
            state.relationships.insert(
                key,
                RelationshipRecord {
                    from_account_id: from_user_id.to_string(),
                    to_account_id: to_user_id.to_string(),
                    status: status.to_string(),
                    last_notified_at: None,
                    created_at: now,
                    updated_at: now,
                },
            );
        }
    }
}

fn should_send_notification(
    state: &crate::storage::db::DatabaseState,
    receiver_user_id: &str,
    sender_user_id: &str,
    status: &str,
) -> bool {
    if status == "rejected" {
        return false;
    }
    let last_notified_at = state
        .relationships
        .get(&(receiver_user_id.to_string(), sender_user_id.to_string()))
        .and_then(|relationship| relationship.last_notified_at);
    let Some(last_notified_at) = last_notified_at else {
        return true;
    };
    last_notified_at.saturating_add(NOTIFICATION_COOLDOWN_MS) < now_ms()
}

fn mark_relationship_notified(
    state: &mut crate::storage::db::DatabaseState,
    from_user_id: &str,
    to_user_id: &str,
) {
    if let Some(relationship) = state
        .relationships
        .get_mut(&(from_user_id.to_string(), to_user_id.to_string()))
    {
        relationship.last_notified_at = Some(now_ms());
        relationship.updated_at = now_ms();
    }
}

fn create_feed_post(
    state: &mut crate::storage::db::DatabaseState,
    account_id: &str,
    body: serde_json::Value,
    repeat_key: Option<String>,
) -> FeedPostRecord {
    if let Some(repeat_key) = repeat_key.as_deref() {
        let to_delete = state
            .feed_posts
            .iter()
            .filter(|(_, post)| {
                post.account_id == account_id && post.repeat_key.as_deref() == Some(repeat_key)
            })
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        for id in to_delete {
            state.feed_posts.remove(&id);
        }
    }

    let now = now_ms();
    let counter = {
        let account = state
            .accounts
            .get_mut(account_id)
            .expect("account should exist");
        account.feed_seq += 1;
        account.feed_seq
    };
    let post = FeedPostRecord {
        id: format!("feed_{}", uuid::Uuid::now_v7()),
        account_id: account_id.to_string(),
        repeat_key,
        body,
        cursor: format!("0-{counter}"),
        created_at: now,
        updated_at: now,
    };
    state.feed_posts.insert(post.id.clone(), post.clone());
    post
}
