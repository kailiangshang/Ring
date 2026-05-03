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

#[derive(Debug, Deserialize)]
pub struct FetchUrlArgs {
    pub url: String,
    pub focus: Option<String>,
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

fn is_url_allowed(url: &str) -> bool {
    let lower = url.to_lowercase();
    let blocked = [
        "localhost",
        "127.0.0.1",
        "0.0.0.0",
        "::1",
        "[::1]",
        "10.",
        "172.16.",
        "172.17.",
        "172.18.",
        "172.19.",
        "172.20.",
        "172.21.",
        "172.22.",
        "172.23.",
        "172.24.",
        "172.25.",
        "172.26.",
        "172.27.",
        "172.28.",
        "172.29.",
        "172.30.",
        "172.31.",
        "192.168.",
    ];
    !blocked.iter().any(|b| lower.contains(b))
}

pub async fn execute_fetch_url(args: &FetchUrlArgs) -> Result<String> {
    if !args.url.starts_with("http://") && !args.url.starts_with("https://") {
        return Err(RingError::BadRequest(
            "URL must start with http:// or https://".into(),
        ));
    }

    if !is_url_allowed(&args.url) {
        return Err(RingError::BadRequest(
            "Access to internal addresses is not allowed".into(),
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| RingError::Internal(format!("Failed to create HTTP client: {e}")))?;

    let response = client
        .get(&args.url)
        .send()
        .await
        .map_err(|e| RingError::BadRequest(format!("Failed to fetch URL: {e}")))?;

    let is_html = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/html"))
        .unwrap_or(false);

    let body = response
        .bytes()
        .await
        .map_err(|e| RingError::Internal(format!("Failed to read response body: {e}")))?;

    if body.len() > 512 * 1024 {
        return Err(RingError::BadRequest(
            "Response too large (max 512KB)".into(),
        ));
    }

    let text = if is_html {
        let html = String::from_utf8_lossy(&body);
        strip_html(&html)
    } else {
        String::from_utf8_lossy(&body).to_string()
    };

    let truncated: String = text.chars().take(15000).collect();

    if let Some(focus) = &args.focus {
        Ok(format!(
            "## Source: {}\n## Focus: {}\n\n{}",
            args.url, focus, truncated
        ))
    } else {
        Ok(format!("## Source: {}\n\n{}", args.url, truncated))
    }
}

fn strip_html(html: &str) -> String {
    let mut result = html.to_string();

    for tag in &["script", "style", "head", "nav", "footer", "noscript"] {
        let close = format!("</{}>", tag);
        loop {
            let lower_result = result.to_lowercase();
            let start = lower_result.find(&format!("<{}", tag));
            if let Some(s) = start {
                let end = lower_result[s..].find(&close).map(|e| s + e + close.len());
                if let Some(e) = end {
                    result = format!("{}{}", &result[..s], &result[e..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    let decoded = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    let mut clean = String::new();
    let mut in_tag = false;
    for ch in decoded.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                clean.push(' ');
            }
            _ if !in_tag => clean.push(ch),
            _ => {}
        }
    }

    let re = regex::Regex::new(r"\s+").unwrap();
    re.replace_all(&clean, " ").trim().to_string()
}
