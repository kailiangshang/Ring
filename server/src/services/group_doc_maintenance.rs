use crate::error::{Result, RingError};
use crate::models::message;
use crate::models::user::UserRow;
use crate::services::llm::LlmClient;
use crate::state::AppState;

const ACTIVE_CONTEXT_PROMPT: &str = r#"基于以下最近的对话历史，生成一个活跃上下文摘要。用 Markdown 格式输出，包含以下部分：

## 近期话题
- 列出最近讨论的 3-5 个主要话题

## 待处理
- 列出尚未解决或需要跟进的事项

## 关注节点
- 列出对话中提到的关键概念或节点

对话历史：
"#;

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

    let prompt = format!("{}\n{}", ACTIVE_CONTEXT_PROMPT, history_text);

    let llm = LlmClient::from_user(user)?;
    let content = llm
        .chat_complete("你是一个群组的上下文分析助手。".into(), prompt)
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

const ARCHIVE_PATTERNS_PROMPT: &str = r#"基于以下归档操作记录，提取归档行为模式偏好。用 Markdown 格式输出，包含以下部分：

## 偏好
- 粒度偏好（按主题/按项目/其他）
- 归档位置偏好

## 模式
- 用户通常将什么类型的内容归入什么节点
- 其他观察到的模式

归档记录：
"#;

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
    let prompt = format!("{}\n{}", ARCHIVE_PATTERNS_PROMPT, archive_content);

    let llm = LlmClient::from_user(user)?;
    let new_patterns = llm
        .chat_complete("你是一个归档模式分析助手。".into(), prompt)
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
