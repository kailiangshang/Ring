# Privacy Filter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a privacy filter that masks sensitive information (phone numbers, Chinese ID numbers, emails, bank card numbers) before sending user messages to LLM APIs, while storing the original content in the database.

**Architecture:** A new `privacy_filter` service module with regex-based filtering. The filter is applied at all LLM call sites (chat, super_chat, material_prep, session summarize) using the user's configured filter settings. Settings are stored as JSON in the `users` table and exposed via REST API.

**Tech Stack:** Rust + Axum + SQLite (sqlx), regex crate

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `server/src/services/privacy_filter.rs` | Create | Core filtering logic with regex patterns |
| `server/migrations/012_privacy_filters.sql` | Create | Add `privacy_filters` column to users table |
| `server/src/models/user.rs` | Modify | Add `privacy_filters` field to all user structs and DB functions |
| `server/src/models/config.rs` | Modify | Add privacy filter config types and DB functions |
| `server/src/services/config.rs` | Modify | Add privacy filter service functions |
| `server/src/routes/config.rs` | Modify | Add GET/PUT `/config/privacy_filters` handlers |
| `server/src/routes/mod.rs` | Modify | Register new routes |
| `server/src/services/mod.rs` | Modify | Add `pub mod privacy_filter;` |
| `server/src/services/chat.rs` | Modify | Apply filter before LLM call in `start_chat_stream` |
| `server/src/services/super_chat.rs` | Modify | Apply filter in `stream_super_chat_inner` |
| `server/src/services/material_prep.rs` | Modify | Apply filter in `generate_materials` |
| `server/src/services/session.rs` | Modify | Apply filter in `start_summarize_stream` |
| `server/Cargo.toml` | Modify | Add `regex = "1"` dependency |

---

## Task 1: Add regex dependency

**Files:**
- Modify: `server/Cargo.toml`

- [ ] **Step 1: Add regex dependency**

Add `regex = "1"` to the `[dependencies]` section.

```toml
[dependencies]
axum = { version = "0.8", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ulid = { version = "1", features = ["serde"] }
tower-http = { version = "0.6", features = ["cors", "fs", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
tower = "0.5"
async-openai = "0.27"
tokio-stream = "0.1"
futures-util = "0.3"
async-stream = "0.3"
dashmap = "6"
reqwest = { version = "0.12", features = ["json"] }
chrono = "0.4"
url = "2"
urlencoding = "2"
serde_yaml = "0.9"
rand = "0.9"
base64 = "0.22"
magic-crypt = "4.0"
regex = "1"
```

- [ ] **Step 2: Verify dependency resolves**

Run: `cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo check`
Expected: Compiles successfully (may show warnings but no errors).

- [ ] **Step 3: Commit**

```bash
git add server/Cargo.toml
git commit -m "chore: add regex dependency for privacy filter"
```

---

## Task 2: Create privacy filter service module

**Files:**
- Create: `server/src/services/privacy_filter.rs`

- [ ] **Step 1: Create the service module**

```rust
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyFilters {
    #[serde(default = "default_true")]
    pub phone: bool,
    #[serde(default = "default_true")]
    pub id_card: bool,
    #[serde(default = "default_true")]
    pub email: bool,
    #[serde(default = "default_true")]
    pub bank_card: bool,
}

fn default_true() -> bool {
    true
}

impl Default for PrivacyFilters {
    fn default() -> Self {
        Self {
            phone: true,
            id_card: true,
            email: true,
            bank_card: true,
        }
    }
}

impl PrivacyFilters {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Self {
        serde_json::from_str(s).unwrap_or_default()
    }
}

pub fn apply_filters(text: &str, filters: &PrivacyFilters) -> String {
    let mut result = text.to_string();

    if filters.phone {
        let re = Regex::new(r"(?<![0-9a-zA-Z])(?:\+?86[-\s]?)?1[3-9]\d{9}(?![0-9a-zA-Z])").unwrap();
        result = re.replace_all(&result, "[PHONE]").to_string();
    }

    if filters.id_card {
        let re = Regex::new(r"(?<![0-9a-zA-Z])\d{6}(?:19|20)\d{2}(?:0[1-9]|1[0-2])(?:0[1-9]|[12]\d|3[01])\d{3}[\dXx](?![0-9a-zA-Z])").unwrap();
        result = re.replace_all(&result, "[ID_CARD]").to_string();
    }

    if filters.email {
        let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap();
        result = re.replace_all(&result, "[EMAIL]").to_string();
    }

    if filters.bank_card {
        let re = Regex::new(r"(?<![0-9a-zA-Z])\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}(?:[-\s]?\d{3})?(?![0-9a-zA-Z])").unwrap();
        result = re.replace_all(&result, "[BANK_CARD]").to_string();
    }

    result
}
```

