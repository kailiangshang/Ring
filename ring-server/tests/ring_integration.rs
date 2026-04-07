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
use ring_server::services::search_service::SearchService;
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
        db: repo.clone(),
        graph_store: Arc::new(PetgraphStore::new()) as Arc<dyn GraphStore>,
        config: Arc::new(Config::default()),
        llm_provider: llm,
        ws_hub: Arc::new(WsHub::new()),
        tool_registry: Arc::new(ToolRegistry::new()),
        search_service: Arc::new(SearchService::new(
            repo.clone(),
            Arc::new(PetgraphStore::new()),
        )),
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

#[tokio::test]
async fn create_and_list_rings() {
    let (app, user_id) = create_test_app().await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/rings")
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&valid_create_ring_body()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/rings")
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    let rings = json["rings"].as_array().unwrap();
    assert_eq!(rings.len(), 1);
    assert_eq!(rings[0]["name"], "Test Ring");
}

#[tokio::test]
async fn create_and_get_ring() {
    let (app, user_id) = create_test_app().await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/rings")
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&valid_create_ring_body()))
                .unwrap(),
        )
        .await
        .unwrap();
    let created = body_to_json(resp.into_body()).await;
    let ring_id = created["id"].as_str().unwrap();

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["id"], ring_id);
    assert_eq!(json["name"], "Test Ring");
    assert_eq!(json["description"], "A test ring");
}

#[tokio::test]
async fn update_ring_name() {
    let (app, user_id) = create_test_app().await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/rings")
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&valid_create_ring_body()))
                .unwrap(),
        )
        .await
        .unwrap();
    let created = body_to_json(resp.into_body()).await;
    let ring_id = created["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/api/v1/rings/{}", ring_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({
                    "name": "Updated Ring",
                    "description": null
                })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["name"], "Updated Ring");

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["name"], "Updated Ring");
}

#[tokio::test]
async fn delete_ring() {
    let (app, user_id) = create_test_app().await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/rings")
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&valid_create_ring_body()))
                .unwrap(),
        )
        .await
        .unwrap();
    let created = body_to_json(resp.into_body()).await;
    let ring_id = created["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/rings/{}", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_ring_empty_name() {
    let (app, user_id) = create_test_app().await;

    let mut body = valid_create_ring_body();
    body["name"] = serde_json::json!("   ");

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/rings")
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_nonexistent_ring() {
    let (app, user_id) = create_test_app().await;

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/rings/nonexistent-id")
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
