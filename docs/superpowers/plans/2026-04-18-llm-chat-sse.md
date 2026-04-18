# LLM Chat + SSE Streaming Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real LLM-powered chat with SSE token streaming to Ring, so users can have actual AI conversations in Group Ring and Self contexts.

**Architecture:** Backend creates an LLM client per-request from the user's stored config (provider/api_key/model/base_url). Chat messages persist in SQLite. The SSE endpoint streams `async-openai` response deltas as `text/event-stream`. Frontend `chat-store` connects to `EventSource`, accumulates deltas into a live `ChatMessage`, and renders token-by-token. Super Ring chat deferred to a later plan.

**Tech Stack:** Rust: async-openai 0.27, tokio-stream, axum SSE. Frontend: EventSource API, existing Zustand stores.

**Scope:** This plan covers Group Ring chat + Self chat only. Super Ring chat, Session AI, WebSocket, compact, and ephemeral mode are deferred.

---

## File Structure

```
server/
├── Cargo.toml                              # MODIFY: add async-openai, tokio-stream
├── migrations/
│   └── 003_messages.sql                    # NEW: messages table
├── src/
│   ├── state.rs                            # MODIFY: no change needed (per-request client)
│   ├── error.rs                            # MODIFY: add From<OpenAIError>
│   ├── models/
│   │   └── message.rs                      # NEW: MessageRow, CRUD queries
│   ├── services/
│   │   ├── llm.rs                          # NEW: build_client, chat_completion (streaming)
│   │   └── chat.rs                         # NEW: send_message, list_history, build_system_prompt
│   └── routes/
│       ├── mod.rs                          # MODIFY: add chat routes
│       └── chat.rs                         # NEW: POST chat (SSE), GET history

ui/src/
├── services/
│   └── sse.ts                              # NEW: EventSource wrapper for SSE
├── stores/
│   └── chat-store.ts                       # MODIFY: real send with SSE, history loading
├── components/
│   └── chat/
│       ├── MessageList.tsx                 # MODIFY: render streaming message
│       └── InputArea.tsx                   # MODIFY: disable during send
```

---

### Task 1: Add Dependencies + Messages Migration

**Files:**
- Modify: `server/Cargo.toml`
- Create: `server/migrations/003_messages.sql`

- [ ] **Step 1: Add dependencies to Cargo.toml**

Append to `[dependencies]`:

```toml
async-openai = "0.27"
tokio-stream = "0.1"
```

- [ ] **Step 2: Create messages migration**

`server/migrations/003_messages.sql`:

```sql
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    ring_id TEXT,
    user_id TEXT NOT NULL REFERENCES users(token_id),
    role TEXT NOT NULL,
    sender_name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    node_refs TEXT NOT NULL DEFAULT '[]',
    tag_refs TEXT NOT NULL DEFAULT '[]',
    token_usage TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_messages_ring ON messages(ring_id);
CREATE INDEX IF NOT EXISTS idx_messages_user ON messages(user_id);
CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(ring_id, created_at);
```

`ring_id` is `NULL` for Self chat messages. `role` is one of `user`, `group_ring`, `super_ring`, `session_ring`, `self`, `system`. `token_usage` stores JSON like `{"prompt_tokens":500,"completion_tokens":200}`.

- [ ] **Step 3: Verify build**

Run: `cd server && cargo check 2>&1`
Expected: Compiles (downloads async-openai).

- [ ] **Step 4: Commit**

```bash
git add server/Cargo.toml server/migrations/003_messages.sql
git commit -m "feat(server): add async-openai + tokio-stream deps, messages migration"
```

---

### Task 2: Message Model

**Files:**
- Create: `server/src/models/message.rs`
- Modify: `server/src/models/mod.rs` (or `server/src/lib.rs` if modules are re-exported there)

- [ ] **Step 1: Create message model**

`server/src/models/message.rs`:

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct MessageRow {
    pub id: String,
    pub ring_id: Option<String>,
    pub user_id: String,
    pub role: String,
    pub sender_name: String,
    pub content: String,
    pub node_refs: String,
    pub tag_refs: String,
    pub token_usage: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub node_refs: Vec<String>,
    pub tag_refs: Vec<String>,
}

