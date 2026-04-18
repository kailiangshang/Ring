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

    AppState::new(pool, std::path::PathBuf::from("/tmp/ring-test-rings"))
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

#[tokio::test]
async fn test_session_crud() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let session_body = r#"{"title":"Test Session","skill":"discussion","archivable":true}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(session_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = read_body(resp).await;
    let session_id = json["id"].as_str().unwrap();
    assert_eq!(json["phase"], "discussion");
    assert_eq!(json["owner"], token);

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/sessions"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["sessions"].as_array().unwrap().len(), 1);

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/sessions/{session_id}"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["title"], "Test Session");
    assert_eq!(json["participants"].as_array().unwrap().len(), 1);

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/close"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["phase"], "closed");

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/reopen"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["phase"], "discussion");

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/close"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(make_request(
            "DELETE",
            &format!("/api/rings/{ring_id}/sessions/{session_id}"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["status"], "deleted");
}

#[tokio::test]
async fn test_session_single_active() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let session_body = r#"{"title":"First Session"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(session_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let session_body2 = r#"{"title":"Second Session"}"#;
    let resp = app
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(session_body2),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_session_archive_toggle() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let session_body = r#"{"title":"Archive Session","skill":"discussion","archivable":true}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(session_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = read_body(resp).await;
    let session_id = json["id"].as_str().unwrap();

    let archive_body = r#"{"enabled":true}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/archive-toggle"),
            Some(archive_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["archive_enabled"], true);
}

#[tokio::test]
async fn test_archive_repo_init() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/repo/status"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["initialized"], false);

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/repo/init"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["initialized"], true);
}

#[tokio::test]
async fn test_archive_list_empty() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/archives"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["archives"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_archive_queue_empty() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let resp = app
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/archive-queue"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["queue"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_close_session_triggers_auto_archive_check() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/mode"),
            Some(r#"{"interaction_mode":"auto"}"#),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let session_body =
        r#"{"title":"Auto Archive Test","skill":"discussion","archivable":true}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(session_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = read_body(resp).await;
    let session_id = json["id"].as_str().unwrap();

    let archive_body = r#"{"enabled":true}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/archive-toggle"),
            Some(archive_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/close"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["phase"], "closed");
}
