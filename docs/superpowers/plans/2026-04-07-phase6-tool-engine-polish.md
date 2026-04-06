# Phase 6: Tool Engine & Polish — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the atomic tool engine framework, LLM tool-calling integration, atomic tool implementations, preset workflows, settings API/UI, and frontend tool event rendering.

**Architecture:** Extend the existing LLM provider trait to support OpenAI function calling / Anthropic tool_use. Build a `ToolEngine` module with registry + dispatcher pattern. Tools are trait objects registered at startup. The AI service manages a tool-call-result loop. Frontend renders tool events inline in chat.

**Tech Stack:** Rust (scraper, regex, pdf-extract crates) + React (toolbar, tool bubbles)

---

## Overview: 8 Modules

This plan is split into 8 modules. Modules 1-4 are backend, modules 5-8 are frontend + integration.

**Module dependency chain:**
```
M1 (tool models + engine) → M2 (LLM provider tools) → M3 (atomic tools) → M4 (workflows + triggers)
M5 (settings backend) — independent
M6 (settings frontend + toolbar) — depends on M5
M7 (chat tool events) — depends on M1
M8 (UI polish) — depends on M6, M7
```

---

## Module 1: Tool Engine Framework

**Files:**
- Create: `ring-server/src/models/tool_model.rs`
- Create: `ring-server/src/services/tool_engine/mod.rs`
- Create: `ring-server/src/services/tool_engine/registry.rs`
- Create: `ring-server/src/services/tool_engine/dispatcher.rs`
- Modify: `ring-server/src/models/mod.rs`
- Modify: `ring-server/src/services/mod.rs`
- Modify: `ring-server/src/state.rs`

### Task 1.1: Tool Models

Create `ring-server/src/models/tool_model.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub tool_call_id: String,
    pub tool_name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultRecord {
    pub tool_call_id: String,
    pub tool_name: String,
    pub output: serde_json::Value,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecution {
    pub call: ToolCallRequest,
    pub result: Option<ToolResultRecord>,
}
```

Register in `models/mod.rs`: `pub mod tool_model;`

### Task 1.2: Tool Trait + Registry

Create `ring-server/src/services/tool_engine/mod.rs`:

```rust
pub mod dispatcher;
pub mod registry;

pub use dispatcher::ToolDispatcher;
pub use registry::ToolRegistry;

use async_trait::async_trait;
use crate::error::Result;
use crate::models::tool_model::{ToolDefinition, ToolResultRecord};

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value>;
}
```

Create `ring-server/src/services/tool_engine/registry.rs`:

```rust
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::tool_model::ToolDefinition;
use super::Tool;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.definition().name;
        self.tools.insert(name, tool);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

Create `ring-server/src/services/tool_engine/dispatcher.rs`:

```rust
use std::sync::Arc;

use crate::error::{Result, RingError};
use crate::models::tool_model::{ToolCallRequest, ToolResultRecord};
use super::registry::ToolRegistry;

pub struct ToolDispatcher {
    registry: Arc<ToolRegistry>,
}

impl ToolDispatcher {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        ToolDispatcher { registry }
    }

    pub async fn dispatch(&self, call: ToolCallRequest) -> ToolResultRecord {
        match self.registry.get(&call.tool_name) {
            Some(tool) => match tool.execute(call.input.clone()).await {
                Ok(output) => ToolResultRecord {
                    tool_call_id: call.tool_call_id,
                    tool_name: call.tool_name,
                    output,
                    success: true,
                },
                Err(e) => ToolResultRecord {
                    tool_call_id: call.tool_call_id,
                    tool_name: call.tool_name,
                    output: serde_json::json!({ "error": e.to_string() }),
                    success: false,
                },
            },
            None => ToolResultRecord {
                tool_call_id: call.tool_call_id,
                tool_name: call.tool_name,
                output: serde_json::json!({ "error": format!("unknown tool: {}", call.tool_name) }),
                success: false,
            },
        }
    }
}
```

### Task 1.3: Add ToolRegistry to AppState

Modify `state.rs` to add `tool_registry: Arc<ToolRegistry>` field.

Modify `main.rs` to create `ToolRegistry` and pass to state.

Register in `services/mod.rs`:
```rust
pub mod tool_engine;
pub use tool_engine::{ToolDispatcher, ToolRegistry};
```

---

## Module 2: LLM Provider Tool Calling

**Files:**
- Modify: `ring-server/src/services/llm_provider.rs`
- Modify: `ring-server/src/services/llm_openai.rs`
- Modify: `ring-server/src/services/llm_anthropic.rs`
- Modify: `ring-server/src/services/ai_service.rs`

### Task 2.1: Extend LlmProvider Trait

Add `tools` parameter to `chat_stream`:

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn chat_stream(
        &self,
        messages: Vec<LlmMessage>,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Pin<Box<dyn Stream<Item = LlmEvent> + Send>>;
}
```

