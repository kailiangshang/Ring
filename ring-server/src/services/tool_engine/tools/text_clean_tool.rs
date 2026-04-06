use async_trait::async_trait;
use serde::Deserialize;

use crate::error::RingError;
use crate::models::tool_model::ToolDefinition;
use crate::services::tool_engine::Tool;

pub struct TextCleanTool;

#[derive(Deserialize)]
struct TextCleanInput {
    text: String,
}

impl Default for TextCleanTool {
    fn default() -> Self {
        TextCleanTool
    }
}

impl TextCleanTool {
    pub fn new() -> Self {
        TextCleanTool
    }
}

#[async_trait]
impl Tool for TextCleanTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "text_clean".to_string(),
            description:
                "Clean and normalize text by stripping extra whitespace and normalizing unicode"
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text to clean" }
                },
                "required": ["text"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value) -> crate::error::Result<serde_json::Value> {
        let parsed: TextCleanInput =
            serde_json::from_value(input).map_err(RingError::Serialization)?;
        let cleaned = parsed
            .text
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");
        Ok(serde_json::json!({ "cleaned_text": cleaned }))
    }
}
