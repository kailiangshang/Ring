# Open Link Join Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 3 new API endpoints for the open-link join flow: public token verification (creator server), public join execution (creator server), and local proxy join (joiner's server).

**Architecture:** Two-layer API. Creator's ring-server exposes 2 public endpoints (no auth, invite token serves as credential). Joiner's local ring-server exposes 1 proxy endpoint that calls creator's endpoints + clones repo. New `Gone` error variant for expired/revoked tokens.

**Tech Stack:** Rust + Axum + SQLite (sqlx), reqwest (HTTP client, already in Cargo.toml)

---

### Task 1: Add `Gone` error variant

**Files:**
- Modify: `server/src/error.rs`

- [ ] **Step 1: Add `Gone` variant to `RingError`**

Add after the `Conflict` variant in `server/src/error.rs` (around line 20):

```rust
    #[error("Gone: {0}")]
    Gone(String),

    #[error("Bad gateway")]
    BadGateway,
```

- [ ] **Step 2: Add `Gone` + `BadGateway` mappings in `IntoResponse`**

Add inside the `match &self` block in `IntoResponse` (after the `Conflict` arm, before `Internal`):

```rust
            RingError::Gone(msg) => (StatusCode::GONE, msg.clone()),
            RingError::BadGateway => (
                StatusCode::BAD_GATEWAY,
                "bad gateway".into(),
            ),
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add server/src/error.rs
git commit -m "Add Gone error variant to RingError"
```

---

### Task 2: Add new model queries for join flow

**Files:**
- Modify: `server/src/models/invite.rs`
- Modify: `server/src/models/user.rs`
- Modify: `server/src/models/member.rs`

- [ ] **Step 1: Add `find_token_by_value` to `models/invite.rs`**

Add at the end of `server/src/models/invite.rs`:

```rust
pub async fn find_token_by_value(
    pool: &sqlx::SqlitePool,
    token: &str,
) -> Result<Option<InviteTokenRow>> {
    sqlx::query_as::<_, InviteTokenRow>(
        "SELECT * FROM invite_tokens WHERE token = ?1",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

pub async fn increment_use_count(pool: &sqlx::SqlitePool, token: &str) -> Result<()> {
    sqlx::query("UPDATE invite_tokens SET use_count = use_count + 1 WHERE token = ?1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_member_count(pool: &sqlx::SqlitePool, ring_id: &str) -> Result<i64> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM members WHERE ring_id = ?1")
            .bind(ring_id)
            .fetch_one(pool)
            .await?;
    Ok(count)
}
```

- [ ] **Step 2: Add `create_joiner_user` to `models/user.rs`**

Add at the end of `server/src/models/user.rs`:

```rust
pub async fn create_joiner_user(
    pool: &sqlx::SqlitePool,
    token_id: &str,
    display_name: &str,
) -> Result<UserRow> {
    sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, llm_base_url, gitlab_url, gitlab_token)
         VALUES (?1, ?2, NULL, 0, 'openai', NULL, 'gpt-4o', NULL, '', '')
         RETURNING *",
    )
    .bind(token_id)
    .bind(display_name)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}
```

- [ ] **Step 3: Add `is_member` check to `models/member.rs`**

Add at the end of `server/src/models/member.rs`:

```rust
pub async fn is_member(pool: &sqlx::SqlitePool, ring_id: &str, user_id: &str) -> bool {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM members WHERE ring_id = ?1 AND user_id = ?2)",
    )
    .bind(ring_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    exists
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles

- [ ] **Step 5: Commit**

```bash
git add server/src/models/invite.rs server/src/models/user.rs server/src/models/member.rs
git commit -m "Add join flow model queries: find_token, increment_use_count, create_joiner_user, is_member"
```

---

### Task 3: Add join service functions

**Files:**
- Modify: `server/src/services/invite.rs`

- [ ] **Step 1: Add imports and join service functions**

Add to the imports at the top of `server/src/services/invite.rs`:

```rust
use serde::{Deserialize, Serialize};
use ulid::Ulid;
```

Add these structs and functions at the end of `server/src/services/invite.rs`:

```rust
#[derive(Debug, Serialize)]
pub struct JoinInfoResponse {
    pub valid: bool,
    pub reason: Option<String>,
    pub ring_id: Option<String>,
    pub ring_name: Option<String>,
    pub member_count: Option<i64>,
    pub role: Option<String>,
    pub token_type: Option<String>,
}

