use async_stream::stream;
use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use serde::Deserialize;
use std::convert::Infallible;

use axum::http::StatusCode;

use crate::error::{Result, RingError};
use crate::extractors::auth::AuthUser;
use crate::models::message;
use crate::models::ring;
use crate::services::{
    chat::{self, ChatParams, CompactResult},
    llm::SseEvent,
};
use crate::state::AppState;

fn trim_code_fence(text: &str) -> String {
    let cleaned = text.trim();
    if cleaned.starts_with("```") {
        cleaned
            .lines()
            .skip(1)
            .take_while(|line| !line.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        cleaned.to_string()
    }
}

fn extract_tag_payload(text: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let start = text.find(&start_tag)?;
    let content_start = start + start_tag.len();
    let end = text[content_start..].find(&end_tag)? + content_start;
    Some(text[content_start..end].trim().to_string())
}

fn extract_balanced_json_object(text: &str) -> Option<String> {
    let mut start = None;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in text.char_indices() {
        if start.is_none() {
            if ch == '{' {
                start = Some(idx);
                depth = 1;
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let start_idx = start?;
                    return Some(text[start_idx..=idx].to_string());
                }
            }
            _ => {}
        }
    }

    None
}

fn extract_knowledge_extraction_json(text: &str) -> Option<String> {
    let cleaned = trim_code_fence(text);
    if let Some(tagged) = extract_tag_payload(&cleaned, "knowledge_extraction") {
        return Some(trim_code_fence(&tagged));
    }
    if serde_json::from_str::<serde_json::Value>(&cleaned).is_ok() {
        return Some(cleaned);
    }
    extract_balanced_json_object(&cleaned)
}

fn normalize_knowledge_extraction(parsed: &serde_json::Value) -> Option<serde_json::Value> {
    let concepts = parsed.get("concepts")?.as_array()?;
    let mut normalized_concepts = Vec::new();
    let mut concept_labels = Vec::new();
    for concept in concepts {
        if let Some(label) = concept.as_str() {
            let trimmed = label.trim();
            if trimmed.is_empty() {
                continue;
            }
            concept_labels.push(trimmed.to_string());
            normalized_concepts.push(serde_json::json!({
                "label": trimmed,
                "node_type": "topic",
                "tags": [],
            }));
            continue;
        }

        if let Some(obj) = concept.as_object() {
            let label = obj.get("label").and_then(|v| v.as_str())?.trim();
            if label.is_empty() {
                continue;
            }
            concept_labels.push(label.to_string());
            let node_type = obj
                .get("node_type")
                .and_then(|v| v.as_str())
                .unwrap_or("topic");
            let tags = obj
                .get("tags")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            normalized_concepts.push(serde_json::json!({
                "label": label,
                "node_type": node_type,
                "tags": tags,
            }));
        }
    }

    let relations = parsed
        .get("relations")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut normalized_relations = Vec::new();
    for relation in relations {
        if let Some(obj) = relation.as_object() {
            let from = obj
                .get("from")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("subject").and_then(|v| v.as_str()));
            let to = obj
                .get("to")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("object").and_then(|v| v.as_str()));
            let rel = obj
                .get("relation")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("predicate").and_then(|v| v.as_str()))
                .unwrap_or("related_to");

            if let (Some(from), Some(to)) = (from, to) {
                normalized_relations.push(serde_json::json!({
                    "from": from,
                    "to": to,
                    "relation": rel,
                }));
            }
        }
    }

    if !concept_labels.is_empty() {
        use std::collections::HashSet;

        let anchor = parsed
            .get("suggested_graph")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .and_then(|suggested| {
                concept_labels
                    .iter()
                    .find(|label| label.eq_ignore_ascii_case(suggested))
                    .cloned()
            })
            .unwrap_or_else(|| concept_labels[0].clone());

        let mut connected = HashSet::new();
        let mut existing_pairs = HashSet::new();
        for relation in &normalized_relations {
            if let Some(obj) = relation.as_object() {
                let from = obj.get("from").and_then(|v| v.as_str()).unwrap_or_default();
                let to = obj.get("to").and_then(|v| v.as_str()).unwrap_or_default();
                if !from.is_empty() {
                    connected.insert(from.to_string());
                }
                if !to.is_empty() {
                    connected.insert(to.to_string());
                }
                if !from.is_empty() && !to.is_empty() {
                    existing_pairs.insert((from.to_string(), to.to_string()));
                }
            }
        }

        let should_seed_backbone = normalized_relations.is_empty() && concept_labels.len() > 1;
        for label in &concept_labels {
            if label == &anchor {
                continue;
            }
            let needs_fallback = should_seed_backbone || !connected.contains(label);
            if !needs_fallback {
                continue;
            }
            let pair = (anchor.clone(), label.clone());
            if existing_pairs.contains(&pair) {
                continue;
            }
            normalized_relations.push(serde_json::json!({
                "from": anchor,
                "to": label,
                "relation": "related_to",
            }));
            existing_pairs.insert(pair);
        }

        if !connected.contains(&anchor) && concept_labels.len() > 1 {
            if let Some(first_neighbor) = concept_labels.iter().find(|label| *label != &anchor) {
                let pair = (anchor.clone(), first_neighbor.clone());
                if !existing_pairs.contains(&pair) {
                    normalized_relations.push(serde_json::json!({
                        "from": anchor,
                        "to": first_neighbor,
                        "relation": "related_to",
                    }));
                }
            }
        }
    }

    let mut normalized = serde_json::Map::new();
    normalized.insert(
        "concepts".into(),
        serde_json::Value::Array(normalized_concepts),
    );
    normalized.insert(
        "relations".into(),
        serde_json::Value::Array(normalized_relations),
    );
    if let Some(summary) = parsed.get("summary") {
        normalized.insert("summary".into(), summary.clone());
    }
    if let Some(suggested_graph) = parsed.get("suggested_graph") {
        normalized.insert("suggested_graph".into(), suggested_graph.clone());
    }
    Some(serde_json::Value::Object(normalized))
}

