use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::db::Repository;
use crate::error::RingError;
use crate::models::tool_model::ToolDefinition;
use crate::services::tool_engine::Tool;

pub struct SearchTool {
    repo: Arc<dyn Repository>,
}

#[derive(Deserialize)]
struct SearchInput {
    query: String,
    graph_ids: Option<Vec<String>>,
    limit: Option<i64>,
}

impl SearchTool {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        SearchTool { repo }
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "search".to_string(),
            description: "Search knowledge graph nodes using full-text search".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query string" },
                    "graph_ids": { "type": "array", "items": { "type": "string" }, "description": "Optional list of graph IDs to filter" },
                    "limit": { "type": "integer", "description": "Maximum number of results (default: 10)" }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, input: serde_json::Value) -> crate::error::Result<serde_json::Value> {
        let parsed: SearchInput =
            serde_json::from_value(input).map_err(RingError::Serialization)?;
        let results = self
            .repo
            .search_nodes_fts(&parsed.query, parsed.graph_ids, parsed.limit.unwrap_or(10))
            .await?;
        Ok(serde_json::json!({ "results": results }))
    }
}
