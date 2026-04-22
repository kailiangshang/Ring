use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::notification;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListNotificationsQuery {
    unread_only: Option<bool>,
}

pub async fn list_notifications(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<Vec<notification::NotificationRow>>> {
    let notifications = notification::list_notifications(
        &state.db,
        &user.token_id,
        query.unread_only.unwrap_or(false),
    )
    .await?;
    Ok(Json(notifications))
}

pub async fn mark_as_read(
    State(state): State<AppState>,
    user: AuthUser,
    Path(notification_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    notification::mark_as_read(&state.db, &notification_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn mark_all_as_read(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    notification::mark_all_as_read(&state.db, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn get_unread_count(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let count = notification::get_unread_count(&state.db, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "count": count })))
}

pub async fn delete_notification(
    State(state): State<AppState>,
    user: AuthUser,
    Path(notification_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    notification::delete_notification(&state.db, &notification_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
