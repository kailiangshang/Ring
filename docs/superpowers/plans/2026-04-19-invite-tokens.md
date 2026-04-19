# Invite Tokens API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add invite_tokens table, CRUD API for generating/listing/revoking Ring invite tokens.

**Architecture:** New migration for invite_tokens table. New model/service/routes modules following existing patterns (same as members.rs). Permission checks use existing `get_user_role()` — only creator/admin can manage tokens. Token generation uses `rand` + `base64`.

**Tech Stack:** Rust + Axum + SQLite (sqlx), rand + base64 crates

---

### Task 1: Add dependencies + migration

**Files:**
- Modify: `server/Cargo.toml`
- Create: `server/migrations/007_invite_tokens.sql`

- [ ] **Step 1: Add rand + base64 to Cargo.toml**

Add to `[dependencies]` in `server/Cargo.toml`:

```toml
rand = "0.9"
base64 = "0.22"
```

- [ ] **Step 2: Create migration file**

Create `server/migrations/007_invite_tokens.sql`:

```sql
CREATE TABLE invite_tokens (
    token TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id),
    type TEXT NOT NULL CHECK(type IN ('open', 'audit')),
    role TEXT NOT NULL DEFAULT 'member' CHECK(role IN ('member', 'readonly')),
    max_uses INTEGER NOT NULL DEFAULT 1,
    use_count INTEGER NOT NULL DEFAULT 0,
    max_members INTEGER,
    expires_at TEXT NOT NULL,
    revoked_at TEXT,
    created_by TEXT NOT NULL REFERENCES users(token_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles (new deps download + migration file exists)

- [ ] **Step 4: Commit**

```bash
git add server/Cargo.toml server/migrations/007_invite_tokens.sql
git commit -m "Add invite_tokens migration and rand/base64 dependencies"
```

---

### Task 2: Create invite model

**Files:**
- Create: `server/src/models/invite.rs`
- Modify: `server/src/models/mod.rs`

- [ ] **Step 1: Create models/invite.rs**

Create `server/src/models/invite.rs`:

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize)]
pub struct InviteTokenRow {
    pub token: String,
    pub ring_id: String,
    pub r#type: String,
    pub role: String,
    pub max_uses: i64,
    pub use_count: i64,
    pub max_members: Option<i64>,
    pub expires_at: String,
    pub revoked_at: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteToken {
    #[serde(default = "default_type")]
    pub r#type: String,
    #[serde(default = "default_role")]
    pub role: String,
    #[serde(default = "default_max_uses")]
    pub max_uses: i64,
    pub max_members: Option<i64>,
    #[serde(default = "default_expires_hours")]
    pub expires_in_hours: i64,
}

fn default_type() -> String {
    "open".to_string()
}

fn default_role() -> String {
    "member".to_string()
}

fn default_max_uses() -> i64 {
    1
}

fn default_expires_hours() -> i64 {
    24
}

pub async fn insert_token(pool: &sqlx::SqlitePool, row: &InviteTokenRow) -> Result<()> {
    sqlx::query(
        "INSERT INTO invite_tokens (token, ring_id, type, role, max_uses, use_count, max_members, expires_at, revoked_at, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )
    .bind(&row.token)
    .bind(&row.ring_id)
    .bind(&row.r#type)
    .bind(&row.role)
    .bind(row.max_uses)
    .bind(row.use_count)
    .bind(row.max_members)
    .bind(&row.expires_at)
    .bind(&row.revoked_at)
    .bind(&row.created_by)
    .bind(&row.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_tokens(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    include_expired: bool,
    include_revoked: bool,
) -> Result<Vec<InviteTokenRow>> {
    let mut query = String::from(
        "SELECT * FROM invite_tokens WHERE ring_id = ?1",
    );
    if !include_expired {
        query.push_str(" AND expires_at > datetime('now')");
    }
    if !include_revoked {
        query.push_str(" AND revoked_at IS NULL");
    }
    query.push_str(" ORDER BY created_at DESC");

    let rows = sqlx::query_as::<_, InviteTokenRow>(&query)
        .bind(ring_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn revoke_token(pool: &sqlx::SqlitePool, ring_id: &str, token: &str) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE invite_tokens SET revoked_at = datetime('now') WHERE ring_id = ?1 AND token = ?2 AND revoked_at IS NULL",
    )
    .bind(ring_id)
    .bind(token)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM invite_tokens WHERE ring_id = ?1 AND token = ?2)",
        )
        .bind(ring_id)
        .bind(token)
        .fetch_one(pool)
        .await?;

        if !exists {
            return Err(RingError::NotFound(format!("invite token not found")));
        }
    }
    Ok(true)
}
```

- [ ] **Step 2: Add mod invite to models/mod.rs**

Add to `server/src/models/mod.rs`:

```rust
pub mod invite;
```

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add server/src/models/invite.rs server/src/models/mod.rs
git commit -m "Add invite token model with SQL queries"
```

---

### Task 3: Create invite service

**Files:**
- Create: `server/src/services/invite.rs`
- Modify: `server/src/services/mod.rs`

- [ ] **Step 1: Create services/invite.rs**

Create `server/src/services/invite.rs`:

```rust
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;

use crate::error::{Result, RingError};
use crate::models::invite::{self, CreateInviteToken, InviteTokenRow};
use crate::models::ring;
use crate::state::AppState;

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn check_admin(pool: &sqlx::SqlitePool, ring_id: &str, user_id: &str) -> crate::error::Result<String> {
    let role = ring::get_user_role(pool, ring_id, user_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden(
            "only creator or admin can manage invite tokens".into(),
        ));
    }
    Ok(role)
}

