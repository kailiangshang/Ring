use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use ring_server::config::Config;
use ring_server::db::sqlite::SqliteRepository;
use ring_server::graph::petgraph_store::PetgraphStore;
use ring_server::graph::store_trait::GraphStore;
use ring_server::routes::build_router;
use ring_server::services::llm_provider::{LlmProvider, MockLlmProvider};
use ring_server::services::tool_engine::ToolRegistry;
use ring_server::services::ws_hub::WsHub;
use ring_server::state::AppState;
use std::sync::Arc;
use tower::ServiceExt;

async fn create_test_app() -> (Router, String) {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    let repo = Arc::new(SqliteRepository::new(pool));
    let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new(vec![]));
    let state = AppState {
        db: repo,
        graph_store: Arc::new(PetgraphStore::new()) as Arc<dyn GraphStore>,
        config: Arc::new(Config::default()),
        llm_provider: llm,
        ws_hub: Arc::new(WsHub::new()),
        tool_registry: Arc::new(ToolRegistry::new()),
    };
    let app = build_router(state);
    let user_id = complete_setup(app.clone()).await;
    (app, user_id)
}

async fn complete_setup(app: Router) -> String {
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
    let user_id = json["user_id"].as_str().unwrap().to_string();

    app.oneshot(
        Request::builder()
            .method(Method::POST)
            .uri("/api/v1/setup/complete")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    user_id
}

fn json_body(value: &serde_json::Value) -> Body {
    Body::from(serde_json::to_vec(value).unwrap())
}

async fn body_to_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn valid_create_ring_body() -> serde_json::Value {
    serde_json::json!({
        "name": "Test Ring",
        "description": "A test ring",
        "role_description": "expert",
        "gitlab_repo": "auto_create",
        "namespace": null
    })
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
                .body(json_body(&valid_create_ring_body()))
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_to_json(resp.into_body()).await;
    json["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn get_queue_returns_200() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}/archive/queue", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert!(json["current_review"].is_null());
    assert!(json["queue"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_prs_returns_200() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}/git/prs", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert!(json["prs"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_prs_with_state_filter() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}/git/prs?state=opened", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert!(json["prs"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn get_commit_log_returns_200() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}/git/commits", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert!(json["commits"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn get_commit_log_with_limit() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}/git/commits?limit=5", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_pr_diff_returns_404_for_missing() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}/git/prs/999/diff", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn merge_pr_returns_404_for_missing() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/rings/{}/git/prs/999/merge", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn reject_pr_returns_404_for_missing() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/rings/{}/git/prs/999/reject", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn confirm_archive_returns_404_for_missing() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!(
                    "/api/v1/rings/{}/archive/nonexistent/confirm",
                    ring_id
                ))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
