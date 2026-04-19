# Installation Navigation Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create `GET /ring/join?token=xxx` page that shows Ring info, OS-specific download links, and a "Continue to Join" button. Served by creator's ring-server as inline HTML.

**Architecture:** New route handler in `join_page.rs` generates HTML dynamically. Route registered outside `/api` nest, before `fallback_service`. Uses existing `verify_join_token()` service. OS detection from User-Agent header.

**Tech Stack:** Rust + Axum, inline HTML via `format!()`, no external dependencies needed.

---

### Task 1: Create join_page.rs with handler + HTML rendering

**Files:**
- Create: `server/src/routes/join_page.rs`

- [ ] **Step 1: Create join_page.rs**

Create `server/src/routes/join_page.rs` with the following content:

```rust
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse};
use serde::Deserialize;

use crate::error::Result;
use crate::services::invite;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct JoinPageQuery {
    pub token: Option<String>,
}

fn detect_os(user_agent: &str) -> &'static str {
    let ua = user_agent.to_lowercase();
    if ua.contains("windows") {
        "windows"
    } else if ua.contains("mac os x") || ua.contains("macos") {
        "macos-arm64"
    } else if ua.contains("linux") {
        "linux"
    } else {
        "unknown"
    }
}

fn extract_host_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.split(':').next().map(|s| s.to_string()))
}

fn render_error_page(title: &str, message: &str) -> Html<String> {
    let html = format!(
        r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title} - Ring</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ background: #0d1117; color: #e6edf3; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; display: flex; justify-content: center; align-items: center; min-height: 100vh; padding: 20px; }}
.container {{ max-width: 480px; width: 100%; text-align: center; }}
.error-icon {{ font-size: 48px; margin-bottom: 16px; }}
h1 {{ font-size: 20px; margin-bottom: 8px; font-weight: 600; }}
p {{ color: #8b949e; font-size: 14px; line-height: 1.5; }}
</style>
</head>
<body>
<div class="container">
<div class="error-icon">&#9888;</div>
<h1>{title}</h1>
<p>{message}</p>
</div>
</body>
</html>"##
    );
    Html(html)
}

fn render_join_page(
    ring_name: &str,
    member_count: i64,
    role: &str,
    token_type: &str,
    token: &str,
    host_ip: Option<String>,
    detected_os: &str,
) -> Html<String> {
    let creator_ip = host_ip.unwrap_or_default();
    let continue_url = format!(
        "http://localhost:7420/ring/join?token={}&creator_ip={}",
        token, creator_ip
    );

    let github_owner = std::env::var("RING_GITHUB_OWNER").unwrap_or_else(|_| "ring-hub".into());
    let base_url = format!(
        "https://github.com/{}/ring/releases/latest/download",
        github_owner
    );

    let platforms = [
        ("windows", "Windows", "ring-server-windows-amd64.zip"),
        ("macos-arm64", "macOS (Apple Silicon)", "ring-server-macos-arm64.tar.gz"),
        ("macos-amd64", "macOS (Intel)", "ring-server-macos-amd64.tar.gz"),
        ("linux", "Linux", "ring-server-linux-amd64.tar.gz"),
    ];

    let download_buttons = platforms
        .iter()
        .map(|(os_id, os_name, filename)| {
            let is_highlight = *os_id == detected_os;
            let bg = if is_highlight { "#238636" } else { "#21262d" };
            let border = if is_highlight { "#2ea043" } else { "#30363d" };
            let tag = if is_highlight {
                r#"<span class="recommended">Recommended</span>"#
            } else {
                ""
            };
            format!(
                r##"<a href="{base}/{filename}" class="download-btn" style="background:{bg};border-color:{border}"><span class="os-name">{os_name}</span>{tag}</a>"##
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let type_label = if token_type == "audit" {
        "audit link"
    } else {
        "open link"
    };

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Join {ring_name} - Ring</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ background: #0d1117; color: #e6edf3; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif; display: flex; justify-content: center; align-items: center; min-height: 100vh; padding: 20px; }}
.container {{ max-width: 480px; width: 100%; }}
.header {{ text-align: center; margin-bottom: 32px; }}
.logo {{ font-size: 32px; font-weight: 700; letter-spacing: -1px; color: #58a6ff; margin-bottom: 4px; }}
.type-badge {{ display: inline-block; padding: 2px 8px; background: #1f2937; border: 1px solid #30363d; border-radius: 12px; font-size: 12px; color: #8b949e; }}
.ring-info {{ text-align: center; margin-bottom: 32px; padding: 24px; background: #161b22; border: 1px solid #30363d; border-radius: 12px; }}
.ring-name {{ font-size: 24px; font-weight: 600; margin-bottom: 12px; }}
.ring-meta {{ color: #8b949e; font-size: 14px; }}
.ring-meta span {{ margin: 0 8px; }}
.steps {{ margin-bottom: 32px; }}
.step {{ display: flex; align-items: flex-start; margin-bottom: 16px; padding: 16px; background: #161b22; border: 1px solid #30363d; border-radius: 8px; }}
.step-num {{ flex-shrink: 0; width: 24px; height: 24px; background: #1f6feb; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 12px; font-weight: 600; margin-right: 12px; }}
.step-content {{ flex: 1; }}
.step-title {{ font-size: 14px; font-weight: 500; margin-bottom: 4px; }}
.step-desc {{ font-size: 12px; color: #8b949e; font-family: 'Cascadia Code', 'Fira Code', monospace; }}
.downloads {{ margin-bottom: 32px; }}
.downloads-title {{ font-size: 14px; font-weight: 500; margin-bottom: 12px; text-align: center; }}
.download-btn {{ display: block; padding: 10px 16px; margin-bottom: 8px; border: 1px solid #30363d; border-radius: 8px; text-decoration: none; color: #e6edf3; text-align: center; transition: opacity 0.2s; }}
.download-btn:hover {{ opacity: 0.85; }}
.os-name {{ font-size: 14px; }}
.recommended {{ display: inline-block; margin-left: 8px; padding: 1px 6px; background: #0d1117; border: 1px solid #2ea043; border-radius: 4px; font-size: 11px; color: #2ea043; }}
.continue-section {{ text-align: center; }}
.continue-btn {{ display: inline-block; padding: 12px 32px; background: #1f6feb; color: #fff; text-decoration: none; border-radius: 8px; font-size: 16px; font-weight: 500; transition: background 0.2s; }}
.continue-btn:hover {{ background: #388bfd; }}
.continue-hint {{ margin-top: 12px; font-size: 12px; color: #8b949e; }}
</style>
</head>
<body>
<div class="container">
<div class="header">
<div class="logo">Ring</div>
<div class="type-badge">{type_label}</div>
</div>
<div class="ring-info">
<div class="ring-name">{ring_name}</div>
<div class="ring-meta">
<span>{member_count} members</span>
<span>Role: {role}</span>
</div>
</div>
<div class="steps">
<div class="step">
<div class="step-num">1</div>
<div class="step-content">
<div class="step-title">Download Ring</div>
</div>
</div>
<div class="step">
<div class="step-num">2</div>
<div class="step-content">
<div class="step-title">Extract and run</div>
<div class="step-desc">./ring-server</div>
</div>
</div>
<div class="step">
<div class="step-num">3</div>
<div class="step-content">
<div class="step-title">Click the button below to join</div>
</div>
</div>
</div>
<div class="downloads">
<div class="downloads-title">Download for your platform</div>
{download_buttons}
</div>
<div class="continue-section">
<a href="{continue_url}" class="continue-btn">Continue to join "{ring_name}"</a>
<div class="continue-hint">Ring must be running locally before joining</div>
</div>
</div>
</body>
</html>"##
    );
    Html(html)
}

pub async fn join_page_handler(
    State(state): State<AppState>,
    Query(query): Query<JoinPageQuery>,
    headers: HeaderMap,
) -> axum::response::Response {
    let token = match query.token {
        Some(t) if !t.is_empty() => t,
        _ => {
            return render_error_page(
                "Missing invite token",
                "Please use a valid invite link.",
            )
            .into_response();
        }
    };

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let detected_os = detect_os(user_agent);
    let host_ip = extract_host_ip(&headers);

    match invite::verify_join_token(&state, &token).await {
        Ok(info) if info.valid => render_join_page(
            info.ring_name.as_deref().unwrap_or("Unknown"),
            info.member_count.unwrap_or(0),
            info.role.as_deref().unwrap_or("member"),
            info.token_type.as_deref().unwrap_or("open"),
            &token,
            host_ip,
            detected_os,
        )
        .into_response(),
        Ok(info) => {
            let reason = info.reason.as_deref().unwrap_or("This invite link is invalid");
            let title = if reason.contains("expired") {
                "Invite link has expired"
            } else if reason.contains("revoked") {
                "Invite link has been revoked"
            } else {
                "Invite link is invalid"
            };
            render_error_page(
                title,
                &format!("{}. Please contact the Ring creator for a new link.", reason),
            )
            .into_response()
        }
        Err(_) => render_error_page(
            "Server error",
            "Failed to verify invite link. Please try again later.",
        )
        .into_response(),
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles (may need import fixes)

- [ ] **Step 3: Commit**

```bash
git add server/src/routes/join_page.rs
git commit -m "Add join page handler with inline HTML rendering"
```

---

### Task 2: Register route in mod.rs

**Files:**
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Add module declaration and route**

In `server/src/routes/mod.rs`:

1. Add `mod join_page;` to the module declarations (after `mod invite;`)

2. Add `.route("/ring/join", get(join_page::join_page_handler))` in `build_router()` AFTER `.nest("/api", api)` and BEFORE `.fallback_service(...)`:

```rust
    Router::new()
        .nest("/api", api)
        .route("/ring/join", get(join_page::join_page_handler))
        .fallback_service(ServeDir::new(&static_dir).append_index_html_on_directories(true))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add server/src/routes/mod.rs
