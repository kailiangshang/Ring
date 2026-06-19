use crate::error::Result;
use crate::models::message::{self, MessageRow};
use crate::services::llm::{LlmClient, SseEvent};
use crate::services::privacy_filter::{apply_filters, PrivacyFilters};
use crate::state::AppState;

const COMPACT_THRESHOLD: usize = 30;
const COMPACT_SUMMARY_MAX_TOKENS: usize = 500;

pub fn detect_archive_intent(content: &str) -> bool {
    let lower = content.to_lowercase();
    let explicit_keywords = ["/save", "/archive", "\u{5f52}\u{6863}", "archive this"];
    explicit_keywords.iter().any(|kw| lower.contains(kw))
}

pub fn detect_graph_intent(content: &str) -> bool {
    let lower = content.to_lowercase();
    let explicit_keywords = [
        "save to graph",
        "archive to graph",
        "generate graph",
        "create graph",
        "add to graph",
        "attach to graph",
        "mount to graph",
        "\u{5b58}\u{5230}\u{56fe}\u{8c31}",
        "\u{8bb0}\u{5f55}\u{5230}\u{56fe}\u{8c31}",
        "\u{751f}\u{6210}\u{56fe}\u{8c31}",
        "\u{521b}\u{5efa}\u{56fe}\u{8c31}",
        "\u{6302}\u{8f7d}\u{5230}\u{56fe}\u{8c31}",
        "\u{4fdd}\u{5b58}\u{56fe}\u{8c31}",
        "\u{90a3}\u{4f60}\u{751f}\u{6210}\u{56fe}\u{8c31}\u{554a}",
        "\u{5e2e}\u{6211}\u{751f}\u{6210}\u{56fe}\u{8c31}\u{5427}",
        "\u{5e2e}\u{6211}\u{6302}\u{8f7d}\u{5230}\u{56fe}\u{8c31}\u{4e0a}\u{53bb}",
    ];
    if explicit_keywords.iter().any(|kw| lower.contains(kw)) {
        return true;
    }

    let trimmed = lower.trim();
    let shorthand_prompts = [
        "\u{56fe}\u{8c31}\u{5462}",
        "\u{77e5}\u{8bc6}\u{56fe}\u{8c31}\u{5462}",
        "graph?",
        "graph please",
    ];
    if shorthand_prompts.contains(&trimmed) {
        return true;
    }

    let action_markers = [
        "\u{751f}\u{6210}",
        "\u{521b}\u{5efa}",
        "\u{6574}\u{7406}",
        "\u{63d0}\u{53d6}",
        "\u{6302}\u{8f7d}",
        "\u{4fdd}\u{5b58}",
        "\u{8f6c}\u{6210}",
        "\u{53d8}\u{6210}",
        "generate",
        "create",
        "build",
        "attach",
        "save",
    ];
    let graph_markers = [
        "\u{56fe}\u{8c31}",
        "\u{77e5}\u{8bc6}\u{56fe}\u{8c31}",
        "graph",
    ];

    graph_markers
        .iter()
        .any(|graph_kw| trimmed.contains(graph_kw))
        && action_markers
            .iter()
            .any(|action_kw| trimmed.contains(action_kw))
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

    compact_messages(state, user, ring_id, user_id, &old_messages).await
}

pub struct CompactResult {
    pub summary: String,
    pub removed_count: usize,
}

pub async fn compact_history(
    state: &AppState,
    user: &crate::models::user::UserRow,
    ring_id: Option<&str>,
    user_id: &str,
) -> Result<CompactResult> {
    let messages = message::list_messages(&state.db, ring_id, user_id, None, 1000).await?;
    if messages.is_empty() {
        return Ok(CompactResult {
            summary: "No messages to compact.".into(),
            removed_count: 0,
        });
    }

    let old_messages: Vec<_> = if messages.len() <= 2 {
        return Ok(CompactResult {
            summary: "Too few messages to compact.".into(),
            removed_count: 0,
        });
    } else {
        messages.iter().rev().take(messages.len() - 2).collect()
    };

    let count = old_messages.len();
    let result = compact_messages(state, user, ring_id, user_id, &old_messages).await?;

    match result {
        Some(summary) => Ok(CompactResult {
            summary,
            removed_count: count,
        }),
        None => Ok(CompactResult {
            summary: "No messages to compact.".into(),
            removed_count: 0,
        }),
    }
}