pub async fn verify_join_token(state: &AppState, token_str: &str) -> Result<JoinInfoResponse> {
    let row = invite::find_token_by_value(&state.db, token_str)
        .await?
        .ok_or_else(|| RingError::NotFound("invite token not found".into()))?;

    if row.revoked_at.is_some() {
        return Ok(JoinInfoResponse {
            valid: false,
            reason: Some("token revoked".into()),
            ring_id: None,
            ring_name: None,
            member_count: None,
            role: None,
            token_type: None,
        });
    }

    let now = Utc::now().to_rfc3339();
    if row.expires_at < now {
        return Ok(JoinInfoResponse {
            valid: false,
            reason: Some("token expired".into()),
            ring_id: None,
            ring_name: None,
            member_count: None,
            role: None,
            token_type: None,
        });
    }

    let ring: Option<(String,)> =
        sqlx::query_as("SELECT name FROM rings WHERE id = ?1")
            .bind(&row.ring_id)
            .fetch_optional(&state.db)
            .await?;

    let ring_name = ring.map(|r| r.0).unwrap_or_default();
    let member_count = invite::get_member_count(&state.db, &row.ring_id).await.unwrap_or(0);

    Ok(JoinInfoResponse {
        valid: true,
        reason: None,
        ring_id: Some(row.ring_id.clone()),
        ring_name: Some(ring_name),
        member_count: Some(member_count),
        role: Some(row.role.clone()),
        token_type: Some(row.r#type.clone()),
    })
}

#[derive(Debug, Deserialize)]
pub struct JoinRequest {
    pub invite_token: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct JoinResponse {
    pub token_id: String,
    pub ring_id: String,
    pub ring_name: String,
    pub role: String,
    pub gitlab_repo_url: Option<String>,
}

pub async fn execute_join(state: &AppState, input: &JoinRequest) -> Result<JoinResponse> {
    if input.display_name.trim().is_empty() {
        return Err(RingError::BadRequest("display_name is required".into()));
    }

    let row = invite::find_token_by_value(&state.db, &input.invite_token)
        .await?
        .ok_or_else(|| RingError::NotFound("invite token not found".into()))?;

    if row.r#type != "open" {
        return Err(RingError::BadRequest(
            "this token is not an open invite".into(),
        ));
    }

    if row.revoked_at.is_some() {
        return Err(RingError::Gone("token has been revoked".into()));
    }

    let now = Utc::now().to_rfc3339();
    if row.expires_at < now {
        return Err(RingError::Gone("token has expired".into()));
    }

    if row.max_uses > 0 && row.use_count >= row.max_uses {
        return Err(RingError::Forbidden("token has reached max uses".into()));
    }

    if let Some(max) = row.max_members {
        let count = invite::get_member_count(&state.db, &row.ring_id).await?;
        if count >= max {
            return Err(RingError::Forbidden("ring has reached max members".into()));
        }
    }

    let token_id = format!("user-{}", Ulid::new());

    if crate::models::member::is_member(&state.db, &row.ring_id, &token_id).await {
        return Err(RingError::Conflict("already a member".into()));
    }

    crate::models::user::create_joiner_user(&state.db, &token_id, &input.display_name).await?;

    crate::models::member::add_member(&state.db, &row.ring_id, &token_id, &row.role).await?;

    invite::increment_use_count(&state.db, &input.invite_token).await?;

    let ring_name: String =
        sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
            .bind(&row.ring_id)
            .fetch_one(&state.db)
            .await
            .unwrap_or_default();

    let gitlab_repo_url: Option<String> =
        sqlx::query_scalar("SELECT gitlab_repo_url FROM rings WHERE id = ?1")
            .bind(&row.ring_id)
            .fetch_optional(&state.db)
            .await?
            .flatten();

    Ok(JoinResponse {
        token_id,
        ring_id: row.ring_id,
        ring_name,
        role: row.role,
        gitlab_repo_url,
    })
}

#[derive(Debug, Deserialize)]
pub struct LocalJoinRequest {
    pub invite_token: String,
    pub creator_ip: String,
}

pub async fn local_join(
    state: &AppState,
    user_id: &str,
    input: &LocalJoinRequest,
) -> Result<serde_json::Value> {
    let base_url = format!("http://{}:7420/api", input.creator_ip);

    let info_url = format!("{}/join/info?token={}", base_url, input.invite_token);
    let info_resp = reqwest::get(&info_url)
        .await
        .map_err(|e| RingError::Internal(format!("failed to contact creator: {e}")))?;

    if !info_resp.status().is_success() {
        return Err(RingError::BadGateway);
    }

    let info: serde_json::Value = info_resp
        .json()
        .await
        .map_err(|e| RingError::Internal(format!("failed to parse creator response: {e}")))?;

    if !info["valid"].as_bool().unwrap_or(false) {
        let reason = info["reason"].as_str().unwrap_or("unknown");
        return Err(RingError::BadRequest(format!("invite invalid: {reason}")));
    }

    let user = crate::models::user::get_user(&state.db, user_id).await?;

    let join_url = format!("{}/join", base_url);
    let join_body = serde_json::json!({
        "invite_token": input.invite_token,
        "display_name": user.display_name,
    });
    let join_resp = reqwest::Client::new()
        .post(&join_url)
        .json(&join_body)
        .send()
        .await
        .map_err(|e| RingError::Internal(format!("failed to join via creator: {e}")))?;

    if !join_resp.status().is_success() {
        let status = join_resp.status();
        let body: serde_json::Value = join_resp.json().await.unwrap_or(json!({}));
        let msg = body["error"]["message"].as_str().unwrap_or("unknown error");
        return Err(RingError::Internal(format!(
            "creator join failed ({status}): {msg}"
        )));
    }

    let join_result: serde_json::Value = join_resp
        .json()
        .await
        .map_err(|e| RingError::Internal(format!("failed to parse join result: {e}")))?;

    if let Some(repo_url) = join_result["gitlab_repo_url"].as_str() {
        if !repo_url.is_empty() {
            let rings_dir = state.rings_dir.clone();
            let ring_id = join_result["ring_id"].as_str().unwrap_or_default().to_string();
            let repo_url = repo_url.to_string();
            tokio::spawn(async move {
                let repo_path = rings_dir.join(&ring_id);
                if !repo_path.join(".git").exists() {
                    if let Err(e) = crate::services::git_service::GitService::clone(&repo_url, &repo_path) {
                        tracing::warn!("git clone failed for ring {ring_id}: {e}");
                        return;
                    }
                    let _ = std::fs::create_dir_all(repo_path.join("archives"));
                    let _ = std::fs::create_dir_all(repo_path.join("graphs"));
                    let _ = std::fs::create_dir_all(repo_path.join(".group"));
                    tracing::info!("git clone completed: ring={ring_id}");
                }
            });
        }
    }

    Ok(join_result)
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add server/src/services/invite.rs
git commit -m "Add join service: verify_join_token, execute_join, local_join"
```

---

### Task 4: Add join route handlers + register routes

**Files:**
- Modify: `server/src/routes/invite.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Add new request structs and handlers to `routes/invite.rs`**

Add these imports at the top of `server/src/routes/invite.rs` (merge with existing imports):

```rust
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::invite::CreateInviteToken;
use crate::services::invite;
use crate::state::AppState;
```

Add these new structs and handlers at the end of `server/src/routes/invite.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct JoinInfoQuery {
    pub token: String,
}

pub async fn join_info(
    State(state): State<AppState>,
    Query(query): Query<JoinInfoQuery>,
) -> Result<Json<serde_json::Value>> {
    let info = invite::verify_join_token(&state, &query.token).await?;
    Ok(Json(json!({
        "valid": info.valid,
        "reason": info.reason,
        "ring_id": info.ring_id,
        "ring_name": info.ring_name,
        "member_count": info.member_count,
        "role": info.role,
        "token_type": info.token_type,
    })))
}

#[derive(Debug, Deserialize)]
pub struct JoinBody {
    pub invite_token: String,
    pub display_name: String,
}

pub async fn join_ring(
    State(state): State<AppState>,
    Json(body): Json<JoinBody>,
) -> Result<Json<serde_json::Value>> {
    let req = invite::JoinRequest {
        invite_token: body.invite_token,
        display_name: body.display_name,
    };
    let result = invite::execute_join(&state, &req).await?;
    Ok(Json(json!({
        "token_id": result.token_id,
        "ring_id": result.ring_id,
        "ring_name": result.ring_name,
        "role": result.role,
        "gitlab_repo_url": result.gitlab_repo_url,
    })))
}

#[derive(Debug, Deserialize)]
pub struct LocalJoinBody {
    pub invite_token: String,
    pub creator_ip: String,
}

pub async fn local_join_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<LocalJoinBody>,
) -> Result<Json<serde_json::Value>> {
    let req = invite::LocalJoinRequest {
        invite_token: body.invite_token,
        creator_ip: body.creator_ip,
    };
    let result = invite::local_join(&state, &user.token_id, &req).await?;
    Ok(result)
}
```

- [ ] **Step 2: Register new routes in `routes/mod.rs`**

Add these 3 routes AFTER the existing invite-tokens routes and BEFORE `.with_state(state)`:

```rust
        .route("/join/info", get(invite::join_info))
        .route("/join", post(invite::join_ring))
        .route("/join/local", post(invite::local_join_handler))
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/invite.rs server/src/routes/mod.rs
git commit -m "Add join API routes: join_info, join_ring, local_join"
```

---

### Task 5: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add join flow tests**

Add at the end of `server/tests/integration.rs`:

```rust
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
    let app = build_router(state);
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let body = r#"{"type":"open","expires_in_hours":0}"#;
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
```

- [ ] **Step 2: Run all tests**

Run: `cargo test` from `server/`
Expected: all tests pass (29 existing + ~7 new)

- [ ] **Step 3: Run fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings` from `server/`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "Add integration tests for open link join flow"
```

---

### Task 6: Final verification

- [ ] **Step 1: Run all backend tests**

Run: `cargo test` from `server/`
Expected: all tests pass

- [ ] **Step 2: Run fmt check + clippy**

Run: `cargo fmt --check && cargo clippy -- -D warnings` from `server/`
Expected: no errors

- [ ] **Step 3: Frontend build**

Run: `cd ui && npm run build`
Expected: build succeeds