pub async fn create_token(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    input: &CreateInviteToken,
) -> Result<InviteTokenRow> {
    check_admin(&state.db, ring_id, user_id).await?;

    if input.r#type != "open" && input.r#type != "audit" {
        return Err(RingError::BadRequest("type must be 'open' or 'audit'".into()));
    }
    if input.role != "member" && input.role != "readonly" {
        return Err(RingError::BadRequest("role must be 'member' or 'readonly'".into()));
    }
    if input.expires_in_hours <= 0 {
        return Err(RingError::BadRequest("expires_in_hours must be positive".into()));
    }

    let token = generate_token();
    let expires_at = Utc::now() + chrono::Duration::hours(input.expires_in_hours);

    let row = InviteTokenRow {
        token,
        ring_id: ring_id.to_string(),
        r#type: input.r#type.clone(),
        role: input.role.clone(),
        max_uses: input.max_uses,
        use_count: 0,
        max_members: input.max_members,
        expires_at: expires_at.to_rfc3339(),
        revoked_at: None,
        created_by: user_id.to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    invite::insert_token(&state.db, &row).await?;
    Ok(row)
}

pub async fn list_tokens(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    include_expired: bool,
    include_revoked: bool,
) -> Result<Vec<InviteTokenRow>> {
    check_admin(&state.db, ring_id, user_id).await?;
    invite::list_tokens(&state.db, ring_id, include_expired, include_revoked).await
}

pub async fn revoke_token(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    token: &str,
) -> Result<String> {
    check_admin(&state.db, ring_id, user_id).await?;
    invite::revoke_token(&state.db, ring_id, token).await?;
    Ok(Utc::now().to_rfc3339())
}
```

**IMPORTANT NOTE**: The `check_admin` function above uses `async` logic but is declared as `async fn`. However, `get_user_role` is async. Since Rust doesn't allow `await` in a non-async function, the implementer MUST make `check_admin` an `async fn`. The code shown above has a bug — it uses `.await` inside a non-async function. The correct version:

```rust
async fn check_admin(pool: &sqlx::SqlitePool, ring_id: &str, user_id: &str) -> crate::error::Result<String> {
    let role = ring::get_user_role(pool, ring_id, user_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden(
            "only creator or admin can manage invite tokens".into(),
        ));
    }
    Ok(role)
}
```

- [ ] **Step 2: Add mod invite to services/mod.rs**

Add to `server/src/services/mod.rs`:

```rust
pub mod invite;
```

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add server/src/services/invite.rs server/src/services/mod.rs
git commit -m "Add invite token service with create/list/revoke"
```

---

### Task 4: Create invite routes

**Files:**
- Create: `server/src/routes/invite.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Create routes/invite.rs**

Create `server/src/routes/invite.rs`:

```rust
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::error::Result;
use crate::extractors::auth::AuthUser;
use crate::models::invite::CreateInviteToken;
use crate::services::invite;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListTokensQuery {
    #[serde(default)]
    pub include_expired: Option<bool>,
    #[serde(default)]
    pub include_revoked: Option<bool>,
}

pub async fn create_invite_token(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateInviteToken>,
) -> Result<Json<serde_json::Value>> {
    let token = invite::create_token(&state, &ring_id, &user.token_id, &body).await?;
    Ok(Json(json!({
        "token": token.token,
        "type": token.r#type,
        "role": token.role,
        "max_uses": token.max_uses,
        "max_members": token.max_members,
        "expires_at": token.expires_at,
        "created_at": token.created_at,
    })))
}

pub async fn list_invite_tokens(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<ListTokensQuery>,
) -> Result<Json<serde_json::Value>> {
    let include_expired = query.include_expired.unwrap_or(false);
    let include_revoked = query.include_revoked.unwrap_or(false);
    let tokens = invite::list_tokens(&state, &ring_id, &user.token_id, include_expired, include_revoked).await?;
    Ok(Json(json!({ "tokens": tokens })))
}

pub async fn revoke_invite_token(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, token)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let revoked_at = invite::revoke_token(&state, &ring_id, &user.token_id, &token).await?;
    Ok(Json(json!({ "ok": true, "revoked_at": revoked_at })))
}
```

- [ ] **Step 2: Register routes in mod.rs**

In `server/src/routes/mod.rs`:

1. Add `mod invite;` to module declarations

2. Add these routes before `.with_state(state)` (after the skills routes):

```rust
        .route(
            "/rings/{ring_id}/invite-tokens",
            post(invite::create_invite_token).get(invite::list_invite_tokens),
        )
        .route(
            "/rings/{ring_id}/invite-tokens/{token}",
            delete(invite::revoke_invite_token),
        )
```

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/invite.rs server/src/routes/mod.rs
git commit -m "Add invite token API routes: create, list, revoke"
```

---

### Task 5: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add invite token tests**

Add at the end of the test file:

```rust
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
```

- [ ] **Step 2: Run all tests**

Run: `cd server && cargo test`
Expected: all tests pass (29 total: 25 existing + 4 new)

- [ ] **Step 3: Run fmt + clippy**

Run: `cd server && cargo fmt && cargo clippy -- -D warnings`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "Add integration tests for invite token CRUD endpoints"
```

---

### Task 6: Final verification

- [ ] **Step 1: Run all backend tests**

Run: `cd server && cargo test`
Expected: all tests pass

- [ ] **Step 2: Run fmt check + clippy**

Run: `cd server && cargo fmt --check && cargo clippy -- -D warnings`
Expected: no errors

- [ ] **Step 3: Frontend build**

Run: `cd ui && npm run build`
Expected: build succeeds

- [ ] **Step 4: Final commit if needed**

```bash
git add -A
git commit -m "Final fixes for invite tokens feature"
```
