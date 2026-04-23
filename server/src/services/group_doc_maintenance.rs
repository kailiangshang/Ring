use crate::error::{Result, RingError};
use crate::models::message;
use crate::models::user::UserRow;
use crate::services::llm::LlmClient;
use crate::state::AppState;

pub async fn update_active_context(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    user: &UserRow,
) -> Result<()> {
    let messages = message::list_messages(&state.db, Some(ring_id), user_id, None, 30).await?;

    if messages.len() < 3 {
        return Ok(());
    }

    let history_text: String = messages
        .iter()
        .rev()
        .map(|m| format!("{}: {}", m.sender_name, m.content))
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!("{}\n{}", crate::prompts::group_docs::ACTIVE_CONTEXT_USER, history_text);

    let llm = LlmClient::from_user(user)?;
    let content = llm
        .chat_complete(crate::prompts::group_docs::ACTIVE_CONTEXT_SYSTEM.into(), prompt)
        .await?;

    sqlx::query(
        "INSERT INTO group_docs (ring_id, doc_name, content, updated_at)
         VALUES (?1, 'active-context', ?2, datetime('now'))
         ON CONFLICT(ring_id, doc_name) DO UPDATE SET
         content = ?2, updated_at = datetime('now')",
    )
    .bind(ring_id)
    .bind(content.trim())
    .execute(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn update_archive_patterns(
    state: &AppState,
    ring_id: &str,
    user: &UserRow,
    archive_content: &str,
) -> Result<()> {
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT content FROM group_docs WHERE ring_id = ?1 AND doc_name = 'archive-patterns'",
    )
    .bind(ring_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    let existing = existing.unwrap_or_default();
    let prompt = format!("{}\n{}", crate::prompts::group_docs::ARCHIVE_PATTERNS_USER, archive_content);

    let llm = LlmClient::from_user(user)?;
    let new_patterns = llm
        .chat_complete(crate::prompts::group_docs::ARCHIVE_PATTERNS_SYSTEM.into(), prompt)
        .await?;

    let merged = if existing.is_empty() {
        new_patterns
    } else {
        format!("{}\n\n---\n\n{}", existing, new_patterns)
    };

    sqlx::query(
        "INSERT INTO group_docs (ring_id, doc_name, content, updated_at)
         VALUES (?1, 'archive-patterns', ?2, datetime('now'))
         ON CONFLICT(ring_id, doc_name) DO UPDATE SET
         content = ?2, updated_at = datetime('now')",
    )
    .bind(ring_id)
    .bind(merged.trim())
    .execute(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn add_correction(
    state: &AppState,
    ring_id: &str,
    correction_detail: &str,
) -> Result<()> {
    let existing: Option<String> = sqlx::query_scalar(
        "SELECT content FROM group_docs WHERE ring_id = ?1 AND doc_name = 'corrections'",
    )
    .bind(ring_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let entry = format!("## {}\n- {}", today, correction_detail);

    let merged = if let Some(existing) = existing {
        format!("{}\n\n{}", entry, existing)
    } else {
        entry
    };

    sqlx::query(
        "INSERT INTO group_docs (ring_id, doc_name, content, updated_at)
         VALUES (?1, 'corrections', ?2, datetime('now'))
         ON CONFLICT(ring_id, doc_name) DO UPDATE SET
         content = ?2, updated_at = datetime('now')",
    )
    .bind(ring_id)
    .bind(merged.trim())
    .execute(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}

const KNOWLEDGE_SUMMARY_PROMPT: &str = r#"基于以下图谱节点和边信息，生成知识库整体摘要。用 Markdown 格式输出，包含以下部分：

## 知识概览
- 总节点数、边数

## 主要分类
- 列出主要节点分类及描述

## 近期变化
- 如果有更新，列出近期变化

图谱数据：
"#;

pub async fn update_knowledge_summary(
    state: &AppState,
    ring_id: &str,
    user: &UserRow,
) -> Result<()> {
    let nodes = sqlx::query_as::<_, crate::models::graph::GraphNodeRow>(
        "SELECT n.* FROM graph_nodes n
         JOIN graphs g ON n.graph_id = g.id
         WHERE g.ring_id = ?1",
    )
    .bind(ring_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    let edges = sqlx::query_as::<_, crate::models::graph::GraphEdgeRow>(
        "SELECT e.* FROM graph_edges e
         JOIN graphs g ON e.graph_id = g.id
         WHERE g.ring_id = ?1",
    )
    .bind(ring_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    let graph_json = serde_json::json!({
        "nodes": nodes,
        "edges": edges,
    });

    let prompt = format!(
        "{}\n{}",
        KNOWLEDGE_SUMMARY_PROMPT,
        serde_json::to_string_pretty(&graph_json).unwrap_or_default()
    );

    let llm = LlmClient::from_user(user)?;
    let content = llm
        .chat_complete("你是一个知识库分析助手。".into(), prompt)
        .await?;

    sqlx::query(
        "INSERT INTO group_docs (ring_id, doc_name, content, updated_at)
         VALUES (?1, 'knowledge-summary', ?2, datetime('now'))
         ON CONFLICT(ring_id, doc_name) DO UPDATE SET
         content = ?2, updated_at = datetime('now')",
    )
    .bind(ring_id)
    .bind(content.trim())
    .execute(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}