async fn compact_messages(
    state: &AppState,
    user: &crate::models::user::UserRow,
    ring_id: Option<&str>,
    user_id: &str,
    old_messages: &[&MessageRow],
) -> Result<Option<String>> {
    if old_messages.is_empty() {
        return Ok(None);
    }

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
            content: &format!("[鍘嗗彶鎽樿] {}", summary),
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
            &format!("[鍘嗗彶鎽樿] {}", summary),
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

pub async fn build_system_prompt(
    pool: &sqlx::SqlitePool,
    ring_name: Option<&str>,
    role_description: Option<&str>,
    user_id: &str,
) -> String {
    let prompt = match ring_name {
        Some(name) => crate::prompts::group_ring::system(name, role_description),
        None => {
            let self_dir = crate::services::self_data::get_self_dir(user_id);
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
        let self_dir = crate::services::self_data::get_self_dir(user_id);
        let mut extra = String::new();
        let memory_ctx = crate::services::self_memory::build_memory_context(&self_dir).await;
        if !memory_ctx.is_empty() {
            extra.push_str(&memory_ctx);
        }
        let activity_ctx = build_recent_activity(pool, &self_dir, user_id).await;
        if !activity_ctx.is_empty() {
            if !extra.is_empty() {
                extra.push_str("\n\n");
            }
            extra.push_str(&activity_ctx);
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

async fn build_recent_activity(
    pool: &sqlx::SqlitePool,
    self_dir: &std::path::Path,
    user_id: &str,
) -> String {
    let metrics = crate::services::self_data::read_metrics(self_dir);
    let chat_patterns = match metrics.get("chat_patterns") {
        Some(v) if v.is_object() => v,
        _ => return String::new(),
    };
    let mut ring_entries: Vec<(String, i64)> = Vec::new();
    if let Some(obj) = chat_patterns.as_object() {
        for (key, val) in obj {
            if let Some(ring_id) = key.strip_prefix("ring_") {
                if let Some(count) = val.as_i64() {
                    ring_entries.push((ring_id.to_string(), count));
                }
            }
        }
    }
    if ring_entries.is_empty() {
        return String::new();
    }
    ring_entries.sort_by_key(|b| std::cmp::Reverse(b.1));
    let most_active_ring_id = &ring_entries[0].0;
    let ring_name = sqlx::query_scalar::<_, String>("SELECT name FROM rings WHERE id = ?1")
        .bind(most_active_ring_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    if ring_name.is_empty() {
        return String::new();
    }
    let messages =
        crate::models::message::list_messages(pool, Some(most_active_ring_id), user_id, None, 3)
            .await
            .unwrap_or_default();
    let summaries: Vec<String> = messages
        .into_iter()
        .rev()
        .filter(|m| m.role != "system")
        .take(3)
        .map(|m| {
            let sender = if m.role == "user" { "鐢ㄦ埛" } else { "AI" };
            let content = if m.content.len() > 100 {
                let s: String = m.content.chars().take(100).collect();
                format!("{s}...")
            } else {
                m.content.clone()
            };
            format!("- {sender}: {content}")
        })
        .collect();
    if summaries.is_empty() {
        return format!(
            "<recent_activity>\n## 鏈€杩戞椿鍔╘n- 娲昏穬 Ring: {ring_name}\n</recent_activity>"
        );
    }
    let summaries_text = summaries.join("\n");
    format!(
        "<recent_activity>\n## 鏈€杩戞椿鍔╘n- 娲昏穬 Ring: {ring_name}\n- 鏈€杩戣璁?\n{summaries_text}\n</recent_activity>"
    )
}

async fn query_doc(pool: &sqlx::SqlitePool, ring_id: &str, doc_name: &str) -> Option<String> {
    sqlx::query_scalar("SELECT content FROM group_docs WHERE ring_id = ?1 AND doc_name = ?2")
        .bind(ring_id)
        .bind(doc_name)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

fn format_doc_ctx(results: [(Option<String>, &str); 3]) -> String {
    let mut ctx = String::new();
    for (content, name) in results {
        if let Some(c) = content {
            if !c.trim().is_empty() {
                ctx.push_str(&format!("### {name}\n{c}\n\n"));
            }
        }
    }
    ctx
}

pub async fn build_group_ring_prompt_with_docs(
    pool: &sqlx::SqlitePool,
    ring_name: &str,
    role_description: Option<&str>,
    ring_id: &str,
    rings_dir: Option<&std::path::Path>,
) -> String {
    let base = crate::prompts::group_ring::system(ring_name, role_description);

    let (role, conventions, active_ctx) = tokio::join!(
        query_doc(pool, ring_id, "role"),
        query_doc(pool, ring_id, "conventions"),
        query_doc(pool, ring_id, "active-context"),
    );
    let (archive_patterns, corrections, knowledge_summary) = tokio::join!(
        query_doc(pool, ring_id, "archive-patterns"),
        query_doc(pool, ring_id, "corrections"),
        query_doc(pool, ring_id, "knowledge-summary"),
    );

    let core_ctx = format_doc_ctx([
        (role, "role"),
        (conventions, "conventions"),
        (active_ctx, "active-context"),
    ]);
    let ext_ctx = format_doc_ctx([
        (archive_patterns, "archive-patterns"),
        (corrections, "corrections"),
        (knowledge_summary, "knowledge-summary"),
    ]);

    let mut extra = String::new();
    if !core_ctx.is_empty() {
        extra.push_str("## Group Context (Core)\n\n");
        extra.push_str(&core_ctx);
    }
    if !ext_ctx.is_empty() {
        extra.push_str("## Group Context (Extended)\n\n");
        extra.push_str(&ext_ctx);
    }

    if let Some(rd) = rings_dir {
        let attached = build_attached_docs_section(pool, rd, ring_id).await;
        if !attached.is_empty() {
            if !extra.is_empty() {
                extra.push_str("\n\n");
            }
            extra.push_str(&attached);
        }
    }

    if extra.is_empty() {
        base
    } else {
        format!("{base}\n\n{extra}")
    }
}

async fn build_attached_docs_section(
    pool: &sqlx::SqlitePool,
    rings_dir: &std::path::Path,
    ring_id: &str,
) -> String {
    let g = match crate::models::graph::ensure_default_graph(pool, ring_id).await {
        Ok(g) => g,
        Err(_) => return String::new(),
    };
    let nodes = match crate::models::graph::list_nodes(pool, &g.id).await {
        Ok(n) => n,
        Err(_) => return String::new(),
    };

    let mut sections = Vec::new();
    for node in &nodes {
        let doc_refs = crate::services::graph::get_node_doc_refs(&node.metadata);
        if doc_refs.is_empty() {
            continue;
        }
        let mut node_docs = String::new();
        for dr in doc_refs.iter().take(3) {
            if let Some(content) =
                crate::services::graph::resolve_doc_content(rings_dir, ring_id, dr)
            {
                node_docs.push_str(&format!(
                    "#### {} ({})\n{}\n\n",
                    dr.title, dr.doc_type, content
                ));
            }
        }
        if !node_docs.is_empty() {
            sections.push(format!("### Node: {}\n\n{}", node.label, node_docs));
        }
    }

    if sections.is_empty() {
        return String::new();
    }
    format!("<attached_docs>\n{}\n</attached_docs>", sections.join("\n"))
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

#[allow(clippy::too_many_arguments)]
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
    let system_prompt = if let (Some(name), Some(ring_id)) = (params.ring_name, params.ring_id) {
        build_group_ring_prompt_with_docs(
            &state.db,
            name,
            params.role_description,
            ring_id,
            Some(&state.rings_dir),
        )
        .await
    } else {
        build_system_prompt(
            &state.db,
            params.ring_name,
            params.role_description,
            &user.token_id,
        )
        .await
    };
    let history = if params.ephemeral {
        vec![]
    } else {
        load_history_context(&state.db, params.ring_id, &user.token_id, 20).await?
    };

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
        async_openai::types::ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "fetch_url".into(),
                description: Some("Fetch and extract text content from a web page URL. Use for research and gathering information from the web.".into()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "The HTTP/HTTPS URL to fetch" },
                        "focus": { "type": "string", "description": "Optional focus area to highlight in the extracted content" }
                    },
                    "required": ["url"]
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
        "fetch_url" => {
            let parsed: crate::services::workflow::FetchUrlArgs = serde_json::from_value(args)
                .map_err(|e| crate::error::RingError::BadRequest(e.to_string()))?;
            crate::services::workflow::execute_fetch_url(&parsed).await
        }
        _ => Err(crate::error::RingError::BadRequest(format!(
            "unknown tool: {tool_name}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_archive_intent_chinese() {
        assert!(detect_archive_intent("请把这段对话归档"));
        assert!(detect_archive_intent("归档这段内容"));
        assert!(detect_archive_intent("/save"));
        assert!(!detect_archive_intent("存到图谱"));
    }

    #[test]
    fn test_detect_archive_intent_english() {
        assert!(detect_archive_intent("archive this conversation"));
        assert!(detect_archive_intent("/archive"));
        assert!(!detect_archive_intent("save to graph"));
    }

    #[test]
    fn test_detect_graph_intent_keywords() {
        assert!(detect_graph_intent("save to graph"));
        assert!(detect_graph_intent("帮我挂载到图谱上去"));
        assert!(detect_graph_intent("那你生成图谱啊"));
        assert!(detect_graph_intent("保存图谱"));
        assert!(detect_graph_intent("图谱呢"));
        assert!(detect_graph_intent("把刚才讨论整理成知识图谱"));
        assert!(!detect_graph_intent("什么是图谱"));
        assert!(!detect_graph_intent("hello world"));
    }

    #[test]
    fn test_detect_archive_intent_negative() {
        assert!(!detect_archive_intent("hello world"));
        assert!(!detect_archive_intent("what is the weather today"));
        assert!(!detect_archive_intent("请保存这个文件"));
        assert!(!detect_archive_intent("记录一下会议纪要"));
        assert!(!detect_archive_intent("mark this as important"));
    }
}
