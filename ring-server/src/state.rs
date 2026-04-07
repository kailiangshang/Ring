use std::sync::Arc;

use crate::config::Config;
use crate::db::traits::Repository;
use crate::graph::store_trait::GraphStore;
use crate::services::llm_anthropic::AnthropicProvider;
use crate::services::llm_openai::OpenAiProvider;
use crate::services::llm_provider::LlmProvider;
use crate::services::search_service::SearchService;
use crate::services::tool_engine::ToolRegistry;
use crate::services::ws_hub::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Repository>,
    pub graph_store: Arc<dyn GraphStore>,
    pub search_service: Arc<SearchService>,
    pub config: Arc<Config>,
    pub llm_provider: Arc<dyn LlmProvider>,
    pub ws_hub: Arc<WsHub>,
    pub tool_registry: Arc<ToolRegistry>,
}

impl AppState {
    pub async fn rebuild_llm(&self) -> Arc<dyn LlmProvider> {
        let provider_name = self.db.get_setting("llm_provider").await.ok().flatten();
        if provider_name.is_none() {
            return self.llm_provider.clone();
        }
        let model = self.db.get_setting("llm_model").await.ok().flatten();
        let api_key = self.db.get_setting("llm_api_key").await.ok().flatten();
        let base_url = self.db.get_setting("llm_base_url").await.ok().flatten();

        match provider_name.as_deref() {
            Some("openai") | Some("ollama") => Arc::new(OpenAiProvider::new(
                api_key.unwrap_or_default(),
                model.unwrap_or_default(),
                base_url,
            )),
            Some("anthropic") => Arc::new(AnthropicProvider::new(
                api_key.unwrap_or_default(),
                model.unwrap_or_default(),
                base_url,
            )),
            _ => self.llm_provider.clone(),
        }
    }
}
