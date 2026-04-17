# Backend API Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Rust + Axum backend foundation with SQLite, covering Setup, Ring CRUD, Members, Config, and static file serving — enough to replace the frontend's mock data with real API calls.

**Architecture:** Handler → Service → Model three-layer separation. Handlers only parse params and return responses. All business logic lives in services. Models handle database queries. SQLite stores all persistent data in `~/.ring/ring.db`. Frontend is served as static files from the binary in production, or via Vite proxy in dev mode.

**Tech Stack:** Rust + Axum 0.8 + SQLx (SQLite) + ULID + Tower-HTTP (CORS, ServeDir, Trace)

**Scope:** This plan covers 7 API domains (Setup, Ring, Members, Config, Health, Mode, Group Docs) out of 20. Chat/SSE, Graph, Session, WebSocket, Archive, PR, Invitations, Export, Notifications, Skills, Blueprint, Self, Super Ring are deferred to later plans.

---

## File Structure

```
server/
├── Cargo.toml
├── migrations/
│   └── 001_init.sql
└── src/
    ├── main.rs                 # Binary entry: start server
    ├── lib.rs                  # Library root: re-export modules
    ├── error.rs                # RingError (single error type for whole crate)
    ├── state.rs                # AppState (DB pool + config)
    ├── extractors/
    │   ├── mod.rs
    │   └── auth.rs             # X-Ring-Token extractor
    ├── routes/
    │   ├── mod.rs              # Combine all route groups into Router
    │   ├── health.rs           # GET /api/health
    │   ├── setup.rs            # GET/POST/PUT /api/setup
    │   ├── rings.rs            # GET/POST /api/rings, GET /api/rings/:id
    │   ├── members.rs          # GET /api/rings/:id/members
    │   ├── config.rs           # GET/PUT /api/config/llm
    │   ├── mode.rs             # GET/PUT /api/rings/:id/mode
    │   └── group_docs.rs       # GET/PUT /api/rings/:id/group-docs/:doc_name
    ├── services/
    │   ├── mod.rs
    │   ├── setup.rs            # Setup business logic
    │   ├── ring.rs             # Ring CRUD logic
    │   ├── member.rs           # Member listing logic
    │   ├── config.rs           # LLM config logic
    │   └── mode.rs             # Mode switching logic
    └── models/
        ├── mod.rs
        ├── user.rs             # User DB model + queries
        ├── ring.rs             # Ring DB model + queries
        ├── member.rs           # Member DB model + queries
        └── config.rs           # Config DB model + queries
```

---

## Task 1: Project Scaffold + Error Handling

**Files:**
- Create: `server/Cargo.toml`
- Create: `server/src/main.rs`
- Create: `server/src/lib.rs`
- Create: `server/src/error.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "ring-server"
version = "0.1.0"
edition = "2021"
default-run = "ring-server"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ulid = { version = "1", features = ["serde"] }
tower-http = { version = "0.6", features = ["cors", "fs", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
```

- [ ] **Step 2: Create error.rs**

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum RingError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for RingError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => RingError::NotFound("resource not found".into()),
            sqlx::Error::Database(ref db) => {
                if db.code().map_or(false, |c| c == "2067") {
                    RingError::Conflict("resource already exists".into())
                } else {
                    RingError::Internal(e.to_string())
                }
            }
            _ => RingError::Internal(e.to_string()),
        }
    }
}

impl IntoResponse for RingError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            RingError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            RingError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            RingError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            RingError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            RingError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            RingError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".into())
            }
        };
        let code = status
            .canonical_reason()
            .unwrap_or("error")
            .to_lowercase()
            .replace(' ', "_");
        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });
        (status, axum::Json(body)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, RingError>;
```

- [ ] **Step 3: Create lib.rs**

```rust
pub mod error;
pub mod extractors;
pub mod models;
pub mod routes;
pub mod services;
pub mod state;
```

- [ ] **Step 4: Create main.rs (minimal placeholder)**

```rust
use ring_server::routes::build_router;
use ring_server::state::AppState;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("ring_server=debug,tower_http=debug")
        .init();

    let data_dir = dirs_data_dir();
    std::fs::create_dir_all(&data_dir).expect("failed to create data dir");

    let db_url = format!("sqlite:{}/ring.db?mode=rwc", data_dir);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("failed to connect to SQLite");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let state = AppState::new(pool);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7420")
        .await
        .expect("failed to bind to port 7420");

    tracing::info!("ring-server listening on http://localhost:7420");
    axum::serve(listener, app).await.expect("server error");
}