Update `MockLlmProvider` in `llm_provider.rs` to accept `tools` parameter (ignore it for mock).

### Task 2.2: OpenAI Tool Calls

Modify `llm_openai.rs`:
- Add `tools` to the `CreateChatCompletionRequest` when provided
- Parse `delta.tool_calls` from streaming events
- Emit `LlmEvent::ToolCall` when tool_calls appear in the stream

### Task 2.3: Anthropic Tool Use

Modify `llm_anthropic.rs`:
- Add `tools` to the request JSON body when provided
- Parse `content_block_start` with `type: "tool_use"` from SSE
- Parse `input_json_delta` for tool input
- Emit `LlmEvent::ToolCall` when tool_use blocks appear

### Task 2.4: AI Service Tool Loop

Modify `ai_service.rs` to implement the tool-call-result loop:

```
1. Build messages + optional tools list
2. Call LLM with tools
3. Stream events to caller (SSE)
4. When ToolCall event arrives:
   a. Emit ToolCall event to SSE
   b. Execute tool via ToolDispatcher
   c. Emit ToolResult event to SSE
   d. Feed tool result back into messages
   e. Call LLM again (loop back to step 2)
5. Maximum 5 tool rounds per user message (safety limit)
```

---

## Module 3: Atomic Tool Implementations

**Files:**
- Create: `ring-server/src/services/tool_engine/tools/mod.rs`
- Create: `ring-server/src/services/tool_engine/tools/search_tool.rs`
- Create: `ring-server/src/services/tool_engine/tools/text_clean_tool.rs`
- Create: `ring-server/src/services/tool_engine/tools/web_scrape_tool.rs`
- Create: `ring-server/src/services/tool_engine/tools/markdown_gen_tool.rs`
- Create: `ring-server/src/services/tool_engine/tools/privacy_filter_tool.rs`

### Task 3.1: Add Dependencies

Add to `Cargo.toml`:
```toml
scraper = "0.21"
regex = "1"
```

### Task 3.2: Search Tool

Wraps existing `SearchService`. Input: `{ query, graph_ids?, limit? }`. Output: `{ results: [...] }`.

### Task 3.3: Text Clean Tool

Pure Rust. Input: `{ text }`. Output: `{ cleaned_text }`. Strips extra whitespace, normalizes unicode.

### Task 3.4: Web Scrape Tool

Uses `reqwest` (already present) + `scraper`. Input: `{ url }`. Output: `{ title, text }`. Fetches URL, extracts text content.

### Task 3.5: Markdown Generation Tool

Pure Rust. Input: `{ title, sections: [{heading, body}] }`. Output: `{ markdown }`. Generates formatted markdown.

### Task 3.6: Privacy Filter Tool

Uses `regex`. Input: `{ text }`. Output: `{ filtered_text, redactions_count }`. Redacts email, phone, ID card patterns.

### Task 3.7: Register All Tools

In `main.rs`, register all tools into the `ToolRegistry`:
```rust
registry.register(Arc::new(SearchTool::new(db.clone())));
registry.register(Arc::new(TextCleanTool::new()));
registry.register(Arc::new(WebScrapeTool::new()));
registry.register(Arc::new(MarkdownGenTool::new()));
registry.register(Arc::new(PrivacyFilterTool::new()));
```

---

## Module 4: Workflows + AI Triggers

