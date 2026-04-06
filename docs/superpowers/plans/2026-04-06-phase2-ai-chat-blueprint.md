# Phase 2 Implementation Plan — AI Chat & Blueprint (TDD)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 LLM 对话（Super Ring + Group Ring）和蓝图构建流程，含 SSE 流式响应。

**Architecture:** 统一 LlmProvider trait 抽象 OpenAI/Anthropic/Ollama 差异。SSE 流式输出通过 Axum Sse 类型。Blueprint 通过 Group Ring 特殊 prompt 多轮对话构建。对话消息存 SQLite。

**Tech Stack:** async-openai (OpenAI+Ollama), reqwest (Anthropic), Axum SSE, SQLite conversations/messages 表

**Reference docs:**
- `docs/technical/llm-prompts.md` — system prompt 模板
- `docs/technical/sse-protocol.md` — SSE event 类型定义
- `docs/technical/api-design.md` sections 3 (蓝图), 4 (对话), 10 (Super Ring)

---

## File Structure

```
ring-server/src/
├── services/
│   ├── llm_provider.rs         # LlmProvider trait
│   ├── llm_openai.rs           # OpenAI/Ollama adapter (async-openai)
│   ├── llm_anthropic.rs        # Anthropic adapter (reqwest)
│   ├── ai_service.rs           # AI dispatcher (Super Ring + Group Ring)
│   ├── blueprint_service.rs    # Blueprint logic
│   └── context_loader.rs       # .ring/ doc loader
├── handlers/
│   ├── ai.rs                   # Super Ring + Group Ring chat endpoints
│   ├── blueprint.rs            # Blueprint endpoints
│   └── conversation.rs         # Conversation CRUD + message send
├── models/
│   ├── conversation.rs         # Conversation + Message models
│   └── blueprint.rs            # BlueprintTemplate model
├── db/
│   ├── traits.rs               # Add conversation/blueprint repo methods
│   └── sqlite.rs               # Implement new repo methods
└── routes.rs                   # Add new routes

ring-frontend/src/
├── pages/
│   ├── RingSpace/
│   │   ├── ChatView.tsx         # Group Ring 对话视图
│   │   └── BlueprintWizard.tsx  # 蓝图构建向导
│   └── RingHub/
│   │   └── SuperRingChat.tsx   # Super Ring 对话
├── components/
│   └── chat/
│       ├── ChatBubble.tsx       # 消息气泡
│       ├── ChatInput.tsx        # 输入框
│       └── SseParser.ts         # SSE 流解析工具
├── stores/
│   └── chatStore.ts            # 对话状态管理
└── api/
    └── client.ts               # Add chat/blueprint API functions
```

---

## Module 1: LLM Provider Trait

**Files:**
- Create: `ring-server/src/services/llm_provider.rs`

- [ ] **Step 1: Write failing test**

Test LlmProvider trait contract: `chat_stream` returns a stream of `SseEvent` items.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn mock_provider_returns_text_events() {
        let provider = MockLlmProvider::new(vec![
            LlmEvent::Text("hello".into()),
            LlmEvent::Done { message_id: None, token_usage: None },
        ]);
        let stream = provider.chat_stream(vec![]).await.unwrap();
        let events: Vec<LlmEvent> = stream.collect().await;
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], LlmEvent::Text(ref s) if s == "hello"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --lib services::llm_provider`
Expected: FAIL

- [ ] **Step 3: Implement LlmProvider trait + LlmEvent enum**

Define `LlmEvent` enum: `Text(String)`, `ToolCall{...}`, `ToolResult{...}`, `Error{code, message}`, `Done{message_id, token_usage}`.
Define `LlmProvider` trait with `async fn chat_stream(&self, messages: Vec<LlmMessage>) -> Result<Pin<Box<dyn Stream<Item = LlmEvent> + Send>>>`.
Define `LlmMessage` struct: `{ role: String, content: String }`.
Define `MockLlmProvider` for testing.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-server && cargo test --lib services::llm_provider`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/services/llm_provider.rs
git commit -m "feat(phase2): add LlmProvider trait with mock implementation"
```

---

## Module 2: OpenAI/Ollama Adapter

**Files:**
- Create: `ring-server/src/services/llm_openai.rs`

- [ ] **Step 1: Write failing test**

Test that `OpenAiProvider` converts `LlmMessage` to `async_openai` types and parses streaming response into `LlmEvent`s. Use a mock server or test with `base_url` pointing to a local test server.

Test at minimum: text event parsing, done event with token usage, error handling on connection failure.

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement OpenAiProvider**

Implement `OpenAiProvider` struct with `api_key`, `model`, `base_url` (None = default OpenAI, Some = Ollama endpoint). Uses `async_openai` crate. Convert `LlmMessage` to `ChatCompletionRequestMessage`. Parse `ChatCompletionResponseStream` into `LlmEvent` stream.

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/services/llm_openai.rs
git commit -m "feat(phase2): add OpenAI/Ollama LLM adapter"
```