fn dirs_data_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    format!("{home}/.ring")
}
```

- [ ] **Step 5: Verify compilation fails (expected — missing modules)**

Run: `cd server && cargo check 2>&1 | head -5`
Expected: Compilation errors for missing modules — this confirms the scaffold is wired up.

- [ ] **Step 6: Commit**

```bash
git add server/Cargo.toml server/src/
git commit -m "feat(server): project scaffold with error handling"
```

---

## Task 2: Database + AppState + Migrations

**Files:**
- Create: `server/migrations/001_init.sql`
- Create: `server/src/state.rs`
- Create: `server/src/models/mod.rs`

- [ ] **Step 1: Create migration 001_init.sql**

```sql
CREATE TABLE IF NOT EXISTS users (
    token_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    avatar TEXT,
    is_creator BOOLEAN NOT NULL DEFAULT 0,
    llm_provider TEXT NOT NULL DEFAULT 'openai',
    llm_api_key TEXT,
    llm_model TEXT NOT NULL DEFAULT 'gpt-4o',
    llm_base_url TEXT,
    gitlab_url TEXT,
    gitlab_token TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS rings (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    creator_id TEXT NOT NULL REFERENCES users(token_id),
    role_description TEXT,
    interaction_mode TEXT NOT NULL DEFAULT 'normal',
    skill_permission_mode TEXT NOT NULL DEFAULT 'plan',
    blueprint_status TEXT NOT NULL DEFAULT 'pending',
    gitlab_repo_url TEXT,
    gitlab_namespace TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS members (
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(token_id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member',
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (ring_id, user_id)
);

CREATE TABLE IF NOT EXISTS setup_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    is_setup BOOLEAN NOT NULL DEFAULT 0
);

INSERT OR IGNORE INTO setup_state (id, is_setup) VALUES (1, 0);

CREATE TABLE IF NOT EXISTS group_docs (
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    doc_name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (ring_id, doc_name)
);
```

- [ ] **Step 2: Create state.rs**

```rust
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}
```

- [ ] **Step 3: Create models/mod.rs (empty placeholder)**

```rust
pub mod config;
pub mod member;
pub mod ring;
pub mod user;
```

- [ ] **Step 4: Add sqlx-cli check (optional — verify migration syntax)**

Run: `cd server && cargo install sqlx-cli --no-default-features --features sqlite && sqlx database create --database-url "sqlite:./test.db" && sqlx migrate run --database-url "sqlite:./test.db" && rm test.db`

Or skip this step — migrations will run on startup.

- [ ] **Step 5: Commit**

```bash
git add server/migrations/ server/src/state.rs server/src/models/mod.rs
git commit -m "feat(server): SQLite schema + AppState"
```

---

## Task 3: Models (User, Ring, Member, Config)

**Files:**
- Create: `server/src/models/user.rs`
- Create: `server/src/models/ring.rs`
- Create: `server/src/models/member.rs`
- Create: `server/src/models/config.rs`

- [ ] **Step 1: Create models/user.rs**

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{RingError, Result};

#[derive(Debug, FromRow, Serialize)]
pub struct UserRow {
    pub token_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub is_creator: bool,
    pub llm_provider: String,
    pub llm_api_key: Option<String>,
    pub llm_model: String,
    pub llm_base_url: Option<String>,
    pub gitlab_url: Option<String>,
    pub gitlab_token: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub display_name: String,
    pub avatar: Option<String>,
    pub llm_provider: String,
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub llm_base_url: Option<String>,
    pub gitlab_url: String,
    pub gitlab_token: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub llm_provider: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub llm_base_url: Option<String>,
    pub gitlab_url: Option<String>,
    pub gitlab_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub token_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
}

impl UserRow {
    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            token_id: self.token_id.clone(),
            display_name: self.display_name.clone(),
            avatar: self.avatar.clone(),
        }
    }
}

pub async fn create_user(pool: &sqlx::SqlitePool, token_id: &str, input: &CreateUser) -> Result<UserRow> {
    let model = input.llm_model.as_deref().unwrap_or("gpt-4o");
    sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, llm_base_url, gitlab_url, gitlab_token)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9)
         RETURNING *"
    )
        .bind(token_id)
        .bind(&input.display_name)
        .bind(&input.avatar)
        .bind(&input.llm_provider)
        .bind(&input.llm_api_key)
        .bind(model)
        .bind(&input.llm_base_url)
        .bind(&input.gitlab_url)
        .bind(&input.gitlab_token)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

pub async fn get_user(pool: &sqlx::SqlitePool, token_id: &str) -> Result<UserRow> {
    sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE token_id = ?1")
        .bind(token_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("user {token_id} not found")))
}

pub async fn update_user(pool: &sqlx::SqlitePool, token_id: &str, input: &UpdateUser) -> Result<UserRow> {
    let current = get_user(pool, token_id).await?;
    sqlx::query_as::<_, UserRow>(
        "UPDATE users SET
            display_name = ?1, avatar = ?2, llm_provider = ?3, llm_api_key = ?4,
            llm_model = ?5, llm_base_url = ?6, gitlab_url = ?7, gitlab_token = ?8
         WHERE token_id = ?9
         RETURNING *"
    )
        .bind(input.display_name.as_deref().unwrap_or(&current.display_name))
        .bind(input.avatar.as_ref().or(current.avatar.as_ref()))
        .bind(input.llm_provider.as_deref().unwrap_or(&current.llm_provider))
        .bind(input.llm_api_key.as_ref().or(current.llm_api_key.as_ref()))
        .bind(input.llm_model.as_deref().unwrap_or(&current.llm_model))
        .bind(input.llm_base_url.as_ref().or(current.llm_base_url.as_ref()))
        .bind(input.gitlab_url.as_ref().or(current.gitlab_url.as_ref()))
        .bind(input.gitlab_token.as_ref().or(current.gitlab_token.as_ref()))
        .bind(token_id)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}
```

- [ ] **Step 2: Create models/ring.rs**

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{RingError, Result};

#[derive(Debug, FromRow, Serialize)]
pub struct RingRow {
    pub id: String,
    pub name: String,
    pub creator_id: String,
    pub role_description: Option<String>,
    pub interaction_mode: String,
    pub skill_permission_mode: String,
    pub blueprint_status: String,
    pub gitlab_repo_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRing {
    pub name: String,
    pub role_description: String,
    pub gitlab_repo_url: Option<String>,
    pub gitlab_namespace: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RingListItem {
    pub id: String,
    pub name: String,
    pub role: String,
    pub member_count: i64,
    pub node_count: i64,
    pub last_activity_at: String,
    pub has_active_session: bool,
}

#[derive(Debug, Serialize)]
pub struct RingDetail {
    pub id: String,
    pub name: String,
    pub role: String,
    pub role_description: Option<String>,
    pub member_count: i64,
    pub node_count: i64,
    pub blueprint_status: String,
    pub interaction_mode: String,
    pub skill_permission_mode: String,
    pub created_at: String,
}

pub async fn create_ring(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    creator_id: &str,
    input: &CreateRing,
) -> Result<RingRow> {
    let ring = sqlx::query_as::<_, RingRow>(
        "INSERT INTO rings (id, name, creator_id, role_description, gitlab_repo_url, gitlab_namespace)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         RETURNING *"
    )
        .bind(ring_id)
        .bind(&input.name)
        .bind(creator_id)
        .bind(&input.role_description)
        .bind(&input.gitlab_repo_url)
        .bind(&input.gitlab_namespace)
        .fetch_one(pool)
        .await?;

    sqlx::query(
        "INSERT INTO members (ring_id, user_id, role) VALUES (?1, ?2, 'creator')"
    )
        .bind(ring_id)
        .bind(creator_id)
        .execute(pool)
        .await?;

    Ok(ring)
}

pub async fn list_rings_for_user(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> Result<Vec<RingListItem>> {
    let rows = sqlx::query_as::<_, RingListItem>(
        "SELECT r.id, r.name, m.role,
                (SELECT COUNT(*) FROM members m2 WHERE m2.ring_id = r.id) as member_count,
                0 as node_count,
                r.created_at as last_activity_at,
                0 as has_active_session
         FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         ORDER BY r.created_at DESC"
    )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_ring_detail(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    user_id: &str,
) -> Result<RingDetail> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, String, String, String)>(
        "SELECT r.id, r.name, r.role_description, r.blueprint_status,
                r.interaction_mode, r.skill_permission_mode
         FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?2
         WHERE r.id = ?1"
    )
        .bind(ring_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("ring {ring_id} not found")))?;

    let member_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM members WHERE ring_id = ?1"
    )
        .bind(ring_id)
        .fetch_one(pool)
        .await?;

    Ok(RingDetail {
        id: row.0,
        name: row.1,
        role: get_user_role(pool, ring_id, user_id).await?,
        role_description: row.2,
        member_count,
        node_count: 0,
        blueprint_status: row.3,
        interaction_mode: row.4,
        skill_permission_mode: row.5,
        created_at: String::new(),
    })
}

pub async fn get_user_role(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    user_id: &str,
) -> Result<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT role FROM members WHERE ring_id = ?1 AND user_id = ?2"
    )
        .bind(ring_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound("not a member".into()))
}
```

- [ ] **Step 3: Create models/member.rs**

```rust
use serde::Serialize;
use sqlx::FromRow;

