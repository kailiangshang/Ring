use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::error::RingError;
use crate::services::notification_service::NotificationService;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListNotificationsQuery {
    pub unread_only: Option<bool>,
}

pub async fn list_notifications(
    State(state): State<AppState>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<serde_json::Value>, RingError> {
    let service = NotificationService::new(state.db.clone());
    let notifications = service
        .list_for_user("user-1", query.unread_only.unwrap_or(false))
        .await?;
    Ok(Json(serde_json::json!({ "notifications": notifications })))
}

pub async fn mark_read(
    State(state): State<AppState>,
    Path(notification_id): Path<String>,
) -> Result<StatusCode, RingError> {
    let service = NotificationService::new(state.db.clone());
    service.mark_read(&notification_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
