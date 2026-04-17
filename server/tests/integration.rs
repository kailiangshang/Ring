use axum::body::Body;
use axum::http::{Request, StatusCode};
use ring_server::routes::build_router;
use ring_server::state::AppState;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn setup_app() -> AppState {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory db");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    AppState::new(pool)
}

fn make_request(method: &str, uri: &str, body: Option<&str>, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json");
    if let Some(t) = token {
        builder = builder.header("X-Ring-Token", t);
    }
    let body = match body {
        Some(b) => Body::from(b.to_string()),
        None => Body::empty(),
    };
    builder.body(body).unwrap()
}

async fn read_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_health_check() {
    let state = setup_app().await;
    let app = build_router(state);

    let resp = app
        .oneshot(make_request("GET", "/api/health", None, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_setup_flow() {
    let state = setup_app().await;
    let app = build_router(state);

    let resp = app
        .clone()
        .oneshot(make_request("GET", "/api/setup/status", None, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["is_setup"], false);

    let setup_body = r#"{"display_name":"TestUser","avatar":"🧪","llm_provider":"openai","llm_api_key":"sk-test","llm_model":"gpt-4o","gitlab_url":"https://gitlab.test.com","gitlab_token":"glpat-test"}"#;
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/setup", Some(setup_body), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let json = read_body(resp).await;
    let token_id = json["token_id"].as_str().unwrap();
    assert!(!token_id.is_empty());
    assert_eq!(json["display_name"], "TestUser");

    let resp = app
        .oneshot(make_request("GET", "/api/setup/status", None, None))
        .await
        .unwrap();
    let json = read_body(resp).await;
    assert_eq!(json["is_setup"], true);
}

#[tokio::test]
async fn test_setup_duplicate_rejected() {
    let state = setup_app().await;
    let app = build_router(state);

    let setup_body = r#"{"display_name":"TestUser","llm_provider":"openai","llm_api_key":"sk-test","gitlab_url":"https://gitlab.test.com","gitlab_token":"glpat-test"}"#;
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/setup", Some(setup_body), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .oneshot(make_request("POST", "/api/setup", Some(setup_body), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_ring_crud() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let ring_body = r#"{"name":"Test Ring","role_description":"You are a test assistant"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/rings",
            Some(ring_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let json = read_body(resp).await;
    let ring_id = json["id"].as_str().unwrap();
    assert_eq!(json["role"], "creator");

    let resp = app
        .clone()
        .oneshot(make_request("GET", "/api/rings", None, Some(&token)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["rings"].as_array().unwrap().len(), 1);

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["name"], "Test Ring");
}

#[tokio::test]
async fn test_members_list() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let resp = app
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/members"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["members"].as_array().unwrap().len(), 1);
    assert_eq!(json["members"][0]["role"], "creator");
}

#[tokio::test]
async fn test_config_llm() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request("GET", "/api/config/llm", None, Some(&token)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["provider"], "openai");
    assert_eq!(json["api_key_set"], true);

    let update_body = r#"{"provider":"anthropic","model":"claude-sonnet-4-20250514"}"#;
    let resp = app
        .oneshot(make_request(
            "PUT",
            "/api/config/llm",
            Some(update_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["provider"], "anthropic");
}

async fn do_setup(app: &axum::Router) -> String {
    let setup_body = r#"{"display_name":"TestUser","llm_provider":"openai","llm_api_key":"sk-test","gitlab_url":"https://gitlab.test.com","gitlab_token":"glpat-test"}"#;
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/setup", Some(setup_body), None))
        .await
        .unwrap();
    let json = read_body(resp).await;
    json["token_id"].as_str().unwrap().to_string()
}

async fn create_ring(app: &axum::Router, token: &str) -> String {
    let ring_body = r#"{"name":"Test Ring","role_description":"Test"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/rings",
            Some(ring_body),
            Some(token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    json["id"].as_str().unwrap().to_string()
}
