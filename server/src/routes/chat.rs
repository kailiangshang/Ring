use async_stream::stream;
use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use serde::Deserialize;
use std::convert::Infallible;

use crate::error::{Result, RingError};
use crate::extractors::auth::AuthUser;
use crate::models::message;
use crate::models::ring;
use crate::models::user;
use crate::services::{
    chat::{self, ChatParams},
    llm::SseEvent,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub content: String,
    #[serde(default)]
    pub node_refs: Vec<String>,
    #[serde(default)]
    pub tag_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub before: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, serde::Serialize)]
pub struct HistoryResponse {
    pub messages: Vec<message::MessageRow>,
    pub has_more: bool,
}

pub async fn ring_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user_row = user::get_user(&state.db, &user.token_id).await?;
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let ring_info = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT name, role_description FROM rings WHERE id = ?1",
    )
    .bind(&ring_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?
    .ok_or_else(|| RingError::NotFound("ring not found".into()))?;

    let mut rx = chat::start_chat_stream(
        &state,
        &user_row,
        &ChatParams {
            ring_id: Some(&ring_id),
            role_description: ring_info.1.as_deref(),
            ring_name: Some(&ring_info.0),
            ai_role: "group_ring",
            content: &body.content,
            node_refs: body.node_refs,
            tag_refs: body.tag_refs,
        },
    )
    .await?;

    let pool = state.db.clone();
    let ring_id_c = ring_id.clone();
    let user_id = user.token_id.clone();

    let s = stream! {
        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::Start { message_id, role } => {
                    let data = serde_json::json!({"message_id": message_id, "role": role});
                    yield Ok(Event::default().event("message_start").data(data.to_string()));
                }
                SseEvent::Delta { content } => {
                    let data = serde_json::json!({ "content": content });
                    yield Ok(Event::default().event("delta").data(data.to_string()));
                }
                SseEvent::End { message_id, full_content, token_usage } => {
                    let usage_json = token_usage.as_deref().and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    let _ = message::insert_message(
                        &pool,
                        &message::NewMessage {
                            id: &message_id,
                            ring_id: Some(&ring_id_c),
                            user_id: &user_id,
                            role: "group_ring",
                            sender_name: "GROUP RING",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: token_usage.as_deref(),
                        },
                    ).await;
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    };

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn ring_history(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let limit = query.limit + 1;
    let messages = chat::get_history(
        &state,
        Some(&ring_id),
        &user.token_id,
        query.before.as_deref(),
        limit,
    )
    .await?;

    let has_more = messages.len() > query.limit as usize;
    let messages = if has_more {
        messages.into_iter().take(query.limit as usize).collect()
    } else {
        messages
    };

    Ok(Json(HistoryResponse { messages, has_more }))
}

pub async fn self_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user_row = user::get_user(&state.db, &user.token_id).await?;

    let mut rx = chat::start_chat_stream(
        &state,
        &user_row,
        &ChatParams {
            ring_id: None,
            role_description: None,
            ring_name: None,
            ai_role: "self",
            content: &body.content,
            node_refs: body.node_refs,
            tag_refs: body.tag_refs,
        },
    )
    .await?;

    let pool = state.db.clone();
    let user_id = user.token_id.clone();

    let s = stream! {
        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::Start { message_id, role } => {
                    let data = serde_json::json!({"message_id": message_id, "role": role});
                    yield Ok(Event::default().event("message_start").data(data.to_string()));
                }
                SseEvent::Delta { content } => {
                    let data = serde_json::json!({ "content": content });
                    yield Ok(Event::default().event("delta").data(data.to_string()));
                }
                SseEvent::End { message_id, full_content, token_usage } => {
                    let usage_json = token_usage.as_deref().and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    let _ = message::insert_message(
                        &pool,
                        &message::NewMessage {
                            id: &message_id,
                            ring_id: None,
                            user_id: &user_id,
                            role: "self",
                            sender_name: "SELF",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: token_usage.as_deref(),
                        },
                    ).await;
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    };

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn self_history(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let limit = query.limit + 1;
    let messages =
        chat::get_history(&state, None, &user.token_id, query.before.as_deref(), limit).await?;

    let has_more = messages.len() > query.limit as usize;
    let messages = if has_more {
        messages.into_iter().take(query.limit as usize).collect()
    } else {
        messages
    };

    Ok(Json(HistoryResponse { messages, has_more }))
}
