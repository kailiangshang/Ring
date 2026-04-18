# Super Ring Basic Chat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Super Ring chat endpoint (POST /api/super/chat) that reuses the existing messages table with `ring_id = "super"`, plus system prompt file management and frontend integration.

**Architecture:** New service (`super_chat.rs`) calls existing `message::insert_message` and `chat::load_history_context` with `ring_id = Some("super")`. New route handler follows the `self_chat` SSE pattern. Frontend `chat-store.ts` adds `'super'` context support. System prompt reads from `~/.ring/hub/system_prompt.md` with code fallback.

**Tech Stack:** Rust, Axum, sqlx, async-openai SSE, tokio, std::fs

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `server/src/main.rs` | Modify | Create `~/.ring/hub/` dir on startup |
| `server/src/services/super_chat.rs` | Create | System prompt I/O + start_super_chat + get_super_history |
| `server/src/services/mod.rs` | Modify | Register super_chat module |
| `server/src/routes/super_chat.rs` | Create | 4 endpoints: chat SSE, history, get/put system prompt |
| `server/src/routes/mod.rs` | Modify | Register routes |
| `ui/src/stores/chat-store.ts` | Modify | Handle `'super'` context in send() and loadHistory() |

---

### Task 1: Create hub directory on startup

**Files:**
- Modify: `server/src/main.rs` (after line 27, before `AppState::new`)

- [ ] **Step 1: Add hub dir creation**

After line 27 (`std::fs::create_dir_all(&rings_dir).expect("failed to create rings dir");`), add:

```rust
    let hub_dir = std::path::PathBuf::from(format!("{data_dir}/hub"));
    std::fs::create_dir_all(&hub_dir).expect("failed to create hub dir");
```

- [ ] **Step 2: Pass hub_dir to AppState**

Change the `AppState::new` call and add `hub_dir` field. First, update `server/src/state.rs`:

Add `pub hub_dir: PathBuf` to `AppState` struct, and update constructor:

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub ws_hub: WsHub,
    pub rings_dir: PathBuf,
    pub hub_dir: PathBuf,
}

impl AppState {
    pub fn new(db: SqlitePool, rings_dir: PathBuf, hub_dir: PathBuf) -> Self {
        Self {
            db,
            ws_hub: WsHub::new(),
            rings_dir,
            hub_dir,
        }
    }
}
```

Then update `main.rs` line 29:

```rust
    let state = AppState::new(pool, rings_dir, hub_dir);
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles. There will be compiler errors from existing code that calls `AppState::new` with 2 args — fix the test file `server/tests/integration.rs` to pass a dummy hub_dir:

In `setup_app()` function, change:
```rust
AppState::new(pool, std::path::PathBuf::from("/tmp/ring-test-rings"))
```
to:
```rust
AppState::new(pool, std::path::PathBuf::from("/tmp/ring-test-rings"), std::path::PathBuf::from("/tmp/ring-test-hub"))
```

- [ ] **Step 4: Run all tests**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all 15 tests pass

- [ ] **Step 5: Commit**

```bash
git add server/src/main.rs server/src/state.rs server/tests/integration.rs
git commit -m "feat: add hub_dir to AppState and create ~/.ring/hub on startup"
```

---

### Task 2: Create super_chat service

**Files:**
- Create: `server/src/services/super_chat.rs`
- Modify: `server/src/services/mod.rs`

- [ ] **Step 1: Create `server/src/services/super_chat.rs`**

