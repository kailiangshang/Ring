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

fn graph_base(ring_id: &str) -> String {
    format!("/api/v1/rings/{}/graphs", ring_id)
}

async fn create_test_node(
    app: &Router,
    ring_id: &str,
    graph_id: &str,
    user_id: &str,
) -> serde_json::Value {
    let body = serde_json::json!({
        "label": "Test Node",
        "node_type": "concept",
        "parent_id": null,
        "description": "a test node"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}/{}/nodes", graph_base(ring_id), graph_id))
                .header("content-type", "application/json")
                .header("X-User-Id", user_id)
                .body(json_body(&body))
                .unwrap(),
        )
        .await
        .unwrap();
    body_to_json(resp.into_body()).await
}

#[tokio::test]
async fn create_and_get_node() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let node = create_test_node(&app, &ring_id, graph_id, &user_id).await;
    let node_id = node["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "{}/{}/nodes/{}",
                    graph_base(&ring_id),
                    graph_id,
                    node_id
                ))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["id"], node_id);
    assert_eq!(json["label"], "Test Node");
    assert_eq!(json["node_type"], "concept");
    assert_eq!(json["description"], "a test node");
}

#[tokio::test]
async fn update_node_label() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let node = create_test_node(&app, &ring_id, graph_id, &user_id).await;
    let node_id = node["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!(
                    "{}/{}/nodes/{}",
                    graph_base(&ring_id),
                    graph_id,
                    node_id
                ))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&serde_json::json!({
                    "label": "Updated Label",
                    "description": null,
                    "node_type": null
                })))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["label"], "Updated Label");
}

#[tokio::test]
async fn delete_node_returns_204_then_404() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let node = create_test_node(&app, &ring_id, graph_id, &user_id).await;
    let node_id = node["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!(
                    "{}/{}/nodes/{}",
                    graph_base(&ring_id),
                    graph_id,
                    node_id
                ))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "{}/{}/nodes/{}",
                    graph_base(&ring_id),
                    graph_id,
                    node_id
                ))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_edge_and_list_in_graph() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let n1 = create_test_node(&app, &ring_id, graph_id, &user_id).await;
    let n2 = create_test_node(&app, &ring_id, graph_id, &user_id).await;
    let n1_id = n1["id"].as_str().unwrap();
    let n2_id = n2["id"].as_str().unwrap();

    let edge_body = serde_json::json!({
        "source_id": n1_id,
        "target_id": n2_id,
        "relation": "related_to",
        "label": "test edge"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}/{}/edges", graph_base(&ring_id), graph_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&edge_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let edge = body_to_json(resp.into_body()).await;
    let edge_id = edge["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("{}/{}", graph_base(&ring_id), graph_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_to_json(resp.into_body()).await;
    let edges = json["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0]["id"], edge_id);
}

#[tokio::test]
async fn delete_edge_removes_from_graph() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let n1 = create_test_node(&app, &ring_id, graph_id, &user_id).await;
    let n2 = create_test_node(&app, &ring_id, graph_id, &user_id).await;
    let n1_id = n1["id"].as_str().unwrap();
    let n2_id = n2["id"].as_str().unwrap();

    let edge_body = serde_json::json!({
        "source_id": n1_id,
        "target_id": n2_id,
        "relation": "related_to",
        "label": "to delete"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}/{}/edges", graph_base(&ring_id), graph_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&edge_body))
                .unwrap(),
        )
        .await
        .unwrap();
    let edge = body_to_json(resp.into_body()).await;
    let edge_id = edge["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!(
                    "{}/{}/edges/{}",
                    graph_base(&ring_id),
                    graph_id,
                    edge_id
                ))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("{}/{}", graph_base(&ring_id), graph_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_to_json(resp.into_body()).await;
    let edges = json["edges"].as_array().unwrap();
    assert!(edges.is_empty());
}

#[tokio::test]
async fn get_children_of_node() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let parent_body = serde_json::json!({
        "label": "Parent",
        "node_type": "category",
        "parent_id": null,
        "description": null
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}/{}/nodes", graph_base(&ring_id), graph_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&parent_body))
                .unwrap(),
        )
        .await
        .unwrap();
    let parent = body_to_json(resp.into_body()).await;
    let parent_id = parent["id"].as_str().unwrap();

    let child_body = serde_json::json!({
        "label": "Child1",
        "node_type": "concept",
        "parent_id": parent_id,
        "description": null
    });
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}/{}/nodes", graph_base(&ring_id), graph_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&child_body))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("{}/{}", graph_base(&ring_id), graph_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_to_json(resp.into_body()).await;
    let nodes = json["nodes"].as_array().unwrap();
    let children: Vec<&serde_json::Value> = nodes
        .iter()
        .filter(|n| n["parent_id"].as_str() == Some(parent_id))
        .collect();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0]["label"], "Child1");
}

#[tokio::test]
async fn get_root_nodes_via_graph_detail() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let root_body = serde_json::json!({
        "label": "Root",
        "node_type": "category",
        "parent_id": null,
        "description": null
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}/{}/nodes", graph_base(&ring_id), graph_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&root_body))
                .unwrap(),
        )
        .await
        .unwrap();
    let root = body_to_json(resp.into_body()).await;
    let root_id = root["id"].as_str().unwrap();

    let child_body = serde_json::json!({
        "label": "Child",
        "node_type": "concept",
        "parent_id": root_id,
        "description": null
    });
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}/{}/nodes", graph_base(&ring_id), graph_id))
                .header("content-type", "application/json")
                .header("X-User-Id", &user_id)
                .body(json_body(&child_body))
                .unwrap(),
        )
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("{}/{}", graph_base(&ring_id), graph_id))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_to_json(resp.into_body()).await;
    let nodes = json["nodes"].as_array().unwrap();
    let roots: Vec<&serde_json::Value> =
        nodes.iter().filter(|n| n["parent_id"].is_null()).collect();
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0]["label"], "Root");
}

#[tokio::test]
async fn get_node_returns_content() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let node = create_test_node(&app, &ring_id, graph_id, &user_id).await;
    let node_id = node["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "{}/{}/nodes/{}/content",
                    graph_base(&ring_id),
                    graph_id,
                    node_id
                ))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_to_json(resp.into_body()).await;
    assert_eq!(json["node_id"], node_id);
    assert_eq!(json["label"], "Test Node");
    assert!(json["last_modified"].is_string());
}

#[tokio::test]
async fn create_node_empty_label_400() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let body = serde_json::json!({
        "label": "   ",
        "node_type": "concept",
        "parent_id": null,
        "description": null
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("{}/{}/nodes", graph_base(&ring_id), graph_id))
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
async fn get_nonexistent_node_404() {
    let (app, user_id) = create_test_app().await;
    let ring_id = create_ring(&app, &user_id).await;
    let graph_id = "graph-1";

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "{}/{}/nodes/nonexistent-node-id",
                    graph_base(&ring_id),
                    graph_id
                ))
                .header("X-User-Id", &user_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
