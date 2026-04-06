use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use ring_server::config::Config;
use ring_server::db::sqlite::SqliteRepository;
use ring_server::graph::petgraph_store::PetgraphStore;
use ring_server::routes::build_router;
use ring_server::services::llm_provider::{LlmProvider, MockLlmProvider};
use ring_server::services::tool_engine::ToolRegistry;
use ring_server::services::ws_hub::WsHub;
use ring_server::state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

async fn create_test_app() -> Router {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = Arc::new(SqliteRepository::new(pool));
    let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new(vec![]));
    let state = AppState {
        db: repo,
        graph_store: Arc::new(RwLock::new(PetgraphStore::new())),
        config: Arc::new(Config::default()),
        llm_provider: llm,
        ws_hub: Arc::new(WsHub::new()),
        tool_registry: Arc::new(ToolRegistry::new()),
    };
    build_router(state)
}

fn json_body(value: &serde_json::Value) -> Body {
    Body::from(serde_json::to_vec(value).unwrap())
}

async fn body_to_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn full_setup_wizard_flow() {
    let app = create_test_app().await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["setup_completed"], false);
    assert_eq!(json["step"], "username");

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
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["display_name"], "TestUser");
    assert!(json["user_id"].is_string());

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/setup/llm")
                .header("content-type", "application/json")
                .body(json_body(&serde_json::json!({
                    "provider": "openai",
                    "model": "gpt-4",
                    "api_key": "sk-test",
                    "base_url": null
                })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/setup/gitlab")
                .header("content-type", "application/json")
                .body(json_body(&serde_json::json!({
                    "repo_url": "git@gitlab.corp:test/repo.git",
                    "auth_type": "ssh_key",
                    "ssh_key_path": "~/.ssh/id_rsa",
                    "auto_create": false
                })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/setup/complete")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["setup_completed"], true);
}

#[tokio::test]
async fn setup_rejects_empty_username() {
    let app = create_test_app().await;

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/setup/username")
                .header("content-type", "application/json")
                .body(json_body(&serde_json::json!({ "display_name": "" })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn setup_rejects_long_username() {
    let app = create_test_app().await;

    let long_name = "x".repeat(51);
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/setup/username")
                .header("content-type", "application/json")
                .body(json_body(&serde_json::json!({ "display_name": long_name })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn setup_twice_returns_conflict() {
    let app = create_test_app().await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/setup/username")
                .header("content-type", "application/json")
                .body(json_body(&serde_json::json!({ "display_name": "User1" })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/setup/complete")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/setup/username")
                .header("content-type", "application/json")
                .body(json_body(&serde_json::json!({ "display_name": "User2" })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}
