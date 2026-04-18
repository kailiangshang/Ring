use async_stream::stream;
use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use serde::Deserialize;
use std::convert::Infallible;

use crate::error::Result;
use crate::extractors::auth::AuthUser;
use crate::models::message;
use crate::models::user;
use crate::services::{llm::SseEvent, super_chat};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub content: String,
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

#[derive(Debug, Deserialize)]
pub struct SystemPromptRequest {
    pub prompt: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SystemPromptResponse {
    pub prompt: String,
    pub is_custom: bool,
}

pub async fn super_chat_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user_row = user::get_user(&state.db, &user.token_id).await?;

    let mut rx = super_chat::start_super_chat(&state, &user_row, &body.content).await?;

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
                SseEvent::End { message_id, full_content } => {
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": { "prompt_tokens": 0, "completion_tokens": 0 }
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    let _ = message::insert_message(
                        &pool,
                        &message::NewMessage {
                            id: &message_id,
                            ring_id: Some("super"),
                            user_id: &user_id,
                            role: "super_ring",
                            sender_name: "SUPER RING",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: None,
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

pub async fn super_history(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let limit = query.limit + 1;
    let messages = super_chat::get_super_history(
        &state,
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

pub async fn get_system_prompt(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<SystemPromptResponse>> {
    let (prompt, is_custom) = super_chat::get_system_prompt_info(&state.hub_dir);
    Ok(Json(SystemPromptResponse { prompt, is_custom }))
}

pub async fn update_system_prompt(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<SystemPromptRequest>,
) -> Result<Json<SystemPromptResponse>> {
    super_chat::update_system_prompt(&state.hub_dir, &body.prompt)?;
    let (prompt, is_custom) = super_chat::get_system_prompt_info(&state.hub_dir);
    Ok(Json(SystemPromptResponse { prompt, is_custom }))
}
