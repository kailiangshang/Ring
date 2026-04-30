use async_stream::stream;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use serde::Deserialize;
use std::convert::Infallible;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::message;
use crate::models::ring;
use crate::services::blueprint_service::{
    get_blueprint, preview_from_template, FromTemplateRequest,
};
use crate::services::llm::SseEvent;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct BlueprintChatRequest {
    pub content: String,
    pub current_blueprint: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BlueprintHistoryQuery {
    pub before: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, serde::Serialize)]
pub struct BlueprintHistoryResponse {
    pub messages: Vec<message::MessageRow>,
    pub has_more: bool,
}

pub async fn get_blueprint_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<crate::services::blueprint_service::BlueprintResponse>> {
    let _ = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let blueprint = get_blueprint(&state, &ring_id).await?;
    Ok(Json(blueprint))
}

pub async fn preview_template(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<FromTemplateRequest>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(crate::error::RingError::Forbidden(
            "only creator/admin can manage blueprint".into(),
        ));
    }
    let preview = preview_from_template(&state, &body.template).await?;
    Ok(Json(serde_json::json!({ "preview": preview })))
}

pub async fn confirm_blueprint_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<crate::services::blueprint_service::ConfirmBlueprintRequest>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(crate::error::RingError::Forbidden(
            "only creator/admin can manage blueprint".into(),
        ));
    }
    crate::services::blueprint_service::confirm_with_blueprint(&state, &ring_id, &body).await?;

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "blueprint") {
        tracing::warn!("failed to record tool usage: {e}");
    }

    Ok(Json(serde_json::json!({ "status": "confirmed" })))
}

pub async fn blueprint_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<BlueprintChatRequest>,
) -> Result<
    Sse<
        axum::response::sse::KeepAliveStream<
            BoxStream<'static, std::result::Result<Event, Infallible>>,
        >,
    >,
> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(crate::error::RingError::Forbidden(
            "only creator/admin can manage blueprint".into(),
        ));
    }

    let status =
        sqlx::query_scalar::<_, String>("SELECT blueprint_status FROM rings WHERE id = ?1")
            .bind(&ring_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    if status == "confirmed" {
        return Err(crate::error::RingError::Forbidden(
            "blueprint already confirmed".into(),
        ));
    }

    let user_row = state.get_user_decrypted(&user.token_id).await?;

    let ring_info = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT name, role_description FROM rings WHERE id = ?1",
    )
    .bind(&ring_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?
    .ok_or_else(|| crate::error::RingError::NotFound("ring not found".into()))?;

    let current_bp_str = body
        .current_blueprint
        .as_ref()
        .map(|v| serde_json::to_string_pretty(v).unwrap_or_default());

    let system_prompt = crate::prompts::blueprint::system(
        &ring_info.0,
        ring_info.1.as_deref(),
        current_bp_str.as_deref(),
    );

    let history_messages =
        message::list_messages(&state.db, Some(&ring_id), &user.token_id, None, 15)
            .await
            .unwrap_or_default()
            .into_iter()
            .rev()
            .filter(|m| m.role == "blueprint" || m.role == "user")
            .map(|m| (m.role, m.content))
            .collect::<Vec<_>>();

    let user_msg_id = ulid::Ulid::new().to_string();
    message::insert_message(
        &state.db,
        &message::NewMessage {
            id: &user_msg_id,
            ring_id: Some(&ring_id),
            user_id: &user.token_id,
            role: "user",
            sender_name: &user_row.display_name,
            content: &body.content,
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await?;

    let llm = crate::services::llm::LlmClient::from_user(&user_row)?;
    let mut rx = llm.chat_stream(
        system_prompt,
        history_messages,
        body.content.clone(),
        "blueprint".to_string(),
    );

    let pool = state.db.clone();
    let ring_id_c = ring_id.clone();
    let user_id = user.token_id.clone();

    let s = stream! {
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
                SseEvent::End { message_id, full_content, token_usage } => {
                    let usage_json = token_usage.as_deref().and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    if let Err(e) = message::insert_message(
                        &pool,
                        &message::NewMessage {
                            id: &message_id,
                            ring_id: Some(&ring_id_c),
                            user_id: &user_id,
                            role: "blueprint",
                            sender_name: "GROUP RING",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: token_usage.as_deref(),
                        },
                    ).await {
                        tracing::warn!("failed to insert message: {e}");
                    }

                    let self_dir = crate::services::self_data::get_self_dir(&user_id);
                    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "blueprint") {
                        tracing::warn!("failed to record tool usage: {e}");
                    }
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    }.boxed();

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn blueprint_history(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    axum::extract::Query(query): axum::extract::Query<BlueprintHistoryQuery>,
) -> Result<Json<BlueprintHistoryResponse>> {
    let _ = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let limit = query.limit + 1;
    let messages = message::list_messages(
        &state.db,
        Some(&ring_id),
        &user.token_id,
        query.before.as_deref(),
        limit,
    )
    .await
    .unwrap_or_default()
    .into_iter()
    .rev()
    .filter(|m| m.role == "blueprint" || m.role == "user")
    .collect::<Vec<_>>();

    let has_more = messages.len() > query.limit as usize;
    let messages = if has_more {
        messages.into_iter().take(query.limit as usize).collect()
    } else {
        messages
    };

    Ok(Json(BlueprintHistoryResponse { messages, has_more }))
}
