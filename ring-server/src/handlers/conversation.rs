use axum::extract::{Extension, Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::RingError;
use crate::handlers::sse_helpers::{spawn_sse_stream, SseStream};
use crate::middleware::auth::AuthUser;
use crate::models::conversation::Conversation;
use crate::services::ai_service::AiService;
use crate::services::tool_engine::ToolDispatcher;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConvRequest {
    pub title: Option<String>,
    pub context_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListConversationsResponse {
    pub conversations: Vec<Conversation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessagesResponse {
    pub messages: Vec<crate::models::conversation::Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesQueryParams {
    pub limit: Option<i64>,
    pub before_id: Option<String>,
}

pub async fn list(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> Result<Json<ListConversationsResponse>, RingError> {
    let conversations = state.db.list_conversations(&ring_id).await?;
    Ok(Json(ListConversationsResponse { conversations }))
}

pub async fn create(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(ring_id): Path<String>,
    Json(req): Json<CreateConvRequest>,
) -> Result<(axum::http::StatusCode, Json<Conversation>), RingError> {
    let user_id = auth_user.user_id;
    let context_mode = req.context_mode.unwrap_or_else(|| "storage".into());
    let conv = state
        .db
        .create_conversation(&ring_id, req.title, &context_mode, &user_id)
        .await?;
    Ok((axum::http::StatusCode::CREATED, Json(conv)))
}

pub async fn get(
    State(state): State<AppState>,
    Path((_ring_id, conv_id)): Path<(String, String)>,
) -> Result<Json<Conversation>, RingError> {
    let conv = state
        .db
        .get_conversation(&conv_id)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("conversation {}", conv_id)))?;
    Ok(Json(conv))
}

pub async fn get_messages(
    State(state): State<AppState>,
    Path((_ring_id, conv_id)): Path<(String, String)>,
    Query(params): Query<MessagesQueryParams>,
) -> Result<Json<ListMessagesResponse>, RingError> {
    let limit = params.limit.unwrap_or(50);
    let before_id = params.before_id.as_deref();
    let messages = state.db.get_messages(&conv_id, limit, before_id).await?;
    Ok(Json(ListMessagesResponse { messages }))
}

pub async fn send_message(
    State(state): State<AppState>,
    Path((ring_id, conv_id)): Path<(String, String)>,
    Json(req): Json<SendMessageRequest>,
) -> Result<SseStream, RingError> {
    if req.message.trim().is_empty() {
        return Err(RingError::Validation("message must not be empty".into()));
    }

    let dispatcher = Arc::new(ToolDispatcher::new(state.tool_registry.clone()));
    let ai = AiService::new(state.db.clone(), state.llm_provider.clone(), dispatcher);
    let llm_stream = ai.group_ring_chat(&ring_id, &conv_id, req.message).await?;

    Ok(spawn_sse_stream(llm_stream))
}
