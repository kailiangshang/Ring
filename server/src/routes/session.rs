use async_stream::stream;
use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;

use crate::error::{Result, RingError};
use crate::extractors::AuthUser;
use crate::models::ring;
use crate::models::session as session_model;
use crate::models::session::{ArchiveToggleInput, CreateSessionInput, InviteParticipantsInput};
use crate::models::user;
use crate::services::llm::SseEvent;
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

pub async fn start_session_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let result = session::start_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn summarize_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user_row = user::get_user(&state.db, &user.token_id).await?;
    let _ = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if !session_model::is_owner(&state.db, &session_id, &user.token_id).await? {
        return Err(RingError::Forbidden("only owner can summarize".into()));
    }

    let sess = session_model::get_session(&state.db, &session_id).await?;
    if sess.phase != "discussion" {
        return Err(RingError::BadRequest(
            "session is not in discussion phase".into(),
        ));
    }

    session_model::update_phase(&state.db, &session_id, "summary").await?;

    let messages = session_model::get_messages(&state.db, &session_id, 0, 10000).await?;
    let messages_text = messages
        .iter()
        .map(|m| format!("[{}] {}: {}", m.created_at, m.sender_name, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let ctx = session::SummarizeContext {
        session_id: session_id.clone(),
        skill: sess.skill.clone(),
        messages_text,
    };

    let mut rx = session::start_summarize_stream(&state, &user_row, ctx)?;

    let pool = state.db.clone();
    let sid = session_id.clone();

    let s = stream! {
        let mut full_content = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::Start { message_id, role } => {
                    let data = serde_json::json!({"message_id": message_id, "role": role});
                    yield Ok(Event::default().event("message_start").data(data.to_string()));
                }
                SseEvent::Delta { content } => {
                    full_content.push_str(&content);
                    let data = serde_json::json!({ "content": content });
                    yield Ok(Event::default().event("delta").data(data.to_string()));
                }
                SseEvent::End { message_id, full_content: fc } => {
                    full_content = fc;
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": { "prompt_tokens": 0, "completion_tokens": 0 }
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    let _ = session_model::set_summary(&pool, &sid, &full_content).await;
                    let _ = session_model::update_phase(&pool, &sid, "closed").await;
                }
                SseEvent::Error(msg) => {
                    let _ = session_model::update_phase(&pool, &sid, "discussion").await;
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    };

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn get_material_prep(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let materials =
        session::get_materials_service(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "materials": materials })))
}

#[derive(Debug, Deserialize)]
pub struct HighlightInput {
    pub material_id: String,
    pub note: String,
}

pub async fn highlight_material_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(body): Json<HighlightInput>,
) -> Result<Json<Value>> {
    let result = session::highlight_material(
        &state,
        &ring_id,
        &session_id,
        &user.token_id,
        &body.material_id,
        &body.note,
    )
    .await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}