git commit -m "Register GET /ring/join route for install navigation page"
```

---

### Task 3: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add join page tests**

Add at the end of `server/tests/integration.rs`:

```rust
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
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );
    let body = axum::body::to_bytes(resp.into_body(), 10000)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Test Ring"));
    assert!(html.contains("Continue to join"));
    assert!(html.contains("localhost:7420"));
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
    let body = axum::body::to_bytes(resp.into_body(), 10000)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Missing invite token"));
}

#[tokio::test]
async fn test_join_page_expired_token() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","max_uses":10,"expires_in_hours":0}"#;
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
    let body = axum::body::to_bytes(resp.into_body(), 10000)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("expired") || html.contains("invalid"));
}

#[tokio::test]
async fn test_join_page_os_detection() {
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
    req.headers_mut().insert(
        "user-agent",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64)".parse().unwrap(),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 10000)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains("Recommended"));
    assert!(html.contains("Windows"));
}
```

- [ ] **Step 2: Run all tests**

Run: `cargo test` from `server/`
Expected: all tests pass (42 existing + 4 new = 46)

- [ ] **Step 3: Run fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings` from `server/`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "Add integration tests for join page navigation"
```

---

### Task 4: Final verification

- [ ] **Step 1: Run all backend tests**

Run: `cargo test` from `server/`
Expected: all tests pass

- [ ] **Step 2: Run fmt check + clippy**

Run: `cargo fmt --check && cargo clippy -- -D warnings` from `server/`
Expected: no errors
