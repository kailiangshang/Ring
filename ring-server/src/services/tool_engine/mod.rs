pub mod dispatcher;
pub mod registry;

pub use dispatcher::ToolDispatcher;
pub use registry::ToolRegistry;

use crate::error::Result;
use crate::models::tool_model::ToolDefinition;
use async_trait::async_trait;

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value>;
}