fn build_graph_preview_message(extraction_json: &str) -> String {
    format!(
        "Graph intent detected. Review the extracted concepts below and confirm before creating graph nodes.\n\n<graph_action>{{\"intent\":\"confirm_create_graph\"}}</graph_action>\n<knowledge_extraction>{}</knowledge_extraction>",
        extraction_json
    )
}

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
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    ring::reject_readonly(&role)?;

    let ring_info = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT name, role_description FROM rings WHERE id = ?1",
    )
    .bind(&ring_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| RingError::NotFound("ring not found".into()))?;

    let graph_intent = chat::detect_graph_intent(&body.content);
    let archive_intent = chat::detect_archive_intent(&body.content);
    tracing::info!(
        ring_id = %ring_id,
        user_id = %user.token_id,
        content = %body.content,
        graph_intent,
        archive_intent,
        "ring_chat intent detection"
    );

    if graph_intent {
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

        let history =
            chat::load_history_context(&state.db, Some(&ring_id), &user.token_id, 100).await?;
        let transcript = history
            .into_iter()
            .filter(|(role, content)| {
                role == "user"
                    && !chat::detect_graph_intent(content)
                    && !chat::detect_archive_intent(content)
                    && content.trim().chars().count() >= 12
            })
            .map(|(_, content)| content)
            .collect::<Vec<_>>()
            .join("\n\n");

        let preview_content = if transcript.trim().is_empty() {
            "I could not find prior discussion content to extract into a graph.".to_string()
        } else {
            match crate::services::workflow::execute_knowledge_extract(
                &user_row,
                &crate::services::workflow::KnowledgeExtractArgs {
                    content: transcript,
                    target_graph: None,
                },
            )
            .await
            {
                Ok(extraction) => {
                    match extract_knowledge_extraction_json(&extraction)
                        .and_then(|payload| serde_json::from_str::<serde_json::Value>(&payload).ok())
                    {
                        Some(parsed) => {
                            let normalized = normalize_knowledge_extraction(&parsed);
                            let has_concepts = normalized
                                .as_ref()
                                .and_then(|value| value.get("concepts"))
                                .and_then(|v| v.as_array())
                                .map(|v| !v.is_empty())
                                .unwrap_or(false);
                            if has_concepts {
                                build_graph_preview_message(
                                    &normalized.unwrap().to_string(),
                                )
                            } else {
                                "I did not find enough structured concepts to create a useful graph from the recent discussion.".to_string()
                            }
                        }
                        None => {
                            "I could not parse a graph extraction preview from the recent discussion.".to_string()
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to extract graph preview: {e}");
                    "I could not build a graph preview from the recent discussion.".to_string()
                }
            }
        };

        let pool = state.db.clone();
        let ring_id_c = ring_id.clone();
        let user_id_c = user.token_id.clone();
        let s = stream! {
            let message_id = ulid::Ulid::new().to_string();
            yield Ok(Event::default().event("message_start").data(serde_json::json!({"message_id": message_id, "role": "group_ring"}).to_string()));
            yield Ok(Event::default().event("delta").data(serde_json::json!({"content": preview_content.clone()}).to_string()));
            yield Ok(Event::default().event("message_end").data(serde_json::json!({"message_id": message_id, "usage": {"prompt_tokens": 0, "completion_tokens": 0}}).to_string()));

            if let Err(e) = message::insert_message(
                &pool,
                &message::NewMessage {
                    id: &message_id,
                    ring_id: Some(&ring_id_c),
                    user_id: &user_id_c,
                    role: "group_ring",
                    sender_name: "GROUP RING",
                    content: &preview_content,
                    node_refs: &[],
                    tag_refs: &[],
                    token_usage: None,
                },
            ).await {
                tracing::warn!("failed to insert graph preview message: {e}");
            }
        }.boxed();

        return Ok(Sse::new(s).keep_alive(KeepAlive::default()));
    }

    if archive_intent {
        let ring_id_c = ring_id.clone();
        let user_id_c = user.token_id.clone();
        let state_c = state.clone();
        let content_c = body.content.clone();
        let self_dir_c = crate::services::self_data::get_self_dir(&user.token_id);
        let backend_c = crate::services::archive_service::get_backend(
            &state.db,
            &ring_id,
            None,
            Some(&state.encryption),
        )
        .await?;
        let s = stream! {
            let message_id = ulid::Ulid::new().to_string();
            yield Ok(Event::default().event("message_start").data(serde_json::json!({"message_id": message_id, "role": "group_ring"}).to_string()));
            yield Ok(Event::default().event("delta").data(serde_json::json!({"content": "检测到归档意图，正在启动归档流程..."}).to_string()));
            yield Ok(Event::default().event("message_end").data(serde_json::json!({"message_id": message_id, "usage": {"prompt_tokens": 0, "completion_tokens": 0}}).to_string()));
            if let Err(e) = crate::services::archive_service::quick_archive(
                &state_c, backend_c.as_ref(), &ring_id_c, &user_id_c, &content_c,
            ).await {
                tracing::warn!("failed to quick archive: {e}");
            }
            if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir_c, "archive") {
                tracing::warn!("failed to record tool usage: {e}");
            }
        }.boxed();
        return Ok(Sse::new(s).keep_alive(KeepAlive::default()));
    }

    {
        let state_c = state.clone();
        let user_row_c = user_row.clone();
        let ring_id_c = ring_id.clone();
        let token_id_c = user.token_id.clone();
        tokio::spawn(async move {
            let _ =
                chat::auto_compact_history(&state_c, &user_row_c, Some(&ring_id_c), &token_id_c)
                    .await;
        });
    }

    let tools = crate::services::chat::get_group_ring_tools();
    let pool_c = state.db.clone();
    let user_row_tool = user_row.clone();

    let mut rx = {
        let llm = crate::services::llm::LlmClient::from_user(&user_row)?;
        let system_prompt = chat::build_group_ring_prompt_with_docs(
            &state.db,
            &ring_info.0,
            ring_info.1.as_deref(),
            &ring_id,
            Some(&state.rings_dir),
        )
        .await;
        let history =
            chat::load_history_context(&state.db, Some(&ring_id), &user.token_id, 20).await?;

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
                Box::pin(async move {
                    crate::services::chat::execute_group_tool(&pool, &user, name, args).await
                })
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
    if let Err(e) =
        crate::services::self_data::record_chat_message(&self_dir, Some(&ring_id_c), content_len)
    {
        tracing::warn!("failed to record chat message: {e}");
    }

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
                            role: "group_ring",
                            sender_name: "GROUP RING",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: token_usage.as_deref(),
                        },
                    ).await {
                        tracing::warn!("failed to insert message: {e}");
                    }

                    let ring_name_search = crate::services::search::get_ring_name(&pool, &ring_id_c).await.unwrap_or_default();
                    if let Err(e) = crate::services::search::upsert_search_index(
                        &pool, "message", &message_id, &ring_id_c, &ring_name_search,
                        "GROUP RING", &full_content,
                        &serde_json::json!({"role": "group_ring"}).to_string(),
                    ).await {
                        tracing::warn!("failed to update search index: {e}");
                    }

                    let state = state_c.clone();
                    let ring_id = ring_id_c.clone();
                    let user_id_for_doc = user_id.clone();
                    let user_row = user_row_c.clone();
                    tokio::spawn(async move {
                        if let Err(e) = crate::services::group_doc_maintenance::update_active_context(
                            &state, &ring_id, &user_id_for_doc, &user_row
                        ).await {
                            tracing::warn!("failed to update active context: {e}");
                        }
                    });

                    // Auto-archive check
                    let pool_auto = pool.clone();
                    let ring_id_auto = ring_id_c.clone();
                    let user_id_auto = user_id.clone();
                    let user_row_auto = user_row_c.clone();
                    let content_auto = full_content.clone();
                    let user_message_auto = user_message.clone();
                    let rings_dir_auto = state_c.rings_dir.clone();
                    let backend_auto = match crate::services::archive_service::get_backend(&state_c.db, &ring_id_auto, Some(&user_row_c), Some(&state_c.encryption)).await {
                        Ok(b) => b,
                        Err(e) => {
                            tracing::warn!("auto_archive_chat: failed to get backend: {e}");
                            return;
                        }
                    };
                    tokio::spawn(async move {
                        let auto_archive: bool = sqlx::query_scalar(
                            "SELECT auto_archive FROM rings WHERE id = ?1",
                        )
                        .bind(&ring_id_auto)
                        .fetch_one(&pool_auto)
                        .await
                        .unwrap_or(false);

                        if auto_archive {
                            crate::services::archive_service::auto_archive_chat(
                                &pool_auto,
                                backend_auto,
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
    let request_start = std::time::Instant::now();
    tracing::info!(
        "self_chat route: request started at {:?}",
        request_start.elapsed().as_secs_f64()
    );

    let user_row = state.get_user_decrypted(&user.token_id).await?;
    tracing::info!(
        "self_chat route: user decrypted at {:?}",
        request_start.elapsed().as_secs_f64()
    );

    {
        let state_c = state.clone();
        let user_row_c = user_row.clone();
        let token_id_c = user.token_id.clone();
        tokio::spawn(async move {
            let _ = chat::auto_compact_history(&state_c, &user_row_c, None, &token_id_c).await;
        });
    }
    tracing::info!(
        "self_chat route: auto_compact_history spawned at {:?}",
        request_start.elapsed().as_secs_f64()
    );

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
    if let Err(e) = crate::services::self_data::record_chat_message(&self_dir, None, content_len) {
        tracing::warn!("failed to record chat message: {e}");
    }
    let user_row_c = user_row.clone();

    let s = stream! {
        tracing::info!("self_chat route: stream consumer started at {:?}", request_start.elapsed().as_secs_f64());
        let mut event_count = 0;
        while let Some(event) = rx.recv().await {
            event_count += 1;
            if event_count == 1 {
                tracing::info!("self_chat route: first event received at {:?}", request_start.elapsed().as_secs_f64());
            }
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
                    tracing::info!("self_chat route: End event received at {:?}, total_events={}", request_start.elapsed().as_secs_f64(), event_count);
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
                            ring_id: None,
                            user_id: &user_id,
                            role: "self",
                            sender_name: "SELF",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: token_usage.as_deref(),
                        },
                    ).await {
                        tracing::warn!("failed to insert message: {e}");
                    }

                    let extract_user_id = user_id.clone();
                    let extract_user_msg = user_message.clone();
                    let extract_ai_msg = full_content.clone();
                    let extract_user_row = user_row_c.clone();
                    tokio::spawn(async move {
                        if let Err(e) = crate::services::self_memory::extract_memories(
                            &extract_user_row,
                            &extract_user_id,
                            &extract_user_msg,
                            &extract_ai_msg,
                        ).await {
                            tracing::warn!("failed to extract memories: {e}");
                        }
                        if let Err(e) = crate::services::self_memory::check_and_compress(
                            &extract_user_row,
                            &extract_user_id,
                        ).await {
                            tracing::warn!("failed to compress memories: {e}");
                        }
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

pub async fn delete_ring_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path((_ring_id, message_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    delete_message_for_user(&state, &user.token_id, &message_id).await
}

pub async fn delete_self_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(message_id): Path<String>,
) -> Result<StatusCode> {
    delete_message_for_user(&state, &user.token_id, &message_id).await
}

pub async fn delete_super_message(
    State(state): State<AppState>,
    user: AuthUser,
    Path(message_id): Path<String>,
) -> Result<StatusCode> {
    delete_message_for_user(&state, &user.token_id, &message_id).await
}

async fn delete_message_for_user(
    state: &AppState,
    user_id: &str,
    message_id: &str,
) -> Result<StatusCode> {
    let msg = message::get_message(&state.db, message_id)
        .await?
        .ok_or_else(|| RingError::NotFound("message not found".into()))?;
    if msg.role != "system" && msg.user_id != user_id {
        return Err(RingError::Forbidden(
            "can only delete your own messages".into(),
        ));
    }
    message::delete_message(&state.db, message_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Serialize)]
pub struct CompactResponse {
    pub summary: String,
    pub removed_count: usize,
}

pub async fn ring_compact(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<CompactResponse>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    ring::reject_readonly(&role)?;

    let CompactResult {
        summary,
        removed_count,
    } = chat::compact_history(&state, &user_row, Some(&ring_id), &user.token_id).await?;

    Ok(Json(CompactResponse {
        summary,
        removed_count,
    }))
}

pub async fn self_compact(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<CompactResponse>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;

    let CompactResult {
        summary,
        removed_count,
    } = chat::compact_history(&state, &user_row, None, &user.token_id).await?;

    Ok(Json(CompactResponse {
        summary,
        removed_count,
    }))
}

#[cfg(test)]
mod tests {
    use super::normalize_knowledge_extraction;

    #[test]
    fn normalize_extraction_adds_backbone_relations_when_missing() {
        let parsed = serde_json::json!({
            "concepts": [
                {"label": "微服务架构"},
                {"label": "API网关"},
                {"label": "订单服务"}
            ],
            "relations": [],
            "suggested_graph": "微服务架构"
        });

        let normalized = normalize_knowledge_extraction(&parsed).expect("normalized");
        let relations = normalized["relations"].as_array().expect("relations");
        assert_eq!(relations.len(), 2);
        assert!(relations
            .iter()
            .any(|rel| rel["from"] == "微服务架构" && rel["to"] == "API网关"));
        assert!(relations
            .iter()
            .any(|rel| rel["from"] == "微服务架构" && rel["to"] == "订单服务"));
    }

    #[test]
    fn normalize_extraction_connects_orphan_concepts() {
        let parsed = serde_json::json!({
            "concepts": [
                {"label": "微服务架构"},
                {"label": "API网关"},
                {"label": "订单服务"}
            ],
            "relations": [
                {"from": "API网关", "to": "订单服务", "relation": "路由到"}
            ],
            "suggested_graph": "微服务架构"
        });

        let normalized = normalize_knowledge_extraction(&parsed).expect("normalized");
        let relations = normalized["relations"].as_array().expect("relations");
        assert_eq!(relations.len(), 2);
        assert!(relations
            .iter()
            .any(|rel| rel["from"] == "API网关" && rel["to"] == "订单服务"));
        assert!(relations
            .iter()
            .any(|rel| rel["from"] == "微服务架构" && rel["to"] == "API网关"
                || rel["from"] == "微服务架构" && rel["to"] == "订单服务"));
    }
}
