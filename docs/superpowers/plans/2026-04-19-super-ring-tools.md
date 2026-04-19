# Super Ring Tool Framework + Cross-Ring Query Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a tool framework to Super Ring chat using async-openai native function calling. Two tools: `query_rings` (list all rings with summary) and `query_ring_detail` (read graph + archives for a specific ring).

**Architecture:** Modified `start_super_chat` does a two-phase LLM call: first `chat_complete_with_tools` (non-streaming, with tool definitions) to check if LLM wants tools, then either returns the direct message or executes tools and calls `chat_stream` for the final streaming response.

**Tech Stack:** Rust, async-openai 0.27 (ChatCompletionTool, FunctionObject, ChatCompletionMessageToolCall), sqlx, std::fs

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `server/src/services/llm.rs` | Modify | Add `chat_complete_with_tools` + `ChatCompleteWithToolsResult` |
| `server/src/services/super_chat.rs` | Modify | Add `get_super_tools`, `execute_tool`, `build_ring_summary`, modify `start_super_chat` |
| `server/src/routes/super_chat.rs` | Modify | Handle two-phase response in handler |

---

### Task 1: Add `chat_complete_with_tools` to LlmClient

**Files:**
- Modify: `server/src/services/llm.rs`

- [ ] **Step 1: Add imports**

Add to the existing imports at the top of the file:

```rust
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequest,
    ChatCompletionTool, FunctionObject, ChatCompletionToolType,
};
```

Note: keep the existing imports, just add `ChatCompletionTool`, `FunctionObject`, `ChatCompletionToolType`.

- [ ] **Step 2: Add result enum**

Add after the `SseEvent` enum (after line 32):

```rust
pub enum ChatCompleteWithToolsResult {
    Message { content: String },
    ToolCalls { tool_calls: Vec<async_openai::types::ChatCompletionMessageToolCall> },
}
```

- [ ] **Step 3: Add `chat_complete_with_tools` method**

Add inside `impl LlmClient`, after the existing `chat_complete` method (after line 188):

```rust
    pub async fn chat_complete_with_tools(
        self,
        system_prompt: String,
        history: Vec<(String, String)>,
        user_message: String,
        tools: Vec<ChatCompletionTool>,
    ) -> crate::error::Result<ChatCompleteWithToolsResult> {
        let mut messages = vec![ChatCompletionRequestMessage::System(
            ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(system_prompt),
                name: None,
            },
        )];

        for (role, content) in history {
            match role.as_str() {
                "user" => {
                    messages.push(ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessage {
                            content: ChatCompletionRequestUserMessageContent::Text(content),
                            name: None,
                        },
                    ));
                }
                _ => {
                    messages.push(ChatCompletionRequestMessage::Assistant(
                        #[allow(deprecated)]
                        ChatCompletionRequestAssistantMessage {
                            content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                                content,
                            )),
                            name: None,
                            tool_calls: None,
                            refusal: None,
                            audio: None,
                            function_call: None,
                        },
                    ));
                }
            }
        }

        messages.push(ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: ChatCompletionRequestUserMessageContent::Text(user_message),
                name: None,
            },
        ));

        let request = CreateChatCompletionRequest {
            messages,
            model: self.model,
            tools: Some(tools),
            tool_choice: Some(async_openai::types::ChatCompletionToolChoiceOption::Auto),
            ..Default::default()
        };

        let response = self.client.chat().create(request).await?;
        let choice = response.choices.first().ok_or_else(|| {
            crate::error::RingError::Internal("no choices in response".into())
        })?;

        if let Some(tool_calls) = &choice.message.tool_calls {
            Ok(ChatCompleteWithToolsResult::ToolCalls {
                tool_calls: tool_calls.clone(),
            })
        } else {
            Ok(ChatCompleteWithToolsResult::Message {
                content: choice.message.content.clone().unwrap_or_default(),
            })
        }
    }
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 5: Commit**

```bash
git add server/src/services/llm.rs
git commit -m "feat: add chat_complete_with_tools for function calling support"
```

---

### Task 2: Add tool definitions and execution to super_chat

**Files:**
- Modify: `server/src/services/super_chat.rs`

- [ ] **Step 1: Add imports and tool definitions**

Replace the top of `server/src/services/super_chat.rs` with:

```rust
use std::path::Path;

use async_openai::types::{ChatCompletionTool, ChatCompletionToolType, FunctionObject};
use serde::Deserialize;

use crate::error::{Result, RingError};
use crate::models::message::{self, MessageRow};
use crate::services::chat;
use crate::services::llm::{ChatCompleteWithToolsResult, LlmClient, SseEvent};
use crate::state::AppState;

const SUPER_RING_ID: &str = "super";
```

- [ ] **Step 2: Add tool structs and get_super_tools**

Add after the `update_system_prompt` function (after line 37):

```rust
#[derive(Debug, Deserialize)]
struct QueryRingDetailArgs {
    ring_name: String,
}

