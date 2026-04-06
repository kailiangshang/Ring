use async_trait::async_trait;
use serde::Deserialize;

use crate::error::RingError;
use crate::models::tool_model::ToolDefinition;
use crate::services::tool_engine::Tool;

pub struct MarkdownGenTool;

#[derive(Deserialize)]
struct Section {
    heading: String,
    body: String,
}

#[derive(Deserialize)]
struct MarkdownGenInput {
    title: String,
    sections: Vec<Section>,
}

impl Default for MarkdownGenTool {
    fn default() -> Self {
        MarkdownGenTool
    }
}

impl MarkdownGenTool {
    pub fn new() -> Self {
        MarkdownGenTool
    }
}

#[async_trait]
impl Tool for MarkdownGenTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "markdown_gen".to_string(),
            description: "Generate formatted markdown from a title and sections".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Document title" },
                    "sections": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "heading": { "type": "string" },
                                "body": { "type": "string" }
                            },
                            "required": ["heading", "body"]
                        },
                        "description": "Sections with heading and body"
                    }
                },
                "required": ["title", "sections"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value) -> crate::error::Result<serde_json::Value> {
        let parsed: MarkdownGenInput =
            serde_json::from_value(input).map_err(RingError::Serialization)?;

        let mut md = format!("# {}\n", parsed.title);

        for section in &parsed.sections {
            md.push_str(&format!("\n## {}\n\n{}\n", section.heading, section.body));
        }

        Ok(serde_json::json!({ "markdown": md }))
    }
}