```rust
use std::path::Path;

use crate::error::Result;
use crate::models::message::{self, MessageRow};
use crate::services::chat;
use crate::services::llm::{LlmClient, SseEvent};
use crate::state::AppState;

const SUPER_RING_ID: &str = "super";

const DEFAULT_SUPER_SYSTEM_PROMPT: &str = "你是 Super Ring，用户的全局 AI 助手和跨 Ring 协调者。\n\n你的职责：\n1. Ring 管理引导 — 帮助用户创建、配置 Ring\n2. 跨 Ring 分析 — 按需读取所有 Ring 的内容，进行汇总、对比、推荐\n3. 使用引导 — 回答关于 Ring 产品功能的问题\n\n请用简洁、专业的方式回答。";

pub fn get_system_prompt(hub_dir: &Path) -> String {
    let prompt_file = hub_dir.join("system_prompt.md");
    match std::fs::read_to_string(&prompt_file) {
        Ok(content) if !content.trim().is_empty() => content,
        _ => DEFAULT_SUPER_SYSTEM_PROMPT.to_string(),
    }
}

pub fn get_system_prompt_info(hub_dir: &Path) -> (String, bool) {
    let prompt_file = hub_dir.join("system_prompt.md");
    match std::fs::read_to_string(&prompt_file) {
        Ok(ref content) if !content.trim().is_empty() => (content.clone(), true),
        _ => (DEFAULT_SUPER_SYSTEM_PROMPT.to_string(), false),
    }
}

pub fn update_system_prompt(hub_dir: &Path, prompt: &str) -> Result<()> {
    let prompt_file = hub_dir.join("system_prompt.md");
    if prompt.trim().is_empty() {
        let _ = std::fs::remove_file(&prompt_file);
    } else {
        std::fs::write(&prompt_file, prompt)?;
    }
    Ok(())
}

pub async fn start_super_chat(
    state: &AppState,
    user: &crate::models::user::UserRow,
    content: &str,
) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
    let user_msg_id = ulid::Ulid::new().to_string();
    message::insert_message(
        &state.db,
        &message::NewMessage {
            id: &user_msg_id,
            ring_id: Some(SUPER_RING_ID),
            user_id: &user.token_id,
            role: "user",
            sender_name: &user.display_name,
            content,
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await?;

    let system_prompt = get_system_prompt(&state.hub_dir);
    let history =
        chat::load_history_context(&state.db, Some(SUPER_RING_ID), &user.token_id, 20).await?;

    let llm = LlmClient::from_user(user)?;
    let rx = llm.chat_stream(
        system_prompt,
        history,
        content.to_string(),
        "super_ring".to_string(),
    );
    Ok(rx)
}

pub async fn get_super_history(
    state: &AppState,
    user_id: &str,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    chat::get_history(state, Some(SUPER_RING_ID), user_id, before_id, limit).await
}
```

- [ ] **Step 2: Register module in `server/src/services/mod.rs`**

Add at the end:

```rust
pub mod super_chat;
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 4: Commit**

```bash
git add server/src/services/super_chat.rs server/src/services/mod.rs
git commit -m "feat: add super_chat service with system prompt management"
```

---

### Task 3: Create super_chat routes

**Files:**
- Create: `server/src/routes/super_chat.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Create `server/src/routes/super_chat.rs`**

```rust
use async_stream::stream;
use axum::extract::{Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use serde::Deserialize;
use std::convert::Infallible;

use crate::error::Result;
use crate::extractors::auth::AuthUser;
use crate::models::message;
use crate::models::user;
use crate::services::{llm::SseEvent, super_chat};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub before: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, serde::Serialize)]
pub struct HistoryResponse {
    pub messages: Vec<message::MessageRow>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
pub struct SystemPromptRequest {
    pub prompt: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SystemPromptResponse {
    pub prompt: String,
    pub is_custom: bool,
}

pub async fn super_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user_row = user::get_user(&state.db, &user.token_id).await?;

    let mut rx = super_chat::start_super_chat(&state, &user_row, &body.content).await?;

    let pool = state.db.clone();
    let user_id = user.token_id.clone();

    let s = stream! {
        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::Start { message_id, role } => {
                    let data = serde_json::json!({"message_id": message_id, "role": role});
                    yield Ok(Event::default().event("message_start").data(data.to_string()));
                }
                SseEvent::Delta { content } => {
                    let data = serde_json::json!({ "content": content });
                    yield Ok(Event::default().event("delta").data(data.to_string()));
                }
                SseEvent::End { message_id, full_content } => {
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": { "prompt_tokens": 0, "completion_tokens": 0 }
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    let _ = message::insert_message(
                        &pool,
                        &message::NewMessage {
                            id: &message_id,
                            ring_id: Some("super"),
                            user_id: &user_id,
                            role: "super_ring",
                            sender_name: "SUPER RING",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: None,
                        },
                    ).await;
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    };

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn super_history(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let limit = query.limit + 1;
    let messages = super_chat::get_super_history(
        &state,
        &user.token_id,
        query.before.as_deref(),
        limit,
    )
    .await?;

    let has_more = messages.len() > query.limit as usize;
    let messages = if has_more {
        messages.into_iter().take(query.limit as usize).collect()
    } else {
        messages
    };

    Ok(Json(HistoryResponse { messages, has_more }))
}

pub async fn get_system_prompt(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<SystemPromptResponse>> {
    let (prompt, is_custom) = super_chat::get_system_prompt_info(&state.hub_dir);
    Ok(Json(SystemPromptResponse { prompt, is_custom }))
}

pub async fn update_system_prompt(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<SystemPromptRequest>,
) -> Result<Json<SystemPromptResponse>> {
    super_chat::update_system_prompt(&state.hub_dir, &body.prompt)?;
    let (prompt, is_custom) = super_chat::get_system_prompt_info(&state.hub_dir);
    Ok(Json(SystemPromptResponse { prompt, is_custom }))
}
```