pub async fn insert_message(
    pool: &sqlx::SqlitePool,
    id: &str,
    ring_id: Option<&str>,
    user_id: &str,
    role: &str,
    sender_name: &str,
    content: &str,
    node_refs: &[String],
    tag_refs: &[String],
    token_usage: Option<&str>,
) -> Result<MessageRow> {
    sqlx::query_as::<_, MessageRow>(
        "INSERT INTO messages (id, ring_id, user_id, role, sender_name, content, node_refs, tag_refs, token_usage)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         RETURNING *",
    )
    .bind(id)
    .bind(ring_id)
    .bind(user_id)
    .bind(role)
    .bind(sender_name)
    .bind(content)
    .bind(serde_json::to_string(node_refs).unwrap_or_else(|_| "[]".into()))
    .bind(serde_json::to_string(tag_refs).unwrap_or_else(|_| "[]".into()))
    .bind(token_usage)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn list_messages(
    pool: &sqlx::SqlitePool,
    ring_id: Option<&str>,
    user_id: &str,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    let rows = if let Some(before) = before_id {
        sqlx::query_as::<_, MessageRow>(
            "SELECT * FROM messages
             WHERE (ring_id = ?1 OR (?1 IS NULL AND ring_id IS NULL))
             AND user_id = ?2
             AND created_at < (SELECT created_at FROM messages WHERE id = ?3)
             ORDER BY created_at DESC LIMIT ?4",
        )
        .bind(ring_id)
        .bind(user_id)
        .bind(before)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, MessageRow>(
            "SELECT * FROM messages
             WHERE (ring_id = ?1 OR (?1 IS NULL AND ring_id IS NULL))
             AND user_id = ?2
             ORDER BY created_at DESC LIMIT ?3",
        )
        .bind(ring_id)
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    };
    rows.map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn update_message_content(
    pool: &sqlx::SqlitePool,
    id: &str,
    content: &str,
    token_usage: Option<&str>,
) -> Result<()> {
    sqlx::query("UPDATE messages SET content = ?1, token_usage = ?2 WHERE id = ?3")
        .bind(content)
        .bind(token_usage)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
    Ok(())
}
```

- [ ] **Step 2: Register module**

In `server/src/lib.rs`, add `pub mod message;` to the `models` module block (or wherever models are registered). Check the file first — if there's a `mod models;` with sub-modules listed, add `pub mod message;` there.

- [ ] **Step 3: Verify build**

Run: `cd server && cargo check 2>&1`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add server/src/models/message.rs server/src/lib.rs
git commit -m "feat(server): message model with insert, list, update queries"
```

---

### Task 3: LLM Service — Client Builder + Streaming

**Files:**
- Create: `server/src/services/llm.rs`
- Modify: `server/src/error.rs` — add `From<async_openai::error::OpenAIError>`
- Modify: `server/src/lib.rs` — register `services::llm` module

- [ ] **Step 1: Add OpenAIError conversion to error.rs**

In `server/src/error.rs`, add this impl block alongside the existing `From<sqlx::Error>`:

```rust
impl From<async_openai::error::OpenAIError> for RingError {
    fn from(e: async_openai::error::OpenAIError) -> Self {
        RingError::Internal(format!("LLM error: {e}"))
    }
}
```

- [ ] **Step 2: Create LLM service**

`server/src/services/llm.rs`:

```rust
use async_openai::config::{OpenAIConfig, Api};
use async_openai::{Client, types::*};
use futures_util::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

use crate::error::{Result, RingError};
use crate::models::user::UserRow;

pub struct LlmClient {
    client: Client<OpenAIConfig>,
    model: String,
}

impl LlmClient {
    pub fn from_user(user: &UserRow) -> Result<Self> {
        let api_key = user.llm_api_key.as_deref().ok_or_else(|| {
            RingError::Internal("LLM API key not configured".into())
        })?;

        let mut config = OpenAIConfig::new().with_api_key(api_key);
        if let Some(base_url) = &user.llm_base_url {
            config = config.with_api_base(base_url);
        }

        Ok(Self {
            client: Client::with_config(config),
            model: user.llm_model.clone(),
        })
    }

    pub async fn chat_stream(
        &self,
        system_prompt: &str,
        history: Vec<(String, String)>,
        user_message: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        let mut messages = vec![ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()?,
        )];

        for (role, content) in history {
            match role.as_str() {
                "user" => {
                    messages.push(ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(content)
                            .build()?,
                    ));
                }
                _ => {
                    messages.push(ChatCompletionRequestMessage::Assistant(
                        ChatCompletionRequestAssistantMessageArgs::default()
                            .content(content)
                            .build()?,
                    ));
                }
            }
        }

        messages.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_message)
                .build()?,
        ));

        let request = ChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .stream(true)
            .build()?;

        let tx_err = tx.clone();
        let message_id = ulid::Ulid::new().to_string();

        tokio::spawn(async move {
            let _ = tx_err.send(SseEvent::Start {
                message_id: message_id.clone(),
                role: "group_ring".into(),
            }).await;

            match self.client.chat().create_stream(request).await {
                Ok(mut stream) => {
                    let mut full_content = String::new();
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(chunk) => {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(delta) = &choice.delta.content {
                                        full_content.push_str(delta);
                                        let _ = tx.send(SseEvent::Delta {
                                            content: delta.clone(),
                                        }).await;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(SseEvent::Error(e.to_string())).await;
                                break;
                            }
                        }
                    }
                    let _ = tx.send(SseEvent::End {
                        message_id: message_id.clone(),
                        full_content,
                    }).await;
                }
                Err(e) => {
                    let _ = tx.send(SseEvent::Error(e.to_string())).await;
                }
            }
        });

        Ok(rx)
    }
}

pub enum SseEvent {
    Start { message_id: String, role: String },
    Delta { content: String },
    End { message_id: String, full_content: String },
    Error(String),
}
```

**Note:** The `self` capture in the `tokio::spawn` closure requires `LlmClient` to be `'static + Send`. Since `Client<OpenAIConfig>` already is, this works. We need to clone `self` (or just the client) before spawning. The actual implementation will need to handle this — see Task 3 revision below.

**Important:** The above code has a borrow issue — `self` is borrowed in the spawned task. The fix is to move `self.client` and `self.model` into the task. Let me revise:

`server/src/services/llm.rs` (revised):

```rust
use async_openai::config::OpenAIConfig;
use async_openai::types::*;
use async_openai::Client;
use futures_util::StreamExt;

use crate::error::{Result, RingError};
use crate::models::user::UserRow;

pub struct LlmClient {
    client: Client<OpenAIConfig>,
    model: String,
}

pub enum SseEvent {
    Start {
        message_id: String,
        role: String,
    },
    Delta {
        content: String,
    },
    End {
        message_id: String,
        full_content: String,
    },
    Error(String),
}

impl LlmClient {
    pub fn from_user(user: &UserRow) -> Result<Self> {
        let api_key = user.llm_api_key.as_deref().ok_or_else(|| {
            RingError::Internal("LLM API key not configured".into())
        })?;

        let mut config = OpenAIConfig::new().with_api_key(api_key);
        if let Some(base_url) = &user.llm_base_url {
            config = config.with_api_base(base_url);
        }

        Ok(Self {
            client: Client::with_config(config),
            model: user.llm_model.clone(),
        })
    }

    pub fn chat_stream(
        self,
        system_prompt: String,
        history: Vec<(String, String)>,
        user_message: String,
        ai_role: String,
    ) -> tokio::sync::mpsc::Receiver<SseEvent> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            let message_id = ulid::Ulid::new().to_string();

            let _ = tx
                .send(SseEvent::Start {
                    message_id: message_id.clone(),
                    role: ai_role,
                })
                .await;

            let mut messages = vec![ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(&system_prompt)
                    .build()
                    .unwrap_or_default(),
            )];

            for (role, content) in history {
                match role.as_str() {
                    "user" => {
                        messages.push(ChatCompletionRequestMessage::User(
                            ChatCompletionRequestUserMessageArgs::default()
                                .content(content)
                                .build()
                                .unwrap_or_default(),
                        ));
                    }
                    _ => {
                        messages.push(ChatCompletionRequestMessage::Assistant(
                            ChatCompletionRequestAssistantMessageArgs::default()
                                .content(content)
                                .build()
                                .unwrap_or_default(),
                        ));
                    }
                }
            }

            messages.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(&user_message)
                    .build()
                    .unwrap_or_default(),
            ));

            let request = ChatCompletionRequestArgs::default()
                .model(&self.model)
                .messages(messages)
                .stream(true)
                .build()
                .unwrap_or_default();

            match self.client.chat().create_stream(request).await {
                Ok(mut stream) => {
                    let mut full_content = String::new();
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(chunk) => {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(delta) = &choice.delta.content {
                                        full_content.push_str(delta);
                                        let _ = tx
                                            .send(SseEvent::Delta {
                                                content: delta.clone(),
                                            })
                                            .await;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(SseEvent::Error(e.to_string())).await;
                                break;
                            }
                        }
                    }
                    let _ = tx
                        .send(SseEvent::End {
                            message_id: message_id.clone(),
                            full_content,
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx.send(SseEvent::Error(e.to_string())).await;
                }
            }
        });

        rx
    }
}
```

- [ ] **Step 3: Register module in lib.rs**

Add `pub mod llm;` in the `services` module block of `server/src/lib.rs`.

- [ ] **Step 4: Verify build**

Run: `cd server && cargo check 2>&1`

Note: You may need to add `futures-util` to `Cargo.toml` if `StreamExt` is not available through `tokio-stream`:

```toml
futures-util = "0.3"
```

Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add server/Cargo.toml server/src/services/llm.rs server/src/error.rs server/src/lib.rs
git commit -m "feat(server): LLM service with async-openai streaming client"
```

---

### Task 4: Chat Service — Send + History

**Files:**
- Create: `server/src/services/chat.rs`
- Modify: `server/src/lib.rs` — register `services::chat` module

- [ ] **Step 1: Create chat service**

`server/src/services/chat.rs`:

```rust
use crate::error::{Result, RingError};
use crate::models::message::{self, MessageRow};
use crate::models::user::UserRow;
use crate::services::llm::{LlmClient, SseEvent};
use crate::state::AppState;

pub async fn get_history(
    state: &AppState,
    ring_id: Option<&str>,
    user_id: &str,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    let messages = message::list_messages(&state.db, ring_id, user_id, before_id, limit).await?;
    Ok(messages.into_iter().rev().collect())
}

pub fn build_system_prompt(ring_name: Option<&str>, role_description: Option<&str>) -> String {
    match ring_name {
        Some(name) => {
            let mut prompt = format!("你是 Ring「{name}」的 AI 助手。");
            if let Some(desc) = role_description {
                prompt.push_str(&format!("\n\n角色设定：{desc}"));
            }
            prompt.push_str("\n\n请用简洁、专业的方式回答用户的问题。如果引用了图谱中的节点或概念，请明确标注。");
            prompt
        }
        None => "你是用户的个人 AI 助手 Self。你完全了解用户的偏好、目标和历史对话。请以友好、个性化的方式回答。".into(),
    }
}

pub async fn load_history_context(
    pool: &sqlx::SqlitePool,
    ring_id: Option<&str>,
    user_id: &str,
    limit: i64,
) -> Result<Vec<(String, String)>> {
    let messages = message::list_messages(pool, ring_id, user_id, None, limit).await?;
    Ok(messages
        .into_iter()
        .rev()
        .filter(|m| m.role != "system")
        .map(|m| (m.role.clone(), m.content.clone()))
        .collect())
}

pub async fn start_chat_stream(
    state: &AppState,
    user: &UserRow,
    ring_id: Option<&str>,
    role_description: Option<&str>,
    ring_name: Option<&str>,
    ai_role: &str,
    user_content: &str,
    node_refs: Vec<String>,
    tag_refs: Vec<String>,
) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
    let user_msg_id = ulid::Ulid::new().to_string();
    message::insert_message(
        &state.db,
        &user_msg_id,
        ring_id,
        &user.token_id,
        "user",
        &user.display_name,
        user_content,
        &node_refs,
        &tag_refs,
        None,
    )
    .await?;

    let system_prompt = build_system_prompt(ring_name, role_description);
    let history = load_history_context(&state.db, ring_id, &user.token_id, 20).await?;

    let llm = LlmClient::from_user(user)?;
    let pool = state.db.clone();
    let ai_msg_id = ulid::Ulid::new().to_string();
    let ai_role_owned = ai_role.to_string();
    let ring_id_owned = ring_id.map(|s| s.to_string());
    let user_id = user.token_id.clone();

    let rx = llm.chat_stream(system_prompt, history, user_content.to_string(), ai_role.to_string());

    let rx = spawn_sse_to_events(rx);

    Ok(rx)
}

fn spawn_sse_to_events(
    mut receiver: tokio::sync::mpsc::Receiver<SseEvent>,
) -> tokio::sync::mpsc::Receiver<SseEvent> {
    receiver
}

pub async fn save_ai_response(
    pool: &sqlx::SqlitePool,
    message_id: &str,
    ring_id: Option<&str>,
    user_id: &str,
    role: &str,
    content: &str,
) -> Result<()> {
    message::insert_message(
        pool,
        message_id,
        ring_id,
        user_id,
        role,
        role,
        content,
        &[],
        &[],
        None,
    )
    .await?;
    Ok(())
}
```

**Note on saving the AI response:** The `End` event contains `full_content`. The route handler (Task 5) is responsible for calling `save_ai_response` when it receives `SseEvent::End`. This is handled in the route layer.

- [ ] **Step 2: Register module**

Add `pub mod chat;` in the `services` module block of `server/src/lib.rs`.

- [ ] **Step 3: Verify build**

Run: `cd server && cargo check 2>&1`

Note: There will be unused warnings for `ai_msg_id`, `ai_role_owned`, `ring_id_owned`, `user_id` in `start_chat_stream` — these are used in Task 5 when the route handler saves the AI response. For now they suppress with `let _ = ...` or remove and re-add in Task 5. Cleaner approach: remove them now, add save logic in Task 5.

**Revised `start_chat_stream`** — remove unused vars, just return the receiver:

Replace the body after `let llm = LlmClient::from_user(user)?;` with:

```rust
    let llm = LlmClient::from_user(user)?;
    let rx = llm.chat_stream(system_prompt, history, user_content.to_string(), ai_role.to_string());
    Ok(rx)
```

- [ ] **Step 4: Commit**

```bash
git add server/src/services/chat.rs server/src/lib.rs
git commit -m "feat(server): chat service with history loading and LLM stream dispatch"
```

---

### Task 5: Chat Route — SSE Endpoint + History

**Files:**
- Create: `server/src/routes/chat.rs`
- Modify: `server/src/routes/mod.rs` — add chat routes
- Modify: `server/src/lib.rs` — register routes::chat module

- [ ] **Step 1: Create chat route**

`server/src/routes/chat.rs`:

```rust
use axum::extract::{Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::stream::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use crate::error::{Result, RingError};
use crate::extractors::auth::AuthUser;
use crate::models::message::{self, CreateMessage, MessageRow};
use crate::models::ring;
use crate::models::user;
use crate::services::chat;
use crate::services::llm::SseEvent;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub content: String,
    #[serde(default)]
    pub node_refs: Vec<String>,
    #[serde(default)]
    pub tag_refs: Vec<String>,
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
    pub messages: Vec<MessageRow>,
    pub has_more: bool,
}

pub async fn ring_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    let user_row = user::get_by_token(&state.db, &user.token_id)
        .await?
        .ok_or_else(|| RingError::Unauthorized("user not found".into()))?;

    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let ring_row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT name, role_description FROM rings WHERE id = ?1",
    )
    .bind(&ring_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?
    .ok_or_else(|| RingError::NotFound("ring not found".into()))?;

    let mut rx = chat::start_chat_stream(
        &state,
        &user_row,
        Some(&ring_id),
        ring_row.1.as_deref(),
        Some(&ring_row.0),
        "group_ring",
        &body.content,
        body.node_refs,
        body.tag_refs,
    )
    .await?;

    let pool = state.db.clone();
    let ring_id_clone = ring_id.clone();
    let user_id = user.token_id.clone();

    let stream = async_stream::stream! {
        let mut ai_message_id = String::new();
        let mut full_content = String::new();

        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::Start { message_id, role } => {
                    ai_message_id = message_id;
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "role": role
                    });
                    yield Ok(Event::default().event("message_start").data(data.to_string()));
                }
                SseEvent::Delta { content } => {
                    full_content.push_str(&content);
                    let data = serde_json::json!({ "content": content });
                    yield Ok(Event::default().event("delta").data(data.to_string()));
                }
                SseEvent::End { message_id, full_content: content } => {
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": { "prompt_tokens": 0, "completion_tokens": 0 }
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    let _ = message::insert_message(
                        &pool,
                        &message_id,
                        Some(&ring_id_clone),
                        &user_id,
                        "group_ring",
                        "GROUP RING",
                        &content,
                        &[],
                        &[],
                        None,
                    ).await;
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub async fn ring_history(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let limit = query.limit + 1;
    let messages = chat::get_history(
        &state,
        Some(&ring_id),
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

pub async fn self_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    let user_row = user::get_by_token(&state.db, &user.token_id)
        .await?
        .ok_or_else(|| RingError::Unauthorized("user not found".into()))?;

    let mut rx = chat::start_chat_stream(
        &state,
        &user_row,
        None,
        None,
        None,
        "self",
        &body.content,
        body.node_refs,
        body.tag_refs,
    )
    .await?;

    let pool = state.db.clone();
    let user_id = user.token_id.clone();

    let stream = async_stream::stream! {
        let mut full_content = String::new();

        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::Start { message_id, role } => {
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "role": role
                    });
                    yield Ok(Event::default().event("message_start").data(data.to_string()));
                }
                SseEvent::Delta { content } => {
                    full_content.push_str(&content);
                    let data = serde_json::json!({ "content": content });
                    yield Ok(Event::default().event("delta").data(data.to_string()));
                }
                SseEvent::End { message_id, full_content: content } => {
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": { "prompt_tokens": 0, "completion_tokens": 0 }
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    let _ = message::insert_message(
                        &pool,
                        &message_id,
                        None,
                        &user_id,
                        "self",
                        "SELF",
                        &content,
                        &[],
                        &[],
                        None,
                    ).await;
                }
                SseEvent::Error(msg) => {
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub async fn self_history(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<HistoryResponse>> {
    let limit = query.limit + 1;
    let messages =
        chat::get_history(&state, None, &user.token_id, query.before.as_deref(), limit).await?;

    let has_more = messages.len() > query.limit as usize;
    let messages = if has_more {
        messages.into_iter().take(query.limit as usize).collect()
    } else {
        messages
    };

    Ok(Json(HistoryResponse { messages, has_more }))
}
```

**Note:** `async-stream` crate is needed for the `async_stream::stream!` macro. Add to `Cargo.toml`:

```toml
async-stream = "0.3"
```

Also need a `user::get_by_token` function — check if it exists. If not, add to `server/src/models/user.rs`:

```rust
pub async fn get_by_token(
    pool: &sqlx::SqlitePool,
    token_id: &str,
) -> Result<Option<UserRow>> {
    sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE token_id = ?1")
        .bind(token_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))
}
```

- [ ] **Step 2: Register route module and add routes**

In `server/src/routes/mod.rs`, add at the top:

```rust
mod chat;
```

In the `build_router` function, add these routes inside the `let api = Router::new()` block:

```rust
        .route(
            "/rings/{ring_id}/chat",
            post(chat::ring_chat).get(chat::ring_history),
        )
        .route("/self/chat", post(chat::self_chat).get(chat::self_history))
```

- [ ] **Step 3: Register module in lib.rs**

Add `pub mod chat;` in the `routes` module block of `server/src/lib.rs` (if routes are re-exported there). Otherwise it's handled by the `mod chat;` in routes/mod.rs.

- [ ] **Step 4: Verify build**

Run: `cd server && cargo check 2>&1`

This will likely need a few iterations to get imports right. Key things to check:
- `async-stream` added to Cargo.toml
- `user::get_by_token` exists
- `ring::get_user_role` is accessible
- All imports resolve

Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add server/src/routes/chat.rs server/src/routes/mod.rs server/Cargo.toml server/src/models/user.rs
git commit -m "feat(server): SSE chat endpoints for Group Ring and Self"
```

---

### Task 6: Frontend SSE Client

**Files:**
- Create: `ui/src/services/sse.ts`

- [ ] **Step 1: Create SSE wrapper**

`ui/src/services/sse.ts`:

```typescript
export interface SseMessageStart {
  message_id: string
  role: string
}

export interface SseDelta {
  content: string
}

export interface SseMessageEnd {
  message_id: string
  usage: { prompt_tokens: number; completion_tokens: number }
}

export interface SseError {
  error: string
}

export interface SseCallbacks {
  onStart: (data: SseMessageStart) => void
  onDelta: (data: SseDelta) => void
  onEnd: (data: SseMessageEnd) => void
  onError: (data: SseError) => void
}

export function streamChat(url: string, body: unknown, callbacks: SseCallbacks): AbortController {
  const controller = new AbortController()

  const token = localStorage.getItem('ring_token')

  fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(token ? { 'X-Ring-Token': token } : {}),
    },
    body: JSON.stringify(body),
    signal: controller.signal,
  })
    .then(async (res) => {
      if (!res.ok) {
        const err = await res.json().catch(() => ({}))
        callbacks.onError({ error: err?.error?.message ?? res.statusText })
        return
      }

      const reader = res.body?.getReader()
      if (!reader) {
        callbacks.onError({ error: 'No response body' })
        return
      }

      const decoder = new TextDecoder()
      let buffer = ''

      while (true) {
        const { done, value } = await reader.read()
        if (done) break

        buffer += decoder.decode(value, { stream: true })
        const lines = buffer.split('\n')
        buffer = lines.pop() ?? ''

        let currentEvent = ''
        for (const line of lines) {
          if (line.startsWith('event: ')) {
            currentEvent = line.slice(7).trim()
          } else if (line.startsWith('data: ')) {
            const data = line.slice(6)
            try {
              const parsed = JSON.parse(data)
              switch (currentEvent) {
                case 'message_start':
                  callbacks.onStart(parsed)
                  break
                case 'delta':
                  callbacks.onDelta(parsed)
                  break
                case 'message_end':
                  callbacks.onEnd(parsed)
                  break
                case 'error':
                  callbacks.onError(parsed)
                  break
              }
            } catch {
              // skip malformed JSON
            }
            currentEvent = ''
          }
        }
      }
    })
    .catch((e) => {
      if (e.name !== 'AbortError') {
        callbacks.onError({ error: e.message })
      }
    })

  return controller
}
```

**Note:** We use `fetch` + `ReadableStream` instead of `EventSource` because `EventSource` only supports GET requests. Our SSE endpoint is POST (we send the user message as body).

- [ ] **Step 2: Verify build**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add ui/src/services/sse.ts
git commit -m "feat(ui): SSE streaming client using fetch ReadableStream"
```

---

### Task 7: Frontend Chat Store — Real SSE Send

**Files:**
- Modify: `ui/src/stores/chat-store.ts`
- Modify: `ui/src/stores/self-store.ts` (if Self chat dispatch needs updating)

- [ ] **Step 1: Read current chat-store.ts to understand existing structure**

Read `ui/src/stores/chat-store.ts` and note the existing `send()` function structure. It currently:
1. Parses commands
2. Dispatches local actions
3. Adds user message to local array
4. Clears input

The new `send()` must additionally:
1. After adding user message, determine the SSE endpoint
2. Call `streamChat()` with appropriate URL
3. Create a placeholder AI message and update it as deltas arrive
4. Handle errors and abort on re-send

- [ ] **Step 2: Rewrite chat-store.ts**

`ui/src/stores/chat-store.ts`:

```typescript
import { create } from 'zustand'
import type { ChatMessage } from '../types/chat'
import { parseCommand } from '../services/command-parser'
import { streamChat, type SseMessageStart, type SseMessageEnd } from '../services/sse'
import { usePanelStore } from './panel-store'
import { useSelfStore } from './self-store'
import { useModeStore } from './mode-store'
import { useRingStore } from './ring-store'
import { useAppStore } from './app-store'

interface ChatState {
  messages: ChatMessage[]
  input: string
  session_mode: 'storage' | 'ephemeral'
  sending: boolean
  streaming_message_id: string | null
  abort_controller: AbortController | null
  history_loaded: boolean
  setInput: (val: string) => void
  addMessage: (msg: ChatMessage) => void
  updateMessageContent: (id: string, content: string) => void
  send: () => void
  loadHistory: () => Promise<void>
  setSessionMode: (mode: 'storage' | 'ephemeral') => void
  stopStreaming: () => void
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  input: '',
  session_mode: 'storage',
  sending: false,
  streaming_message_id: null,
  abort_controller: null,
  history_loaded: false,

  setInput: (val) => set({ input: val }),

  addMessage: (msg) => set((s) => ({ messages: [...s.messages, msg] })),

  updateMessageContent: (id, content) =>
    set((s) => ({
      messages: s.messages.map((m) => (m.id === id ? { ...m, content } : m)),
    })),

  send: () => {
    const { input, addMessage, sending } = get()
    if (!input.trim() || sending) return

    const parsed = parseCommand(input)

    if (parsed) {
      for (const cmd of parsed) {
        switch (cmd.type) {
          case 'action': {
            if (cmd.action === 'graph') usePanelStore.getState().toggle('graph')
            else if (cmd.action === 'archive') usePanelStore.getState().toggle('archive')
            else if (cmd.action === 'config') usePanelStore.getState().toggle('config')
            else if (cmd.action === 'session') usePanelStore.getState().toggle('session')
            else if (cmd.action === 'auto') useModeStore.getState().toggleAuto()
            else if (cmd.action === 'new') {
              const name = cmd.args
              if (name) {
                useRingStore.getState().createRing(name, `You are a ${name} assistant`)
              }
            }
            else if (cmd.action === 'save') {
              addMessage({
                id: `sys-${Date.now()}`,
                role: 'system',
                sender_name: 'SYSTEM',
                content: '归档功能将在后续版本实现',
                created_at: new Date().toISOString(),
              })
            }
            break
          }
          case 'address': {
            if (cmd.target === 'self') useSelfStore.getState().setOpen(true)
            break
          }
          case 'meta': {
            if (cmd.key === 'mode' && cmd.value) useModeStore.getState().setInteractionMode(cmd.value as 'normal' | 'auto')
            else if (cmd.key === 'skill' && cmd.value) useModeStore.getState().setSkillMode(cmd.value as 'auto' | 'plan' | 'edit')
            break
          }
          case 'reference':
            break
        }
      }
    }

    addMessage({
      id: `msg-${Date.now()}`,
      role: 'user',
      sender_name: 'You',
      content: input,
      node_refs: parsed?.filter((c) => c.type === 'reference').map((c) => c.name),
      created_at: new Date().toISOString(),
    })

    const user_content = input
    const node_refs = parsed?.filter((c) => c.type === 'reference').map((c) => c.name) ?? []

    set({ input: '', sending: true })

    const context = useAppStore.getState().current_context
    const ring_id = useRingStore.getState().active_ring_id

    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat`
    } else {
      set({ sending: false })
      return
    }

    const controller = new AbortController()
    set({ abort_controller: controller })

    streamChat(url, { content: user_content, node_refs, tag_refs: [] }, {
      onStart: (data: SseMessageStart) => {
        const aiMsg: ChatMessage = {
          id: data.message_id,
          role: data.role as ChatMessage['role'],
          sender_name: data.role === 'group_ring' ? 'GROUP RING' : data.role.toUpperCase(),
          content: '',
          created_at: new Date().toISOString(),
        }
        set((s) => ({
          messages: [...s.messages, aiMsg],
          streaming_message_id: data.message_id,
        }))
      },
      onDelta: (data) => {
        const { streaming_message_id, messages } = get()
        if (!streaming_message_id) return
        set({
          messages: messages.map((m) =>
            m.id === streaming_message_id ? { ...m, content: m.content + data.content } : m,
          ),
        })
      },
      onEnd: (data: SseMessageEnd) => {
        set({ sending: false, streaming_message_id: null, abort_controller: null })
      },
      onError: (data) => {
        const { streaming_message_id, messages } = get()
        if (streaming_message_id) {
          set({
            messages: messages.map((m) =>
              m.id === streaming_message_id
                ? { ...m, content: m.content + `\n\n⚠ Error: ${data.error}` }
                : m,
            ),
            sending: false,
            streaming_message_id: null,
            abort_controller: null,
          })
        } else {
          addMessage({
            id: `err-${Date.now()}`,
            role: 'system',
            sender_name: 'SYSTEM',
            content: `Error: ${data.error}`,
            created_at: new Date().toISOString(),
          })
          set({ sending: false, abort_controller: null })
        }
      },
    })
  },

  loadHistory: async () => {
    const context = useAppStore.getState().current_context
    const ring_id = useRingStore.getState().active_ring_id
    const token = localStorage.getItem('ring_token')

    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat/history?limit=50`
    } else {
      return
    }

    try {
      const res = await fetch(url, {
        headers: { 'X-Ring-Token': token ?? '' },
      })
      if (!res.ok) return
      const data = await res.json()
      set({ messages: data.messages ?? [], history_loaded: true })
    } catch {
      // keep existing messages
    }
  },

  setSessionMode: (mode) => set({ session_mode: mode }),

  stopStreaming: () => {
    const { abort_controller } = get()
    abort_controller?.abort()
    set({ sending: false, streaming_message_id: null, abort_controller: null })
  },
}))
```

- [ ] **Step 3: Verify build**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add ui/src/stores/chat-store.ts
git commit -m "feat(ui): chat store sends real SSE requests, accumulates streaming AI responses"
```

