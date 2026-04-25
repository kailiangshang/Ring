use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::{Result, RingError};
use crate::models::user::UserRow;
use crate::services::llm::LlmClient;

#[derive(Debug, Deserialize)]
pub struct FileParseArgs {
    pub file_reference: String,
    pub focus: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeExtractArgs {
    pub content: String,
    pub target_graph: Option<String>,
}

pub async fn execute_file_parse(
    pool: &SqlitePool,
    user: &UserRow,
    args: &FileParseArgs,
) -> Result<String> {
    let row = sqlx::query_as::<_, (String,)>("SELECT content FROM messages WHERE id = ?1")
        .bind(&args.file_reference)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("message {} not found", args.file_reference)))?;

    let file_text = row.0;
    let truncated: String = file_text.chars().take(30000).collect();

    let prompt = crate::prompts::workflow::file_parse_extraction(args.focus.as_deref());
    let llm = LlmClient::from_user(user)?;
    let result = llm.chat_complete(prompt, truncated).await?;
    Ok(result)
}

pub async fn execute_knowledge_extract(
    user: &UserRow,
    args: &KnowledgeExtractArgs,
) -> Result<String> {
    let prompt =
        crate::prompts::workflow::knowledge_extraction_prompt(args.target_graph.as_deref());
    let llm = LlmClient::from_user(user)?;
    let truncated: String = args.content.chars().take(30000).collect();
    let result = llm.chat_complete(prompt, truncated).await?;
    Ok(result)
}
