use ring_server::config::Config;
use ring_server::db::sqlite::SqliteRepository;
use ring_server::db::traits::Repository;
use ring_server::graph::petgraph_store::PetgraphStore;
use ring_server::graph::store_trait::GraphStore;
use ring_server::routes::build_router;
use ring_server::services::llm_anthropic::AnthropicProvider;
use ring_server::services::llm_openai::OpenAiProvider;
use ring_server::services::llm_provider::{LlmProvider, MockLlmProvider};
use ring_server::services::tool_engine::tools::{
    MarkdownGenTool, PrivacyFilterTool, SearchTool, TextCleanTool, WebScrapeTool,
};
use ring_server::services::tool_engine::ToolRegistry;
use ring_server::services::ws_hub::WsHub;
use ring_server::state::AppState;

use std::sync::Arc;

use sqlx::sqlite::SqlitePoolOptions;

async fn build_llm_provider(db: &dyn Repository) -> Arc<dyn LlmProvider> {
    let provider_name = db.get_setting("llm_provider").await.ok().flatten();
    let model = db.get_setting("llm_model").await.ok().flatten();
    let api_key = db.get_setting("llm_api_key").await.ok().flatten();
    let base_url = db.get_setting("llm_base_url").await.ok().flatten();

    match provider_name.as_deref() {
        Some("openai") | Some("ollama") => {
            let api_key = api_key.unwrap_or_default();
            let model = model.unwrap_or_default();
            Arc::new(OpenAiProvider::new(api_key, model, base_url))
        }
        Some("anthropic") => {
            let api_key = api_key.unwrap_or_default();
            let model = model.unwrap_or_default();
            Arc::new(AnthropicProvider::new(api_key, model, base_url))
        }
        _ => Arc::new(MockLlmProvider::new(vec![])),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::default();

    let db_path = config.data_dir.join("data");
    std::fs::create_dir_all(&db_path).expect("failed to create data dir");

    let pool = SqlitePoolOptions::new()
        .connect(&config.database_url)
        .await
        .expect("failed to connect to sqlite");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let db = Arc::new(SqliteRepository::new(pool));
    let graph_store: Arc<dyn GraphStore> = Arc::new(PetgraphStore::new());
    let config = Arc::new(config);
    let llm_provider = build_llm_provider(db.as_ref()).await;
    let ws_hub = Arc::new(WsHub::new());
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(SearchTool::new(db.clone())));
    registry.register(Arc::new(TextCleanTool::new()));
    registry.register(Arc::new(WebScrapeTool::new()));
    registry.register(Arc::new(MarkdownGenTool::new()));
    registry.register(Arc::new(PrivacyFilterTool::new()));
    let tool_registry = Arc::new(registry);

    let state = AppState {
        db,
        graph_store,
        config,
        llm_provider,
        ws_hub,
        tool_registry,
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7420")
        .await
        .expect("failed to bind port 7420");

    tracing::info!("Ring server listening on http://0.0.0.0:7420");
    axum::serve(listener, app).await.expect("server error");
}
