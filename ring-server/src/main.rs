use ring_server::config::Config;
use ring_server::db::sqlite::SqliteRepository;
use ring_server::graph::petgraph_store::PetgraphStore;
use ring_server::graph::store_trait::GraphStore;
use ring_server::routes::build_router;
use ring_server::services::llm_provider::MockLlmProvider;
use ring_server::services::tool_engine::tools::{
    MarkdownGenTool, PrivacyFilterTool, SearchTool, TextCleanTool, WebScrapeTool,
};
use ring_server::services::tool_engine::ToolRegistry;
use ring_server::services::ws_hub::WsHub;
use ring_server::state::AppState;

use std::sync::Arc;

use sqlx::sqlite::SqlitePoolOptions;

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
    let ws_hub = Arc::new(WsHub::new());
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(SearchTool::new(db.clone())));
    registry.register(Arc::new(TextCleanTool::new()));
    registry.register(Arc::new(WebScrapeTool::new()));
    registry.register(Arc::new(MarkdownGenTool::new()));
    registry.register(Arc::new(PrivacyFilterTool::new()));
    let tool_registry = Arc::new(registry);

    let llm_provider = Arc::new(MockLlmProvider::new(vec![]));
    let port = config.port;
    let state = AppState {
        db: db.clone(),
        graph_store,
        config,
        llm_provider,
        ws_hub,
        tool_registry,
    };
    let llm_provider = state.rebuild_llm().await;
    let state = AppState {
        llm_provider,
        ..state
    };

    let app = build_router(state);

    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|_| panic!("failed to bind {}", bind_addr));

    tracing::info!("Ring server listening on http://{}", bind_addr);
    axum::serve(listener, app).await.expect("server error");
}
