# Audit Link + Approval Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add join_requests table and 5 API endpoints for audit link join flow: submit application, poll status, list requests, approve, reject.

**Architecture:** New migration for join_requests table. Reuse open join's verify + create_joiner_user logic. Apply is public (no auth), approve/reject require creator/admin. Polling for status checks.

**Tech Stack:** Rust + Axum + SQLite (sqlx), ulid (already in Cargo.toml)

---

### Task 1: Create migration for join_requests table

**Files:**
- Create: `server/migrations/008_join_requests.sql`

- [ ] **Step 1: Create migration file**

Create `server/migrations/008_join_requests.sql`:

```sql
CREATE TABLE join_requests (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id),
    invite_token TEXT NOT NULL REFERENCES invite_tokens(token),
    display_name TEXT NOT NULL,
    message TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'approved', 'rejected')),
    reviewer_id TEXT REFERENCES users(token_id),
    review_note TEXT,
    reviewed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add server/migrations/008_join_requests.sql
git commit -m "Add join_requests migration for audit approval flow"
```

---

### Task 2: Add join_requests model queries

**Files:**
- Modify: `server/src/models/invite.rs`

- [ ] **Step 1: Add JoinRequestRow struct and queries**

Add at the END of `server/src/models/invite.rs`:

```rust
#[derive(Debug, FromRow, Serialize)]
pub struct JoinRequestRow {
    pub id: String,
    pub ring_id: String,
    pub invite_token: String,
    pub display_name: String,
    pub message: Option<String>,
    pub status: String,
    pub reviewer_id: Option<String>,
    pub review_note: Option<String>,
    pub reviewed_at: Option<String>,
    pub created_at: String,
}

pub async fn insert_join_request(pool: &sqlx::SqlitePool, row: &JoinRequestRow) -> Result<()> {
    sqlx::query(
        "INSERT INTO join_requests (id, ring_id, invite_token, display_name, message, status, reviewer_id, review_note, reviewed_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind(&row.id)
    .bind(&row.ring_id)
    .bind(&row.invite_token)
    .bind(&row.display_name)
    .bind(&row.message)
    .bind(&row.status)
    .bind(&row.reviewer_id)
    .bind(&row.review_note)
    .bind(&row.reviewed_at)
    .bind(&row.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_join_request(pool: &sqlx::SqlitePool, id: &str) -> Result<Option<JoinRequestRow>> {
    sqlx::query_as::<_, JoinRequestRow>(
        "SELECT * FROM join_requests WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

pub async fn list_pending_requests(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    status_filter: &str,
) -> Result<Vec<JoinRequestRow>> {
    let query = if status_filter == "all" {
        "SELECT * FROM join_requests WHERE ring_id = ?1 ORDER BY created_at DESC".to_string()
    } else {
        "SELECT * FROM join_requests WHERE ring_id = ?1 AND status = 'pending' ORDER BY created_at DESC".to_string()
    };
    let rows = sqlx::query_as::<_, JoinRequestRow>(&query)
        .bind(ring_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn update_join_request_status(
    pool: &sqlx::SqlitePool,
    id: &str,
    status: &str,
    reviewer_id: &str,
    review_note: Option<&str>,
) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE join_requests SET status = ?1, reviewer_id = ?2, review_note = ?3, reviewed_at = datetime('now') WHERE id = ?4 AND status = 'pending'",
    )
    .bind(status)
    .bind(reviewer_id)
    .bind(review_note)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM join_requests WHERE id = ?1)",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        if !exists {
            return Err(RingError::NotFound("join request not found".into()));
        }
        return Err(RingError::Conflict("request is not pending".into()));
    }
    Ok(true)
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add server/src/models/invite.rs
git commit -m "Add join_requests model: JoinRequestRow + CRUD queries"
```

---

### Task 3: Add approval service functions

**Files:**
- Modify: `server/src/services/invite.rs`

- [ ] **Step 1: Add 4 new service functions**