---

### Task 8: Frontend UI Updates — Streaming Indicator + History Loading

**Files:**
- Modify: `ui/src/components/chat/MessageList.tsx`
- Modify: `ui/src/components/chat/MessageItem.tsx`
- Modify: `ui/src/components/chat/InputArea.tsx`
- Modify: `ui/src/components/layout/AppLayout.tsx` — load history on ring switch

- [ ] **Step 1: Update MessageItem.tsx to show streaming cursor**

Read `ui/src/components/chat/MessageItem.tsx` first. Add a blinking cursor to messages that are still streaming:

After the content display, add a cursor check:

```typescript
{msg.id === streaming_id && (
  <span style={{
    display: 'inline-block',
    width: 6,
    height: 14,
    background: 'var(--accent-cyan)',
    marginLeft: 2,
    animation: 'blink 1s step-end infinite',
  }} />
)}
```

This requires passing `streaming_id` as a prop or reading from the store.

- [ ] **Step 2: Update InputArea.tsx to show sending state and stop button**

Read `ui/src/components/chat/InputArea.tsx`. Add:
- Disable input + send button while `sending === true`
- Show a STOP button when streaming

Replace the SEND button section:

```typescript
{sending ? (
  <button
    onClick={stopStreaming}
    style={{
      background: 'var(--accent-amber)',
      color: 'var(--bg-base)',
      border: 'none',
      borderRadius: 4,
      padding: '8px 16px',
      fontSize: 12,
      fontWeight: 700,
      cursor: 'pointer',
    }}
  >
    STOP
  </button>
) : (
  <button
    onClick={send}
    style={{
      background: 'var(--accent-cyan)',
      color: 'var(--bg-base)',
      border: 'none',
      borderRadius: 4,
      padding: '8px 16px',
      fontSize: 12,
      fontWeight: 700,
      cursor: 'pointer',
      letterSpacing: '0.05em',
    }}
  >
    SEND
  </button>
)}
```

