use async_stream::stream;
use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use serde::Deserialize;
use std::convert::Infallible;

use crate::error::{Result, RingError};
use crate::extractors::AuthUser;
use crate::models::ring;
use crate::models::session as session_model;
use crate::models::session::{
    ArchiveToggleInput, CreateSessionInput, InviteParticipantsInput, SessionMaterialRow,
    SessionParticipantRow,
};
use crate::services::llm::SseEvent;
use crate::services::session::{self, SessionResponse};
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
) -> Result<(axum::http::StatusCode, Json<SessionResponse>)> {
    let result = session::create_session(&state, &ring_id, &user.token_id, body).await?;

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_session_created(&self_dir) {
        tracing::warn!("failed to record session created: {e}");
    }

    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

pub async fn list_sessions(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<Json<serde_json::Value>> {
    let sessions =
        session::list_sessions(&state, &ring_id, &user.token_id, query.status.as_deref()).await?;
    Ok(Json(serde_json::json!({ "sessions": sessions })))
}

pub async fn get_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<SessionResponse>> {
    let result = session::get_session_detail(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(result))
}

pub async fn close_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<SessionResponse>> {
    let result = session::close_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(result))
}

pub async fn reopen_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<SessionResponse>> {
    let result = session::reopen_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(result))
}

pub async fn delete_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    session::delete_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub async fn invite_participants(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(body): Json<InviteParticipantsInput>,
) -> Result<Json<Vec<SessionParticipantRow>>> {
    let participants =
        session::invite_participants(&state, &ring_id, &session_id, &user.token_id, body).await?;
    Ok(Json(participants))
}

pub async fn remove_participant(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id, target_id)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>> {
    session::remove_participant(&state, &ring_id, &session_id, &target_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "status": "removed" })))
}

pub async fn archive_toggle(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(body): Json<ArchiveToggleInput>,
) -> Result<Json<SessionResponse>> {
    let result =
        session::toggle_archive(&state, &ring_id, &session_id, &user.token_id, body.enabled)
            .await?;
    Ok(Json(result))
}

#[derive(Debug, serde::Deserialize)]
pub struct TransferOwnershipInput {
    pub new_owner_id: String,
}

pub async fn transfer_ownership(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(body): Json<TransferOwnershipInput>,
) -> Result<Json<serde_json::Value>> {
    let role = crate::models::ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" {
        return Err(crate::error::RingError::Forbidden(
            "only creator can transfer session ownership".into(),
        ));
    }

    let is_participant: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM session_participants WHERE session_id = ?1 AND token_id = ?2",
    )
    .bind(&session_id)
    .bind(&body.new_owner_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    if !is_participant {
        return Err(crate::error::RingError::BadRequest(
            "new owner must be a session participant".into(),
        ));
    }

    sqlx::query("UPDATE sessions SET owner = ?1 WHERE id = ?2")
        .bind(&body.new_owner_id)
        .bind(&session_id)
        .execute(&state.db)
        .await
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    sqlx::query(
        "UPDATE session_participants SET role = 'owner' WHERE session_id = ?1 AND token_id = ?2",
    )
    .bind(&session_id)
    .bind(&body.new_owner_id)
    .execute(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    sqlx::query("UPDATE session_participants SET role = 'participant' WHERE session_id = ?1 AND token_id = ?2 AND role = 'owner'")
        .bind(&session_id)
        .bind(&user.token_id)
        .execute(&state.db)
        .await
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    Ok(Json(serde_json::json!({"status": "transferred"})))
}

pub async fn get_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<serde_json::Value>> {
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
) -> Result<Json<SessionResponse>> {
    let result = session::start_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(result))
}

pub async fn summarize_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;
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
    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);

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
                SseEvent::End { message_id, full_content: fc, token_usage: _ } => {
                    full_content = fc;
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": { "prompt_tokens": 0, "completion_tokens": 0 }
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    if let Err(e) = session_model::set_summary(&pool, &sid, &full_content).await {
                        tracing::warn!("failed to update session summary: {e}");
                    }
                    if let Err(e) = session_model::update_phase(&pool, &sid, "closed").await {
                        tracing::warn!("failed to update session phase: {e}");
                    }

                    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "session_summarize") {
                        tracing::warn!("failed to record tool usage: {e}");
                    }

                    if let Ok(updated_sess) = session_model::get_session(&pool, &sid).await {
                        let ring_name = crate::services::search::get_ring_name(&pool, &updated_sess.ring_id).await.unwrap_or_default();
                        let content = format!("{} {}", &updated_sess.description, updated_sess.summary.as_deref().unwrap_or(""));
                        let metadata = serde_json::json!({"skill": &updated_sess.skill, "phase": &updated_sess.phase}).to_string();
                        if let Err(e) = crate::services::search::upsert_search_index(
                            &pool, "session", &updated_sess.id, &updated_sess.ring_id, &ring_name,
                            &updated_sess.title, &content, &metadata,
                        ).await {
                            tracing::warn!("failed to update search index: {e}");
                        }
                    }
                }
                SseEvent::Error(msg) => {
                    if let Err(e) = session_model::update_phase(&pool, &sid, "discussion").await {
                        tracing::warn!("failed to update session phase: {e}");
                    }
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
) -> Result<Json<serde_json::Value>> {
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
) -> Result<Json<SessionMaterialRow>> {
    let result = session::highlight_material(
        &state,
        &ring_id,
        &session_id,
        &user.token_id,
        &body.material_id,
        &body.note,
    )
    .await?;
    Ok(Json(result))
}
