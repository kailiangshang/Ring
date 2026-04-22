use async_stream::stream;
use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, KeepAliveStream, Sse};
use axum::Json;
use serde::Deserialize;
use std::convert::Infallible;
use std::pin::Pin;

use crate::error::Result;
use crate::extractors::auth::AuthUser;
use crate::models::message;
use crate::services::{llm::SseEvent, super_chat};
use crate::state::AppState;

type BoxedSseStream =
    Pin<Box<dyn tokio_stream::Stream<Item = std::result::Result<Event, Infallible>> + Send>>;

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

#[derive(Debug, serde::Serialize)]
pub struct PreferencesResponse {
    pub content: String,
    pub is_custom: bool,
}

#[derive(Debug, Deserialize)]
pub struct PreferencesRequest {
    pub content: String,
}

pub async fn super_chat_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<KeepAliveStream<BoxedSseStream>>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;
    let mut rx = super_chat::stream_super_chat(state, user_row, body.content);

    let s: BoxedSseStream = Box::pin(stream! {
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
                SseEvent::End { message_id, full_content: _, token_usage } => {
                    let usage_json = token_usage.as_deref().and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    });
    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn super_history(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let limit = query.limit + 1;
    let messages =
        super_chat::get_super_history(&state, &user.token_id, query.before.as_deref(), limit)
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

pub async fn get_preferences(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<PreferencesResponse>> {
    let (content, is_custom) = super_chat::get_user_preferences_info(&state.hub_dir);
    Ok(Json(PreferencesResponse { content, is_custom }))
}

pub async fn update_preferences(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<PreferencesRequest>,
) -> Result<Json<PreferencesResponse>> {
    super_chat::update_user_preferences(&state.hub_dir, &body.content)?;
    let (content, is_custom) = super_chat::get_user_preferences_info(&state.hub_dir);
    Ok(Json(PreferencesResponse { content, is_custom }))
}

#[derive(Debug, Deserialize)]
pub struct CrossRingQueryRequest {
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct CrossRingAnalysisRequest {
    pub ring_names: Vec<String>,
    pub analysis_type: String,
    pub question: Option<String>,
}

pub async fn cross_ring_query_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CrossRingQueryRequest>,
) -> Result<Sse<KeepAliveStream<BoxedSseStream>>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;
    let mut rx = super_chat::stream_cross_ring_query(state, user_row, body.query);

    let s: BoxedSseStream = Box::pin(stream! {
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
                SseEvent::End { message_id, full_content: _, token_usage } => {
                    let usage_json = token_usage.as_deref().and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    });
    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn cross_ring_analysis_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CrossRingAnalysisRequest>,
) -> Result<Sse<KeepAliveStream<BoxedSseStream>>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;
    let request = super_chat::CrossRingAnalysisRequest {
        ring_names: body.ring_names,
        analysis_type: body.analysis_type,
        question: body.question,
    };
    let mut rx = super_chat::stream_cross_ring_analysis(state, user_row, request);

    let s: BoxedSseStream = Box::pin(stream! {
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
                SseEvent::End { message_id, full_content: _, token_usage } => {
                    let usage_json = token_usage.as_deref().and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    });
    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}
