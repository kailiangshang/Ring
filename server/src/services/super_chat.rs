use std::path::Path;

use async_openai::types::{ChatCompletionTool, ChatCompletionToolType, FunctionObject};
use serde::Deserialize;

use crate::error::{Result, RingError};
use crate::models::message::{self, MessageRow};
use crate::services::chat;
use crate::services::llm::{ChatCompleteWithToolsResult, LlmClient};
use crate::state::AppState;

const SUPER_RING_ID: &str = "super";

const DEFAULT_SUPER_SYSTEM_PROMPT: &str = "你是 Super Ring，用户的全局 AI 助手和跨 Ring 协调者。\n\n你的职责：\n1. Ring 管理引导 — 帮助用户创建、配置 Ring\n2. 跨 Ring 分析 — 按需读取所有 Ring 的内容，进行汇总、对比、推荐\n3. 使用引导 — 回答关于 Ring 产品功能的问题\n\n请用简洁、专业的方式回答。";

pub fn get_system_prompt(hub_dir: &Path) -> String {
    let prompt_file = hub_dir.join("system_prompt.md");
    match std::fs::read_to_string(&prompt_file) {
        Ok(content) if !content.trim().is_empty() => content,
        _ => DEFAULT_SUPER_SYSTEM_PROMPT.to_string(),
    }
}

pub fn get_system_prompt_info(hub_dir: &Path) -> (String, bool) {
    let prompt_file = hub_dir.join("system_prompt.md");
    match std::fs::read_to_string(&prompt_file) {
        Ok(ref content) if !content.trim().is_empty() => (content.clone(), true),
        _ => (DEFAULT_SUPER_SYSTEM_PROMPT.to_string(), false),
    }
}

pub fn update_system_prompt(hub_dir: &Path, prompt: &str) -> Result<()> {
    let prompt_file = hub_dir.join("system_prompt.md");
    if prompt.trim().is_empty() {
        let _ = std::fs::remove_file(&prompt_file);
    } else {
        std::fs::write(&prompt_file, prompt)?;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct QueryRingDetailArgs {
    ring_name: String,
}

pub fn get_super_tools() -> Vec<ChatCompletionTool> {
    vec![
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: "query_rings".to_string(),
                description: Some(
                    "列出用户所有 Ring 的摘要信息，包括名称、成员数和最近归档标题。当用户询问关于 Ring 的概况时使用。"
                        .to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: "query_ring_detail".to_string(),
                description: Some(
                    "读取指定 Ring 的详细数据，包括图谱节点和最近归档内容。当用户想了解某个 Ring 的具体内容时使用。"
                        .to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "ring_name": {
                                "type": "string",
                                "description": "要查询的 Ring 名称"
                            }
                        },
                        "required": ["ring_name"]
                    }),
                ),
                strict: None,
            },
        },
    ]
}