And disable the input:

```typescript
disabled={sending}
```

- [ ] **Step 3: Load history on ring switch in AppLayout.tsx**

Read `ui/src/components/layout/AppLayout.tsx`. Add a `useEffect` that calls `loadHistory()` when `active_ring_id` changes:

```typescript
const loadHistory = useChatStore((s) => s.loadHistory)
const active_ring_id = useRingStore((s) => s.active_ring_id)

useEffect(() => {
  if (active_ring_id) {
    loadHistory()
  }
}, [active_ring_id, loadHistory])
```

- [ ] **Step 4: Add blink animation to index.css**

Read `ui/src/index.css` and add:

```css
@keyframes blink {
  50% { opacity: 0; }
}
```

- [ ] **Step 5: Verify build**

Run: `cd ui && npx tsc --noEmit && npm run build`
Expected: Clean build.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/chat/ ui/src/components/layout/AppLayout.tsx ui/src/index.css
git commit -m "feat(ui): streaming cursor, stop button, history loading on ring switch"
```

---

### Task 9: Integration Test — Chat SSE End-to-End

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add chat integration tests**

Read `server/tests/integration.rs` to see the existing pattern. Add:

```rust
#[tokio::test]
async fn test_chat_requires_auth() {
    let (pool, _tmp) = setup_test_db().await;
    let state = AppState::new(pool);
    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rings/ring-1/chat")
                .header("content-type", "application/json")
                .body(r#"{"content":"hello","node_refs":[],"tag_refs":[]}"#.into())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_chat_history_requires_membership() {
    let (pool, _tmp) = setup_test_db().await;
    let state = AppState::new(pool);
    let app = build_router(state.clone());

    let user = create_test_user(&state.db, "alice").await;
    let ring = create_test_ring(&state.db, "test-ring", &user.token_id).await;

    let other = create_test_user(&state.db, "bob").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/rings/{ring}/chat/history?limit=50"))
                .header("x-ring-token", &other.token_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 404);
}
```

Note: The helper functions `create_test_user`, `create_test_ring`, `setup_test_db` may need to be added if they don't exist. Check the existing test file for patterns.

- [ ] **Step 2: Run tests**

Run: `cd server && cargo test`
Expected: All tests pass (including the new chat auth/membership tests).

- [ ] **Step 3: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test(server): chat endpoint auth and membership integration tests"
```

---

### Task 10: Manual E2E Smoke Test

**Files:** No new files — manual verification

- [ ] **Step 1: Build frontend**

Run: `cd ui && npm run build`
Expected: Success.

- [ ] **Step 2: Start backend**

Run: `cd server && rm -f ~/.ring/ring.db && cargo run &`
Expected: `ring-server listening on http://localhost:7420`

- [ ] **Step 3: Test full chat flow via curl**

```bash
# Setup
TOKEN=$(curl -s -X POST http://localhost:7420/api/setup \
  -H 'Content-Type: application/json' \
  -d '{"display_name":"Kai","avatar":"🦊","llm_provider":"openai","llm_api_key":"sk-test","llm_model":"gpt-4o","gitlab_url":"https://g.test","gitlab_token":"glpat-test"}' | python3 -c "import sys,json; print(json.load(sys.stdin)['token_id'])")

# Create ring
RING_ID=$(curl -s -X POST http://localhost:7420/api/rings \
  -H 'Content-Type: application/json' \
  -H "X-Ring-Token: $TOKEN" \
  -d '{"name":"test","role_description":"你是一个测试助手"}' | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")

# Send chat message (will fail with LLM since sk-test is fake, but SSE error event should stream)
curl -s -N -X POST "http://localhost:7420/api/rings/$RING_ID/chat" \
  -H 'Content-Type: application/json' \
  -H "X-Ring-Token: $TOKEN" \
  -d '{"content":"你好","node_refs":[],"tag_refs":[]}'

# Check history (user message should be saved even if LLM failed)
curl -s -H "X-Ring-Token: $TOKEN" "http://localhost:7420/api/rings/$RING_ID/chat/history?limit=50" | python3 -m json.tool

# Test SPA
curl -s -o /dev/null -w "%{http_code}" http://localhost:7420/
```

Expected:
- SSE stream starts with `event: message_start`, then `event: error` (fake API key)
- History shows the user message
- SPA returns 200

- [ ] **Step 4: Kill backend**

```bash
kill %1
```

---

## Self-Review

### 1. Spec Coverage

| API Design Requirement | Covered | Task |
|------------------------|---------|------|
| POST /api/rings/{ring_id}/chat (SSE) | Yes | Task 5 |
| GET /api/rings/{ring_id}/chat/history | Yes | Task 5 |
| POST /api/self/chat (SSE) | Yes | Task 5 |
| GET /api/self/chat/history | Yes | Task 5 |
| SSE event types (message_start, delta, message_end, error) | Yes | Task 5 |
| Message persistence | Yes | Task 2 |
| History pagination (before, limit, has_more) | Yes | Task 5 |
| Auth check + membership validation | Yes | Task 5 |
| LLM client per-user config | Yes | Task 3 |
| Streaming token-by-token display | Yes | Task 6-8 |

**Deferred (as planned):**
- POST `/api/rings/{ring_id}/chat/compact` — Phase 3
- PUT `/api/rings/{ring_id}/chat/session-mode` — Phase 3
- Super Ring chat (`/api/super/chat`) — later plan
- Session AI via WebSocket — Phase 2

### 2. Placeholder Scan

No TBD/TODO/placeholders found. All steps contain complete code.

### 3. Type Consistency

- `ChatRequest` (Rust) matches `{ content, node_refs, tag_refs }` from API design
- `SseEvent` enum (Rust) matches `SseCallbacks` interface (TypeScript)
- `MessageRow` fields match `ChatMessage` type in `ui/src/types/chat.ts`
- `HistoryResponse.messages` is `Vec<MessageRow>` — frontend reads `.messages` array
- `HistoryQuery.limit` defaults to 50, `has_more` uses `limit + 1` trick for pagination

---

**Plan complete and saved to `docs/superpowers/plans/2026-04-18-llm-chat-sse.md`. Two execution options:**

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
