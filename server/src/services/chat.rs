use crate::error::Result;
use crate::models::message::{self, MessageRow};
use crate::services::llm::{LlmClient, SseEvent};
use crate::services::privacy_filter::{apply_filters, PrivacyFilters};
use crate::state::AppState;

const COMPACT_THRESHOLD: usize = 30;
const COMPACT_SUMMARY_MAX_TOKENS: usize = 500;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_archive_intent_chinese() {
        assert!(detect_archive_intent("请把这段对话归档"));
        assert!(detect_archive_intent("保存到图谱"));
        assert!(detect_archive_intent("记录到图谱"));
        assert!(detect_archive_intent("值得归档"));
        assert!(detect_archive_intent("mark this"));
    }

    #[test]
    fn test_detect_archive_intent_english() {
        assert!(detect_archive_intent("archive this conversation"));
        assert!(detect_archive_intent("save to graph"));
        assert!(detect_archive_intent("save this for later"));
    }

    #[test]
    fn test_detect_archive_intent_negative() {
        assert!(!detect_archive_intent("hello world"));
        assert!(!detect_archive_intent("what is the weather today"));
    }

    #[test]
    fn test_should_recommend_archive_with_indicators() {
        assert!(should_recommend_archive("我们达成了共识，结论是采用方案A"));
        assert!(should_recommend_archive(
            "The team decided to go with option B"
        ));
        assert!(should_recommend_archive("Final conclusion: use Rust"));
    }

    #[test]
    fn test_should_recommend_archive_negative() {
        assert!(!should_recommend_archive("hello"));
        assert!(!should_recommend_archive("what do you think"));
    }
}

pub fn detect_archive_intent(content: &str) -> bool {
    let lower = content.to_lowercase();
    let keywords = [
        "归档",
        "保存",
        "记录到图谱",
        "archive",
        "save to graph",
        "值得归档",
        "mark this",
        "save this",
        "archive this",
        "记录一下",
        "记下来",
        "存到图谱",
    ];
    keywords.iter().any(|kw| lower.contains(kw))
}

pub fn should_recommend_archive(content: &str) -> bool {
    let lower = content.to_lowercase();
    let indicators = [
        "结论",
        "总结",
        "决策",
        "方案",
        "决定",
        "agreed",
        "decided",
        "conclusion",
        "resolved",
        "solution",
        "finalized",
        "确定",
        "共识",
        "一致同意",
    ];
    indicators.iter().any(|ind| lower.contains(ind))
}

pub async fn get_history(
    state: &AppState,
    ring_id: Option<&str>,
    user_id: &str,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    let messages = message::list_messages(&state.db, ring_id, user_id, before_id, limit).await?;
    Ok(messages.into_iter().rev().collect())
}