**Files:**
- Create: `ring-server/src/services/workflow_service.rs`
- Create: `ring-server/src/services/trigger_service.rs`
- Create: `ring-server/src/services/settings_service.rs`
- Create: `ring-server/src/handlers/settings.rs`
- Modify: `ring-server/src/routes.rs`
- Modify: `ring-server/src/handlers/mod.rs`
- Modify: `ring-server/src/services/mod.rs`

### Task 4.1: Settings Service + Handler

Thin wrapper around existing `get_setting`/`set_setting` DB methods.

`GET /api/v1/settings` returns all settings as JSON.
`PUT /api/v1/settings` accepts `{ llm: {...}, privacy: {...} }` and stores.

### Task 4.2: Workflow Service

Defines 3 preset workflows (meeting_archive, deep_research, learning_center). Each workflow is a sequence of tool calls orchestrated by the AI service. The workflow service provides the initial prompt + tool set for each scenario.

### Task 4.3: Trigger Service

Background rules that fire based on context:
- After AI response, check if archive is appropriate → emit `ArchiveSuggestion`
- When graph has < 3 nodes and user sends first message → emit empty graph guidance
- These are checked in the AI service after each response

---

## Module 5: Frontend — Settings + Toolbar

**Files:**
- Create: `ring-frontend/src/pages/Settings/SettingsPage.tsx`
- Create: `ring-frontend/src/stores/settingsStore.ts`
- Modify: `ring-frontend/src/api/client.ts`
- Modify: `ring-frontend/src/App.tsx`

### Task 5.1: Settings API + Store

Add to `client.ts`:
```typescript
export async function get_settings(): Promise<Settings> { ... }
export async function update_settings(settings: Partial<Settings>): Promise<void> { ... }
```

Create `settingsStore.ts` with Zustand pattern matching existing stores.

### Task 5.2: Settings Page

React component with LLM config form (provider, model, api_key, base_url) + privacy toggle.

### Task 5.3: Add Settings Route

Add `/settings` route to `App.tsx`.

---

## Module 6: Frontend — Tool Event Rendering

**Files:**
- Modify: `ring-frontend/src/stores/chatStore.ts`
- Modify: `ring-frontend/src/components/chat/ChatBubble.tsx`
- Create: `ring-frontend/src/components/chat/ToolCallBubble.tsx`
- Create: `ring-frontend/src/components/chat/ToolResultBubble.tsx`
- Create: `ring-frontend/src/components/chat/ArchiveSuggestion.tsx`

### Task 6.1: Handle Tool Events in ChatStore

Update `chatStore.ts` SSE handler to process `tool_call`, `tool_result`, and `archive_suggestion` events instead of silently dropping them. Store them as part of the message stream.

### Task 6.2: Tool Call/Result Bubbles

`ToolCallBubble.tsx`: Shows tool name + spinner (executing) or checkmark (done).
`ToolResultBubble.tsx`: Shows tool output in a collapsible section.
`ArchiveSuggestion.tsx`: Shows AI recommendation with Accept/Dismiss buttons.

### Task 6.3: ChatBubble Dispatch

Update `ChatBubble.tsx` to render different bubble types based on event type.

---

## Module 7: Frontend — Toolbar UI

**Files:**
- Create: `ring-frontend/src/components/toolbar/Toolbar.tsx`
- Modify: `ring-frontend/src/components/chat/ChatInput.tsx`

### Task 7.1: Toolbar Component

Simple toolbar that shows available tools with status. Integrates above the chat input.

### Task 7.2: Integrate Toolbar with ChatInput

Add toolbar component to the chat area.

---

## Module 8: Polish + Verification

### Task 8.1: Error Handling

Ensure all new handlers return proper error types. Frontend shows error toasts.

### Task 8.2: Clippy + Build Verification

Run `cargo clippy -- -D warnings`, `cargo fmt`, `npm run build`, `npm test`.

### Task 8.3: Commit + Merge

```bash
git add -A
git commit -m "feat(phase6): add tool engine, workflows, settings, and UI polish"
git checkout main
git merge feat/phase6-tool-engine --no-ff
```
