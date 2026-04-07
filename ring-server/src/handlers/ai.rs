use axum::extract::{Extension, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::RingError;
use crate::handlers::sse_helpers::{spawn_sse_stream, SseStream};
use crate::middleware::auth::AuthUser;
use crate::services::ai_service::AiService;
use crate::services::tool_engine::ToolDispatcher;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperRingRequest {
    pub message: String,
}

pub async fn super_ring_chat(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<SuperRingRequest>,
) -> Result<SseStream, RingError> {
    if req.message.trim().is_empty() {
        return Err(RingError::Validation("message must not be empty".into()));
    }

    let dispatcher = Arc::new(ToolDispatcher::new(state.tool_registry.clone()));
    let ai = AiService::new(state.db.clone(), state.llm_provider.clone(), dispatcher);
    let llm_stream = ai.super_ring_chat(req.message).await?;
    let _ = auth_user;

    Ok(spawn_sse_stream(llm_stream))
}
