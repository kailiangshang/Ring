use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use ring_server::config::Config;
use ring_server::db::sqlite::SqliteRepository;
use ring_server::graph::petgraph_store::PetgraphStore;
use ring_server::graph::store_trait::GraphStore;
use ring_server::routes::build_router;
use ring_server::services::llm_provider::{LlmEvent, LlmProvider, MockLlmProvider, TokenUsage};
use ring_server::services::tool_engine::ToolRegistry;
use ring_server::services::ws_hub::WsHub;
use ring_server::state::AppState;
use std::sync::Arc;
use tower::ServiceExt;

fn json_body(value: &serde_json::Value) -> Body {
    Body::from(serde_json::to_vec(value).unwrap())
}

async fn body_to_bytes(body: Body) -> Vec<u8> {
    body.collect().await.unwrap().to_bytes().to_vec()
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
        db: repo.clone(),
        graph_store: Arc::new(PetgraphStore::new()) as Arc<dyn GraphStore>,
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

async fn create_ring(app: &Router, user_id: &str) -> String {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/rings")
                .header("content-type", "application/json")
                .header("X-User-Id", user_id)
                .body(json_body(&serde_json::json!({
                    "name": "Test Ring",
                    "description": "A test ring",
                    "role_description": "expert",
                    "gitlab_repo": "auto_create"
                })))
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_to_json(resp.into_body()).await;
    json["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_conversation_success() {
    let (app, user_id) = create_app_with_events(vec![]);
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/rings/{}/conversations", ring_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({
                    "title": "My Chat",
                    "context_mode": "storage"
                })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["title"], "My Chat");
    assert_eq!(json["context_mode"], "storage");
    assert_eq!(json["mode"], "chat");
    assert!(!json["id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn list_conversations_by_ring() {
    let (app, user_id) = create_app_with_events(vec![]);
    let ring_id = create_ring(&app, &user_id).await;

    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/rings/{}/conversations", ring_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({ "title": "Conv 1" })))
                .unwrap(),
        )
        .await
        .unwrap();

    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/rings/{}/conversations", ring_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({ "title": "Conv 2" })))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}/conversations", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    let convs = json["conversations"].as_array().unwrap();
    assert_eq!(convs.len(), 2);
}

#[tokio::test]
async fn send_message_returns_sse_stream() {
    let events = vec![
        LlmEvent::Text {
            content: "Hello!".into(),
        },
        LlmEvent::Done {
            message_id: Some("msg-1".into()),
            token_usage: Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        },
    ];
    let (app, user_id) = create_app_with_events(events);
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/rings/{}/conversations", ring_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({ "title": "SSE Test" })))
                .unwrap(),
        )
        .await
        .unwrap();
    let conv = body_to_json(resp.into_body()).await;
    let conv_id = conv["id"].as_str().unwrap();

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/api/v1/rings/{}/conversations/{}/messages",
                    ring_id, conv_id
                ))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({ "message": "Hi there" })))
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

    let body_bytes = body_to_bytes(resp.into_body()).await;
    let body_str = String::from_utf8(body_bytes).unwrap();
    assert!(body_str.contains("event:message") || body_str.contains("event: message"));
    assert!(body_str.contains("text"));
    assert!(body_str.contains("done"));
}