use crate::error::{RingError, Result};

#[derive(Debug, FromRow, Serialize)]
pub struct MemberRow {
    pub user_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub token_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub role: String,
    pub joined_at: String,
    pub online: bool,
}

pub async fn list_members(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
) -> Result<Vec<MemberResponse>> {
    let rows = sqlx::query_as::<_, MemberRow>(
        "SELECT m.user_id, u.display_name, u.avatar, m.role, m.joined_at
         FROM members m
         JOIN users u ON u.token_id = m.user_id
         WHERE m.ring_id = ?1
         ORDER BY m.joined_at"
    )
        .bind(ring_id)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|r| MemberResponse {
            token_id: r.user_id,
            display_name: r.display_name,
            avatar: r.avatar,
            role: r.role,
            joined_at: r.joined_at,
            online: false,
        })
        .collect())
}

pub async fn update_role(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    user_id: &str,
    new_role: &str,
) -> Result<()> {
    let result = sqlx::query(
        "UPDATE members SET role = ?1 WHERE ring_id = ?2 AND user_id = ?3"
    )
        .bind(new_role)
        .bind(ring_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("member not found".into()));
    }
    Ok(())
}

pub async fn remove_member(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    user_id: &str,
) -> Result<()> {
    let result = sqlx::query(
        "DELETE FROM members WHERE ring_id = ?1 AND user_id = ?2"
    )
        .bind(ring_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("member not found".into()));
    }
    Ok(())
}
```

- [ ] **Step 4: Create models/config.rs**

```rust
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Serialize)]
pub struct LLMConfigResponse {
    pub provider: String,
    pub model: String,
    pub api_key_set: bool,
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLLMConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

pub async fn get_llm_config(pool: &sqlx::SqlitePool, user_id: &str) -> Result<LLMConfigResponse> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        "SELECT llm_provider, llm_model, llm_api_key, llm_base_url FROM users WHERE token_id = ?1"
    )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| crate::error::RingError::NotFound("user not found".into()))?;

    Ok(LLMConfigResponse {
        provider: row.0,
        model: row.1,
        api_key_set: row.2.is_some(),
        base_url: row.3,
    })
}

pub async fn update_llm_config(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    input: &UpdateLLMConfig,
) -> Result<LLMConfigResponse> {
    let current = get_llm_config(pool, user_id).await?;

    let provider = input.provider.as_deref().unwrap_or(&current.provider);
    let model = input.model.as_deref().unwrap_or(&current.model);
    let api_key = input.api_key.as_deref().or_else(|| {
        if current.api_key_set { None } else { Some("") }
    });

    if let Some(key) = api_key {
        sqlx::query("UPDATE users SET llm_provider = ?1, llm_model = ?2, llm_api_key = ?3, llm_base_url = ?4 WHERE token_id = ?5")
            .bind(provider)
            .bind(model)
            .bind(if key.is_empty() { None as Option<&str> } else { Some(key) })
            .bind(input.base_url.as_deref().or(current.base_url.as_deref()))
            .bind(user_id)
            .execute(pool)
            .await?;
    } else {
        sqlx::query("UPDATE users SET llm_provider = ?1, llm_model = ?2, llm_base_url = ?3 WHERE token_id = ?4")
            .bind(provider)
            .bind(model)
            .bind(input.base_url.as_deref().or(current.base_url.as_deref()))
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    get_llm_config(pool, user_id).await
}

pub async fn get_setup_done(pool: &sqlx::SqlitePool) -> Result<bool> {
    let done = sqlx::query_scalar::<_, bool>(
        "SELECT is_setup FROM setup_state WHERE id = 1"
    )
        .fetch_one(pool)
        .await?;
    Ok(done)
}

pub async fn set_setup_done(pool: &sqlx::SqlitePool) -> Result<()> {
    sqlx::query("UPDATE setup_state SET is_setup = 1 WHERE id = 1")
        .execute(pool)
        .await?;
    Ok(())
}
```

- [ ] **Step 5: Verify models compile**

Run: `cd server && cargo check 2>&1 | tail -5`
Expected: Errors about missing extractors/services/routes modules — models should compile.

- [ ] **Step 6: Commit**

```bash
git add server/src/models/
git commit -m "feat(server): user, ring, member, config models with DB queries"
```

---

## Task 4: Auth Extractor + Services Layer

**Files:**
- Create: `server/src/extractors/mod.rs`
- Create: `server/src/extractors/auth.rs`
- Create: `server/src/services/mod.rs`
- Create: `server/src/services/setup.rs`
- Create: `server/src/services/ring.rs`
- Create: `server/src/services/member.rs`
- Create: `server/src/services/config.rs`

- [ ] **Step 1: Create extractors/auth.rs**

```rust
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::error::RingError;
use crate::state::AppState;

pub struct AuthUser {
    pub token_id: String,
}

impl<S: Send + Sync> FromRequestParts<S> for AuthUser
where
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = RingError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let token = parts
            .headers
            .get("X-Ring-Token")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| RingError::Unauthorized("missing X-Ring-Token header".into()))?;

        let exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM users WHERE token_id = ?1"
        )
            .bind(token)
            .fetch_one(&app_state.db)
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;

        if !exists {
            return Err(RingError::Unauthorized("invalid token".into()));
        }

        Ok(AuthUser {
            token_id: token.to_string(),
        })
    }
}