---

## Module 3: Anthropic Adapter

**Files:**
- Create: `ring-server/src/services/llm_anthropic.rs`

- [ ] **Step 1: Write failing test**

Test Anthropic request format: system message at top level, tool results as user messages. Test SSE parsing from Anthropic's streaming format.

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement AnthropicProvider**

Use `reqwest` to call Anthropic Messages API (`https://api.anthropic.com/v1/messages`). Convert `LlmMessage` to Anthropic format (system at top level, tool_result as user content block). Parse Anthropic SSE events into `LlmEvent`.

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/services/llm_anthropic.rs
git commit -m "feat(phase2): add Anthropic LLM adapter"
```

---

## Module 4: Conversation Models + DB

**Files:**
- Create: `ring-server/src/models/conversation.rs`
- Create: `ring-server/src/models/blueprint.rs`
- Modify: `ring-server/src/db/traits.rs` — add conversation/blueprint methods
- Modify: `ring-server/src/db/sqlite.rs` — implement new methods

- [ ] **Step 1: Write failing tests**

```rust
#[tokio::test]
async fn create_and_list_conversations() {
    let repo = setup_test_db().await;
    let user = repo.create_user(NewUser { display_name: "张三".into() }).await.unwrap();
    let ring = repo.create_ring(/* ... */).await.unwrap();
    let conv = repo.create_conversation(ring.id.clone(), Some("测试".into()), "storage", user.id.clone()).await.unwrap();
    let list = repo.list_conversations(&ring.id).await.unwrap();
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn create_and_get_messages() {
    let repo = setup_test_db().await;
    // create user + ring + conversation
    let msg = repo.create_message(conv.id.clone(), "user", "你好", Some(user.id.clone())).await.unwrap();
    let msgs = repo.get_messages(&conv.id, 50, None).await.unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content, "你好");
}