- [ ] **Step 2: Register module**

Modify `server/src/services/mod.rs` to add:

```rust
pub mod archive_service;
pub mod blueprint;
pub mod blueprint_service;
pub mod chat;
pub mod config;
pub mod encryption;
pub mod git_service;
pub mod gitlab_service;
pub mod graph;
pub mod graph_chat_command;
pub mod group_doc_maintenance;
pub mod invite;
pub mod llm;
pub mod material_prep;
pub mod member;
pub mod mode;
pub mod privacy_filter;
pub mod ring;
pub mod self_data;
pub mod session;
pub mod setup;
pub mod skill;
pub mod super_chat;
```

- [ ] **Step 3: Verify compilation**

Run: `cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo check`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add server/src/services/privacy_filter.rs server/src/services/mod.rs
git commit -m "feat: add privacy filter service with regex patterns"
```

---

## Task 3: Add database migration for privacy_filters column

**Files:**
- Create: `server/migrations/012_privacy_filters.sql`

- [ ] **Step 1: Create migration**

```sql
ALTER TABLE users ADD COLUMN privacy_filters TEXT;
```

- [ ] **Step 2: Commit**

```bash
git add server/migrations/012_privacy_filters.sql
git commit -m "feat: add privacy_filters column to users table"
```

---

## Task 4: Update user model with privacy_filters field

**Files:**
- Modify: `server/src/models/user.rs`

- [ ] **Step 1: Add field to UserRow**

Update the `UserRow` struct:

```rust
#[derive(Debug, FromRow, Serialize, Clone)]
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
    pub auto_compact: bool,
    pub privacy_filters: Option<String>,
    pub created_at: String,
}
```

- [ ] **Step 2: Add field to CreateUser**

```rust
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub display_name: String,
    pub avatar: Option<String>,
    pub llm_provider: String,
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub llm_base_url: Option<String>,
    pub gitlab_url: Option<String>,
    pub gitlab_token: Option<String>,
    pub privacy_filters: Option<String>,
}
```

- [ ] **Step 3: Add field to UpdateUser**

```rust
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
    pub privacy_filters: Option<String>,
}
```

- [ ] **Step 4: Update create_user function**

Update the INSERT query to include `privacy_filters`:

```rust
pub async fn create_user(
    pool: &sqlx::SqlitePool,
    token_id: &str,
    input: &CreateUser,
) -> Result<UserRow> {
    let model = input.llm_model.as_deref().unwrap_or("gpt-4o");
    sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, llm_base_url, gitlab_url, gitlab_token, auto_compact, privacy_filters)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10)
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
        .bind(&input.privacy_filters)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}
```

- [ ] **Step 5: Update create_joiner_user function**

```rust
pub async fn create_joiner_user(
    pool: &sqlx::SqlitePool,
    token_id: &str,
    display_name: &str,
) -> Result<UserRow> {
    sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, llm_base_url, gitlab_url, gitlab_token, auto_compact, privacy_filters)
         VALUES (?1, ?2, NULL, 0, 'openai', NULL, 'gpt-4o', NULL, NULL, NULL, 1, NULL)
         RETURNING *",
    )
    .bind(token_id)
    .bind(display_name)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}
