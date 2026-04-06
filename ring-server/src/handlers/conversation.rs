use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, Sse};
use axum::Json;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;

use crate::error::RingError;
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

async fn get_first_user_id(state: &AppState) -> Result<String, RingError> {
    let users = state.db.list_all_users().await?;
    users
        .into_iter()
        .next()
        .map(|u| u.id)
        .ok_or_else(|| RingError::Validation("no user found, run setup first".into()))
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
    Path(ring_id): Path<String>,
    Json(req): Json<CreateConvRequest>,
) -> Result<(axum::http::StatusCode, Json<Conversation>), RingError> {
    let user_id = get_first_user_id(&state).await?;
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
) -> Result<Sse<ReceiverStream<Result<Event, Infallible>>>, RingError> {
    if req.message.trim().is_empty() {
        return Err(RingError::Validation("message must not be empty".into()));
    }

    let dispatcher = Arc::new(ToolDispatcher::new(state.tool_registry.clone()));
    let ai = AiService::new(state.db.clone(), state.llm_provider.clone(), dispatcher);
    let llm_stream = ai.group_ring_chat(&ring_id, &conv_id, req.message).await?;

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(32);

    tokio::spawn(async move {
        let mut stream = std::pin::pin!(llm_stream);
        while let Some(event) = stream.next().await {
            let json = serde_json::to_string(&event).unwrap_or_default();
            let sse_event = Event::default().event("message").data(json);
            if tx.send(Ok(sse_event)).await.is_err() {
                break;
            }
        }
    });

    Ok(Sse::new(ReceiverStream::new(rx)))
}
