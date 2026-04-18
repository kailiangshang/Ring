use std::path::Path;

use crate::error::Result;
use crate::models::message::{self, MessageRow};
use crate::services::chat;
use crate::services::llm::{LlmClient, SseEvent};
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

pub async fn start_super_chat(
    state: &AppState,
    user: &crate::models::user::UserRow,
    content: &str,
) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
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

    let system_prompt = get_system_prompt(&state.hub_dir);
    let history =
        chat::load_history_context(&state.db, Some(SUPER_RING_ID), &user.token_id, 20).await?;

    let llm = LlmClient::from_user(user)?;
    let rx = llm.chat_stream(
        system_prompt,
        history,
        content.to_string(),
        "super_ring".to_string(),
    );
    Ok(rx)
}

pub async fn get_super_history(
    state: &AppState,
    user_id: &str,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    chat::get_history(state, Some(SUPER_RING_ID), user_id, before_id, limit).await
}
