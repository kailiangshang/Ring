use crate::error::Result;
use crate::models::conversation_token::{self, TOKEN_THRESHOLD};
use crate::models::message::{self, MessageRow};
use crate::services::llm::{LlmClient, SseEvent};
use crate::state::AppState;

const COMPACT_THRESHOLD: usize = 30;
const COMPACT_SUMMARY_MAX_TOKENS: usize = 500;

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

pub async fn check_token_threshold(
    state: &AppState,
    user: &crate::models::user::UserRow,
    ring_id: Option<&str>,
) -> Result<(i64, bool)> {
    let token_count = conversation_token::get_token_count(&state.db, &user.token_id, ring_id).await?;
    let should_compact = user.auto_compact && token_count >= TOKEN_THRESHOLD;
    Ok((token_count, should_compact))
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

    let prompt = format!(
        "请对以下对话历史进行压缩总结，保留关键信息、决策、行动项和重要上下文。限制在{}字以内：\n\n{}",
        COMPACT_SUMMARY_MAX_TOKENS, history_text
    );

    let llm = LlmClient::from_user(user)?;
    let summary = llm
        .chat_complete("你是一个对话压缩助手。".into(), prompt)
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

    for msg in old_messages {
        let _ = sqlx::query("DELETE FROM messages WHERE id = ?1")
            .bind(&msg.id)
            .execute(&state.db)
            .await;
    }

    let _ = conversation_token::reset_tokens(&state.db, user_id, ring_id
    ).await;

    Ok(Some(summary))
}

pub fn build_system_prompt(ring_name: Option<&str>, role_description: Option<&str>) -> String {
    match ring_name {
        Some(name) => {
            let mut prompt = format!("你是 Ring「{name}」的 AI 助手。");
            if let Some(desc) = role_description {
                prompt.push_str(&format!("\n\n角色设定：{desc}"));
            }
            prompt.push_str("\n\n请用简洁、专业的方式回答用户的问题。如果引用了图谱中的节点或概念，请明确标注。");
            prompt
        }
        None => {
            let mut prompt =
                "你是用户的个人 AI 助手 Self。你完全了解用户的偏好、目标和历史对话。".to_string();

            let self_dir = crate::services::self_data::get_self_dir("");
            let (identity, identity_exists) =
                crate::services::self_data::read_self_file(&self_dir, "identity")
                    .unwrap_or_default();
            if identity_exists && !identity.is_empty() {
                prompt.push_str("\n\n用户身份定义：\n");
                prompt.push_str(&identity);
            }

            let (style, style_exists) =
                crate::services::self_data::read_self_file(&self_dir, "style").unwrap_or_default();
            if style_exists && !style.is_empty() {
                prompt.push_str("\n\n对话风格偏好：\n");
                prompt.push_str(&style);
            }

            let personality = crate::services::self_data::read_self_file(&self_dir, "personality")
                .unwrap_or_default();
            if personality.1 && !personality.0.is_empty() {
                if let Ok(p) = serde_json::from_str::<serde_json::Value>(&personality.0) {
                    if let Some(tone) = p.get("tone").and_then(|v| v.as_str()) {
                        prompt.push_str(&format!("\n\n语气风格：{tone}"));
                    }
                }
            }

            prompt.push_str("\n\n请以友好、个性化的方式回答。");
            prompt
        }
    }
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

pub async fn start_chat_stream(
    state: &AppState,
    user: &crate::models::user::UserRow,
    params: &ChatParams<'_>,
) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
    let user_msg_id = ulid::Ulid::new().to_string();

    if !params.ephemeral {
        message::insert_message(
            &state.db,
            &message::NewMessage {
                id: &user_msg_id,
                ring_id: params.ring_id,
                user_id: &user.token_id,
                role: "user",
                sender_name: &user.display_name,
                content: params.content,
                node_refs: &params.node_refs,
                tag_refs: &params.tag_refs,
                token_usage: None,
            },
        )
        .await?;
    }

    let system_prompt = build_system_prompt(params.ring_name, params.role_description);
    let history = if params.ephemeral {
        vec![]
    } else {
        load_history_context(&state.db, params.ring_id, &user.token_id, 20).await?
    };

    let llm = LlmClient::from_user(user)?;
    let rx = llm.chat_stream(
        system_prompt,
        history,
        params.content.to_string(),
        params.ai_role.to_string(),
    );
    Ok(rx)
}
