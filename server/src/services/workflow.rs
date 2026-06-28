use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::{Result, RingError};
use crate::models::user::UserRow;
use crate::services::llm::LlmClient;
use crate::state::AppState;

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

#[derive(Debug, Deserialize)]
pub struct GraphMutationArgs {
    pub action: String,
    pub node_id: Option<String>,
    pub label: Option<String>,
    pub node_type: Option<String>,
    pub parent_id: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
    pub source_id: Option<String>,
    pub target_id: Option<String>,
    pub relation: Option<String>,
    pub edge_id: Option<String>,
}

pub async fn execute_graph_mutation(
    state: &AppState,
    ring_id: &str,
    args: &GraphMutationArgs,
) -> Result<String> {
    match args.action.as_str() {
        "create_node" => {
            let input = crate::models::graph::CreateNodeInput {
                label: args.label.clone().unwrap_or_else(|| "New Node".into()),
                parent_id: args.parent_id.clone(),
                node_type: args.node_type.clone().unwrap_or_else(|| "topic".into()),
                tags: args.tags.clone().unwrap_or_default(),
                content: args.content.clone().unwrap_or_default(),
                markdown_path: None,
                metadata: serde_json::json!({}),
                graph_id: None,
            };
            let node = crate::services::graph::create_node(state, ring_id, &input).await?;
            Ok(format!("Created node: {} ({})", node.label, node.id))
        }
        "update_node" => {
            let node_id = args
                .node_id
                .as_deref()
                .ok_or_else(|| RingError::BadRequest("node_id required for update_node".into()))?;
            let input = crate::models::graph::UpdateNodeInput {
                label: args.label.clone(),
                tags: args.tags.clone(),
                content: args.content.clone(),
                markdown_path: None,
                metadata: None,
            };
            let node = crate::services::graph::update_node(state, node_id, &input).await?;
            Ok(format!("Updated node: {} ({})", node.label, node.id))
        }
        "delete_node" => {
            let node_id = args
                .node_id
                .as_deref()
                .ok_or_else(|| RingError::BadRequest("node_id required for delete_node".into()))?;
            crate::services::graph::delete_node(state, node_id).await?;
            Ok("Deleted node".into())
        }
        "create_edge" => {
            let input = crate::models::graph::CreateEdgeInput {
                source_id: args
                    .source_id
                    .clone()
                    .ok_or_else(|| RingError::BadRequest("source_id required".into()))?,
                target_id: args
                    .target_id
                    .clone()
                    .ok_or_else(|| RingError::BadRequest("target_id required".into()))?,
                relation: args.relation.clone().unwrap_or_else(|| "related_to".into()),
                label: String::new(),
                graph_id: None,
            };
            let edge = crate::services::graph::create_edge(state, ring_id, &input).await?;
            Ok(format!(
                "Created edge: {} -[{}]-> {} ({})",
                edge.source_id, edge.relation, edge.target_id, edge.id
            ))
        }
        "delete_edge" => {
            let edge_id = args
                .edge_id
                .as_deref()
                .ok_or_else(|| RingError::BadRequest("edge_id required for delete_edge".into()))?;
            crate::services::graph::delete_edge(state, edge_id).await?;
            Ok("Deleted edge".into())
        }
        _ => Err(RingError::BadRequest(format!(
            "unknown graph mutation action: {}",
            args.action
        ))),
    }
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

    let ring_id: Option<String> = sqlx::query_scalar("SELECT ring_id FROM messages WHERE id = ?1")
        .bind(&args.file_reference)
        .fetch_optional(pool)
        .await?
        .flatten();

    let existing_labels = if let Some(ref rid) = ring_id {
        let g = crate::models::graph::ensure_default_graph(pool, rid)
            .await
            .ok();
        if let Some(graph) = g {
            let nodes = crate::models::graph::list_nodes(pool, &graph.id)
                .await
                .unwrap_or_default();
            let labels: Vec<String> = nodes.iter().map(|n| n.label.clone()).collect();
            if labels.is_empty() {
                String::new()
            } else {
                labels.join(", ")
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let prompt = crate::prompts::workflow::file_parse_extraction(
        args.focus.as_deref(),
        if existing_labels.is_empty() {
            None
        } else {
            Some(&existing_labels)
        },
    );
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

pub fn is_url_allowed(url: &str) -> bool {
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
        let title = extract_html_title(&html);
        let main_content = extract_main_content(&html);
        if !main_content.is_empty() {
            format!(
                "Title: {}\n\n{}",
                title.unwrap_or_default(),
                strip_html(&main_content)
            )
        } else {
            let stripped = strip_html(&html);
            if let Some(t) = title {
                format!("Title: {}\n\n{}", t, stripped)
            } else {
                stripped
            }
        }
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

fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title>")?;
    let end = lower.find("</title>")?;
    if end <= start + 7 {
        return None;
    }
    let title = html[start + 7..end].trim().to_string();
    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

fn extract_main_content(html: &str) -> String {
    let lower = html.to_lowercase();
    for tag in &["article", "main"] {
        let open = format!("<{}", tag);
        let close = format!("</{}>", tag);
        if let Some(s) = lower.find(&open) {
            if let Some(gt) = lower[s..].find('>') {
                let tag_end = s + gt + 1;
                if let Some(e) = lower[tag_end..].find(&close) {
                    return html[tag_end..tag_end + e].to_string();
                }
            }
        }
    }
    String::new()
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