#[tokio::test]
async fn list_blueprint_templates() {
    let repo = setup_test_db().await;
    repo.create_blueprint_template("tpl-1", "产品研究", "适合分析", r#"[{"name":"知识图谱"}]"#, true).await.unwrap();
    let templates = repo.list_blueprint_templates().await.unwrap();
    assert_eq!(templates.len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

Define `Conversation`, `Message`, `BlueprintTemplate` models.
Add to Repository trait: `create_conversation`, `list_conversations`, `get_conversation`, `create_message`, `get_messages`, `list_blueprint_templates`, `create_blueprint_template`.
Implement in SqliteRepository.

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/models/conversation.rs ring-server/src/models/blueprint.rs ring-server/src/db/
git commit -m "feat(phase2): add conversation/blueprint models and repo methods"
```

---

## Module 5: AI Service (Core Dispatcher)

**Files:**
- Create: `ring-server/src/services/ai_service.rs`
- Create: `ring-server/src/services/context_loader.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[tokio::test]
async fn group_ring_chat_returns_sse_events() {
    let svc = setup_ai_service_with_mock().await;
    let events = svc.group_ring_chat("ring-1", "conv-1", "你好").await.unwrap();
    // verify events contain text + done
}

#[tokio::test]
async fn super_ring_chat_returns_sse_events() {
    let svc = setup_ai_service_with_mock().await;
    let events = svc.super_ring_chat("帮我创建一个Ring").await.unwrap();
    // verify events
}
```

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`AiService` takes `Arc<dyn Repository>` + `Arc<dyn LlmProvider>`.

Methods:
- `super_ring_chat(message: &str) -> Result<impl Stream<Item = LlmEvent>>` — load Super Ring system prompt from `docs/technical/llm-prompts.md` section 1, call LLM
- `group_ring_chat(ring_id, conv_id, message) -> Result<impl Stream<Item = LlmEvent>>` — load .ring/ role.md + conventions.md, build system prompt per section 2.1, append user message to conversation history, call LLM, save messages to DB
- `blueprint_chat(ring_id, message) -> Result<impl Stream<Item = LlmEvent>>` — load blueprint system prompt per section 3 + role.md, call LLM

`context_loader.rs` — functions to load .ring/ docs from the Git repo path:
- `load_role_md(ring_path) -> Option<String>`
- `load_conventions_md(ring_path) -> Option<String>`
- `load_active_context_md(ring_path) -> Option<String>`
- `build_group_ring_prompt(role, conventions, active_context) -> String`

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/services/ai_service.rs ring-server/src/services/context_loader.rs
git commit -m "feat(phase2): add AI service with Super Ring + Group Ring + blueprint chat"
```

---

## Module 6: SSE Handler + Conversation API

**Files:**
- Create: `ring-server/src/handlers/conversation.rs`
- Create: `ring-server/src/handlers/ai.rs`
- Modify: `ring-server/src/routes.rs` — add new routes
- Modify: `ring-server/src/state.rs` — add llm_provider field

- [ ] **Step 1: Write integration tests**

Test against Axum test router with mock LLM provider:

```rust
#[tokio::test]
async fn send_message_returns_sse_stream() {
    let app = create_test_app_with_mock_llm().await;
    // POST /api/v1/rings/{ringId}/conversations → create conv
    // POST /api/v1/rings/{ringId}/conversations/{convId}/messages
    // Assert response is text/event-stream
    // Parse SSE events, verify text + done
}

#[tokio::test]
async fn super_ring_chat_returns_sse_stream() {
    let app = create_test_app_with_mock_llm().await;
    // POST /api/v1/super-ring/chat {"message": "hello"}
    // Assert SSE response
}

#[tokio::test]
async fn get_conversations_returns_list() {
    // Create conversation, GET list, verify present
}

#[tokio::test]
async fn get_messages_returns_history() {
    // Create conv + send message, GET messages, verify content
}
```

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement handlers**

`conversation.rs`:
- `list` — GET conversations by ring_id
- `create` — POST create new conversation
- `get` — GET single conversation
- `send_message` — POST message, return SSE stream. Parse SSE events from LLM, re-emit as Axum SSE with correct event types. Save user message + AI response to DB.
- `get_messages` — GET paginated message history

`ai.rs`:
- `super_ring_chat` — POST, return SSE stream

SSE response format: use `axum::response::sse::{Sse, Event}`. Each event: `Event::default().data(json_string).event("message")`.

Update `AppState` to include `llm_provider: Arc<dyn LlmProvider>`.

Update `routes.rs`:
```
/api/v1/rings/{ringId}/conversations           → GET list, POST create
/api/v1/rings/{ringId}/conversations/{convId}  → GET get
/api/v1/rings/{ringId}/conversations/{convId}/messages → GET history, POST send_message
/api/v1/super-ring/chat                         → POST super_ring_chat
```

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/handlers/conversation.rs ring-server/src/handlers/ai.rs ring-server/src/routes.rs ring-server/src/state.rs
git commit -m "feat(phase2): add conversation and Super Ring SSE endpoints"
```

---

## Module 7: Blueprint API

**Files:**
- Create: `ring-server/src/handlers/blueprint.rs`
- Modify: `ring-server/src/routes.rs` — add blueprint routes
- Create: `ring-server/tests/blueprint_integration.rs`

- [ ] **Step 1: Write integration tests**

```rust
#[tokio::test]
async fn list_blueprint_templates() {
    // GET /api/v1/rings/{ringId}/blueprint/templates → 200 with template list
}

#[tokio::test]
async fn blueprint_chat_returns_sse() {
    // POST /api/v1/rings/{ringId}/blueprint/chat {"message": "我需要竞品分析"}
    // SSE stream with text + blueprint_proposal
}

#[tokio::test]
async fn preview_blueprint() {
    // POST /api/v1/rings/{ringId}/blueprint/preview with graphs data
    // Returns node+edge preview for D3.js
}

#[tokio::test]
async fn confirm_blueprint() {
    // POST /api/v1/rings/{ringId}/blueprint/confirm
    // Creates graphs in petgraph, changes ring status to active
}
```

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`blueprint.rs` handlers:
- `list_templates` — query DB for system + user templates
- `blueprint_chat` — SSE stream, uses `AiService::blueprint_chat`. Parse LLM response for `blueprint_proposal` events.
- `preview_blueprint` — given graph definitions, create nodes+edges in petgraph, return preview data
- `confirm_blueprint` — persist blueprint to DB + petgraph, update ring status

Routes:
```
/api/v1/rings/{ringId}/blueprint/templates  → GET
/api/v1/rings/{ringId}/blueprint/chat       → POST (SSE)
/api/v1/rings/{ringId}/blueprint/preview    → POST
/api/v1/rings/{ringId}/blueprint/confirm    → POST
```

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/handlers/blueprint.rs ring-server/src/routes.rs ring-server/tests/blueprint_integration.rs
git commit -m "feat(phase2): add blueprint endpoints with template list, chat, preview, confirm"
```

---

## Module 8: Frontend — Chat Components

**Files:**
- Create: `ring-frontend/src/components/chat/ChatBubble.tsx`
- Create: `ring-frontend/src/components/chat/ChatInput.tsx`
- Create: `ring-frontend/src/components/chat/SseParser.ts`
- Create: `ring-frontend/src/stores/chatStore.ts`

- [ ] **Step 1: Write tests**

- `SseParser` — test parsing SSE text chunks into structured events
- `ChatInput` — test renders input + send button, calls onSend
- `ChatBubble` — test renders user/assistant messages differently

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`SseParser.ts` — utility to parse `text/event-stream` response body into typed events (text, tool_call, tool_result, archive_suggestion, blueprint_proposal, done, error).

`chatStore.ts` — Zustand store: `messages[]`, `isStreaming`, `sendMessage()` (POST with fetch + SSE parsing), `currentConversationId`.

`ChatBubble.tsx` — renders a single message with role-based styling.

`ChatInput.tsx` — text input + send button, disabled during streaming.

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/components/chat/ ring-frontend/src/stores/chatStore.ts
git commit -m "feat(phase2): add chat components with SSE parser"
```

---

## Module 9: Frontend — Chat View + Super Ring + Blueprint Wizard

**Files:**
- Create: `ring-frontend/src/pages/RingSpace/ChatView.tsx`
- Create: `ring-frontend/src/pages/RingSpace/BlueprintWizard.tsx`
- Create: `ring-frontend/src/pages/RingHub/SuperRingChat.tsx`
- Modify: `ring-frontend/src/App.tsx` — add new routes
- Modify: `ring-frontend/src/api/client.ts` — add chat/blueprint API functions

- [ ] **Step 1: Write tests**

- `ChatView` renders message list + input
- `BlueprintWizard` renders template cards + chat area
- `SuperRingChat` renders chat input + messages

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`ChatView.tsx` — main chat page for a Ring. Shows message history (ChatBubble list), ChatInput at bottom. On mount, load conversation history. On send, POST message and parse SSE stream.

`SuperRingChat.tsx` — standalone chat page at Ring Hub level. Simpler: no ring context, just Super Ring conversation.

`BlueprintWizard.tsx` — two modes:
1. Template selection: show template cards from API, click to preview
2. Custom chat: chat with Group Ring in blueprint mode, show D3.js preview of proposed graphs

`App.tsx` — add routes:
- `/ring/:ringId` → ChatView
- `/ring/:ringId/blueprint` → BlueprintWizard

`client.ts` — add functions: `sendMessage(ringId, convId, content)`, `listConversations(ringId)`, `createConversation(ringId, title)`, `listBlueprintTemplates(ringId)`, `blueprintChat(ringId, message)`, `blueprintPreview(ringId, graphs)`, `blueprintConfirm(ringId, graphs)`, `superRingChat(message)`.

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/pages/ ring-frontend/src/App.tsx ring-frontend/src/api/client.ts
git commit -m "feat(phase2): add ChatView, BlueprintWizard, SuperRingChat pages"
```

---

## Module 10: Integration Verification

- [ ] **Step 1: Run all backend tests**

Run: `cd ring-server && cargo test`
Expected: ALL PASS

- [ ] **Step 2: Run all frontend tests**

Run: `cd ring-frontend && npm test`
Expected: ALL PASS

- [ ] **Step 3: Run clippy + fmt**

```bash
cd ring-server && cargo fmt --check && cargo clippy -- -D warnings
```

- [ ] **Step 4: Manual smoke test**

Start backend, open frontend:
1. Complete setup → Create Ring → Enter Ring space
2. Send message to Group Ring → verify SSE streaming works
3. Go to Ring Hub → Super Ring chat → verify response
4. Go to blueprint wizard → verify template list + chat

- [ ] **Step 5: Final commit**

```bash
git commit --allow-empty -m "milestone: Phase 2 complete — AI chat and blueprint"
```