pub struct OptionalUser {
    pub token_id: Option<String>,
}

impl<S: Send + Sync> FromRequestParts<S> for OptionalUser
where
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> std::result::Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("X-Ring-Token")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        Ok(OptionalUser { token_id: token })
    }
}
```

- [ ] **Step 2: Create extractors/mod.rs**

```rust
pub mod auth;

pub use auth::{AuthUser, OptionalUser};
```

- [ ] **Step 3: Create services/setup.rs**

```rust
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::error::{RingError, Result};
use crate::models::config::{get_setup_done, set_setup_done};
use crate::models::user::{self, UserResponse};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SetupRequest {
    pub display_name: String,
    pub avatar: Option<String>,
    pub llm_provider: String,
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub llm_base_url: Option<String>,
    pub gitlab_url: String,
    pub gitlab_token: String,
}

#[derive(Debug, Serialize)]
pub struct SetupStatusResponse {
    pub is_setup: bool,
    pub step: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SetupResponse {
    pub token_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
}

pub async fn get_status(state: &AppState) -> Result<SetupStatusResponse> {
    let is_setup = get_setup_done(&state.db).await?;
    Ok(SetupStatusResponse {
        is_setup,
        step: if is_setup { None } else { Some("identity".into()) },
    })
}

pub async fn submit_setup(state: &AppState, input: SetupRequest) -> Result<SetupResponse> {
    let done = get_setup_done(&state.db).await?;
    if done {
        return Err(RingError::Conflict("setup already completed".into()));
    }

    if input.llm_provider != "ollama" && input.llm_api_key.is_none() {
        return Err(RingError::BadRequest("llm_api_key required for non-ollama providers".into()));
    }

    let token_id = format!("user-{}", Ulid::new());
    let create_input = user::CreateUser {
        display_name: input.display_name,
        avatar: input.avatar,
        llm_provider: input.llm_provider,
        llm_api_key: input.llm_api_key,
        llm_model: input.llm_model,
        llm_base_url: input.llm_base_url,
        gitlab_url: input.gitlab_url,
        gitlab_token: input.gitlab_token,
    };
    let user = user::create_user(&state.db, &token_id, &create_input).await?;
    set_setup_done(&state.db).await?;

    Ok(SetupResponse {
        token_id: user.token_id,
        display_name: user.display_name,
        avatar: user.avatar,
    })
}

pub async fn update_setup(state: &AppState, token_id: &str, input: SetupRequest) -> Result<SetupResponse> {
    let update_input = user::UpdateUser {
        display_name: Some(input.display_name),
        avatar: input.avatar,
        llm_provider: Some(input.llm_provider),
        llm_api_key: input.llm_api_key,
        llm_model: input.llm_model,
        llm_base_url: input.llm_base_url,
        gitlab_url: Some(input.gitlab_url),
        gitlab_token: Some(input.gitlab_token),
    };
    let user = user::update_user(&state.db, token_id, &update_input).await?;
    Ok(SetupResponse {
        token_id: user.token_id,
        display_name: user.display_name,
        avatar: user.avatar,
    })
}
```

- [ ] **Step 4: Create services/ring.rs**

```rust
use serde::Serialize;
use ulid::Ulid;

use crate::error::{RingError, Result};
use crate::models::ring::{self, CreateRing, RingDetail, RingListItem};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CreateRingResponse {
    pub id: String,
    pub name: String,
    pub role: String,
    pub blueprint_status: String,
}

pub async fn list_rings(state: &AppState, user_id: &str) -> Result<Vec<RingListItem>> {
    ring::list_rings_for_user(&state.db, user_id).await
}

pub async fn create_ring(
    state: &AppState,
    user_id: &str,
    input: CreateRing,
) -> Result<CreateRingResponse> {
    let id = Ulid::new().to_string();
    let row = ring::create_ring(&state.db, &id, user_id, &input).await?;
    Ok(CreateRingResponse {
        id: row.id,
        name: row.name,
        role: "creator".into(),
        blueprint_status: row.blueprint_status,
    })
}

pub async fn get_ring_detail(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
) -> Result<RingDetail> {
    ring::get_ring_detail(&state.db, ring_id, user_id).await
}
```

- [ ] **Step 5: Create services/member.rs**

```rust
use crate::error::{RingError, Result};
use crate::models::member::{self, MemberResponse};
use crate::models::ring;
use crate::state::AppState;

pub async fn list_members(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
) -> Result<Vec<MemberResponse>> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    member::list_members(&state.db, ring_id).await
}

pub async fn update_member_role(
    state: &AppState,
    ring_id: &str,
    caller_id: &str,
    target_id: &str,
    new_role: &str,
) -> Result<()> {
    let caller_role = ring::get_user_role(&state.db, ring_id, caller_id).await?;
    if caller_role != "creator" && caller_role != "admin" {
        return Err(RingError::Forbidden("only creator or admin can change roles".into()));
    }
    if new_role != "admin" && new_role != "member" && new_role != "readonly" {
        return Err(RingError::BadRequest("invalid role".into()));
    }
    member::update_role(&state.db, ring_id, target_id, new_role).await
}

pub async fn remove_member(
    state: &AppState,
    ring_id: &str,
    caller_id: &str,
    target_id: &str,
) -> Result<()> {
    let caller_role = ring::get_user_role(&state.db, ring_id, caller_id).await?;
    if caller_role != "creator" && caller_role != "admin" {
        return Err(RingError::Forbidden("only creator or admin can remove members".into()));
    }
    member::remove_member(&state.db, ring_id, target_id).await
}
```

- [ ] **Step 6: Create services/config.rs**

```rust
use crate::error::Result;
use crate::models::config::{self, LLMConfigResponse, UpdateLLMConfig};
use crate::state::AppState;