pub fn get_super_tools() -> Vec<ChatCompletionTool> {
    vec![
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: "query_rings".to_string(),
                description: Some("列出用户所有 Ring 的摘要信息，包括名称、成员数和最近归档标题。当用户询问关于 Ring 的概况时使用。".to_string()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }).to_string()),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: "query_ring_detail".to_string(),
                description: Some("读取指定 Ring 的详细数据，包括图谱节点和最近归档内容。当用户想了解某个 Ring 的具体内容时使用。".to_string()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "ring_name": {
                            "type": "string",
                            "description": "要查询的 Ring 名称"
                        }
                    },
                    "required": ["ring_name"]
                }).to_string()),
                strict: None,
            },
        },
    ]
}
```

- [ ] **Step 3: Add build_ring_summary**

```rust
pub async fn build_ring_summary(pool: &sqlx::SqlitePool, user_id: &str) -> String {
    let rings = sqlx::query_as::<_, (String, String)>(
        "SELECT r.id, r.name FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         ORDER BY r.created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if rings.is_empty() {
        return "用户目前没有任何 Ring。".to_string();
    }

    let mut summary = String::from("## 用户的所有 Ring\n\n");

    for (ring_id, ring_name) in &rings {
        let member_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM members WHERE ring_id = ?1",
        )
        .bind(ring_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        let archive_titles: Vec<String> = sqlx::query_scalar(
            "SELECT title FROM archive_records
             WHERE ring_id = ?1 AND status IN ('pushed', 'committed')
             ORDER BY created_at DESC LIMIT 3",
        )
        .bind(ring_id)
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        summary.push_str(&format!("### {ring_name} ({member_count} 成员)\n"));
        if archive_titles.is_empty() {
            summary.push_str("- 暂无归档\n\n");
        } else {
            summary.push_str(&format!("- 最近归档: {}\n\n", archive_titles.join(", ")));
        }
    }

    summary
}
```

- [ ] **Step 4: Add execute_tool**

```rust
pub async fn execute_tool(
    pool: &sqlx::SqlitePool,
    rings_dir: &Path,
    user_id: &str,
    tool_name: &str,
    arguments: &str,
) -> Result<String> {
    match tool_name {
        "query_rings" => execute_query_rings(pool, user_id).await,
        "query_ring_detail" => {
            let args: QueryRingDetailArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            execute_query_ring_detail(pool, rings_dir, user_id, &args.ring_name).await
        }
        _ => Err(RingError::BadRequest(format!("unknown tool: {tool_name}"))),
    }
}

async fn execute_query_rings(pool: &sqlx::SqlitePool, user_id: &str) -> Result<String> {
    Ok(build_ring_summary(pool, user_id).await)
}

