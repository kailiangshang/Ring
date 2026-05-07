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
use crate::services::{
    chat::{self, CompactResult},
    llm,
    super_chat,
};
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

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "search") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    let s: BoxedSseStream = Box::pin(stream! {
        while let Some(event) = rx.recv().await {
            yield Ok(llm::sse_event_to_axum(event));
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
    let (prompt, is_custom) = super_chat::get_system_prompt_info(&state.hub_dir).await;
    Ok(Json(SystemPromptResponse { prompt, is_custom }))
}

pub async fn update_system_prompt(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<SystemPromptRequest>,
) -> Result<Json<SystemPromptResponse>> {
    let user_row = crate::models::user::get_user(&state.db, &user.token_id).await?;
    if !user_row.is_creator {
        return Err(crate::error::RingError::Forbidden(
            "only setup creator can modify system prompt".into(),
        ));
    }
    super_chat::update_system_prompt(&state.hub_dir, &body.prompt).await?;
    let (prompt, is_custom) = super_chat::get_system_prompt_info(&state.hub_dir).await;
    Ok(Json(SystemPromptResponse { prompt, is_custom }))
}

pub async fn get_preferences(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<PreferencesResponse>> {
    let (content, is_custom) = super_chat::get_user_preferences_info(&state.hub_dir).await;
    Ok(Json(PreferencesResponse { content, is_custom }))
}

pub async fn update_preferences(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<PreferencesRequest>,
) -> Result<Json<PreferencesResponse>> {
    let user_row = crate::models::user::get_user(&state.db, &user.token_id).await?;
    if !user_row.is_creator {
        return Err(crate::error::RingError::Forbidden(
            "only setup creator can modify preferences".into(),
        ));
    }
    super_chat::update_user_preferences(&state.hub_dir, &body.content).await?;
    let (content, is_custom) = super_chat::get_user_preferences_info(&state.hub_dir).await;
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
            yield Ok(llm::sse_event_to_axum(event));
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
            yield Ok(llm::sse_event_to_axum(event));
        }
    });
    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

#[derive(Debug, serde::Serialize)]
pub struct CompactResponse {
    pub summary: String,
    pub removed_count: usize,
}

pub async fn super_compact(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<CompactResponse>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;

    let super_user_id = format!("super:{}", user.token_id);
    let CompactResult {
        summary,
        removed_count,
    } = chat::compact_history(&state, &user_row, None, &super_user_id).await?;

    Ok(Json(CompactResponse {
        summary,
        removed_count,
    }))
}