pub async fn get_llm_config(state: &AppState, user_id: &str) -> Result<LLMConfigResponse> {
    config::get_llm_config(&state.db, user_id).await
}

pub async fn update_llm_config(
    state: &AppState,
    user_id: &str,
    input: UpdateLLMConfig,
) -> Result<LLMConfigResponse> {
    config::update_llm_config(&state.db, user_id, &input).await
}
```

- [ ] **Step 7: Create services/mod.rs**

```rust
pub mod config;
pub mod member;
pub mod ring;
pub mod setup;
```

- [ ] **Step 8: Verify compilation**

Run: `cd server && cargo check 2>&1 | tail -5`
Expected: Only errors about missing routes module.

- [ ] **Step 9: Commit**

```bash
git add server/src/extractors/ server/src/services/
git commit -m "feat(server): auth extractor + services layer"
```

---

## Task 5: Route Handlers (Health + Setup)

**Files:**
- Create: `server/src/routes/mod.rs`
- Create: `server/src/routes/health.rs`
- Create: `server/src/routes/setup.rs`

- [ ] **Step 1: Create routes/health.rs**

```rust
use axum::Json;
use serde_json::{json, Value};

pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
```

- [ ] **Step 2: Create routes/setup.rs**

```rust
use axum::extract::State;
use axum::Json;
use serde_json::Value;

use crate::error::Result;
use crate::extractors::OptionalUser;
use crate::services::setup;
use crate::state::AppState;

pub async fn get_status(
    State(state): State<AppState>,
) -> Result<Json<setup::SetupStatusResponse>> {
    let status = setup::get_status(&state).await?;
    Ok(Json(status))
}