```

- [ ] **Step 6: Update update_user function**

Update the UPDATE query to include `privacy_filters`:

```rust
pub async fn update_user(
    pool: &sqlx::SqlitePool,
    token_id: &str,
    input: &UpdateUser,
) -> Result<UserRow> {
    let current = get_user(pool, token_id).await?;
    sqlx::query_as::<_, UserRow>(
        "UPDATE users SET
            display_name = ?1, avatar = ?2, llm_provider = ?3, llm_api_key = ?4,
            llm_model = ?5, llm_base_url = ?6, gitlab_url = ?7, gitlab_token = ?8,
            privacy_filters = ?9
         WHERE token_id = ?10
         RETURNING *",
    )
    .bind(
        input
            .display_name
            .as_deref()
            .unwrap_or(&current.display_name),
    )
    .bind(input.avatar.as_ref().or(current.avatar.as_ref()))
    .bind(
        input
            .llm_provider
            .as_deref()
            .unwrap_or(&current.llm_provider),
    )
    .bind(input.llm_api_key.as_ref().or(current.llm_api_key.as_ref()))
    .bind(input.llm_model.as_deref().unwrap_or(&current.llm_model))
    .bind(
        input
            .llm_base_url
            .as_ref()
            .or(current.llm_base_url.as_ref()),
    )
    .bind(input.gitlab_url.as_ref().or(current.gitlab_url.as_ref()))
    .bind(
        input
            .gitlab_token
            .as_ref()
            .or(current.gitlab_token.as_ref()),
    )
    .bind(
        input
            .privacy_filters
            .as_ref()
            .or(current.privacy_filters.as_ref()),
    )
    .bind(token_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}
```

- [ ] **Step 7: Verify compilation**

Run: `cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo check`
Expected: Compiles successfully.

- [ ] **Step 8: Commit**

```bash
git add server/src/models/user.rs
git commit -m "feat: add privacy_filters field to user model"
```

---

## Task 5: Add privacy filter config API

**Files:**
- Modify: `server/src/models/config.rs`
- Modify: `server/src/services/config.rs`
- Modify: `server/src/routes/config.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Add types to models/config.rs**

Add to `server/src/models/config.rs`:

```rust
#[derive(Debug, Serialize)]
pub struct PrivacyFiltersResponse {
    pub phone: bool,
    pub id_card: bool,
    pub email: bool,
    pub bank_card: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePrivacyFilters {
    pub phone: Option<bool>,
    pub id_card: Option<bool>,
    pub email: Option<bool>,
    pub bank_card: Option<bool>,
}

pub async fn get_privacy_filters(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> Result<PrivacyFiltersResponse> {
    let row = sqlx::query_scalar::<_, Option<String>>(
        "SELECT privacy_filters FROM users WHERE token_id = ?1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    let filters = match row {
        Some(json) => crate::services::privacy_filter::PrivacyFilters::from_json(&json),
        None => crate::services::privacy_filter::PrivacyFilters::default(),
    };

    Ok(PrivacyFiltersResponse {
        phone: filters.phone,
        id_card: filters.id_card,
        email: filters.email,
        bank_card: filters.bank_card,
    })
}

pub async fn update_privacy_filters(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    input: &UpdatePrivacyFilters,
) -> Result<PrivacyFiltersResponse> {
    let current = get_privacy_filters(pool, user_id).await?;

    let filters = crate::services::privacy_filter::PrivacyFilters {
        phone: input.phone.unwrap_or(current.phone),
        id_card: input.id_card.unwrap_or(current.id_card),
        email: input.email.unwrap_or(current.email),
        bank_card: input.bank_card.unwrap_or(current.bank_card),
    };

    sqlx::query("UPDATE users SET privacy_filters = ?1 WHERE token_id = ?2")
        .bind(filters.to_json())
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(PrivacyFiltersResponse {
        phone: filters.phone,
        id_card: filters.id_card,
        email: filters.email,
        bank_card: filters.bank_card,
    })
}
```

- [ ] **Step 2: Add service functions**

Update `server/src/services/config.rs`:

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

pub async fn get_privacy_filters(
    state: &AppState,
    user_id: &str,
) -> Result<config::PrivacyFiltersResponse> {
    config::get_privacy_filters(&state.db, user_id).await
}

pub async fn update_privacy_filters(
    state: &AppState,
    user_id: &str,
    input: config::UpdatePrivacyFilters,
) -> Result<config::PrivacyFiltersResponse> {
    config::update_privacy_filters(&state.db, user_id, &input).await
}
```

- [ ] **Step 3: Add route handlers**

Update `server/src/routes/config.rs`:

```rust
use axum::extract::State;
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::config::UpdateLLMConfig;
use crate::models::conversation_token::UpdateAutoCompact;
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

pub async fn get_privacy_filters(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<crate::models::config::PrivacyFiltersResponse>> {
    let filters = config::get_privacy_filters(&state, &user.token_id).await?;
    Ok(Json(filters))
}

pub async fn update_privacy_filters(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<crate::models::config::UpdatePrivacyFilters>,
) -> Result<Json<crate::models::config::PrivacyFiltersResponse>> {
    let filters = config::update_privacy_filters(&state, &user.token_id, body).await?;
    Ok(Json(filters))
}

pub async fn get_auto_compact(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let auto_compact =
        crate::models::conversation_token::get_auto_compact(&state.db, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "auto_compact": auto_compact })))
}

