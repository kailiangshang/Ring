use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::error::RingError;
use crate::models::session_model::*;
use crate::services::session_service::SessionService;
use crate::state::AppState;

pub async fn create_session(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionDetailResponse>), RingError> {
    let service = SessionService::new(state.db.clone());
    let session = service.create_session(&ring_id, &req, "user-1").await?;
    Ok((StatusCode::CREATED, Json(session)))
}

#[derive(Deserialize)]
pub struct ListSessionsQuery {
    pub status: Option<String>,
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<Json<SessionListResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let resp = service
        .list_sessions(&ring_id, query.status.as_deref())
        .await?;
    Ok(Json(resp))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<SessionDetailResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let detail = service.get_session_detail(&ring_id, &session_id).await?;
    Ok(Json(detail))
}

pub async fn close_session(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service
        .close_session(&ring_id, &session_id, "user-1")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn leave_session(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service
        .leave_session(&ring_id, &session_id, "user-1")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn toggle_archive(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(req): Json<ArchiveToggleRequest>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service
        .toggle_archive(&ring_id, &session_id, req.archive_enabled, "user-1")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn invite_member(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(req): Json<InviteSessionRequest>,
) -> Result<Json<serde_json::Value>, RingError> {
    let service = SessionService::new(state.db.clone());
    let invited = service
        .invite_member(&ring_id, &session_id, &req.member_ids, "user-1")
        .await?;
    Ok(Json(serde_json::json!({ "invited": invited })))
}

pub async fn delete_session(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service
        .delete_session(&ring_id, &session_id, "user-1")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    pub after_seq: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_messages(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<SessionMessagesResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let resp = service
        .get_messages(
            &ring_id,
            &session_id,
            query.after_seq,
            query.limit.unwrap_or(50),
        )
        .await?;
    Ok(Json(resp))
}