pub async fn build_ring_summary(pool: &sqlx::SqlitePool, user_id: &str) -> String {
    let rings = sqlx::query_as::<_, (String, String)>(
        "SELECT r.id, r.name FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         ORDER BY r.created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if rings.is_empty() {
        return "用户目前没有任何 Ring。".to_string();
    }

    let mut summary = String::from("## 用户的所有 Ring\n\n");

    for (ring_id, ring_name) in &rings {
        let member_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM members WHERE ring_id = ?1",
        )
        .bind(ring_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let archive_titles: Vec<String> = sqlx::query_scalar(
            "SELECT title FROM archive_records
             WHERE ring_id = ?1 AND status IN ('pushed', 'committed')
             ORDER BY created_at DESC LIMIT 3",
        )
        .bind(ring_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        summary.push_str(&format!("### {ring_name} ({member_count} 成员)\n"));
        if archive_titles.is_empty() {
            summary.push_str("- 暂无归档\n\n");
        } else {
            summary.push_str(&format!("- 最近归档: {}\n\n", archive_titles.join(", ")));
        }
    }

    summary
}

pub async fn execute_tool(
    pool: &sqlx::SqlitePool,
    rings_dir: &Path,
    user_id: &str,
    tool_name: &str,
    arguments: &str,
) -> Result<String> {
    match tool_name {
        "query_rings" => execute_query_rings(pool, user_id).await,
        "query_ring_detail" => {
            let args: QueryRingDetailArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            execute_query_ring_detail(pool, rings_dir, user_id, &args.ring_name).await
        }
        _ => Err(RingError::BadRequest(format!("unknown tool: {tool_name}"))),
    }
}

async fn execute_query_rings(pool: &sqlx::SqlitePool, user_id: &str) -> Result<String> {
    Ok(build_ring_summary(pool, user_id).await)
}

async fn execute_query_ring_detail(
    pool: &sqlx::SqlitePool,
    rings_dir: &Path,
    user_id: &str,
    ring_name: &str,
) -> Result<String> {
    let ring_id: Option<String> = sqlx::query_scalar(
        "SELECT r.id FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         WHERE r.name LIKE ?2",
    )
    .bind(user_id)
    .bind(format!("%{ring_name}%"))
    .fetch_optional(pool)
    .await?
    .flatten();

    let ring_id = match ring_id {
        Some(id) => id,
        None => return Ok(format!("未找到名为「{ring_name}」的 Ring。")),
    };

    let mut result = String::new();

    let graph_path = rings_dir.join(&ring_id).join("graph.json");
    if graph_path.exists() {
        match std::fs::read_to_string(&graph_path) {
            Ok(content) => {
                if let Ok(graph) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(nodes) = graph.get("nodes").and_then(|n| n.as_array()) {
                        result.push_str(&format!(
                            "### 图谱节点（共 {} 个，显示前 50 个）\n",
                            nodes.len()
                        ));
                        for node in nodes.iter().take(50) {
                            let label = node
                                .get("label")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            let desc = node
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if desc.is_empty() {
                                result.push_str(&format!("- {label}\n"));
                            } else {
                                result.push_str(&format!("- {label}: {desc}\n"));
                            }
                        }
                        result.push('\n');
                    }
                }
            }
            Err(e) => {
                tracing::warn!("failed to read graph.json: {e}");
            }
        }
    }

    let archives_dir = rings_dir.join(&ring_id).join("archives");
    if archives_dir.exists() {
        let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(&archives_dir)
            .map(|rd| rd.filter_map(|e| e.ok()).collect())
            .unwrap_or_default();
        entries.sort_by_key(|e| {
            std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok()))
        });

        result.push_str("### 最近归档\n\n");
        for entry in entries.iter().take(3) {
            if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".md") {
                    match std::fs::read_to_string(entry.path()) {
                        Ok(content) => {
                            let truncated = if content.len() > 500 {
                                format!("{}...（截断）", &content[..500])
                            } else {
                                content
                            };
                            result.push_str(&format!("#### {name}\n{truncated}\n\n"));
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }

    if result.is_empty() {
        Ok(format!("Ring「{ring_name}」暂无图谱和归档数据。"))
    } else {
        Ok(result)
    }
}

pub enum SuperChatResult {
    DirectMessage { content: String },
    NeedsStream { system_prompt: String, history: Vec<(String, String)>, user_content: String },
}

pub async fn start_super_chat(
    state: &AppState,
    user: &crate::models::user::UserRow,
    content: &str,
) -> Result<SuperChatResult> {
    let user_msg_id = ulid::Ulid::new().to_string();
    message::insert_message(
        &state.db,
        &message::NewMessage {
            id: &user_msg_id,
            ring_id: Some(SUPER_RING_ID),
            user_id: &user.token_id,
            role: "user",
            sender_name: &user.display_name,
            content,
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await?;

    let base_prompt = get_system_prompt(&state.hub_dir);
    let ring_summary = build_ring_summary(&state.db, &user.token_id).await;
    let system_prompt = format!("{base_prompt}\n\n{ring_summary}");

    let history =
        chat::load_history_context(&state.db, Some(SUPER_RING_ID), &user.token_id, 20).await?;

    let llm = LlmClient::from_user(user)?;
    let tools = get_super_tools();

    let result = llm
        .chat_complete_with_tools(
            system_prompt.clone(),
            history.clone(),
            content.to_string(),
            tools,
        )
        .await?;

    match result {
        ChatCompleteWithToolsResult::Message { content: msg } => {
            Ok(SuperChatResult::DirectMessage { content: msg })
        }
        ChatCompleteWithToolsResult::ToolCalls { tool_calls } => {
            let mut tool_results = Vec::new();
            for tc in &tool_calls {
                let args = &tc.function.arguments;
                let tool_result = execute_tool(
                    &state.db,
                    &state.rings_dir,
                    &user.token_id,
                    &tc.function.name,
                    args,
                )
                .await
                .unwrap_or_else(|e| format!("Tool error: {e}"));

                tool_results.push((tc.function.name.clone(), tool_result));
            }

            let mut user_content = content.to_string();
            user_content.push_str("\n\n[Tool Results]\n");
            for (name, result_text) in &tool_results {
                user_content.push_str(&format!("**{name}**:\n{result_text}\n\n"));
            }

            Ok(SuperChatResult::NeedsStream {
                system_prompt,
                history,
                user_content,
            })
        }
    }
}

pub async fn get_super_history(
    state: &AppState,
    user_id: &str,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    chat::get_history(state, Some(SUPER_RING_ID), user_id, before_id, limit).await
}
