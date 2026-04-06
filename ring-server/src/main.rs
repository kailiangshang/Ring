use ring_server::config::Config;
use ring_server::db::sqlite::SqliteRepository;
use ring_server::graph::petgraph_store::PetgraphStore;
use ring_server::routes::build_router;
use ring_server::services::llm_provider::{LlmProvider, MockLlmProvider};
use ring_server::services::ws_hub::WsHub;
use ring_server::state::AppState;

use std::sync::Arc;

use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::RwLock;

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
    let graph_store = Arc::new(RwLock::new(PetgraphStore::new()));
    let config = Arc::new(config);
    let llm_provider: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new(vec![]));
    let ws_hub = Arc::new(WsHub::new());

    let state = AppState {
        db,
        graph_store,
        config,
        llm_provider,
        ws_hub,
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7420")
        .await
        .expect("failed to bind port 7420");

    tracing::info!("Ring server listening on http://0.0.0.0:7420");
    axum::serve(listener, app).await.expect("server error");
}
