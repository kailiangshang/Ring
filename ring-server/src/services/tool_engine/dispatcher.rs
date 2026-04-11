use std::sync::Arc;

use super::registry::ToolRegistry;
use crate::models::tool_model::{ToolCallRequest, ToolDefinition, ToolResultRecord};

pub struct ToolDispatcher {
    registry: Arc<ToolRegistry>,
}

impl ToolDispatcher {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        ToolDispatcher { registry }
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.registry.definitions()
    }

    pub fn definitions_filtered(&self, names: Option<&[String]>) -> Vec<ToolDefinition> {
        let all = self.definitions();
        match names {
            Some(filter) if !filter.is_empty() => all
                .into_iter()
                .filter(|t| filter.contains(&t.name))
                .collect(),
            _ => all,
        }
    }

    pub async fn dispatch(&self, call: ToolCallRequest) -> ToolResultRecord {
        match self.registry.get(&call.tool_name) {
            Some(tool) => match tool.execute(call.input.clone()).await {
                Ok(output) => ToolResultRecord {
                    tool_call_id: call.tool_call_id,
                    tool_name: call.tool_name,
                    output,
                    success: true,
                },
                Err(e) => ToolResultRecord {
                    tool_call_id: call.tool_call_id,
                    tool_name: call.tool_name,
                    output: serde_json::json!({ "error": e.to_string() }),
                    success: false,
                },
            },
            None => {
                let name = call.tool_name.clone();
                ToolResultRecord {
                    tool_call_id: call.tool_call_id,
                    tool_name: call.tool_name,
                    output: serde_json::json!({ "error": format!("unknown tool: {}", name) }),
                    success: false,
                }
            }
        }
    }
}
