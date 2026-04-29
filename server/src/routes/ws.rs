use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

use crate::error::RingError;
use crate::models::session;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct WsParams {
    pub token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, RingError> {
    let exists: bool =
        sqlx::query_scalar::<_, bool>("SELECT COUNT(*) > 0 FROM users WHERE token_id = ?1")
            .bind(&params.token)
            .fetch_one(&state.db)
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;

    if !exists {
        return Err(RingError::Unauthorized("invalid token".into()));
    }

    let token_id = params.token;
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, token_id)))
}

async fn handle_socket(socket: WebSocket, state: AppState, token_id: String) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<String>(crate::ws_hub::WS_CHANNEL_SIZE);

    state.ws_hub.register(token_id.clone(), tx);

    {
        let resumed_sessions: Vec<String> = state.ws_hub.sessions_owned_by(&token_id);
        for session_id in &resumed_sessions {
            let msg = serde_json::json!({
                "type": "session_resumed",
                "session_id": session_id,
            });
            state
                .ws_hub
                .broadcast_to_session(session_id, &msg.to_string());
        }
    }

    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let db = state.db.clone();
    let ws_hub = state.ws_hub.clone();
    let ws_hub_recv = ws_hub.clone();
    let token_id_recv = token_id.clone();

    let recv_task = tokio::spawn(async move {
        let mut last_activity = tokio::time::Instant::now();
        loop {
            let timeout = tokio::time::sleep(tokio::time::Duration::from_secs(60));
            tokio::select! {
                msg = ws_receiver.next() => {
                    last_activity = tokio::time::Instant::now();
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            handle_text_message(&db, &ws_hub_recv, &token_id_recv, &text).await;
                        }
                        Some(Ok(Message::Ping(data))) => {
                            ws_hub_recv.send_to_user(
                                &token_id_recv,
                                &format!(
                                    r#"{{"type":"pong","data":"{}"}}"#,
                                    String::from_utf8_lossy(&data)
                                ),
                            );
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => {}
                    }
                }
                _ = timeout => {
                    if last_activity.elapsed().as_secs() > 60 {
                        tracing::debug!("WebSocket idle timeout for {}", token_id_recv);
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    let paused = ws_hub.unregister(&token_id);
    for session_id in paused {
        let msg = json!({
            "type": "session_paused",
            "session_id": session_id,
            "reason": "owner_offline"
        });
        ws_hub.broadcast_to_session(&session_id, &msg.to_string());
    }
}

async fn handle_text_message(
    db: &sqlx::SqlitePool,
    ws_hub: &crate::ws_hub::WsHub,
    token_id: &str,
    text: &str,
) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return;
    };

    let msg_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match msg_type {
        "session_message" => {
            let Some(session_id) = value.get("session_id").and_then(|v| v.as_str()) else {
                return;
            };
            let Some(content) = value.get("content").and_then(|v| v.as_str()) else {
                return;
            };

            let Ok(is_participant) = session::is_participant(db, session_id, token_id).await else {
                return;
            };
            if !is_participant {
                return;
            }

            let sess = match session::get_session(db, session_id).await {
                Ok(s) => s,
                Err(_) => return,
            };

            if sess.phase == "closed" {
                return;
            }

            let Ok(role) = crate::models::ring::get_user_role(db, &sess.ring_id, token_id).await
            else {
                return;
            };
            if role == "readonly" {
                return;
            }

            let sender_name: String = sqlx::query_scalar::<_, String>(
                "SELECT display_name FROM users WHERE token_id = ?1",
            )
            .bind(token_id)
            .fetch_one(db)
            .await
            .unwrap_or_else(|_| "Unknown".to_string());

            let id = ulid::Ulid::new().to_string();
            let seq_num = match session::next_seq_num(db, session_id).await {
                Ok(n) => n,
                Err(_) => return,
            };

            let Ok(msg_row) = session::insert_message(
                db,
                &id,
                session_id,
                seq_num,
                token_id,
                &sender_name,
                content,
                "user",
            )
            .await
            else {
                return;
            };

            let ring_name = crate::services::search::get_ring_name(db, &sess.ring_id)
                .await
                .unwrap_or_default();
            let metadata =
                serde_json::json!({"session_id": session_id, "message_type": "user"}).to_string();
            let _ = crate::services::search::upsert_search_index(
                db,
                "session_message",
                &id,
                &sess.ring_id,
                &ring_name,
                &sender_name,
                content,
                &metadata,
            )
            .await;

            let broadcast = json!({
                "type": "session_message",
                "session_id": session_id,
                "seq_num": msg_row.seq_num,
                "sender": msg_row.sender,
                "sender_name": msg_row.sender_name,
                "content": msg_row.content,
                "created_at": msg_row.created_at,
            });

            ws_hub.broadcast_to_session(session_id, &broadcast.to_string());
        }
        "session_catchup" => {
            let Some(session_id) = value.get("session_id").and_then(|v| v.as_str()) else {
                return;
            };
            let last_seq = value.get("last_seq").and_then(|v| v.as_i64()).unwrap_or(0);

            let Ok(is_participant) = session::is_participant(db, session_id, token_id).await else {
                return;
            };
            if !is_participant {
                return;
            }

            let Ok(messages) = session::get_messages(db, session_id, last_seq, 100).await else {
                return;
            };

            let catchup = json!({
                "type": "session_catchup",
                "session_id": session_id,
                "messages": messages,
            });

            ws_hub.send_to_user(token_id, &catchup.to_string());
        }
        _ => {}
    }
}