pub async fn submit_setup(
    State(state): State<AppState>,
    Json(body): Json<setup::SetupRequest>,
) -> Result<Json<Value>> {
    let result = setup::submit_setup(state, body).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn update_setup(
    State(state): State<AppState>,
    _user: crate::extractors::AuthUser,
    Json(body): Json<setup::SetupRequest>,
) -> Result<Json<Value>> {
    let result = setup::update_setup(&state, &_user.token_id, body).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}
```

- [ ] **Step 3: Create routes/mod.rs (partial — health + setup only)**

```rust
use axum::Router;
use axum::routing::{get, post, put};
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

mod health;
mod setup;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/health", get(health::health_check))
        .route("/setup/status", get(setup::get_status))
        .route("/setup", post(setup::submit_setup).put(setup::update_setup))
        .with_state(state);

    Router::new()
        .nest("/api", api)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
```

- [ ] **Step 4: Verify compilation and test**

Run: `cd server && cargo check`
Expected: Compiles successfully.

Run: `cd server && cargo build`
Expected: Builds `ring-server` binary.

Run (manual smoke test):
```bash
cd server && cargo run &
sleep 3
curl http://localhost:7420/api/health
curl http://localhost:7420/api/setup/status
kill %1
```

Expected: `{"status":"ok"}` and `{"is_setup":false,"step":"identity"}`.

- [ ] **Step 5: Commit**

```bash
git add server/src/routes/
git commit -m "feat(server): health + setup routes"
```

---

## Task 6: Route Handlers (Rings + Members + Config)

**Files:**
- Create: `server/src/routes/rings.rs`
- Create: `server/src/routes/members.rs`
- Create: `server/src/routes/config.rs`
- Create: `server/src/routes/mode.rs`
- Create: `server/src/services/mode.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Create routes/rings.rs**

```rust
use axum::extract::{Path, State};
use axum::Json;
use serde_json::Value;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::ring::CreateRing;
use crate::services::ring;
use crate::state::AppState;

pub async fn list_rings(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Value>> {
    let rings = ring::list_rings(&state, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "rings": rings })))
}

pub async fn create_ring(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateRing>,
) -> Result<(axum::http::StatusCode, Json<Value>)> {
    let result = ring::create_ring(&state, &user.token_id, body).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::to_value(result).unwrap()),
    ))
}

pub async fn get_ring(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<Value>> {
    let detail = ring::get_ring_detail(&state, &ring_id, &user.token_id).await?;
    Ok(Json(serde_json::to_value(detail).unwrap()))
}
```

- [ ] **Step 2: Create routes/members.rs**

```rust
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::services::member;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub async fn list_members(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<Value>> {
    let members = member::list_members(&state, &ring_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}

pub async fn update_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, target_id)): Path<(String, String)>,
    Json(body): Json<UpdateRoleRequest>,
) -> Result<axum::http::StatusCode> {
    member::update_member_role(&state, &ring_id, &user.token_id, &target_id, &body.role).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn remove_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, target_id)): Path<(String, String)>,
) -> Result<axum::http::StatusCode> {
    member::remove_member(&state, &ring_id, &user.token_id, &target_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
```

- [ ] **Step 3: Create routes/config.rs**

```rust
use axum::extract::State;
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::config::UpdateLLMConfig;
use crate::services::config;
use crate::state::AppState;

pub async fn get_llm_config(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<crate::models::config::LLMConfigResponse>> {
    let cfg = config::get_llm_config(&state, &user.token_id).await?;
    Ok(Json(cfg))
}

pub async fn update_llm_config(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateLLMConfig>,
) -> Result<Json<crate::models::config::LLMConfigResponse>> {
    let cfg = config::update_llm_config(&state, &user.token_id, body).await?;
    Ok(Json(cfg))
}
```

- [ ] **Step 4: Create services/mode.rs**

```rust
use serde::{Deserialize, Serialize};

use crate::error::{RingError, Result};
use crate::models::ring;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ModeResponse {
    pub interaction_mode: String,
    pub skill_permission_mode: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModeRequest {
    pub interaction_mode: Option<String>,
    pub skill_permission_mode: Option<String>,
}

pub async fn get_mode(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
) -> Result<ModeResponse> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT interaction_mode, skill_permission_mode FROM rings WHERE id = ?1"
    )
        .bind(ring_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| RingError::NotFound("ring not found".into()))?;

    Ok(ModeResponse {
        interaction_mode: row.0,
        skill_permission_mode: row.1,
    })
}

pub async fn update_mode(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    input: &UpdateModeRequest,
) -> Result<ModeResponse> {
    let role = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if role == "readonly" {
        return Err(RingError::Forbidden("readonly members cannot change mode".into()));
    }

    if let Some(ref mode) = input.interaction_mode {
        if mode != "normal" && mode != "auto" {
            return Err(RingError::BadRequest("interaction_mode must be 'normal' or 'auto'".into()));
        }
    }
    if let Some(ref mode) = input.skill_permission_mode {
        if mode != "auto" && mode != "plan" && mode != "edit" {
            return Err(RingError::BadRequest("skill_permission_mode must be 'auto', 'plan', or 'edit'".into()));
        }
    }

    let current = get_mode(state, ring_id, user_id).await?;
    let im = input.interaction_mode.as_deref().unwrap_or(&current.interaction_mode);
    let spm = input.skill_permission_mode.as_deref().unwrap_or(&current.skill_permission_mode);

    sqlx::query(
        "UPDATE rings SET interaction_mode = ?1, skill_permission_mode = ?2 WHERE id = ?3"
    )
        .bind(im)
        .bind(spm)
        .bind(ring_id)
        .execute(&state.db)
        .await?;

    Ok(ModeResponse {
        interaction_mode: im.to_string(),
        skill_permission_mode: spm.to_string(),
    })
}
```

Add to `services/mod.rs`:

```rust
pub mod config;
pub mod member;
pub mod mode;
pub mod ring;
pub mod setup;
```

- [ ] **Step 5: Create routes/mode.rs**

```rust
use axum::extract::{Path, State};
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::services::mode::{self, UpdateModeRequest};
use crate::state::AppState;

pub async fn get_mode(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<mode::ModeResponse>> {
    let result = mode::get_mode(&state, &ring_id, &user.token_id).await?;
    Ok(Json(result))
}

pub async fn update_mode(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<UpdateModeRequest>,
) -> Result<Json<mode::ModeResponse>> {
    let result = mode::update_mode(&state, &ring_id, &user.token_id, &body).await?;
    Ok(Json(result))
}
```

- [ ] **Step 6: Create routes/group_docs.rs**

```rust
use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{RingError, Result};
use crate::extractors::AuthUser;
use crate::models::ring;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct GroupDocResponse {
    pub doc_name: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupDocRequest {
    pub content: String,
}

pub async fn get_group_doc(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, doc_name)): Path<(String, String)>,
) -> Result<Json<GroupDocResponse>> {
    let _ = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let valid_docs = ["role", "conventions", "active-context", "archive-patterns", "corrections", "knowledge-summary"];
    if !valid_docs.contains(&doc_name.as_str()) {
        return Err(RingError::BadRequest(format!("invalid doc_name: {doc_name}")));
    }

    let content: Option<String> = sqlx::query_scalar(
        "SELECT content FROM group_docs WHERE ring_id = ?1 AND doc_name = ?2"
    )
        .bind(&ring_id)
        .bind(&doc_name)
        .fetch_optional(&state.db)
        .await?;

    Ok(Json(GroupDocResponse {
        doc_name,
        content: content.unwrap_or_default(),
    }))
}

pub async fn update_group_doc(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, doc_name)): Path<(String, String)>,
    Json(body): Json<UpdateGroupDocRequest>,
) -> Result<Json<GroupDocResponse>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let admin_only_docs = ["role", "conventions"];
    if admin_only_docs.contains(&doc_name.as_str()) && role != "creator" && role != "admin" {
        return Err(RingError::Forbidden("only creator/admin can edit this doc".into()));
    }

    let valid_docs = ["role", "conventions", "active-context", "archive-patterns", "corrections", "knowledge-summary"];
    if !valid_docs.contains(&doc_name.as_str()) {
        return Err(RingError::BadRequest(format!("invalid doc_name: {doc_name}")));
    }

    sqlx::query(
        "INSERT INTO group_docs (ring_id, doc_name, content, updated_at) VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(ring_id, doc_name) DO UPDATE SET content = ?3, updated_at = datetime('now')"
    )
        .bind(&ring_id)
        .bind(&doc_name)
        .bind(&body.content)
        .execute(&state.db)
        .await?;

    Ok(Json(GroupDocResponse {
        doc_name,
        content: body.content,
    }))
}
```

- [ ] **Step 7: Update routes/mod.rs to include all routes**

```rust
use axum::Router;
use axum::routing::{delete, get, post, put};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

mod config;
mod group_docs;
mod health;
mod members;
mod mode;
mod rings;
mod setup;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/health", get(health::health_check))
        .route("/setup/status", get(setup::get_status))
        .route("/setup", post(setup::submit_setup).put(setup::update_setup))
        .route("/rings", get(rings::list_rings).post(rings::create_ring))
        .route("/rings/{ring_id}", get(rings::get_ring))
        .route("/rings/{ring_id}/members", get(members::list_members))
        .route(
            "/rings/{ring_id}/members/{target_id}/role",
            put(members::update_role),
        )
        .route(
            "/rings/{ring_id}/members/{target_id}",
            delete(members::remove_member),
        )
        .route("/config/llm", get(config::get_llm_config).put(config::update_llm_config))
        .route("/rings/{ring_id}/mode", get(mode::get_mode).put(mode::update_mode))
        .route(
            "/rings/{ring_id}/group-docs/{doc_name}",
            get(group_docs::get_group_doc).put(group_docs::update_group_doc),
        )
        .with_state(state);

    Router::new()
        .nest("/api", api)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
```

- [ ] **Step 8: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles successfully.

- [ ] **Step 9: Commit**

```bash
git add server/src/routes/ server/src/services/mod.rs server/src/services/mode.rs
git commit -m "feat(server): rings, members, config, mode, group_docs routes"
```

---

## Task 7: Integration Tests

**Files:**
- Create: `server/tests/integration.rs`

- [ ] **Step 1: Create tests/integration.rs**

```rust
use axum::body::Body;
use axum::http::{Request, StatusCode};
use ring_server::routes::build_router;
use ring_server::state::AppState;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn setup_app() -> (AppState, axum::Router) {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to create in-memory db");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let state = AppState::new(pool);
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
async fn test_health_check() {
    let (_state, app) = setup_app().await;

    let req = Request::builder()
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_setup_flow() {
    let (_state, app) = setup_app().await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["is_setup"], false);

    let setup_body = serde_json::json!({
        "display_name": "TestUser",
        "avatar": "🧪",
        "llm_provider": "openai",
        "llm_api_key": "sk-test",
        "llm_model": "gpt-4o",
        "gitlab_url": "https://gitlab.test.com",
        "gitlab_token": "glpat-test"
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&setup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token_id = json["token_id"].as_str().unwrap();

    assert!(!token_id.is_empty());
    assert_eq!(json["display_name"], "TestUser");

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/setup/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["is_setup"], true);
}

#[tokio::test]
async fn test_setup_duplicate_rejected() {
    let (_state, app) = setup_app().await;

    let setup_body = serde_json::json!({
        "display_name": "TestUser",
        "llm_provider": "openai",
        "llm_api_key": "sk-test",
        "gitlab_url": "https://gitlab.test.com",
        "gitlab_token": "glpat-test"
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&setup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&setup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_ring_crud() {
    let (_state, app) = setup_app().await;

    let setup_body = serde_json::json!({
        "display_name": "TestUser",
        "llm_provider": "openai",
        "llm_api_key": "sk-test",
        "gitlab_url": "https://gitlab.test.com",
        "gitlab_token": "glpat-test"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&setup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let token = serde_json::from_slice::<serde_json::Value>(&body).unwrap()["token_id"]
        .as_str()
        .unwrap()
        .to_string();

    let ring_body = serde_json::json!({
        "name": "Test Ring",
        "role_description": "You are a test assistant"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rings")
                .header("Content-Type", "application/json")
                .header("X-Ring-Token", &token)
                .body(Body::from(serde_json::to_string(&ring_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let ring_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let ring_id = ring_json["id"].as_str().unwrap();
    assert_eq!(ring_json["role"], "creator");

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/rings")
                .header("X-Ring-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 2048).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["rings"].as_array().unwrap().len(), 1);

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(&format!("/api/rings/{ring_id}"))
                .header("X-Ring-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_members_list() {
    let (_state, app) = setup_app().await;

    let setup_body = serde_json::json!({
        "display_name": "TestUser",
        "llm_provider": "openai",
        "llm_api_key": "sk-test",
        "gitlab_url": "https://gitlab.test.com",
        "gitlab_token": "glpat-test"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&setup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let token = serde_json::from_slice::<serde_json::Value>(&body).unwrap()["token_id"]
        .as_str()
        .unwrap()
        .to_string();

    let ring_body = serde_json::json!({
        "name": "Test Ring",
        "role_description": "Test"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rings")
                .header("Content-Type", "application/json")
                .header("X-Ring-Token", &token)
                .body(Body::from(serde_json::to_string(&ring_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let ring_id = serde_json::from_slice::<serde_json::Value>(&body).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = app
        .oneshot(
            Request::builder()
                .uri(&format!("/api/rings/{ring_id}/members"))
                .header("X-Ring-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 2048).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["members"].as_array().unwrap().len(), 1);
    assert_eq!(json["members"][0]["role"], "creator");
}

#[tokio::test]
async fn test_config_llm() {
    let (_state, app) = setup_app().await;

    let setup_body = serde_json::json!({
        "display_name": "TestUser",
        "llm_provider": "openai",
        "llm_api_key": "sk-test",
        "gitlab_url": "https://gitlab.test.com",
        "gitlab_token": "glpat-test"
    });
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/setup")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&setup_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let token = serde_json::from_slice::<serde_json::Value>(&body).unwrap()["token_id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/config/llm")
                .header("X-Ring-Token", &token)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["provider"], "openai");
    assert_eq!(json["api_key_set"], true);

    let update_body = serde_json::json!({
        "provider": "anthropic",
        "model": "claude-sonnet-4-20250514"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/config/llm")
                .header("Content-Type", "application/json")
                .header("X-Ring-Token", &token)
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["provider"], "anthropic");
}
```

- [ ] **Step 2: Run tests**

Run: `cd server && cargo test --test integration`
Expected: All 6 tests PASS.

- [ ] **Step 3: Commit**

```bash
git add server/tests/
git commit -m "test(server): integration tests for setup, rings, members, config"
```

---

## Task 8: Static File Serving + SPA Fallback

**Files:**
- Modify: `server/src/routes/mod.rs`
- Modify: `server/src/main.rs`

- [ ] **Step 1: Update routes/mod.rs to add SPA fallback**

The key idea: in dev mode, the frontend runs on Vite's dev server (proxied via `vite.config.ts`). In production, the Rust binary serves the built frontend from `ui/dist/`. The fallback serves `index.html` for any non-API route.

```rust
use axum::Router;
use axum::routing::{delete, get, post, put};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::state::AppState;

mod config;
mod group_docs;
mod health;
mod members;
mod mode;
mod rings;
mod setup;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/health", get(health::health_check))
        .route("/setup/status", get(setup::get_status))
        .route("/setup", post(setup::submit_setup).put(setup::update_setup))
        .route("/rings", get(rings::list_rings).post(rings::create_ring))
        .route("/rings/{ring_id}", get(rings::get_ring))
        .route("/rings/{ring_id}/members", get(members::list_members))
        .route(
            "/rings/{ring_id}/members/{target_id}/role",
            put(members::update_role),
        )
        .route(
            "/rings/{ring_id}/members/{target_id}",
            delete(members::remove_member),
        )
        .route("/config/llm", get(config::get_llm_config).put(config::update_llm_config))
        .route("/rings/{ring_id}/mode", get(mode::get_mode).put(mode::update_mode))
        .route(
            "/rings/{ring_id}/group-docs/{doc_name}",
            get(group_docs::get_group_doc).put(group_docs::update_group_doc),
        )
        .with_state(state);

    Router::new()
        .nest("/api", api)
        .fallback_service(ServeDir::new("ui/dist").append_index_html_on_directories(true))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles successfully.

- [ ] **Step 3: Full smoke test**

```bash
cd server && cargo run &
sleep 3

curl -s http://localhost:7420/api/health | python3 -m json.tool
curl -s http://localhost:7420/api/setup/status | python3 -m json.tool

curl -s -X POST http://localhost:7420/api/setup \
  -H 'Content-Type: application/json' \
  -d '{"display_name":"Kai","llm_provider":"openai","llm_api_key":"sk-test","gitlab_url":"https://gitlab.test.com","gitlab_token":"glpat-test"}'

kill %1
```

Expected: All return valid JSON.

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/mod.rs
git commit -m "feat(server): static file serving with SPA fallback"
```

---

## Task 9: Fix Compilation Issues + `cargo clippy` + `cargo fmt`

**Files:**
- Modify: Various files as needed to pass clippy and fmt

- [ ] **Step 1: Run cargo fmt**

Run: `cd server && cargo fmt`

- [ ] **Step 2: Run cargo clippy**

Run: `cd server && cargo clippy -- -D warnings 2>&1`

Fix all warnings. Common fixes:
- Unused imports → remove
- Redundant clones → remove where possible
- Missing docs → not required (AGENTS.md: no comments)

- [ ] **Step 3: Run all tests again**

Run: `cd server && cargo test`
Expected: All tests PASS.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore(server): clippy + fmt fixes"
```

---

## Task 10: Verify Frontend-Backend Connection

**Files:**
- Modify: `ui/src/services/mock-data.ts` (optional — replace with API calls later)
- No code changes in this task — purely verification

- [ ] **Step 1: Build frontend**

Run: `cd ui && npm run build`
Expected: `ui/dist/` contains built frontend.

- [ ] **Step 2: Start backend serving frontend**

Run: `cd server && cargo run`
Expected: `ring-server listening on http://localhost:7420`

- [ ] **Step 3: Open browser**

Navigate to `http://localhost:7420` in browser.

Expected:
- Frontend loads (index.html served)
- `GET /api/setup/status` returns `{"is_setup":false,"step":"identity"}`
- SetupWizard should display (because `is_setup` is false)
- After setup, Ring list is empty (no rings yet)

Note: The frontend still uses mock data for most things. The real API integration will happen in Plan 4. This task verifies the server is running and the API endpoints are reachable.

- [ ] **Step 4: Verify API endpoints via curl**

```bash
curl -s http://localhost:7420/api/setup/status
# {"is_setup":false,"step":"identity"}

curl -s -X POST http://localhost:7420/api/setup \
  -H 'Content-Type: application/json' \
  -d '{"display_name":"Kai","avatar":"🦊","llm_provider":"openai","llm_api_key":"sk-test","llm_model":"gpt-4o","gitlab_url":"https://gitlab.company.com","gitlab_token":"glpat-xxx"}'
# {"token_id":"user-XXX","display_name":"Kai","avatar":"🦊"}

# Use the token_id from above:
TOKEN="user-XXX"

curl -s -X POST http://localhost:7420/api/rings \
  -H 'Content-Type: application/json' \
  -H "X-Ring-Token: $TOKEN" \
  -d '{"name":"竞品分析组","role_description":"你是一个产品分析专家"}'
# {"id":"XXX","name":"竞品分析组","role":"creator","blueprint_status":"pending"}

curl -s http://localhost:7420/api/rings -H "X-Ring-Token: $TOKEN"
# {"rings":[...]}

curl -s http://localhost:7420/api/config/llm -H "X-Ring-Token: $TOKEN"
# {"provider":"openai","model":"gpt-4o","api_key_set":true,"base_url":null}
```

Expected: All return correct JSON responses.

- [ ] **Step 5: Final commit (if any changes)**

```bash
git add -A
git commit -m "chore: verify frontend-backend connection"
```

---

## Self-Review

### 1. Spec Coverage

| API Domain | Covered in Plan | Task |
|------------|----------------|------|
| Setup (GET/POST/PUT /api/setup) | Yes | Task 5 |
| Ring CRUD (GET/POST /api/rings, GET /api/rings/:id) | Yes | Task 6 |
| Members (GET/PUT/DELETE /api/rings/:id/members) | Yes | Task 6 |
| Config/LLM (GET/PUT /api/config/llm) | Yes | Task 6 |
| Mode (GET/PUT /api/rings/:id/mode) | Yes | Task 6 |
| Group Docs (GET/PUT /api/rings/:id/group-docs/:name) | Yes | Task 6 |
| Health (GET /api/health) | Yes | Task 5 |
| Static file serving | Yes | Task 8 |
| Chat/SSE | No — Plan 4 | — |
| Graph | No — Plan 4 | — |
| Session | No — Plan 5 | — |
| WebSocket | No — Plan 5 | — |
| Archive/PR | No — Plan 5 | — |
| Invitations/Join | No — Plan 5 | — |
| Export | No — later | — |
| Notifications | No — later | — |
| Skills | No — later | — |
| Blueprint | No — later | — |
| Self | No — Plan 4 | — |
| Super Ring | No — Plan 4 | — |

### 2. Placeholder Scan

No TBD/TODO/placeholders found. All steps contain complete code.

### 3. Type Consistency

- `UserResponse` (models/user.rs) matches setup response shape
- `RingListItem` (models/ring.rs) matches frontend `Ring` type (ring.ts)
- `MemberResponse` (models/member.rs) matches frontend `Member` type
- `LLMConfigResponse` (models/config.rs) matches frontend `LLMConfig` type
- `ModeResponse` (services/mode.rs) matches frontend `RingMode` type
- API field names are `snake_case` throughout (AGENTS.md convention)
- `X-Ring-Token` header used consistently in auth extractor and tests

---

## API Endpoint Summary

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | /api/health | No | Health check |
| GET | /api/setup/status | No | Check setup status |
| POST | /api/setup | No | Initial setup (creates creator) |
| PUT | /api/setup | Token | Update setup/config |
| GET | /api/rings | Token | List user's rings |
| POST | /api/rings | Token | Create new ring |
| GET | /api/rings/:id | Token | Get ring detail |
| GET | /api/rings/:id/members | Token | List ring members |
| PUT | /api/rings/:id/members/:uid/role | Token | Change member role |
| DELETE | /api/rings/:id/members/:uid | Token | Remove member |
| GET | /api/config/llm | Token | Get LLM config |
| PUT | /api/config/llm | Token | Update LLM config |
| GET | /api/rings/:id/mode | Token | Get ring mode |
| PUT | /api/rings/:id/mode | Token | Set ring mode |
| GET | /api/rings/:id/group-docs/:name | Token | Get group doc |
| PUT | /api/rings/:id/group-docs/:name | Token | Update group doc |
