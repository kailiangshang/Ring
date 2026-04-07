use axum::extract::{Extension, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::RingError;
use crate::handlers::sse_helpers::{spawn_sse_stream, SseStream};
use crate::middleware::auth::AuthUser;
use crate::services::ai_service::AiService;
use crate::services::llm_provider::LlmMessage;
use crate::services::tool_engine::ToolDispatcher;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperRingRequest {
    pub message: String,
    pub history: Option<Vec<HistoryMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
}

pub async fn super_ring_chat(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SuperRingRequest>,
) -> Result<SseStream, RingError> {
    if req.message.trim().is_empty() {
        return Err(RingError::Validation("message must not be empty".into()));
    }

    let llm = state.rebuild_llm().await;
    let dispatcher = Arc::new(ToolDispatcher::new(state.tool_registry.clone()));
    let ai = AiService::new(state.db.clone(), llm, dispatcher);
    let history = req.history.unwrap_or_default();
    let history: Vec<LlmMessage> = history
        .into_iter()
        .filter(|m| m.role == "user" || m.role == "assistant")
        .map(|m| LlmMessage { role: m.role, content: m.content })
        .collect();
    let llm_stream = ai.super_ring_chat(auth_user.user_id.clone(), req.message, history).await?;

    Ok(spawn_sse_stream(llm_stream))
}
