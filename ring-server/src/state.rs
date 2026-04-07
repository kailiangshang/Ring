use std::sync::Arc;

use crate::config::Config;
use crate::db::traits::Repository;
use crate::graph::store_trait::GraphStore;
use crate::services::llm_provider::LlmProvider;
use crate::services::tool_engine::ToolRegistry;
use crate::services::ws_hub::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Repository>,
    pub graph_store: Arc<dyn GraphStore>,
    pub config: Arc<Config>,
    pub llm_provider: Arc<dyn LlmProvider>,
    pub ws_hub: Arc<WsHub>,
    pub tool_registry: Arc<ToolRegistry>,
}
