use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use ring_server::config::Config;
use ring_server::db::sqlite::SqliteRepository;
use ring_server::db::Repository;
use ring_server::graph::petgraph_store::PetgraphStore;
use ring_server::graph::store_trait::GraphStore;
use ring_server::routes::build_router;
use ring_server::services::llm_provider::{LlmProvider, MockLlmProvider};
use ring_server::services::tool_engine::ToolRegistry;
use ring_server::services::ws_hub::WsHub;
use ring_server::state::AppState;
use std::sync::Arc;
use tower::ServiceExt;

async fn body_to_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn join_page_valid_token_returns_html() {
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
    };

    let user = repo
        .create_user(ring_server::models::user::NewUser {
            display_name: "TestUser".into(),
        })
        .await
        .unwrap();

    let ring = repo
        .create_ring(ring_server::models::ring::NewRing {
            name: "Test Ring".into(),
            description: Some("A test ring".into()),
            creator_id: user.id.clone(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
            role_description: "expert".into(),
        })
        .await
        .unwrap();

    repo.create_invite_token(&ring.id, "valid-test-token", "open", &user.id)
        .await
        .unwrap();

    let app = build_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/join?token=valid-test-token")
                .body(Body::empty())
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
    assert!(ct.contains("text/html"));

    let body = body_to_string(resp.into_body()).await;
    assert!(body.contains("Test Ring"));
    assert!(body.contains("window.__RING_JOIN_DATA__"));
}

#[tokio::test]
async fn join_page_html_contains_ring_data() {
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
    };

    let user = repo
        .create_user(ring_server::models::user::NewUser {
            display_name: "TestUser".into(),
        })
        .await
        .unwrap();

    let ring = repo
        .create_ring(ring_server::models::ring::NewRing {
            name: "My Ring".into(),
            description: Some("Ring description".into()),
            creator_id: user.id.clone(),
            gitlab_repo: "auto_create".into(),
            namespace: None,
            role_description: "expert".into(),
        })
        .await
        .unwrap();

    repo.create_invite_token(&ring.id, "data-test-token", "open", &user.id)
        .await
        .unwrap();

    let app = build_router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/join?token=data-test-token&creator_ip=192.168.1.1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = body_to_string(resp.into_body()).await;
    assert!(body.contains("My Ring"));
    assert!(body.contains("Ring description"));
    assert!(body.contains("releases/latest/download"));
    assert!(body.contains("ring-server-windows"));
    assert!(body.contains("ring-server-linux"));
    assert!(body.contains("ring-server-macos-aarch64"));
    assert!(body.contains("ring-server-macos-x86_64"));
}

#[tokio::test]
async fn join_page_invalid_token_returns_404() {
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

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/join?token=invalid-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body = body_to_string(resp.into_body()).await;
    assert!(body.contains("Not Found"));
}

#[tokio::test]
async fn join_page_missing_token_returns_400() {
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

    let resp = app
        .oneshot(Request::builder().uri("/join").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_to_string(resp.into_body()).await;
    assert!(body.contains("Bad Request"));
}
