use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::RingError;
use crate::handlers::sse_helpers::{spawn_sse_stream_with_callback, SseStream};
use crate::middleware::auth::AuthUser;
use crate::models::session_model::*;
use crate::services::ai_service::AiService;
use crate::services::session_service::SessionService;
use crate::services::tool_engine::ToolDispatcher;
use crate::state::AppState;

use std::sync::Arc;

pub async fn create_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(ring_id): Path<String>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionDetailResponse>), RingError> {
    let service = SessionService::new(state.db.clone());
    let session = service
        .create_session(&ring_id, &req, &auth_user.user_id)
        .await?;
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
    Extension(auth_user): Extension<AuthUser>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service
        .close_session(&ring_id, &session_id, &auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn leave_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service
        .leave_session(&ring_id, &session_id, &auth_user.user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn toggle_archive(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(req): Json<ArchiveToggleRequest>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service
        .toggle_archive(
            &ring_id,
            &session_id,
            req.archive_enabled,
            &auth_user.user_id,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn invite_member(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(req): Json<InviteSessionRequest>,
) -> Result<Json<serde_json::Value>, RingError> {
    let service = SessionService::new(state.db.clone());
    let invited = service
        .invite_member(&ring_id, &session_id, &req.member_ids, &auth_user.user_id)
        .await?;
    Ok(Json(serde_json::json!({ "invited": invited })))
}

pub async fn delete_session(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service
        .delete_session(&ring_id, &session_id, &auth_user.user_id)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionChatRequest {
    pub message: String,
}

pub async fn send_message(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(req): Json<SessionChatRequest>,
) -> Result<SseStream, RingError> {
    if req.message.trim().is_empty() {
        return Err(RingError::Validation("message must not be empty".into()));
    }

    let session_svc = SessionService::new(state.db.clone());
    let session = session_svc.get_session_detail(&ring_id, &session_id).await?;
    let ring = state
        .db
        .get_ring(&ring_id)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;

    let llm = state.rebuild_llm().await;
    let dispatcher = Arc::new(ToolDispatcher::new(state.tool_registry.clone()));
    let ai = AiService::new(state.db.clone(), llm, dispatcher);
    let llm_stream = ai
        .session_chat(
            &ring_id,
            &session_id,
            &auth_user.user_id,
            &ring.name,
            &session.scenario,
            req.message,
        )
        .await?;

    let db = state.db.clone();
    let sid = session_id.clone();
    let on_complete = move |content: String| {
        let db = db.clone();
        let session_id = sid.clone();
        tokio::spawn(async move {
            let msgs = db.get_session_messages(&session_id, None, 1).await.ok();
            let seq = msgs
                .and_then(|m| m.last().map(|msg| msg.seq_num + 1))
                .unwrap_or(1);
            let _ = db
                .create_session_message(&session_id, "system", "assistant", &content, seq)
                .await;
        });
    };


    Ok(spawn_sse_stream_with_callback(llm_stream, Some(on_complete)))
}