pub async fn auto_compact_history(
    state: &AppState,
    user: &crate::models::user::UserRow,
    ring_id: Option<&str>,
    user_id: &str,
) -> Result<Option<String>> {
    let messages = message::list_messages(&state.db, ring_id, user_id, None, 1000).await?;
    if messages.len() < COMPACT_THRESHOLD {
        return Ok(None);
    }

    let old_messages: Vec<_> = messages.iter().rev().take(messages.len() - 10).collect();

    let history_text: String = old_messages
        .iter()
        .map(|m| format!("{}: {}", m.sender_name, m.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = crate::prompts::compact::user(&history_text, COMPACT_SUMMARY_MAX_TOKENS as i64);

    let llm = LlmClient::from_user(user)?;
    let summary = llm
        .chat_complete(crate::prompts::compact::SYSTEM.into(), prompt)
        .await?;

    let summary_id = ulid::Ulid::new().to_string();
    message::insert_message(
        &state.db,
        &message::NewMessage {
            id: &summary_id,
            ring_id,
            user_id,
            role: "system",
            sender_name: "SYSTEM",
            content: &format!("[历史摘要] {}", summary),
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await?;

    if let Some(ring_id) = ring_id {
        let ring_name = crate::services::search::get_ring_name(&state.db, ring_id)
            .await
            .unwrap_or_default();
        if let Err(e) = crate::services::search::upsert_search_index(
            &state.db,
            "message",
            &summary_id,
            ring_id,
            &ring_name,
            "SYSTEM",
            &format!("[历史摘要] {}", summary),
            &serde_json::json!({"role": "system"}).to_string(),
        )
        .await
        {
            tracing::warn!("failed to update search index: {e}");
        }
    }

    for msg in old_messages {
        let _ = sqlx::query("DELETE FROM messages WHERE id = ?1")
            .bind(&msg.id)
            .execute(&state.db)
            .await;
    }

    Ok(Some(summary))
}

pub fn build_system_prompt(ring_name: Option<&str>, role_description: Option<&str>) -> String {
    let prompt = match ring_name {
        Some(name) => crate::prompts::group_ring::system(name, role_description),
        None => {
            let self_dir = crate::services::self_data::get_self_dir("");
            let (identity, identity_exists) =
                crate::services::self_data::read_self_file(&self_dir, "identity")
                    .unwrap_or_default();
            let identity = if identity_exists && !identity.is_empty() {
                Some(identity.as_str())
            } else {
                None
            };

            let (style, style_exists) =
                crate::services::self_data::read_self_file(&self_dir, "style").unwrap_or_default();
            let style = if style_exists && !style.is_empty() {
                Some(style.as_str())
            } else {
                None
            };

            let personality = crate::services::self_data::read_self_file(&self_dir, "personality")
                .unwrap_or_default();
            let tone = if personality.1 && !personality.0.is_empty() {
                serde_json::from_str::<serde_json::Value>(&personality.0)
                    .ok()
                    .and_then(|p| {
                        p.get("tone")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
            } else {
                None
            };

            crate::prompts::self_chat::system(identity, style, tone.as_deref())
        }
    };
    if ring_name.is_none() {
        let self_dir = crate::services::self_data::get_self_dir("");
        let mut extra = String::new();
        let memory_ctx = crate::services::self_memory::build_memory_context(&self_dir);
        if !memory_ctx.is_empty() {
            extra.push_str(&memory_ctx);
        }
        let metrics = crate::services::self_data::read_metrics(&self_dir);
        let metrics_ctx = crate::prompts::self_chat::metrics_context(&metrics);
        if !metrics_ctx.is_empty() {
            if !extra.is_empty() {
                extra.push_str("\n\n");
            }
            extra.push_str(&metrics_ctx);
        }
        if !extra.is_empty() {
            return format!("{prompt}\n\n{extra}");
        }
    }
    prompt
}

pub async fn load_history_context(
    pool: &sqlx::SqlitePool,
    ring_id: Option<&str>,
    user_id: &str,
    limit: i64,
) -> Result<Vec<(String, String)>> {
    let messages = message::list_messages(pool, ring_id, user_id, None, limit).await?;
    Ok(messages
        .into_iter()
        .rev()
        .filter(|m| m.role != "system")
        .map(|m| (m.role.clone(), m.content.clone()))
        .collect())
}

pub struct ChatParams<'a> {
    pub ring_id: Option<&'a str>,
    pub role_description: Option<&'a str>,
    pub ring_name: Option<&'a str>,
    pub ai_role: &'a str,
    pub content: &'a str,
    pub node_refs: Vec<String>,
    pub tag_refs: Vec<String>,
    pub ephemeral: bool,
}

pub async fn save_user_message(
    pool: &sqlx::SqlitePool,
    ring_id: Option<&str>,
    user_id: &str,
    display_name: &str,
    content: &str,
    node_refs: &[String],
    tag_refs: &[String],
    ephemeral: bool,
) -> Result<String> {
    let msg_id = ulid::Ulid::new().to_string();
    if !ephemeral {
        message::insert_message(
            pool,
            &message::NewMessage {
                id: &msg_id,
                ring_id,
                user_id,
                role: "user",
                sender_name: display_name,
                content,
                node_refs,
                tag_refs,
                token_usage: None,
            },
        )
        .await?;
    }

    if let Some(rid) = ring_id {
        let ring_name = crate::services::search::get_ring_name(pool, rid)
            .await
            .unwrap_or_default();
        if let Err(e) = crate::services::search::upsert_search_index(
            pool,
            "message",
            &msg_id,
            rid,
            &ring_name,
            display_name,
            content,
            &serde_json::json!({"role": "user"}).to_string(),
        )
        .await
        {
            tracing::warn!("failed to update search index: {e}");
        }
    }

    Ok(msg_id)
}

pub async fn start_chat_stream(
    state: &AppState,
    user: &crate::models::user::UserRow,
    params: &ChatParams<'_>,
) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
    let _user_msg_id = save_user_message(
        &state.db,
        params.ring_id,
        &user.token_id,
        &user.display_name,
        params.content,
        &params.node_refs,
        &params.tag_refs,
        params.ephemeral,
    )
    .await?;

    let system_prompt = build_system_prompt(params.ring_name, params.role_description);
    let history = if params.ephemeral {
        vec![]
    } else {
        load_history_context(&state.db, params.ring_id, &user.token_id, 20).await?
    };

    let filters = user
        .privacy_filters
        .as_deref()
        .map(PrivacyFilters::from_json)
        .unwrap_or_default();
    let filtered_content = apply_filters(params.content, &filters);

    let llm = LlmClient::from_user(user)?;
    let rx = llm.chat_stream(
        system_prompt,
        history,
        filtered_content,
        params.ai_role.to_string(),
    );
    Ok(rx)
}

pub fn get_group_ring_tools() -> Vec<async_openai::types::ChatCompletionTool> {
    vec![
        async_openai::types::ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "file_parse".into(),
                description: Some("Parse an uploaded file and extract structured knowledge. Recommend graph nodes.".into()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_reference": { "type": "string", "description": "The message_id of the file upload message" },
                        "focus": { "type": "string", "description": "Optional focus area for extraction" }
                    },
                    "required": ["file_reference"]
                })),
                strict: Some(true),
            },
        },
        async_openai::types::ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "knowledge_extract".into(),
                description: Some("Extract knowledge concepts from text and generate graph node recommendations.".into()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "Text or topic to extract knowledge from" },
                        "target_graph": { "type": "string", "description": "Optional target graph name" }
                    },
                    "required": ["content"]
                })),
                strict: Some(true),
            },
        },
    ]
}

pub async fn execute_group_tool(
    pool: &sqlx::SqlitePool,
    user: &crate::models::user::UserRow,
    tool_name: String,
    args: serde_json::Value,
) -> crate::error::Result<String> {
    match tool_name.as_str() {
        "file_parse" => {
            let parsed: crate::services::workflow::FileParseArgs = serde_json::from_value(args)
                .map_err(|e| crate::error::RingError::BadRequest(e.to_string()))?;
            crate::services::workflow::execute_file_parse(pool, user, &parsed).await
        }
        "knowledge_extract" => {
            let parsed: crate::services::workflow::KnowledgeExtractArgs =
                serde_json::from_value(args)
                    .map_err(|e| crate::error::RingError::BadRequest(e.to_string()))?;
            crate::services::workflow::execute_knowledge_extract(user, &parsed).await
        }
        _ => Err(crate::error::RingError::BadRequest(format!(
            "unknown tool: {tool_name}"
        ))),
    }
}
