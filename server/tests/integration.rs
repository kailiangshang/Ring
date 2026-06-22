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

    let rings_dir = std::path::PathBuf::from("/tmp/ring-test-rings");
    let hub_dir = std::path::PathBuf::from("/tmp/ring-test-hub");
    let skills_dir = std::path::PathBuf::from("/tmp/ring-test-skills");
    let _ = std::fs::create_dir_all(&rings_dir);
    let _ = std::fs::create_dir_all(&hub_dir);
    let _ = std::fs::create_dir_all(&skills_dir);

    AppState::new(pool, rings_dir, hub_dir, skills_dir)
}

async fn setup_unique_app() -> AppState {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory db");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let rings_dir = std::path::PathBuf::from(format!("/tmp/ring-test-rings-{id}"));
    let hub_dir = std::path::PathBuf::from(format!("/tmp/ring-test-hub-{id}"));
    let skills_dir = std::path::PathBuf::from(format!("/tmp/ring-test-skills-{id}"));
    let _ = std::fs::create_dir_all(&rings_dir);
    let _ = std::fs::create_dir_all(&hub_dir);
    let _ = std::fs::create_dir_all(&skills_dir);

    AppState::new(pool, rings_dir, hub_dir, skills_dir)
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

async fn create_second_user(pool: &SqlitePool) -> String {
    let token_id = format!(
        "user-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    sqlx::query(
        "INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, gitlab_url, gitlab_token)
         VALUES (?1, 'Bob', '🧑', 0, 'openai', 'sk-test', 'gpt-4o', 'https://gitlab.test.com', 'glpat-test')",
    )
    .bind(&token_id)
    .execute(pool)
    .await
    .unwrap();
    token_id
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

    let session_body = r#"{"title":"Auto Archive Test","skill":"discussion","archivable":true}"#;
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

#[tokio::test]
async fn test_add_member() {
    let state = setup_app().await;
    let pool = state.db.clone();
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;
    let bob_token = create_second_user(&pool).await;

    let add_body = &format!(r#"{{"user_id":"{bob_token}"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["token_id"], bob_token);
    assert_eq!(json["role"], "member");

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_add_member_forbidden() {
    let state = setup_app().await;
    let pool = state.db.clone();
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;
    let bob_token = create_second_user(&pool).await;

    let add_body = &format!(r#"{{"user_id":"{bob_token}"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&bob_token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_super_chat_history_empty() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/super/chat/history?limit=50",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["messages"].as_array().unwrap().len(), 0);
    assert_eq!(json["has_more"], false);
}

#[tokio::test]
async fn test_super_system_prompt_default() {
    let state = setup_unique_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/super/system-prompt",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["is_custom"], false);
}

async fn setup_unique_skills_app() -> (AppState, String) {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp = format!("/tmp/ring-skill-test-{id}");
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");
    let rings_dir = std::path::PathBuf::from(format!("{tmp}/rings"));
    let hub_dir = std::path::PathBuf::from(format!("{tmp}/hub"));
    let skills_dir = std::path::PathBuf::from(format!("{tmp}/skills"));
    std::fs::create_dir_all(&rings_dir).unwrap();
    std::fs::create_dir_all(&hub_dir).unwrap();
    std::fs::create_dir_all(&skills_dir).unwrap();
    let state = AppState::new(pool, rings_dir, hub_dir, skills_dir);
    (state, tmp)
}

#[tokio::test]
async fn test_skills_list_includes_builtins() {
    let (state, tmp) = setup_unique_skills_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let resp = app
        .clone()
        .oneshot(make_request("GET", "/api/skills", None, Some(&token)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    let skills = json["skills"].as_array().unwrap();
    assert!(skills.len() >= 5);
    let names: Vec<&str> = skills.iter().filter_map(|s| s["name"].as_str()).collect();
    assert!(names.contains(&"decision"));
    assert!(names.contains(&"research"));
    let _ = std::fs::remove_dir_all(std::path::Path::new(&tmp));
}

#[tokio::test]
async fn test_skill_detail_builtin() {
    let (state, tmp) = setup_unique_skills_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/skills/decision",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["name"], "decision");
    assert_eq!(json["source"], "builtin");
    assert!(json["content"].as_str().unwrap().contains("---"));
    let _ = std::fs::remove_dir_all(std::path::Path::new(&tmp));
}

#[tokio::test]
async fn test_skill_remove_builtin_rejected() {
    let (state, tmp) = setup_unique_skills_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let resp = app
        .clone()
        .oneshot(make_request(
            "DELETE",
            "/api/skills/decision",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let _ = std::fs::remove_dir_all(std::path::Path::new(&tmp));
}

#[tokio::test]
async fn test_skill_remove_nonexistent() {
    let (state, tmp) = setup_unique_skills_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let resp = app
        .clone()
        .oneshot(make_request(
            "DELETE",
            "/api/skills/nonexistent",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let _ = std::fs::remove_dir_all(std::path::Path::new(&tmp));
}

#[tokio::test]
async fn test_super_system_prompt_update() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let update_body = r#"{"prompt":"Custom prompt for testing"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            "/api/super/system-prompt",
            Some(update_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["is_custom"], true);
    assert_eq!(json["prompt"], "Custom prompt for testing");

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/super/system-prompt",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    assert_eq!(json["prompt"], "Custom prompt for testing");

    let reset_body = r#"{"prompt":""}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            "/api/super/system-prompt",
            Some(reset_body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    assert_eq!(json["is_custom"], false);
}

#[tokio::test]
async fn test_super_preferences_default() {
    let state = setup_unique_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/super/preferences",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["is_custom"], false);
    assert!(json["content"].as_str().unwrap().contains("zh-CN"));
    assert!(json["content"].as_str().unwrap().contains("openai"));
}

#[tokio::test]
async fn test_super_preferences_update() {
    let state = setup_unique_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let update_body = "{\"content\":\"## \u{8bed}\u{8a00}\\n- default: en\\n\\n## LLM\\n- default_provider: ollama\"}";
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            "/api/super/preferences",
            Some(update_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["is_custom"], true);
    assert!(json["content"].as_str().unwrap().contains("default: en"));

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/super/preferences",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    assert!(json["content"]
        .as_str()
        .unwrap()
        .contains("default_provider: ollama"));
}

#[tokio::test]
async fn test_super_preferences_reset() {
    let state = setup_unique_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let update_body = "{\"content\":\"## \u{8bed}\u{8a00}\\n- default: en\"}";
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            "/api/super/preferences",
            Some(update_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let reset_body = r#"{"content":""}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            "/api/super/preferences",
            Some(reset_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["is_custom"], false);
}

#[tokio::test]
async fn test_create_invite_token() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","role":"member","max_uses":0,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["type"], "open");
    assert_eq!(json["role"], "member");
    assert_eq!(json["max_uses"], 0);
    assert!(json["token"].as_str().unwrap().len() > 10);
}

#[tokio::test]
async fn test_list_invite_tokens() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open"}"#;
    let _ = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["tokens"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_revoke_invite_token() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"audit"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "DELETE",
            &format!("/api/rings/{ring_id}/invite-tokens/{invite_token}"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn test_invite_token_forbidden_for_member() {
    let state = setup_app().await;
    let pool = state.db.clone();
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;
    let bob_token = create_second_user(&pool).await;

    let add_body = &format!(r#"{{"user_id":"{bob_token}"}}"#);
    let _ = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&token),
        ))
        .await
        .unwrap();

    let body = r#"{"type":"open"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&bob_token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_join_info_valid() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/join/info?token={invite_token}"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["valid"], true);
    assert_eq!(json["ring_name"], "Test Ring");
    assert_eq!(json["role"], "member");
}

#[tokio::test]
async fn test_join_info_expired() {
    let state = setup_app().await;
    let pool = state.db.clone();
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let past = chrono::Utc::now() - chrono::Duration::hours(1);
    sqlx::query("UPDATE invite_tokens SET expires_at = ?1 WHERE token = ?2")
        .bind(past.to_rfc3339())
        .bind(invite_token)
        .execute(&pool)
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/join/info?token={invite_token}"),
            None,
            None,
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    assert_eq!(json["valid"], false);
    assert_eq!(json["reason"], "token expired");
}

#[tokio::test]
async fn test_join_info_not_found() {
    let state = setup_app().await;
    let app = build_router(state);
    let resp = app
        .oneshot(make_request(
            "GET",
            "/api/join/info?token=nonexistent",
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_join_ring_success() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let join_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Bob"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/join", Some(join_body), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert!(json["token_id"].as_str().unwrap().starts_with("user-"));
    assert_eq!(json["ring_name"], "Test Ring");
    assert_eq!(json["role"], "member");
}

#[tokio::test]
async fn test_join_ring_empty_name_rejected() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let join_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"  "}}"#);
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/join", Some(join_body), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_join_ring_revoked_token() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let _ = app
        .clone()
        .oneshot(make_request(
            "DELETE",
            &format!("/api/rings/{ring_id}/invite-tokens/{invite_token}"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();

    let join_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Bob"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/join", Some(join_body), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::GONE);
}

#[tokio::test]
async fn test_join_ring_max_uses_exhausted() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":1,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let join_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Alice"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/join", Some(join_body), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let join_body2 = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Bob"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/join", Some(join_body2), None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_audit_apply_success() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"audit","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let apply_body = &format!(
        r#"{{"invite_token":"{invite_token}","display_name":"Bob","message":"I want to join"}}"#
    );
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/join/apply",
            Some(apply_body),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert!(json["request_id"].as_str().unwrap().starts_with("req-"));
    assert_eq!(json["status"], "pending");
}

#[tokio::test]
async fn test_audit_apply_wrong_type() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let apply_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Bob"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/join/apply",
            Some(apply_body),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_audit_apply_status() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"audit","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let apply_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Bob"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/join/apply",
            Some(apply_body),
            None,
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let request_id = json["request_id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/join/apply/status?id={request_id}"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["status"], "pending");
}

#[tokio::test]
async fn test_audit_approve_flow() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"audit","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let apply_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Alice"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/join/apply",
            Some(apply_body),
            None,
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let request_id = json["request_id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/join-requests"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["requests"].as_array().unwrap().len(), 1);

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/join-requests/{request_id}/approve"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["ok"], true);
    assert!(json["token_id"].as_str().unwrap().starts_with("user-"));

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/join/apply/status?id={request_id}"),
            None,
            None,
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    assert_eq!(json["status"], "approved");
}

#[tokio::test]
async fn test_audit_reject_flow() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"audit","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let apply_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Eve"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/join/apply",
            Some(apply_body),
            None,
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let request_id = json["request_id"].as_str().unwrap();

    let reject_body = r#"{"note":"Team is full"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/join-requests/{request_id}/reject"),
            Some(reject_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["ok"], true);

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/join/apply/status?id={request_id}"),
            None,
            None,
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    assert_eq!(json["status"], "rejected");
    assert_eq!(json["review_note"], "Team is full");
}

#[tokio::test]
async fn test_audit_approve_forbidden_for_member() {
    let state = setup_app().await;
    let pool = state.db.clone();
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;
    let bob_token = create_second_user(&pool).await;

    let add_body = &format!(r#"{{"user_id":"{bob_token}"}}"#);
    let _ = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&token),
        ))
        .await
        .unwrap();

    let body = r#"{"type":"audit","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let apply_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Charlie"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/join/apply",
            Some(apply_body),
            None,
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let request_id = json["request_id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/join-requests/{request_id}/approve"),
            None,
            Some(&bob_token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

fn make_request_with_ua(
    method: &str,
    uri: &str,
    body: Option<&str>,
    token: Option<&str>,
    user_agent: &str,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Content-Type", "application/json")
        .header("User-Agent", user_agent);
    if let Some(t) = token {
        builder = builder.header("X-Ring-Token", t);
    }
    let body = match body {
        Some(b) => Body::from(b.to_string()),
        None => Body::empty(),
    };
    builder.body(body).unwrap()
}

async fn read_body_string(resp: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn test_join_page_valid_token() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    std::env::set_var("RING_DOWNLOAD_URL", "https://internal.example.com/ring");
    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/ring/join?token={invite_token}"),
            None,
            None,
        ))
        .await
        .unwrap();
    std::env::remove_var("RING_DOWNLOAD_URL");
    assert_eq!(resp.status(), StatusCode::OK);
    let html = read_body_string(resp).await;
    assert!(html.contains("Test Ring"));
    assert!(html.contains("Continue to Join"));
    assert!(html.contains("localhost:7420"));
    assert!(html.contains("ring-server-windows-amd64.zip"));
    assert!(html.contains("ring-server-macos-arm64.tar.gz"));
}

#[tokio::test]
async fn test_join_page_missing_token() {
    let state = setup_app().await;
    let app = build_router(state);

    let resp = app
        .clone()
        .oneshot(make_request("GET", "/ring/join", None, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let html = read_body_string(resp).await;
    assert!(html.contains("Missing Invite Token"));
}

#[tokio::test]
async fn test_join_page_expired_token() {
    let state = setup_app().await;
    let pool = state.db.clone();
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let past = "2020-01-01T00:00:00+00:00";
    sqlx::query("UPDATE invite_tokens SET expires_at = ?1 WHERE token = ?2")
        .bind(past)
        .bind(invite_token)
        .execute(&pool)
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/ring/join?token={invite_token}"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let html = read_body_string(resp).await;
    assert!(html.contains("Expired") || html.contains("expired"));
}

#[tokio::test]
async fn test_join_page_os_detection_windows() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    std::env::set_var("RING_DOWNLOAD_URL", "https://internal.example.com/ring");
    let req = make_request_with_ua(
        "GET",
        &format!("/ring/join?token={invite_token}"),
        None,
        None,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    std::env::remove_var("RING_DOWNLOAD_URL");
    let html = read_body_string(resp).await;
    assert!(html.contains("Recommended"));
    assert!(html.contains("Windows"));
}

#[tokio::test]
async fn test_join_page_os_detection_macos() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    std::env::set_var("RING_DOWNLOAD_URL", "https://internal.example.com/ring");
    let req = make_request_with_ua(
        "GET",
        &format!("/ring/join?token={invite_token}"),
        None,
        None,
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    std::env::remove_var("RING_DOWNLOAD_URL");
    let html = read_body_string(resp).await;
    assert!(html.contains("Recommended"));
    assert!(html.contains("macOS"));
}

#[tokio::test]
async fn test_join_page_audit_token() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"audit","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/ring/join?token={invite_token}"),
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let html = read_body_string(resp).await;
    assert!(html.contains("Test Ring"));
    assert!(html.contains("audit"));
}

#[tokio::test]
async fn test_join_page_nonexistent_token() {
    let state = setup_app().await;
    let app = build_router(state);

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/ring/join?token=nonexistent123",
            None,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let html = read_body_string(resp).await;
    assert!(html.contains("Invalid") || html.contains("error") || html.contains("Error"));
}

#[tokio::test]
async fn test_join_page_creator_ip_in_continue_link() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":48}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/invite-tokens"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let invite_token = json["token"].as_str().unwrap();

    let mut req = make_request(
        "GET",
        &format!("/ring/join?token={invite_token}"),
        None,
        None,
    );
    req.headers_mut()
        .insert("host", "192.168.1.100:7420".parse().unwrap());
    let resp = app.clone().oneshot(req).await.unwrap();
    let html = read_body_string(resp).await;
    assert!(html.contains("creator_ip=192.168.1.100"));
}

#[tokio::test]
async fn test_llm_test_endpoint_missing_key() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let body = r#"{"provider":"openai","model":"gpt-4o","api_key":null,"base_url":null}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/config/llm/test",
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_recover_token() {
    let state = setup_unique_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/setup/recover",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["token_id"], token);
}

#[tokio::test]
async fn test_recover_token_before_setup() {
    let state = setup_unique_app().await;
    let app = build_router(state);

    let resp = app
        .clone()
        .oneshot(make_request("GET", "/api/setup/recover", None, None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_gitlab_test_endpoint_invalid_url() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let body = r#"{"url":"http://not-a-real-gitlab.local","token":"fake"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/config/gitlab/test",
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["ok"], false);
}

#[tokio::test]
async fn test_self_identity_crud() {
    let _ = std::fs::remove_dir_all(std::path::PathBuf::from(
        std::env::var("HOME").unwrap_or_else(|_| ".".into()) + "/.ring/self",
    ));
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/self/identity",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["exists"], false);

    let body = r#"{"content":"I am a personal AI assistant"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            "/api/self/identity",
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/self/identity",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    assert_eq!(json["exists"], true);
    assert_eq!(json["content"], "I am a personal AI assistant");
}

#[tokio::test]
async fn test_auto_archive_toggle() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    // Get initial mode
    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/mode"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["auto_archive"], false);

    // Toggle auto_archive on
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/mode"),
            Some(r#"{"auto_archive":true}"#),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["auto_archive"], true);

    // Toggle auto_archive off
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/mode"),
            Some(r#"{"auto_archive":false}"#),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["auto_archive"], false);
}

#[tokio::test]
async fn test_search_index_upsert_and_query() {
    let state = setup_unique_app().await;
    let db = state.db.clone();
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    ring_server::services::search::upsert_search_index(
        &db,
        "graph_node",
        "node-1",
        &ring_id,
        "Test Ring",
        "API设计",
        "REST API with JWT authentication",
        "{}",
    )
    .await
    .unwrap();

    let ring_ids = ring_server::services::search::get_user_ring_ids(&db, &token)
        .await
        .unwrap();
    assert!(!ring_ids.is_empty());

    let results = ring_server::services::search::search_cross_ring(&db, &ring_ids, "API JWT", 10)
        .await
        .unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].source_type, "graph_node");
    assert_eq!(results[0].title, "API设计");
    assert_eq!(results[0].ring_name, "Test Ring");

    ring_server::services::search::delete_search_index(&db, "graph_node", "node-1")
        .await
        .unwrap();

    let results_after_delete =
        ring_server::services::search::search_cross_ring(&db, &ring_ids, "API JWT", 10)
            .await
            .unwrap();
    assert!(results_after_delete.is_empty());
}

#[tokio::test]
async fn test_file_upload_to_chat() {
    let state = setup_unique_app().await;
    let app = build_router(state.clone());
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let content = "Hello, this is a test file content.";
    let filename = "test.txt";

    ring_server::services::upload::validate_file(filename, content.len()).unwrap();

    let msgs = ring_server::services::upload::upload_to_chat(
        &state.db,
        Some(&ring_id),
        &token,
        "TestUser",
        filename,
        content.as_bytes(),
    )
    .await
    .unwrap();

    assert_eq!(msgs[0].role, "system");
    assert!(msgs[0].content.contains("test.txt"));
    assert!(msgs[0]
        .content
        .contains("Hello, this is a test file content."));

    let results = ring_server::services::search::search_cross_ring(
        &state.db,
        &[ring_id],
        "test file content",
        10,
    )
    .await
    .unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_file_upload_rejects_large_file() {
    let result = ring_server::services::upload::validate_file("big.txt", 11 * 1024 * 1024);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_file_upload_rejects_bad_extension() {
    let result = ring_server::services::upload::validate_file("evil.exe", 100);
    assert!(result.is_err());
}

fn make_self_dir() -> std::path::PathBuf {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ring-self-test-{id}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_record_tool_usage() {
    let dir = make_self_dir();
    ring_server::services::self_data::record_tool_usage(&dir, "search").unwrap();
    ring_server::services::self_data::record_tool_usage(&dir, "search").unwrap();
    ring_server::services::self_data::record_tool_usage(&dir, "upload").unwrap();

    let metrics = ring_server::services::self_data::read_metrics(&dir);
    let tools = &metrics["tool_usage"]["tools"];
    assert_eq!(tools["search"].as_i64().unwrap(), 2);
    assert_eq!(tools["upload"].as_i64().unwrap(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_record_dwell_heartbeat() {
    let dir = make_self_dir();
    ring_server::services::self_data::record_dwell_heartbeat(&dir, "ring_chat", 30).unwrap();
    ring_server::services::self_data::record_dwell_heartbeat(&dir, "ring_chat", 30).unwrap();

    let metrics = ring_server::services::self_data::read_metrics(&dir);
    let views = &metrics["dwell_time"]["views"];
    assert_eq!(views["ring_chat"].as_i64().unwrap(), 60);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_heartbeat_endpoint() {
    let state = setup_app().await;
    let app = build_router(state.clone());
    let token = do_setup(&app).await;

    let body = r#"{"view":"ring_chat"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/self/metrics/heartbeat",
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let buf = state.dwell_buffer.lock().await;
    let user_buf = buf.get(&token).expect("user buffer exists");
    assert_eq!(user_buf.get("ring_chat"), Some(&30));
}

#[tokio::test]
async fn test_heartbeat_invalid_view() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let body = r#"{"view":"invalid_view"}"#;
    let resp = app
        .oneshot(make_request(
            "POST",
            "/api/self/metrics/heartbeat",
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_blueprint_confirm_with_graphs() {
    let state = setup_app().await;
    let pool = state.db.clone();
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"blueprint":{"graphs":[{"name":"Test Graph","nodes":[{"label":"Root","node_type":"category","tags":[]},{"label":"Child","node_type":"topic","tags":["test"]}],"edges":[{"from":"Root","to":"Child","relation":"related_to"}]}]}}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/blueprint/confirm"),
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["status"], "confirmed");

    let graph_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM graphs WHERE ring_id = ?1")
        .bind(&ring_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(graph_count.0, 1);

    let node_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM graph_nodes WHERE ring_id = ?1")
        .bind(&ring_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(node_count.0, 2);

    let edge_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM graph_edges WHERE ring_id = ?1")
        .bind(&ring_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(edge_count.0, 1);

    let status: (String,) = sqlx::query_as("SELECT blueprint_status FROM rings WHERE id = ?1")
        .bind(&ring_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status.0, "confirmed");
}

#[tokio::test]
async fn test_blueprint_chat_requires_creator() {
    let state = setup_app().await;
    let pool = state.db.clone();
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;
    let bob_token = create_second_user(&pool).await;

    let add_body = &format!(r#"{{"user_id":"{bob_token}"}}"#);
    let _ = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/members"),
            Some(add_body),
            Some(&token),
        ))
        .await
        .unwrap();

    let chat_body = r#"{"content":"hello","current_blueprint":null}"#;
    let resp = app
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/blueprint/chat"),
            Some(chat_body),
            Some(&bob_token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_group_ring_tools_defined(_pool: SqlitePool) {
    let tools = ring_server::services::chat::get_group_ring_tools();
    assert_eq!(tools.len(), 3);

    let names: Vec<&str> = tools.iter().map(|t| t.function.name.as_str()).collect();
    assert!(names.contains(&"file_parse"));
    assert!(names.contains(&"knowledge_extract"));
    assert!(names.contains(&"fetch_url"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cross_ring_cache_summary_and_invalidation(pool: SqlitePool) {
    let cache = ring_server::services::cross_ring_cache::new_cache();

    let summary =
        ring_server::services::cross_ring_cache::get_summary(&cache, &pool, "nonexistent-user")
            .await;
    assert!(summary.contains("没有任何 Ring"));

    let map = cache.lock().await;
    assert!(map.contains_key("summary:nonexistent-user"));
    drop(map);

    ring_server::services::cross_ring_cache::invalidate_summary(&cache, "nonexistent-user").await;

    let map = cache.lock().await;
    assert!(!map.contains_key("summary:nonexistent-user"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cross_ring_cache_invalidate_ring(_pool: SqlitePool) {
    let cache = ring_server::services::cross_ring_cache::new_cache();

    let map = cache.lock().await;
    assert!(map.is_empty());
    drop(map);

    let mut map = cache.lock().await;
    map.insert(
        "detail:test-ring-id".to_string(),
        ("test data".to_string(), std::time::Instant::now()),
    );
    map.insert(
        "graph:test-ring-id".to_string(),
        ("graph data".to_string(), std::time::Instant::now()),
    );
    map.insert(
        "summary:user-1".to_string(),
        ("summary data".to_string(), std::time::Instant::now()),
    );
    drop(map);

    ring_server::services::cross_ring_cache::invalidate_ring(&cache, "test-ring-id").await;

    let map = cache.lock().await;
    assert!(!map.contains_key("detail:test-ring-id"));
    assert!(!map.contains_key("graph:test-ring-id"));
    assert!(map.contains_key("summary:user-1"));
}

#[tokio::test]
async fn test_export_node_path_traversal_blocked() {
    let state = setup_unique_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let body = r#"{"name":"Test Ring","role_description":"Test","storage_mode":"local"}"#;
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/rings", Some(body), Some(&token)))
        .await
        .unwrap();
    let ring_json = read_body(resp).await;
    let ring_id = ring_json["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!(
                "/api/rings/{}/export/node?node_id=../../../etc/passwd",
                ring_id
            ),
            None,
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_skills_auth_required() {
    let state = setup_unique_app().await;
    let app = build_router(state);

    let resp = app
        .clone()
        .oneshot(make_request("GET", "/api/skills", None, None))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_graph_node_delete_cascades_edges() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");

    let ring_id = "test-ring";
    sqlx::query("INSERT INTO users (token_id, display_name) VALUES ('test-user', 'Test')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO rings (id, name, storage_mode, creator_id) VALUES (?1, ?2, 'local', 'test-user')")
        .bind(ring_id)
        .bind("Test Ring")
        .execute(&pool)
        .await
        .unwrap();

    let graph = ring_server::models::graph::ensure_default_graph(&pool, ring_id)
        .await
        .unwrap();

    let node_a = ring_server::models::graph::create_node(
        &pool,
        "node-a",
        &graph.id,
        ring_id,
        &ring_server::models::graph::CreateNodeInput {
            label: "Node A".into(),
            graph_id: None,
            parent_id: None,
            node_type: "topic".into(),
            tags: vec![],
            content: "".into(),
            markdown_path: None,
            metadata: serde_json::json!({}),
        },
    )
    .await
    .unwrap();

    let node_b = ring_server::models::graph::create_node(
        &pool,
        "node-b",
        &graph.id,
        ring_id,
        &ring_server::models::graph::CreateNodeInput {
            label: "Node B".into(),
            graph_id: None,
            parent_id: None,
            node_type: "topic".into(),
            tags: vec![],
            content: "".into(),
            markdown_path: None,
            metadata: serde_json::json!({}),
        },
    )
    .await
    .unwrap();

    let node_c = ring_server::models::graph::create_node(
        &pool,
        "node-c",
        &graph.id,
        ring_id,
        &ring_server::models::graph::CreateNodeInput {
            label: "Node C".into(),
            graph_id: None,
            parent_id: None,
            node_type: "topic".into(),
            tags: vec![],
            content: "".into(),
            markdown_path: None,
            metadata: serde_json::json!({}),
        },
    )
    .await
    .unwrap();

    ring_server::models::graph::create_edge(
        &pool,
        "edge-ab",
        &graph.id,
        ring_id,
        &ring_server::models::graph::CreateEdgeInput {
            graph_id: None,
            source_id: node_a.id.clone(),
            target_id: node_b.id.clone(),
            relation: "related_to".into(),
            label: "A-B".into(),
        },
    )
    .await
    .unwrap();

    ring_server::models::graph::create_edge(
        &pool,
        "edge-ac",
        &graph.id,
        ring_id,
        &ring_server::models::graph::CreateEdgeInput {
            graph_id: None,
            source_id: node_a.id.clone(),
            target_id: node_c.id.clone(),
            relation: "related_to".into(),
            label: "A-C".into(),
        },
    )
    .await
    .unwrap();

    let edges_before = ring_server::models::graph::list_edges(&pool, &graph.id)
        .await
        .unwrap();
    assert_eq!(edges_before.len(), 2);

    ring_server::models::graph::delete_node(&pool, &node_a.id)
        .await
        .unwrap();

    let edges_after = ring_server::models::graph::list_edges(&pool, &graph.id)
        .await
        .unwrap();
    assert_eq!(edges_after.len(), 0);

    let nodes_after = ring_server::models::graph::list_nodes(&pool, &graph.id)
        .await
        .unwrap();
    assert_eq!(nodes_after.len(), 2);
    assert!(nodes_after.iter().all(|n| n.id != node_a.id));
}

#[tokio::test]
async fn test_graph_node_delete_without_edges() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");

    let ring_id = "test-ring-2";
    sqlx::query("INSERT INTO users (token_id, display_name) VALUES ('test-user-2', 'Test 2')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO rings (id, name, storage_mode, creator_id) VALUES (?1, ?2, 'local', 'test-user-2')")
        .bind(ring_id)
        .bind("Test Ring 2")
        .execute(&pool)
        .await
        .unwrap();

    let graph = ring_server::models::graph::ensure_default_graph(&pool, ring_id)
        .await
        .unwrap();

    ring_server::models::graph::create_node(
        &pool,
        "node-solo",
        &graph.id,
        ring_id,
        &ring_server::models::graph::CreateNodeInput {
            label: "Solo Node".into(),
            graph_id: None,
            parent_id: None,
            node_type: "topic".into(),
            tags: vec![],
            content: "".into(),
            markdown_path: None,
            metadata: serde_json::json!({}),
        },
    )
    .await
    .unwrap();

    let nodes_before = ring_server::models::graph::list_nodes(&pool, &graph.id)
        .await
        .unwrap();
    assert_eq!(nodes_before.len(), 1);

    ring_server::models::graph::delete_node(&pool, "node-solo")
        .await
        .unwrap();

    let nodes_after = ring_server::models::graph::list_nodes(&pool, &graph.id)
        .await
        .unwrap();
    assert_eq!(nodes_after.len(), 0);
}

#[tokio::test]
async fn test_graph_service_respects_explicit_graph_id() {
    let state = setup_unique_app().await;
    let ring_id = "graph-target-ring";

    sqlx::query("INSERT INTO users (token_id, display_name) VALUES ('graph-user', 'Graph User')")
        .execute(&state.db)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO rings (id, name, storage_mode, creator_id) VALUES (?1, ?2, 'local', 'graph-user')",
    )
    .bind(ring_id)
    .bind("Graph Target Ring")
    .execute(&state.db)
    .await
    .unwrap();

    let default_graph = ring_server::models::graph::ensure_default_graph(&state.db, ring_id)
        .await
        .unwrap();
    let alt_graph = ring_server::models::graph::create_graph(&state.db, ring_id, "secondary")
        .await
        .unwrap();

    let first_node = ring_server::services::graph::create_node(
        &state,
        ring_id,
        &ring_server::models::graph::CreateNodeInput {
            label: "API Gateway".into(),
            graph_id: Some(alt_graph.id.clone()),
            parent_id: None,
            node_type: "topic".into(),
            tags: vec!["gateway".into()],
            content: "".into(),
            markdown_path: None,
            metadata: serde_json::json!({}),
        },
    )
    .await
    .unwrap();
    assert_eq!(first_node.graph_id, alt_graph.id);

    let second_node = ring_server::services::graph::create_node(
        &state,
        ring_id,
        &ring_server::models::graph::CreateNodeInput {
            label: "Order Service".into(),
            graph_id: Some(alt_graph.id.clone()),
            parent_id: None,
            node_type: "topic".into(),
            tags: vec![],
            content: "".into(),
            markdown_path: None,
            metadata: serde_json::json!({}),
        },
    )
    .await
    .unwrap();
    assert_eq!(second_node.graph_id, alt_graph.id);

    let edge = ring_server::services::graph::create_edge(
        &state,
        ring_id,
        &ring_server::models::graph::CreateEdgeInput {
            graph_id: Some(alt_graph.id.clone()),
            source_id: first_node.id.clone(),
            target_id: second_node.id.clone(),
            relation: "related_to".into(),
            label: "calls".into(),
        },
    )
    .await
    .unwrap();
    assert_eq!(edge.graph_id, alt_graph.id);

    let default_nodes = ring_server::models::graph::list_nodes(&state.db, &default_graph.id)
        .await
        .unwrap();
    let alt_nodes = ring_server::models::graph::list_nodes(&state.db, &alt_graph.id)
        .await
        .unwrap();
    let default_edges = ring_server::models::graph::list_edges(&state.db, &default_graph.id)
        .await
        .unwrap();
    let alt_edges = ring_server::models::graph::list_edges(&state.db, &alt_graph.id)
        .await
        .unwrap();

    assert!(default_nodes.is_empty());
    assert!(default_edges.is_empty());
    assert_eq!(alt_nodes.len(), 2);
    assert_eq!(alt_edges.len(), 1);
}

#[tokio::test]
async fn test_graph_edge_update_route() {
    let state = setup_app().await;
    let app = build_router(state.clone());

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let graph = ring_server::models::graph::ensure_default_graph(&state.db, &ring_id)
        .await
        .unwrap();

    let node_a = ring_server::models::graph::create_node(
        &state.db,
        "node-a",
        &graph.id,
        &ring_id,
        &ring_server::models::graph::CreateNodeInput {
            label: "Node A".into(),
            graph_id: Some(graph.id.clone()),
            parent_id: None,
            node_type: "topic".into(),
            tags: vec![],
            content: "".into(),
            markdown_path: None,
            metadata: serde_json::json!({}),
        },
    )
    .await
    .unwrap();

    let node_b = ring_server::models::graph::create_node(
        &state.db,
        "node-b",
        &graph.id,
        &ring_id,
        &ring_server::models::graph::CreateNodeInput {
            label: "Node B".into(),
            graph_id: Some(graph.id.clone()),
            parent_id: None,
            node_type: "topic".into(),
            tags: vec![],
            content: "".into(),
            markdown_path: None,
            metadata: serde_json::json!({}),
        },
    )
    .await
    .unwrap();

    let edge = ring_server::models::graph::create_edge(
        &state.db,
        "edge-ab",
        &graph.id,
        &ring_id,
        &ring_server::models::graph::CreateEdgeInput {
            graph_id: Some(graph.id.clone()),
            source_id: node_a.id.clone(),
            target_id: node_b.id.clone(),
            relation: "related_to".into(),
            label: "".into(),
        },
    )
    .await
    .unwrap();

    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/graph/edges/{}", edge.id),
            Some(r#"{"relation":"depends_on","label":"A depends on B"}"#),
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["relation"], "depends_on");
    assert_eq!(json["label"], "A depends on B");

    let saved = ring_server::models::graph::list_edges(&state.db, &graph.id)
        .await
        .unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].relation, "depends_on");
    assert_eq!(saved[0].label, "A depends on B");
}
