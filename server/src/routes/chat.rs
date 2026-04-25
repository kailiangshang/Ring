use async_stream::stream;
use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use serde::Deserialize;
use std::convert::Infallible;

use crate::error::{Result, RingError};
use crate::extractors::auth::AuthUser;
use crate::models::message;
use crate::models::ring;
use crate::services::{
    chat::{self, ChatParams},
    llm::SseEvent,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub content: String,
    #[serde(default)]
    pub node_refs: Vec<String>,
    #[serde(default)]
    pub tag_refs: Vec<String>,
    #[serde(default)]
    pub ephemeral: bool,
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

pub async fn ring_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<ChatRequest>,
) -> Result<
    Sse<
        axum::response::sse::KeepAliveStream<
            BoxStream<'static, std::result::Result<Event, Infallible>>,
        >,
    >,
> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let ring_info = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT name, role_description FROM rings WHERE id = ?1",
    )
    .bind(&ring_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?
    .ok_or_else(|| RingError::NotFound("ring not found".into()))?;

    if chat::detect_archive_intent(&body.content) {
        let ring_id_c = ring_id.clone();
        let user_id_c = user.token_id.clone();
        let state_c = state.clone();
        let content_c = body.content.clone();
        let self_dir_c = crate::services::self_data::get_self_dir(&user.token_id);
        let s = stream! {
            let message_id = ulid::Ulid::new().to_string();
            yield Ok(Event::default().event("message_start").data(serde_json::json!({"message_id": message_id, "role": "group_ring"}).to_string()));
            yield Ok(Event::default().event("delta").data(serde_json::json!({"content": "检测到归档意图，正在启动归档流程..."}).to_string()));
            yield Ok(Event::default().event("message_end").data(serde_json::json!({"message_id": message_id, "usage": {"prompt_tokens": 0, "completion_tokens": 0}}).to_string()));
            let _ = crate::services::archive_service::quick_archive(
                &state_c, &ring_id_c, &user_id_c, &content_c,
            ).await;
            let _ = crate::services::self_data::record_tool_usage(&self_dir_c, "archive");
        }.boxed();
        return Ok(Sse::new(s).keep_alive(KeepAlive::default()));
    }

    {
        let state_c = state.clone();
        let user_row_c = user_row.clone();
        let ring_id_c = ring_id.clone();
        let token_id_c = user.token_id.clone();
        tokio::spawn(async move {
            let _ = chat::auto_compact_history(&state_c, &user_row_c, Some(&ring_id_c), &token_id_c).await;
        });
    }

    let _user_msg_id = chat::save_user_message(
        &state.db,
        Some(&ring_id),
        &user.token_id,
        &user_row.display_name,
        &body.content,
        &body.node_refs,
        &body.tag_refs,
        body.ephemeral,
    )
    .await?;

    let tools = crate::services::chat::get_group_ring_tools();
    let pool_c = state.db.clone();
    let user_row_tool = user_row.clone();

    let mut rx = {
        let llm = crate::services::llm::LlmClient::from_user(&user_row)?;
        let system_prompt = chat::build_system_prompt(Some(&ring_info.0), ring_info.1.as_deref());
        let history =
            chat::load_history_context(&state.db, Some(&ring_id), &user.token_id, 20).await?;
        let filters = user_row
            .privacy_filters
            .as_deref()
            .map(crate::services::privacy_filter::PrivacyFilters::from_json)
            .unwrap_or_default();
        let filtered_content =
            crate::services::privacy_filter::apply_filters(&body.content, &filters);
        let pool_t = pool_c.clone();
        let user_t = user_row_tool.clone();
        llm.chat_stream_with_tools(
            system_prompt,
            history,
            filtered_content,
            "group_ring".to_string(),
            tools,
            move |name: String, args: serde_json::Value| {
                let pool = pool_t.clone();
                let user = user_t.clone();
                async move {
                    crate::services::chat::execute_group_tool(&pool, &user, name, args).await
                }
            },
        )
    };

    let pool = state.db.clone();
    let ring_id_c = ring_id.clone();
    let user_id = user.token_id.clone();
    let state_c = state.clone();
    let user_row_c = user_row.clone();
    let self_dir = crate::services::self_data::get_self_dir(&user_id);
    let content_len = body.content.len();
    let user_message = body.content.clone();
    let _ =
        crate::services::self_data::record_chat_message(&self_dir, Some(&ring_id_c), content_len);

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

                    let _ = message::insert_message(
                        &pool,
                        &message::NewMessage {
                            id: &message_id,
                            ring_id: Some(&ring_id_c),
                            user_id: &user_id,
                            role: "group_ring",
                            sender_name: "GROUP RING",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: token_usage.as_deref(),
                        },
                    ).await;

                    let ring_name_search = crate::services::search::get_ring_name(&pool, &ring_id_c).await.unwrap_or_default();
                    let _ = crate::services::search::upsert_search_index(
                        &pool, "message", &message_id, &ring_id_c, &ring_name_search,
                        "GROUP RING", &full_content,
                        &serde_json::json!({"role": "group_ring"}).to_string(),
                    ).await;

                    let state = state_c.clone();
                    let ring_id = ring_id_c.clone();
                    let user_id_for_doc = user_id.clone();
                    let user_row = user_row_c.clone();
                    tokio::spawn(async move {
                        let _ = crate::services::group_doc_maintenance::update_active_context(
                            &state, &ring_id, &user_id_for_doc, &user_row
                        ).await;
                    });

                    // Auto-archive check
                    let pool_auto = pool.clone();
                    let ring_id_auto = ring_id_c.clone();
                    let user_id_auto = user_id.clone();
                    let user_row_auto = user_row_c.clone();
                    let content_auto = full_content.clone();
                    let user_message_auto = user_message.clone();
                    let rings_dir_auto = state_c.rings_dir.clone();
                    tokio::spawn(async move {
                        let auto_archive: bool = sqlx::query_scalar(
                            "SELECT auto_archive FROM rings WHERE id = ?1",
                        )
                        .bind(&ring_id_auto)
                        .fetch_one(&pool_auto)
                        .await
                        .unwrap_or(false);

                        if auto_archive {
                            let git = crate::services::git_service::GitService::new();
                            crate::services::archive_service::auto_archive_chat(
                                &pool_auto,
                                &git,
                                &rings_dir_auto,
                                &ring_id_auto,
                                &user_message_auto,
                                &content_auto,
                                &user_id_auto,
                                &user_row_auto,
                            )
                            .await;
                        }
                    });
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

pub async fn ring_history(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let limit = query.limit + 1;
    let messages = chat::get_history(
        &state,
        Some(&ring_id),
        &user.token_id,
        query.before.as_deref(),
        limit,
    )
    .await?;

    let has_more = messages.len() > query.limit as usize;
    let messages = if has_more {
        messages.into_iter().take(query.limit as usize).collect()
    } else {
        messages
    };

    Ok(Json(HistoryResponse { messages, has_more }))
}

pub async fn self_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;

    let _ = chat::auto_compact_history(&state, &user_row, None, &user.token_id).await;

    let mut rx = chat::start_chat_stream(
        &state,
        &user_row,
        &ChatParams {
            ring_id: None,
            role_description: None,
            ring_name: None,
            ai_role: "self",
            content: &body.content,
            node_refs: body.node_refs,
            tag_refs: body.tag_refs,
            ephemeral: false,
        },
    )
    .await?;

    let pool = state.db.clone();
    let user_id = user.token_id.clone();
    let self_dir = crate::services::self_data::get_self_dir(&user_id);
    let content_len = body.content.len();
    let user_message = body.content.clone();
    let _ = crate::services::self_data::record_chat_message(&self_dir, None, content_len);
    let user_row_c = user_row.clone();

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

                    let _ = message::insert_message(
                        &pool,
                        &message::NewMessage {
                            id: &message_id,
                            ring_id: None,
                            user_id: &user_id,
                            role: "self",
                            sender_name: "SELF",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: token_usage.as_deref(),
                        },
                    ).await;

                    let extract_user_id = user_id.clone();
                    let extract_user_msg = user_message.clone();
                    let extract_ai_msg = full_content.clone();
                    let extract_user_row = user_row_c.clone();
                    tokio::spawn(async move {
                        let _ = crate::services::self_memory::extract_memories(
                            &extract_user_row,
                            &extract_user_id,
                            &extract_user_msg,
                            &extract_ai_msg,
                        ).await;
                        let _ = crate::services::self_memory::check_and_compress(
                            &extract_user_row,
                            &extract_user_id,
                        ).await;
                    });
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    };

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn self_history(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let limit = query.limit + 1;
    let messages =
        chat::get_history(&state, None, &user.token_id, query.before.as_deref(), limit).await?;

    let has_more = messages.len() > query.limit as usize;
    let messages = if has_more {
        messages.into_iter().take(query.limit as usize).collect()
    } else {
        messages
    };

    Ok(Json(HistoryResponse { messages, has_more }))
}