Add at the END of `server/src/services/invite.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct ApplyRequest {
    pub invite_token: String,
    pub display_name: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApplyResponse {
    pub request_id: String,
    pub status: String,
    pub ring_name: String,
}

pub async fn submit_apply(state: &AppState, input: &ApplyRequest) -> Result<ApplyResponse> {
    if input.display_name.trim().is_empty() {
        return Err(RingError::BadRequest("display_name is required".into()));
    }

    let row = invite::find_token_by_value(&state.db, &input.invite_token)
        .await?
        .ok_or_else(|| RingError::NotFound("invite token not found".into()))?;

    if row.r#type != "audit" {
        return Err(RingError::BadRequest(
            "this token is not an audit invite".into(),
        ));
    }

    if row.revoked_at.is_some() {
        return Err(RingError::Gone("token has been revoked".into()));
    }

    let now = Utc::now().to_rfc3339();
    if row.expires_at < now {
        return Err(RingError::Gone("token has expired".into()));
    }

    let request_id = format!("req-{}", Ulid::new());
    let ring_name: String = sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
        .bind(&row.ring_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    let req_row = invite::JoinRequestRow {
        id: request_id.clone(),
        ring_id: row.ring_id.clone(),
        invite_token: input.invite_token.clone(),
        display_name: input.display_name.clone(),
        message: input.message.clone(),
        status: "pending".to_string(),
        reviewer_id: None,
        review_note: None,
        reviewed_at: None,
        created_at: now,
    };

    invite::insert_join_request(&state.db, &req_row).await?;

    Ok(ApplyResponse {
        request_id,
        status: "pending".to_string(),
        ring_name,
    })
}

#[derive(Debug, Serialize)]
pub struct ApplyStatusResponse {
    pub request_id: String,
    pub status: String,
    pub ring_name: Option<String>,
    pub ring_id: Option<String>,
    pub role: Option<String>,
    pub review_note: Option<String>,
    pub token_id: Option<String>,
}

pub async fn check_apply_status(
    state: &AppState,
    request_id: &str,
) -> Result<ApplyStatusResponse> {
    let req = invite::find_join_request(&state.db, request_id)
        .await?
        .ok_or_else(|| RingError::NotFound("join request not found".into()))?;

    let ring_name: String = sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
        .bind(&req.ring_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    let (ring_id, role, token_id) = if req.status == "approved" {
        let token_row = invite::find_token_by_value(&state.db, &req.invite_token)
            .await?
            .map(|t| t.role.clone())
            .unwrap_or_default();
        let user_id: Option<String> = sqlx::query_scalar(
            "SELECT user_id FROM members WHERE ring_id = ?1 AND user_id IN (SELECT token_id FROM users WHERE display_name = ?2) LIMIT 1",
        )
        .bind(&req.ring_id)
        .bind(&req.display_name)
        .fetch_optional(&state.db)
        .await?;
        (Some(req.ring_id.clone()), Some(token_row), user_id)
    } else {
        (None, None, None)
    };

    Ok(ApplyStatusResponse {
        request_id: req.id,
        status: req.status,
        ring_name: Some(ring_name),
        ring_id,
        role,
        review_note: req.review_note,
        token_id,
    })
}

pub async fn list_join_requests(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    status_filter: &str,
) -> Result<Vec<invite::JoinRequestRow>> {
    check_admin(&state.db, ring_id, user_id).await?;
    invite::list_pending_requests(&state.db, ring_id, status_filter).await
}

pub async fn approve_join_request(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    request_id: &str,
) -> Result<serde_json::Value> {
    check_admin(&state.db, ring_id, user_id).await?;

    let req = invite::find_join_request(&state.db, request_id)
        .await?
        .ok_or_else(|| RingError::NotFound("join request not found".into()))?;

    if req.ring_id != ring_id {
        return Err(RingError::NotFound("join request not found".into()));
    }

    if req.status != "pending" {
        return Err(RingError::Conflict("request is not pending".into()));
    }

    let token_row = invite::find_token_by_value(&state.db, &req.invite_token)
        .await?
        .ok_or_else(|| RingError::Gone("invite token no longer exists".into()))?;

    if token_row.revoked_at.is_some() {
        return Err(RingError::Gone("token has been revoked".into()));
    }

    let now = Utc::now().to_rfc3339();
    if token_row.expires_at < now {
        return Err(RingError::Gone("token has expired".into()));
    }

    if token_row.max_uses > 0 && token_row.use_count >= token_row.max_uses {
        return Err(RingError::Forbidden("token has reached max uses".into()));
    }

    if let Some(max) = token_row.max_members {
        let count = invite::get_member_count(&state.db, &token_row.ring_id).await?;
        if count >= max {
            return Err(RingError::Forbidden("ring has reached max members".into()));
        }
    }

    let new_token_id = format!("user-{}", Ulid::new());

    crate::models::user::create_joiner_user(&state.db, &new_token_id, &req.display_name).await?;

    crate::models::member::add_member(&state.db, ring_id, &new_token_id, &token_row.role).await?;

    invite::increment_use_count(&state.db, &req.invite_token).await?;

    invite::update_join_request_status(&state.db, request_id, "approved", user_id, None).await?;

    let ring_name: String = sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
        .bind(ring_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    Ok(serde_json::json!({
        "ok": true,
        "token_id": new_token_id,
        "ring_name": ring_name,
        "role": token_row.role,
    }))
}

pub async fn reject_join_request(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    request_id: &str,
    note: Option<&str>,
) -> Result<serde_json::Value> {
    check_admin(&state.db, ring_id, user_id).await?;

    let req = invite::find_join_request(&state.db, request_id)
        .await?
        .ok_or_else(|| RingError::NotFound("join request not found".into()))?;

    if req.ring_id != ring_id {
        return Err(RingError::NotFound("join request not found".into()));
    }

    invite::update_join_request_status(&state.db, request_id, "rejected", user_id, note).await?;

    Ok(serde_json::json!({ "ok": true }))
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add server/src/services/invite.rs
git commit -m "Add approval service: submit_apply, check_status, approve, reject"
```

