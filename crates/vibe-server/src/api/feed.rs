use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
};

use crate::{
    api::types::{ApiError, FeedListResponse, FeedPostResponse, FeedQuery},
    auth::AuthenticatedUser,
    context::AppContext,
};

pub fn routes() -> Router<AppContext> {
    Router::new().route("/v1/feed", get(list_feed))
}

async fn list_feed(
    State(ctx): State<AppContext>,
    user: AuthenticatedUser,
    Query(query): Query<FeedQuery>,
) -> Result<Json<FeedListResponse>, ApiError> {
    let limit = query.limit.unwrap_or(50);
    if !(1..=200).contains(&limit) {
        return Err(ApiError::bad_request("limit must be between 1 and 200"));
    }
    let mut posts = ctx.db().read(|state| {
        state
            .feed_posts
            .values()
            .filter(|post| post.account_id == user.user_id)
            .cloned()
            .collect::<Vec<_>>()
    });
    posts.sort_by_key(|post| std::cmp::Reverse(parse_cursor(&post.cursor).unwrap_or(0)));
    if let Some(before) = query.before.as_deref() {
        let before =
            parse_cursor(before).ok_or_else(|| ApiError::bad_request("Invalid cursor format"))?;
        posts.retain(|post| parse_cursor(&post.cursor).is_some_and(|cursor| cursor < before));
    } else if let Some(after) = query.after.as_deref() {
        let after =
            parse_cursor(after).ok_or_else(|| ApiError::bad_request("Invalid cursor format"))?;
        posts.retain(|post| parse_cursor(&post.cursor).is_some_and(|cursor| cursor > after));
    }
    let has_more = posts.len() > limit;
    if has_more {
        posts.truncate(limit);
    }
    Ok(Json(FeedListResponse {
        items: posts.iter().map(FeedPostResponse::from_record).collect(),
        has_more,
    }))
}

fn parse_cursor(cursor: &str) -> Option<u64> {
    cursor.strip_prefix("0-")?.parse().ok()
}
