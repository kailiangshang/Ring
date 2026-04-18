use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::session::{ArchiveToggleInput, CreateSessionInput, InviteParticipantsInput};
use crate::services::session;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListSessionsQuery {
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    pub after_seq: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn create_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateSessionInput>,
) -> Result<(axum::http::StatusCode, Json<Value>)> {
    let result = session::create_session(&state, &ring_id, &user.token_id, body).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::to_value(result).unwrap()),
    ))
}

pub async fn list_sessions(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<Json<Value>> {
    let sessions =
        session::list_sessions(&state, &ring_id, &user.token_id, query.status.as_deref()).await?;
    Ok(Json(serde_json::json!({ "sessions": sessions })))
}

pub async fn get_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let result = session::get_session_detail(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn close_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let result = session::close_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn reopen_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let result = session::reopen_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn delete_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    session::delete_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub async fn invite_participants(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(body): Json<InviteParticipantsInput>,
) -> Result<Json<Value>> {
    let participants =
        session::invite_participants(&state, &ring_id, &session_id, &user.token_id, body).await?;
    Ok(Json(serde_json::to_value(participants).unwrap()))
}

pub async fn remove_participant(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id, target_id)): Path<(String, String, String)>,
) -> Result<Json<Value>> {
    session::remove_participant(&state, &ring_id, &session_id, &target_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

pub async fn archive_toggle(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(body): Json<ArchiveToggleInput>,
) -> Result<Json<Value>> {
    let result =
        session::toggle_archive(&state, &ring_id, &session_id, &user.token_id, body.enabled)
            .await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn get_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<Value>> {
    let messages = session::get_messages(
        &state,
        &ring_id,
        &session_id,
        &user.token_id,
        query.after_seq.unwrap_or(0),
        query.limit.unwrap_or(50),
    )
    .await?;
    Ok(Json(serde_json::json!({ "messages": messages })))
}