---

### Task 4: Add approval route handlers + register routes

**Files:**
- Modify: `server/src/routes/invite.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Add new handlers to `routes/invite.rs`**

Add at the END of `server/src/routes/invite.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct ApplyBody {
    pub invite_token: String,
    pub display_name: String,
    pub message: Option<String>,
}

pub async fn apply_join(
    State(state): State<AppState>,
    Json(body): Json<ApplyBody>,
) -> Result<Json<serde_json::Value>> {
    let req = invite::ApplyRequest {
        invite_token: body.invite_token,
        display_name: body.display_name,
        message: body.message,
    };
    let result = invite::submit_apply(&state, &req).await?;
    Ok(Json(json!({
        "request_id": result.request_id,
        "status": result.status,
        "ring_name": result.ring_name,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ApplyStatusQuery {
    pub id: String,
}

pub async fn apply_status(
    State(state): State<AppState>,
    Query(query): Query<ApplyStatusQuery>,
) -> Result<Json<serde_json::Value>> {
    let result = invite::check_apply_status(&state, &query.id).await?;
    Ok(Json(json!({
        "request_id": result.request_id,
        "status": result.status,
        "ring_name": result.ring_name,
        "ring_id": result.ring_id,
        "role": result.role,
        "review_note": result.review_note,
        "token_id": result.token_id,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ListRequestsQuery {
    #[serde(default)]
    pub status: Option<String>,
}

pub async fn list_join_requests_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<ListRequestsQuery>,
) -> Result<Json<serde_json::Value>> {
    let status_filter = query.status.as_deref().unwrap_or("pending");
    let requests = invite::list_join_requests(&state, &ring_id, &user.token_id, status_filter).await?;
    Ok(Json(json!({ "requests": requests })))
}

pub async fn approve_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, request_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let result = invite::approve_join_request(&state, &ring_id, &user.token_id, &request_id).await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct RejectBody {
    pub note: Option<String>,
}

pub async fn reject_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, request_id)): Path<(String, String)>,
    Json(body): Json<RejectBody>,
) -> Result<Json<serde_json::Value>> {
    let result = invite::reject_join_request(
        &state,
        &ring_id,
        &user.token_id,
        &request_id,
        body.note.as_deref(),
    )
    .await?;
    Ok(Json(result))
}
```

- [ ] **Step 2: Register new routes in `routes/mod.rs`**

Add these routes AFTER the existing `/join/local` route and BEFORE `.with_state(state)`:

```rust
        .route("/join/apply", post(invite::apply_join))
        .route("/join/apply/status", get(invite::apply_status))
        .route(
            "/rings/{ring_id}/join-requests",
            get(invite::list_join_requests_handler),
        )
        .route(
            "/rings/{ring_id}/join-requests/{request_id}/approve",
            post(invite::approve_request),
        )
        .route(
            "/rings/{ring_id}/join-requests/{request_id}/reject",
            post(invite::reject_request),
        )
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check` from `server/`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/invite.rs server/src/routes/mod.rs
git commit -m "Add audit approval routes: apply, status, list, approve, reject"
```

---

### Task 5: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add approval flow tests**

Add at the end of `server/tests/integration.rs`:

```rust
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

    let apply_body = &format!(r#"{{"invite_token":"{invite_token}","display_name":"Bob","message":"I want to join"}}"#);
    let resp = app
        .clone()
        .oneshot(make_request("POST", "/api/join/apply", Some(apply_body), None))
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
        .oneshot(make_request("POST", "/api/join/apply", Some(apply_body), None))
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
        .oneshot(make_request("POST", "/api/join/apply", Some(apply_body), None))
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
        .oneshot(make_request("POST", "/api/join/apply", Some(apply_body), None))
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
        .oneshot(make_request("POST", "/api/join/apply", Some(apply_body), None))
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
        .oneshot(make_request("POST", "/api/join/apply", Some(apply_body), None))
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
```

- [ ] **Step 2: Run all tests**

Run: `cargo test` from `server/`
Expected: all tests pass (36 existing + 6 new = 42)

- [ ] **Step 3: Run fmt + clippy**

Run: `cargo fmt && cargo clippy -- -D warnings` from `server/`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "Add integration tests for audit approval flow"
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
