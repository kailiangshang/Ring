use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use ring_server::config::Config;
use ring_server::db::sqlite::SqliteRepository;
use ring_server::graph::petgraph_store::PetgraphStore;
use ring_server::routes::build_router;
use ring_server::services::llm_provider::{LlmEvent, LlmProvider, MockLlmProvider, TokenUsage};
use ring_server::services::tool_engine::ToolRegistry;
use ring_server::services::ws_hub::WsHub;
use ring_server::state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

fn json_body(value: &serde_json::Value) -> Body {
    Body::from(serde_json::to_vec(value).unwrap())
}

async fn body_to_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn create_app_with_events(events: Vec<LlmEvent>) -> (Router, String) {
    let pool = futures::executor::block_on(async {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    });
    let repo = Arc::new(SqliteRepository::new(pool));
    let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new(events));
    let state = AppState {
        db: repo,
        graph_store: Arc::new(RwLock::new(PetgraphStore::new())),
        config: Arc::new(Config::default()),
        llm_provider: llm,
        ws_hub: Arc::new(WsHub::new()),
        tool_registry: Arc::new(ToolRegistry::new()),
    };
    let app = build_router(state);

    let user_id = futures::executor::block_on(async {
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/setup/username")
                    .header("content-type", "application/json")
                    .body(json_body(
                        &serde_json::json!({ "display_name": "TestUser" }),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let json = body_to_json(resp.into_body()).await;
        let uid = json["user_id"].as_str().unwrap().to_string();

        app.clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/setup/complete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        uid
    });

    (app, user_id)
}

#[tokio::test]
async fn super_ring_chat_returns_sse() {
    let events = vec![
        LlmEvent::Text {
            content: "I am Super Ring.".into(),
        },
        LlmEvent::Done {
            message_id: Some("msg-sr-1".into()),
            token_usage: Some(TokenUsage {
                prompt_tokens: 20,
                completion_tokens: 10,
                total_tokens: 30,
            }),
        },
    ];
    let (app, user_id) = create_app_with_events(events);

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/super-ring/chat")
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({ "message": "Hello" })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("text/event-stream"));
}
