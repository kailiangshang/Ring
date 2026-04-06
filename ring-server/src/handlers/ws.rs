use axum::extract::{
    ws::{Message, WebSocket},
    Path, State, WebSocketUpgrade,
};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};

use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, ring_id))
}

async fn handle_socket(socket: WebSocket, state: AppState, ring_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_hub.subscribe(&ring_id).await;

    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if msg.is_err() {
                break;
            }
        }
    });

    let mut send_task = tokio::spawn(async move {
        while let Ok(ws_msg) = rx.recv().await {
            let json = match serde_json::to_string(&ws_msg) {
                Ok(j) => j,
                Err(_) => continue,
            };
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    }
}