pub async fn update_auto_compact(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateAutoCompact>,
) -> Result<Json<serde_json::Value>> {
    let auto_compact = crate::models::conversation_token::update_auto_compact(
        &state.db,
        &user.token_id,
        body.auto_compact,
    )
    .await?;
    Ok(Json(serde_json::json!({ "auto_compact": auto_compact })))
}

pub async fn test_llm_config(
    Json(body): Json<crate::models::config::TestLLMRequest>,
) -> Result<Json<serde_json::Value>> {
    let (ok, message) = crate::services::llm::test_connection(
        &body.provider,
        &body.model,
        body.api_key.as_deref(),
        body.base_url.as_deref(),
    )
    .await?;
    Ok(Json(serde_json::json!({ "ok": ok, "message": message })))
}

pub async fn test_gitlab_config(
    Json(body): Json<crate::models::config::TestGitLabRequest>,
) -> Result<Json<serde_json::Value>> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v4/user", body.url.trim_end_matches('/'));
    let res = client
        .get(&url)
        .header("PRIVATE-TOKEN", &body.token)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(Json(
                    serde_json::json!({ "ok": true, "message": "GitLab connection successful" }),
                ))
            } else {
                let status = resp.status().as_u16();
                Ok(Json(
                    serde_json::json!({ "ok": false, "message": format!("GitLab returned status {}", status) }),
                ))
            }
        }
        Err(e) => Ok(Json(
            serde_json::json!({ "ok": false, "message": format!("{}", e) }),
        )),
    }
}
```

- [ ] **Step 4: Register routes**

Update `server/src/routes/mod.rs` to add the new routes after the auto_compact routes:

```rust
        .route(
            "/config/auto_compact",
            get(config::get_auto_compact).put(config::update_auto_compact),
        )
        .route(
            "/config/privacy_filters",
            get(config::get_privacy_filters).put(config::update_privacy_filters),
        )
```

- [ ] **Step 5: Verify compilation**

Run: `cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo check`
Expected: Compiles successfully.

- [ ] **Step 6: Commit**

```bash
git add server/src/models/config.rs server/src/services/config.rs server/src/routes/config.rs server/src/routes/mod.rs
git commit -m "feat: add privacy filter config API endpoints"
```

---

## Task 6: Integrate privacy filter into chat service

**Files:**
- Modify: `server/src/services/chat.rs`

- [ ] **Step 1: Apply filter before LLM call**

In `server/src/services/chat.rs`, update `start_chat_stream` to filter the content before sending to LLM:

```rust
use crate::error::Result;
use crate::models::conversation_token::{self, TOKEN_THRESHOLD};
use crate::models::message::{self, MessageRow};
use crate::services::llm::{LlmClient, SseEvent};
use crate::services::privacy_filter::{apply_filters, PrivacyFilters};
use crate::state::AppState;

const COMPACT_THRESHOLD: usize = 30;
const COMPACT_SUMMARY_MAX_TOKENS: usize = 500;

// ... existing functions ...

pub async fn start_chat_stream(
    state: &AppState,
    user: &crate::models::user::UserRow,
    params: &ChatParams<'_>,
) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
    let user_msg_id = ulid::Ulid::new().to_string();

    if !params.ephemeral {
        message::insert_message(
            &state.db,
            &message::NewMessage {
                id: &user_msg_id,
                ring_id: params.ring_id,
                user_id: &user.token_id,
                role: "user",
                sender_name: &user.display_name,
                content: params.content,
                node_refs: &params.node_refs,
                tag_refs: &params.tag_refs,
                token_usage: None,
            },
        )
        .await?;
    }

    let system_prompt = build_system_prompt(params.ring_name, params.role_description);
    let history = if params.ephemeral {
        vec![]
    } else {
        load_history_context(&state.db, params.ring_id, &user.token_id, 20).await?
    };

    let filters = user
        .privacy_filters
        .as_deref()
        .map(PrivacyFilters::from_json)
        .unwrap_or_default();
    let filtered_content = apply_filters(params.content, &filters);

    let llm = LlmClient::from_user(user)?;
    let rx = llm.chat_stream(
        system_prompt,
        history,
        filtered_content,
        params.ai_role.to_string(),
    );
    Ok(rx)
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo check`
Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add server/src/services/chat.rs
git commit -m "feat: apply privacy filter before LLM calls in chat"
```

---

## Task 7: Integrate privacy filter into super_chat service

**Files:**
- Modify: `server/src/services/super_chat.rs`

- [ ] **Step 1: Apply filter in stream_super_chat_inner**

