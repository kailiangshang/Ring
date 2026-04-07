use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use ring_server::config::Config;
use ring_server::db::sqlite::SqliteRepository;
use ring_server::db::Repository;
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

async fn body_to_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn body_to_bytes(body: Body) -> Vec<u8> {
    body.collect().await.unwrap().to_bytes().to_vec()
}

fn create_app_with_events(events: Vec<LlmEvent>) -> (Router, Arc<SqliteRepository>, String) {
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

    (app, repo, user_id)
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
async fn list_templates_returns_list() {
    let (app, repo, user_id) = create_app_with_events(vec![]);

    repo.create_blueprint_template(
        "tpl-1",
        "Knowledge Graph",
        Some("A knowledge graph template"),
        r#"[{"name":"concepts","graph_type":"knowledge"}]"#,
        true,
    )
    .await
    .unwrap();

    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}/blueprint/templates", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_to_json(resp.into_body()).await;
    let templates = json["templates"].as_array().unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0]["name"], "Knowledge Graph");
}

#[tokio::test]
async fn blueprint_chat_returns_sse() {
    let events = vec![
        LlmEvent::Text {
            content: "I suggest creating a knowledge graph...".into(),
        },
        LlmEvent::BlueprintProposal {
            data: serde_json::json!({"graphs": [{"name": "知识图谱", "graph_type": "knowledge"}]}),
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
    let (app, _, user_id) = create_app_with_events(events);
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/rings/{}/blueprint/chat", ring_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({
                    "message": "I need a competitor research graph"
                })))
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
    assert!(body_str.contains("blueprint_proposal"));
    assert!(body_str.contains("done"));
}

#[tokio::test]
async fn preview_blueprint_returns_nodes() {
    let (app, _, user_id) = create_app_with_events(vec![]);
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/rings/{}/blueprint/preview", ring_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({
                    "graphs": [
                        {
                            "name": "Knowledge Graph",
                            "graph_type": "knowledge",
                            "categories": ["Concepts", "Methods", "Tools"]
                        },
                        {
                            "name": "Competitor Graph",
                            "graph_type": "competitor",
                            "categories": ["Competitor A"]
                        }
                    ]
                })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_to_json(resp.into_body()).await;
    let graphs = json["graphs"].as_array().unwrap();
    assert_eq!(graphs.len(), 2);

    let kg = &graphs[0];
    assert_eq!(kg["name"], "Knowledge Graph");
    let nodes = kg["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 4);
    let edges = kg["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 3);

    let cg = &graphs[1];
    assert_eq!(cg["name"], "Competitor Graph");
    let cg_nodes = cg["nodes"].as_array().unwrap();
    assert_eq!(cg_nodes.len(), 2);
}

#[tokio::test]
async fn confirm_blueprint_creates_graphs() {
    let (app, _, user_id) = create_app_with_events(vec![]);
    let ring_id = create_ring(&app, &user_id).await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/v1/rings/{}/blueprint/confirm", ring_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({
                    "graphs": [
                        {
                            "name": "Knowledge Graph",
                            "graph_type": "knowledge",
                            "categories": ["Concepts", "Methods"]
                        }
                    ]
                })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["status"], "confirmed");
    assert!(!json["blueprint_id"].as_str().unwrap().is_empty());

    let graphs = json["graphs"].as_array().unwrap();
    assert_eq!(graphs.len(), 1);
    assert_eq!(graphs[0]["name"], "Knowledge Graph");
    assert_eq!(graphs[0]["graph_type"], "knowledge");
    assert!(!graphs[0]["id"].as_str().unwrap().is_empty());

    let ring_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/rings/{}", ring_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let ring_json = body_to_json(ring_resp.into_body()).await;
    assert_eq!(ring_json["status"], "active");
}
