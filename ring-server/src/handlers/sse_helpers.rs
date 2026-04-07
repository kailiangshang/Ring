use axum::response::sse::{Event, Sse};
use futures::{Stream, StreamExt};
use std::convert::Infallible;
use std::pin::Pin;
use tokio_stream::wrappers::ReceiverStream;

use crate::services::llm_provider::LlmEvent;

pub type SseStream = Sse<ReceiverStream<Result<Event, Infallible>>>;

pub fn spawn_sse_stream(llm_stream: Pin<Box<dyn Stream<Item = LlmEvent> + Send>>) -> SseStream {
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

    Sse::new(ReceiverStream::new(rx))
}