- [ ] **Step 2: Register routes in `server/src/routes/mod.rs`**

Add a new module declaration after `mod ws;`:

```rust
mod super_chat;
```

Add routes after the `/api/self/chat/history` route, before `.with_state(state)`:

```rust
        .route("/super/chat", post(super_chat::super_chat))
        .route("/super/chat/history", get(super_chat::super_history))
        .route(
            "/super/system-prompt",
            get(super_chat::get_system_prompt).put(super_chat::update_system_prompt),
        )
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/super_chat.rs server/src/routes/mod.rs
git commit -m "feat: add Super Ring chat, history, and system prompt endpoints"
```

---

### Task 4: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add Super Ring tests**

Add at the end of `server/tests/integration.rs`:

```rust
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
    let state = setup_app().await;
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
    assert!(json["prompt"].as_str().unwrap().len() > 0);
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
```

- [ ] **Step 2: Run new tests**

Run: `cargo test --manifest-path server/Cargo.toml test_super`
Expected: all 3 new tests pass

- [ ] **Step 3: Run all tests**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all 18 tests pass (15 existing + 3 new)

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test: add Super Ring history and system prompt integration tests"
```

---

### Task 5: Frontend chat-store integration

**Files:**
- Modify: `ui/src/stores/chat-store.ts`

- [ ] **Step 1: Update `send()` function**

In `ui/src/stores/chat-store.ts`, find the section (around line 151-157):

```typescript
    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat`
    } else {
      set({ sending: false })
      return
    }
```

Replace with:

```typescript
    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat`
    } else if (context === 'super') {
      url = '/api/super/chat'
    } else {
      set({ sending: false })
      return
    }
```

- [ ] **Step 2: Update `onStart` sender_name**

Find (around line 164):

```typescript
          sender_name: data.role === 'group_ring' ? 'GROUP RING' : data.role.toUpperCase(),
```

Replace with:

```typescript
          sender_name: data.role === 'group_ring' ? 'GROUP RING' : data.role === 'super_ring' ? 'SUPER RING' : data.role.toUpperCase(),
```

- [ ] **Step 3: Update `loadHistory()` function**

Find (around line 218-223):

```typescript
    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat/history?limit=50`
    } else {
      return
    }
```

Replace with:

```typescript
    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat/history?limit=50`
    } else if (context === 'super') {
      url = '/api/super/chat/history?limit=50'
    } else {
      return
    }
```

- [ ] **Step 4: Run TypeScript check**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add ui/src/stores/chat-store.ts
git commit -m "feat: wire Super Ring chat and history into frontend chat-store"
```

---

### Task 6: Final verification

- [ ] **Step 1: Run cargo clippy**

Run: `cargo clippy --manifest-path server/Cargo.toml -- -D warnings`
Expected: no warnings

- [ ] **Step 2: Run cargo fmt**

Run: `cargo fmt --manifest-path server/Cargo.toml`

- [ ] **Step 3: Run full test suite**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all 18 tests pass

- [ ] **Step 4: Commit fmt changes if any**

```bash
git add -A && git commit -m "style: cargo fmt" || echo "no fmt changes"
```
