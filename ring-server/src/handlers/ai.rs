use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::Json;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;

use crate::error::RingError;
use crate::services::ai_service::AiService;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuperRingRequest {
    pub message: String,
}

pub async fn super_ring_chat(
    State(state): State<AppState>,
    Json(req): Json<SuperRingRequest>,
) -> Result<Sse<ReceiverStream<Result<Event, Infallible>>>, RingError> {
    if req.message.trim().is_empty() {
        return Err(RingError::Validation("message must not be empty".into()));
    }

    let ai = AiService::new(state.db.clone(), state.llm_provider.clone());
    let llm_stream = ai.super_ring_chat(req.message).await?;

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