async fn execute_query_ring_detail(
    pool: &sqlx::SqlitePool,
    rings_dir: &Path,
    user_id: &str,
    ring_name: &str,
) -> Result<String> {
    let ring_id: Option<String> = sqlx::query_scalar(
        "SELECT r.id FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         WHERE r.name LIKE ?2",
    )
    .bind(user_id)
    .bind(format!("%{ring_name}%"))
    .fetch_optional(pool)
    .await?
    .flatten();

    let ring_id = match ring_id {
        Some(id) => id,
        None => return Ok(format!("未找到名为「{ring_name}」的 Ring。")),
    };

    let mut result = String::new();

    let graph_path = rings_dir.join(&ring_id).join("graph.json");
    if graph_path.exists() {
        match std::fs::read_to_string(&graph_path) {
            Ok(content) => {
                if let Ok(graph) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(nodes) = graph.get("nodes").and_then(|n| n.as_array()) {
                        result.push_str(&format!("### 图谱节点（共 {} 个，显示前 50 个）\n", nodes.len()));
                        for node in nodes.iter().take(50) {
                            let label = node.get("label").and_then(|v| v.as_str()).unwrap_or("");
                            let desc = node.get("description").and_then(|v| v.as_str()).unwrap_or("");
                            if desc.is_empty() {
                                result.push_str(&format!("- {label}\n"));
                            } else {
                                result.push_str(&format!("- {label}: {desc}\n"));
                            }
                        }
                        result.push('\n');
                    }
                }
            }
            Err(e) => {
                tracing::warn!("failed to read graph.json: {e}");
            }
        }
    }

    let archives_dir = rings_dir.join(&ring_id).join("archives");
    if archives_dir.exists() {
        let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(&archives_dir)
            .map(|rd| rd.filter_map(|e| e.ok()).collect())
            .unwrap_or_default();
        entries.sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));

        result.push_str("### 最近归档\n\n");
        for entry in entries.iter().take(3) {
            if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".md") {
                    match std::fs::read_to_string(entry.path()) {
                        Ok(content) => {
                            let truncated = if content.len() > 500 {
                                format!("{}...（截断）", &content[..500])
                            } else {
                                content
                            };
                            result.push_str(&format!("#### {name}\n{truncated}\n\n"));
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    }

    if result.is_empty() {
        Ok(format!("Ring「{ring_name}」暂无图谱和归档数据。"))
    } else {
        Ok(result)
    }
}
```

- [ ] **Step 5: Modify `start_super_chat` to use tools**

Replace the existing `start_super_chat` function with:

```rust
pub enum SuperChatResult {
    DirectMessage { content: String },
    NeedsStream { system_prompt: String, history: Vec<(String, String)>, user_content: String },
}

pub async fn start_super_chat(
    state: &AppState,
    user: &crate::models::user::UserRow,
    content: &str,
) -> Result<SuperChatResult> {
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

    let base_prompt = get_system_prompt(&state.hub_dir);
    let ring_summary = build_ring_summary(&state.db, &user.token_id).await;
    let system_prompt = format!("{base_prompt}\n\n{ring_summary}");

    let history =
        chat::load_history_context(&state.db, Some(SUPER_RING_ID), &user.token_id, 20).await?;

    let llm = LlmClient::from_user(user)?;
    let tools = get_super_tools();

    let result = llm
        .chat_complete_with_tools(
            system_prompt.clone(),
            history.clone(),
            content.to_string(),
            tools,
        )
        .await?;

    match result {
        ChatCompleteWithToolsResult::Message { content: msg } => {
            Ok(SuperChatResult::DirectMessage { content: msg })
        }
        ChatCompleteWithToolsResult::ToolCalls { tool_calls } => {
            let mut tool_results = Vec::new();
            for tc in &tool_calls {
                let args = tc.function.arguments.as_deref().unwrap_or("{}");
                let tool_result = execute_tool(
                    &state.db,
                    &state.rings_dir,
                    &user.token_id,
                    &tc.function.name,
                    args,
                )
                .await
                .unwrap_or_else(|e| format!("Tool error: {e}"));

                tool_results.push((tc.function.name.clone(), tool_result));
            }

            let mut user_content = content.to_string();
            user_content.push_str("\n\n[Tool Results]\n");
            for (name, result_text) in &tool_results {
                user_content.push_str(&format!("**{name}**:\n{result_text}\n\n"));
            }

            Ok(SuperChatResult::NeedsStream {
                system_prompt,
                history,
                user_content,
            })
        }
    }
}
```

- [ ] **Step 6: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 7: Commit**

```bash
git add server/src/services/super_chat.rs
git commit -m "feat: add tool framework with query_rings and query_ring_detail"
```

---

### Task 3: Update route handler for two-phase response

**Files:**
- Modify: `server/src/routes/super_chat.rs`

- [ ] **Step 1: Update the `super_chat_handler` function**

Read the file first. Then replace the `super_chat_handler` function body with two-phase logic:

```rust
pub async fn super_chat_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ChatRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user_row = user::get_user(&state.db, &user.token_id).await?;

    let result = super_chat::start_super_chat(&state, &user_row, &body.content).await?;

    match result {
        super_chat::SuperChatResult::DirectMessage { content } => {
            let msg_id = ulid::Ulid::new().to_string();

            let _ = message::insert_message(
                &state.db,
                &message::NewMessage {
                    id: &msg_id,
                    ring_id: Some("super"),
                    user_id: &user.token_id,
                    role: "super_ring",
                    sender_name: "SUPER RING",
                    content: &content,
                    node_refs: &[],
                    tag_refs: &[],
                    token_usage: None,
                },
            )
            .await;

            let s = stream! {
                let data = serde_json::json!({"message_id": msg_id, "role": "super_ring"});
                yield Ok(Event::default().event("message_start").data(data.to_string()));
                let data = serde_json::json!({ "content": content });
                yield Ok(Event::default().event("delta").data(data.to_string()));
                let data = serde_json::json!({
                    "message_id": msg_id,
                    "usage": { "prompt_tokens": 0, "completion_tokens": 0 }
                });
                yield Ok(Event::default().event("message_end").data(data.to_string()));
            };
            Ok(Sse::new(s).keep_alive(KeepAlive::default()))
        }
        super_chat::SuperChatResult::NeedsStream {
            system_prompt,
            history,
            user_content,
        } => {
            let llm = crate::services::llm::LlmClient::from_user(&user_row)?;
            let mut rx = llm.chat_stream(
                system_prompt,
                history,
                user_content,
                "super_ring".to_string(),
            );

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
    }
}
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 3: Run all tests**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all 18 tests pass

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/super_chat.rs
git commit -m "feat: update super_chat handler for two-phase tool response"
```

---

### Task 4: Final verification

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