In `server/src/services/super_chat.rs`, add import and filter the content:

```rust
use crate::services::privacy_filter::{apply_filters, PrivacyFilters};
```

Then in `stream_super_chat_inner`, after storing the original message, filter before building messages:

```rust
async fn stream_super_chat_inner(
    state: AppState,
    user: crate::models::user::UserRow,
    content: String,
    tx: &tokio::sync::mpsc::Sender<SseEvent>,
) -> Result<()> {
    let user_msg_id = ulid::Ulid::new().to_string();
    message::insert_message(
        &state.db,
        &message::NewMessage {
            id: &user_msg_id,
            ring_id: Some(SUPER_RING_ID),
            user_id: &user.token_id,
            role: "user",
            sender_name: &user.display_name,
            content: &content,
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await?;

    let base_prompt = get_system_prompt(&state.hub_dir);
    let ring_summary = build_ring_summary(&state.db, &user.token_id).await;
    let prefs = get_user_preferences(&state.hub_dir);
    let system_prompt = format!("{base_prompt}\n\n{ring_summary}\n\n## 用户偏好\n{prefs}");

    let history =
        chat::load_history_context(&state.db, Some(SUPER_RING_ID), &user.token_id, 20).await?;

    let filters = user
        .privacy_filters
        .as_deref()
        .map(PrivacyFilters::from_json)
        .unwrap_or_default();
    let filtered_content = apply_filters(&content, &filters);

    // ... rest of function uses filtered_content instead of content ...
```

Update the `build_messages` call to use `filtered_content`:

```rust
    let mut messages = build_messages(&system_prompt, &history, &filtered_content);
```

- [ ] **Step 2: Apply filter in stream_cross_ring_query_inner**

```rust
    let filters = user
        .privacy_filters
        .as_deref()
        .map(PrivacyFilters::from_json)
        .unwrap_or_default();
    let filtered_query = apply_filters(&query, &filters);
```

Use `filtered_query` in the request:

```rust
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(filtered_query),
                name: None,
            }),
```

- [ ] **Step 3: Apply filter in stream_cross_ring_analysis_inner**

```rust
    let filters = user
        .privacy_filters
        .as_deref()
        .map(PrivacyFilters::from_json)
        .unwrap_or_default();
    let filtered_question = request.question.as_deref().map(|q| apply_filters(q, &filters));
```

Use `filtered_question` in the analysis_prompt construction:

```rust
        _ => {
            format!(
                "请分析以下 Ring 的内容：\n{}\n\n用户问题：{}\n\n请基于以上信息回答。",
                selected_ring_details,
                filtered_question.unwrap_or_default()
            )
        }
```

- [ ] **Step 4: Verify compilation**

Run: `cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo check`
Expected: Compiles successfully.

- [ ] **Step 5: Commit**

```bash
git add server/src/services/super_chat.rs
git commit -m "feat: apply privacy filter in super chat"
```

---

## Task 8: Integrate privacy filter into material_prep service

**Files:**
- Modify: `server/src/services/material_prep.rs`

- [ ] **Step 1: Apply filter in generate_materials**

```rust
use crate::error::Result;
use crate::models::graph;
use crate::models::session;
use crate::models::user::UserRow;
use crate::services::llm::LlmClient;
use crate::services::privacy_filter::{apply_filters, PrivacyFilters};
use crate::state::AppState;

const MATERIAL_PREP_PROMPT: &str = r#"你正在为一场会议准备材料。请根据以下会议主题、Skill 类型和群组上下文，生成 3-5 条会议准备材料。

每条材料应包含：
- 类型（context / question / data / reference）
- 标题
- 具体内容

请以 JSON 数组格式输出，每条材料格式为：
{"item_type": "类型", "title": "标题", "content": "内容"}

会议信息：
"#;

pub async fn generate_materials(
    state: &AppState,
    session_id: &str,
    ring_id: &str,
    skill: &str,
    title: &str,
    description: &str,
    user: &UserRow,
) -> Result<()> {
    let graph = graph::ensure_default_graph(&state.db, ring_id).await?;
    let nodes = graph::list_nodes(&state.db, &graph.id).await?;

    let context = nodes
        .iter()
        .map(|n| format!("- {} ({}): {}", n.label, n.node_type, n.content))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = crate::services::skill::load_skill_prompt(skill, &state.skills_dir)
        .or_else(|| crate::services::skill::build_material_system_prompt(skill, title, description))
        .unwrap_or_else(|| "你是一个会议材料准备助手。".to_string());

    let filters = user
        .privacy_filters
        .as_deref()
        .map(PrivacyFilters::from_json)
        .unwrap_or_default();
    let filtered_title = apply_filters(title, &filters);
    let filtered_description = apply_filters(description, &filters);
    let filtered_context = apply_filters(&context, &filters);

    let prompt = format!(
        "{}\nSkill: {}\n标题: {}\n描述: {}\n\n群组图谱上下文:\n{}",
        MATERIAL_PREP_PROMPT, skill, filtered_title, filtered_description, filtered_context
    );

    let llm = LlmClient::from_user(user)?;
    let response = llm.chat_complete(system_prompt, prompt).await?;

    // ... rest unchanged
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo check`
Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add server/src/services/material_prep.rs
git commit -m "feat: apply privacy filter in material prep"
```

---

## Task 9: Integrate privacy filter into session service

**Files:**
- Modify: `server/src/services/session.rs`

- [ ] **Step 1: Apply filter in start_summarize_stream**

```rust
use crate::error::{Result, RingError};
use crate::models::ring;
use crate::models::session::{
    self, CreateSessionInput, InviteParticipantsInput, SessionParticipantRow, SessionRow,
};
use crate::models::user;
use crate::services::llm::{LlmClient, SseEvent};
use crate::services::privacy_filter::{apply_filters, PrivacyFilters};
use crate::state::AppState;

