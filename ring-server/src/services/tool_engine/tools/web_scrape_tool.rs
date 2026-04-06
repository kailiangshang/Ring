use async_trait::async_trait;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::error::RingError;
use crate::models::tool_model::ToolDefinition;
use crate::services::tool_engine::Tool;

pub struct WebScrapeTool {
    client: Client,
}

#[derive(Deserialize)]
struct WebScrapeInput {
    url: String,
}

impl Default for WebScrapeTool {
    fn default() -> Self {
        WebScrapeTool {
            client: Client::new(),
        }
    }
}

impl WebScrapeTool {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Tool for WebScrapeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "web_scrape".to_string(),
            description: "Fetch a web page and extract its title and text content".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "URL to fetch" }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value) -> crate::error::Result<serde_json::Value> {
        let parsed: WebScrapeInput =
            serde_json::from_value(input).map_err(RingError::Serialization)?;

        let response = self
            .client
            .get(&parsed.url)
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("request failed: {e}")))?;

        let html_str = response
            .text()
            .await
            .map_err(|e| RingError::Internal(format!("failed to read body: {e}")))?;

        let document = Html::parse_document(&html_str);

        let title = document
            .select(&Selector::parse("title").unwrap())
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let content_selectors = ["p", "h1", "h2", "h3", "h4", "h5", "h6", "li", "td"];
        let whitespace_re = Regex::new(r"\s+").unwrap();
        let mut text_parts: Vec<String> = Vec::new();

        for tag in &content_selectors {
            if let Ok(selector) = Selector::parse(tag) {
                for element in document.select(&selector) {
                    let text = whitespace_re
                        .replace_all(&element.text().collect::<String>(), " ")
                        .trim()
                        .to_string();
                    if !text.is_empty() {
                        text_parts.push(text);
                    }
                }
            }
        }

        Ok(serde_json::json!({
            "title": title,
            "text": text_parts.join("\n")
        }))
    }
}