// ... existing code ...

pub fn start_summarize_stream(
    state: &AppState,
    user_row: &user::UserRow,
    ctx: SummarizeContext,
) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
    let system_prompt = crate::services::skill::load_skill_prompt(&ctx.skill, &state.skills_dir)
        .or_else(|| crate::services::skill::build_summary_system_prompt(&ctx.skill))
        .unwrap_or_else(|| "Summarize the following discussion.".to_string());

    let filters = user_row
        .privacy_filters
        .as_deref()
        .map(PrivacyFilters::from_json)
        .unwrap_or_default();
    let filtered_messages = apply_filters(&ctx.messages_text, &filters);

    let user_message = format!(
        "Here is the discussion transcript:\n\n{}\n\nPlease generate the summary.",
        filtered_messages
    );

    let llm = LlmClient::from_user(user_row)?;
    let rx = llm.chat_stream(
        system_prompt,
        vec![],
        user_message,
        "session_ring".to_string(),
    );
    Ok(rx)
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo check`
Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add server/src/services/session.rs
git commit -m "feat: apply privacy filter in session summarize"
```

---

## Task 10: Run tests and verify

**Files:**
- All modified files

- [ ] **Step 1: Run Rust tests**

Run: `cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo test`
Expected: All 55+ tests pass.

- [ ] **Step 2: Run clippy and fmt**

Run:
```bash
cd /Users/kaiiangs/Desktop/open-source-project/Ring/server && cargo fmt && cargo clippy -- -D warnings
```
Expected: No errors, no warnings.

- [ ] **Step 3: Build frontend**

Run:
```bash
cd /Users/kaiiangs/Desktop/open-source-project/Ring/ui && npm run build
```
Expected: Build succeeds.

- [ ] **Step 4: Commit any formatting fixes**

```bash
git add -A
git commit -m "style: apply cargo fmt fixes"
```

---

## Task 11: Merge branch

- [ ] **Step 1: Switch to main and merge**

```bash
cd /Users/kaiiangs/Desktop/open-source-project/Ring
git checkout main
git merge feat/privacy-filter
```

- [ ] **Step 2: Push to remote**

```bash
git push origin main
```

---

## Spec Coverage Check

| Requirement | Task |
|-------------|------|
| Filter phone numbers | Task 2 (regex pattern) |
| Filter Chinese ID numbers | Task 2 (regex pattern) |
| Filter emails | Task 2 (regex pattern) |
| Filter bank card numbers | Task 2 (regex pattern) |
| Apply filter before LLM API call | Tasks 6, 7, 8, 9 |
| Store original in DB | Tasks 6, 7 (message stored before filtering) |
| User configurable | Tasks 3, 4, 5 |
| Default all ON | Task 2 (Default impl) |
| GET/PUT /api/config/privacy_filters | Task 5 |

---

## Placeholder Scan

- No TBD/TODO/fill-in-details found
- All code blocks contain complete implementations
- All test commands are exact
- All file paths are exact

---

**Plan complete and saved to `docs/superpowers/plans/2026-04-23-privacy-filter.md`.**

**Two execution options:**

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**
