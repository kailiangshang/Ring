# Preset Workflow Tools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add two preset workflow tools (file_parse, knowledge_extract) to Group Ring via tool_calls, with LLM-driven invocation and structured node proposals.

**Architecture:** Group Ring chat gains a tool_calls loop (same pattern as Super Ring's `stream_super_chat_inner`). LLM decides when to call tools. Backend executes tool pipelines synchronously, re-calls LLM with tool results, streams final response. Node proposals embedded as `<file_analysis>` / `<knowledge_extraction>` XML blocks in AI messages.

**Tech Stack:** Rust + Axum + async-openai (backend), React + TypeScript (frontend)

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `server/src/prompts.rs` | Add `workflow` module |
| Create | `server/src/services/workflow.rs` | Tool execution pipelines |
| Modify | `server/src/services/llm.rs` | Add `chat_stream_with_tools()` method |
| Modify | `server/src/services/mod.rs` | Register `workflow` module |
| Modify | `server/src/services/chat.rs` | Add `get_group_ring_tools()`, `execute_group_tool()` |
| Modify | `server/src/routes/chat.rs` | Use tool-calling path for Group Ring |
| Modify | `ui/src/components/chat/MessageItem.tsx` | Render `<file_analysis>` / `<knowledge_extraction>` cards |
| Modify | `ui/src/stores/graph-store.ts` | Add `createNodesFromExtraction()` |
| Modify | `server/tests/integration.rs` | Tests |

---

### Task 1: Add `workflow` prompt module + `workflow` service skeleton

**Files:**
- Modify: `server/src/prompts.rs`
- Create: `server/src/services/workflow.rs`
- Modify: `server/src/services/mod.rs`

- [ ] **Step 1: Add `workflow` module to `prompts.rs`**

Add at the end of `server/src/prompts.rs`:

```rust
pub mod workflow {
    pub fn file_parse_extraction(focus: Option<&str>) -> String {
        let mut prompt = String::from(
            "分析以下文件内容，提取结构化知识。\n\n\
            输出格式：\n\
            <file_analysis>\n\
            {\"summary\": \"文件摘要\", \"concepts\": [{\"label\": \"概念名\", \"node_type\": \"category|topic|leaf\", \"tags\": []}], \"relations\": [{\"from\": \"概念A\", \"to\": \"概念B\", \"relation\": \"related_to\"}]}\n\
            </file_analysis>\n\n\
            规则：\n\
            - 提取 3-10 个核心概念作为建议的图谱节点\n\
            - node_type: category（顶层分类）/ topic（具体主题）/ leaf（细节）\n\
            - relation: depends_on / related_to / derives_from / contradicts\n\
            - 每个概念有意义的标签\n\
            - 简洁摘要，不超过 3 句",
        );
        if let Some(f) = focus {
            if !f.is_empty() {
                prompt.push_str(&format!("\n\n重点关注：{f}"));
            }
        }
        prompt
    }

    pub fn knowledge_extraction_prompt(target_graph: Option<&str>) -> String {
        let mut prompt = String::from(
            "从以下内容中提取知识概念和关系。\n\n\
            输出格式：\n\
            <knowledge_extraction>\n\
            {\"concepts\": [{\"label\": \"概念名\", \"node_type\": \"category|topic|leaf\", \"tags\": []}], \"relations\": [{\"from\": \"概念A\", \"to\": \"概念B\", \"relation\": \"related_to\"}], \"suggested_graph\": \"图谱名\"}\n\
            </knowledge_extraction>\n\n\
            规则：\n\
            - 识别核心实体、概念和它们之间的关系\n\
            - 生成适合图谱结构的节点和边\n\
            - 建议节点类型和标签\n\
            - relation: depends_on / related_to / derives_from / contradicts",
        );
        if let Some(g) = target_graph {
            if !g.is_empty() {
                prompt.push_str(&format!("\n\n目标图谱：{g}"));
            }
        }
        prompt
    }
}
```

- [ ] **Step 2: Create `workflow.rs` service**

Create `server/src/services/workflow.rs`:

```rust
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::error::{Result, RingError};
use crate::models::user::UserRow;
use crate::services::llm::LlmClient;

#[derive(Debug, Deserialize)]
pub struct FileParseArgs {
    pub file_reference: String,
    pub focus: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeExtractArgs {
    pub content: String,
    pub target_graph: Option<String>,
}

pub async fn execute_file_parse(
    pool: &SqlitePool,
    user: &UserRow,
    args: &FileParseArgs,
) -> Result<String> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT content FROM messages WHERE id = ?1",
    )
    .bind(&args.file_reference)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound(format!("message {} not found", args.file_reference)))?;

    let file_text = row.0;
    let truncated: String = file_text.chars().take(30000).collect();

    let prompt = crate::prompts::workflow::file_parse_extraction(args.focus.as_deref());
    let llm = LlmClient::from_user(user)?;
    let result = llm.chat_complete(prompt, truncated).await?;
    Ok(result)
}

pub async fn execute_knowledge_extract(
    user: &UserRow,
    args: &KnowledgeExtractArgs,
) -> Result<String> {
    let prompt =
        crate::prompts::workflow::knowledge_extraction_prompt(args.target_graph.as_deref());
    let llm = LlmClient::from_user(user)?;
    let truncated: String = args.content.chars().take(30000).collect();
    let result = llm.chat_complete(prompt, truncated).await?;
    Ok(result)
}
```

- [ ] **Step 3: Register module in `mod.rs`**

Add `pub mod workflow;` to `server/src/services/mod.rs`.

- [ ] **Step 4: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 5: Commit**

```bash
git add server/src/prompts.rs server/src/services/workflow.rs server/src/services/mod.rs
git commit -m "feat(workflow): add workflow prompts and service skeleton"
```

---

### Task 2: Add `chat_stream_with_tools()` to `LlmClient`

**Files:**
- Modify: `server/src/services/llm.rs`

- [ ] **Step 1: Add the streaming-with-tools method**

Add a new method to `impl LlmClient` in `server/src/services/llm.rs`, after `chat_complete_with_tools` (after line 278). This method handles the full tool_calls loop: first LLM call with tools → detect tool_calls → execute externally (via callback) → second LLM call → stream result.

```rust
    pub fn chat_stream_with_tools<F, Fut>(
        self,
        system_prompt: String,
        history: Vec<(String, String)>,
        user_message: String,
        ai_role: String,
        tools: Vec<async_openai::types::ChatCompletionTool>,
        tool_executor: F,
    ) -> tokio::sync::mpsc::Receiver<SseEvent>
    where
        F: Fn(String, serde_json::Value) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = crate::error::Result<String>> + Send,
    {
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
                messages: messages.clone(),
                model: self.model.clone(),
                tools: Some(tools),
                tool_choice: Some(
                    async_openai::types::ChatCompletionToolChoiceOption::Auto,
                ),
                ..Default::default()
            };

            let client = self.client.clone();
            let model = self.model.clone();

            match client.chat().create(request).await {
                Ok(response) => {
                    if let Some(choice) = response.choices.first() {
                        if let Some(tool_calls) = &choice.message.tool_calls {
                            let assistant_content =
                                choice.message.content.clone().unwrap_or_default();

                            let mut tool_messages = vec![];
                            let mut tc_list = vec![];

                            for tc in tool_calls {
                                let args: serde_json::Value =
                                    serde_json::from_str(&tc.function.arguments)
                                        .unwrap_or(serde_json::json!({}));
                                let result = tool_executor(tc.function.name.clone(), args).await;

                                let tool_result = match result {
                                    Ok(r) => r,
                                    Err(e) => format!("Error: {e}"),
                                };

                                tc_list.push(
                                    async_openai::types::ChatCompletionMessageToolCall {
                                        id: tc.id.clone(),
                                        r#type: Some(
                                            async_openai::types::ChatCompletionToolType::Function,
                                        ),
                                        function: async_openai::types::FunctionCall {
                                            name: tc.function.name.clone(),
                                            arguments: tc.function.arguments.clone(),
                                        },
                                    },
                                );

                                tool_messages.push(ChatCompletionRequestMessage::Tool(
                                    ChatCompletionRequestToolMessage {
                                        content: ChatCompletionRequestToolMessageContent::Text(
                                            tool_result,
                                        ),
                                        tool_call_id: tc.id.clone(),
                                    },
                                ));
                            }

                            messages.push(ChatCompletionRequestMessage::Assistant(
                                #[allow(deprecated)]
                                ChatCompletionRequestAssistantMessage {
                                    content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                                        assistant_content,
                                    )),
                                    name: None,
                                    tool_calls: Some(tc_list),
                                    refusal: None,
                                    audio: None,
                                    function_call: None,
                                },
                            ));

                            for tm in tool_messages {
                                messages.push(tm);
                            }

                            let second_request = CreateChatCompletionRequest {
                                messages,
                                model,
                                stream: Some(true),
                                ..Default::default()
                            };

                            match client.chat().create_stream(second_request).await {
                                Ok(mut stream) => {
                                    let mut full_content = String::new();
                                    let mut token_usage: Option<String> = None;
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
                                                if let Some(usage) = &chunk.usage {
                                                    token_usage = Some(
                                                        serde_json::to_string(usage)
                                                            .unwrap_or_default(),
                                                    );
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
                                            token_usage,
                                        })
                                        .await;
                                }
                                Err(e) => {
                                    let _ = tx.send(SseEvent::Error(e.to_string())).await;
                                }
                            }
                        } else {
                            let content = choice.message.content.clone().unwrap_or_default();
                            let _ = tx
                                .send(SseEvent::Delta {
                                    content: content.clone(),
                                })
                                .await;
                            let _ = tx
                                .send(SseEvent::End {
                                    message_id: message_id.clone(),
                                    full_content: content,
                                    token_usage: None,
                                })
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(SseEvent::Error(e.to_string())).await;
                }
            }
        });

        rx
    }
```

Note: Requires `Clone` on `LlmClient` — add `#[derive(Clone)]` to `LlmClient` struct, and add `Clone` bound to `OpenAIConfig` (it already implements Clone). Also requires `use async_openai::types::{ChatCompletionRequestToolMessage, ChatCompletionRequestToolMessageContent};` at the top.

- [ ] **Step 2: Add Clone derive to LlmClient**

Add `#[derive(Clone)]` to `pub struct LlmClient`.

- [ ] **Step 3: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 4: Commit**

```bash
git add server/src/services/llm.rs
git commit -m "feat(workflow): add chat_stream_with_tools to LlmClient"
```

---

### Task 3: Add `get_group_ring_tools()` and `execute_group_tool()` to chat.rs

**Files:**
- Modify: `server/src/services/chat.rs`

- [ ] **Step 1: Add tool definitions**

Add at the end of `server/src/services/chat.rs`:

```rust
pub fn get_group_ring_tools() -> Vec<async_openai::types::ChatCompletionTool> {
    vec![
        async_openai::types::ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "file_parse".into(),
                description: Some("Parse an uploaded file and extract structured knowledge. Recommend graph nodes.".into()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_reference": { "type": "string", "description": "The message_id of the file upload message" },
                        "focus": { "type": "string", "description": "Optional focus area for extraction" }
                    },
                    "required": ["file_reference"]
                })),
            },
        },
        async_openai::types::ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: async_openai::types::FunctionObject {
                name: "knowledge_extract".into(),
                description: Some("Extract knowledge concepts from text and generate graph node recommendations.".into()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "Text or topic to extract knowledge from" },
                        "target_graph": { "type": "string", "description": "Optional target graph name" }
                    },
                    "required": ["content"]
                })),
            },
        },
    ]
}

pub async fn execute_group_tool(
    pool: &sqlx::SqlitePool,
    user: &crate::models::user::UserRow,
    tool_name: String,
    args: serde_json::Value,
) -> crate::error::Result<String> {
    match tool_name.as_str() {
        "file_parse" => {
            let parsed: crate::services::workflow::FileParseArgs =
                serde_json::from_value(args)
                    .map_err(|e| crate::error::RingError::BadRequest(e.to_string()))?;
            crate::services::workflow::execute_file_parse(pool, user, &parsed).await
        }
        "knowledge_extract" => {
            let parsed: crate::services::workflow::KnowledgeExtractArgs =
                serde_json::from_value(args)
                    .map_err(|e| crate::error::RingError::BadRequest(e.to_string()))?;
            crate::services::workflow::execute_knowledge_extract(user, &parsed).await
        }
        _ => Err(crate::error::RingError::BadRequest(format!(
            "unknown tool: {tool_name}"
        ))),
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 3: Commit**

```bash
git add server/src/services/chat.rs
git commit -m "feat(workflow): add get_group_ring_tools and execute_group_tool"
```

---

### Task 4: Update `ring_chat` handler to use tools

**Files:**
- Modify: `server/src/routes/chat.rs`

- [ ] **Step 1: Update ring_chat to use tool-calling path**

Read `server/src/routes/chat.rs` and find the `ring_chat` handler (around line 48). After the graph command detection and archive intent detection (around line 105-107, before `auto_compact_history`), change the flow to use `chat_stream_with_tools` when it's a normal Group Ring chat.

Find the section where `chat::start_chat_stream` is called (around line 109) and replace it with a tool-calling path. The key change: instead of calling `chat::start_chat_stream`, call `LlmClient::chat_stream_with_tools` with group ring tools and a tool executor closure.

The new flow for the normal chat path (after graph command and archive intent checks):

```rust
    let tools = crate::services::chat::get_group_ring_tools();
    let pool_c = state.db.clone();
    let user_row_tool = user_row.clone();

    let mut rx = {
        let llm = crate::services::llm::LlmClient::from_user(&user_row)?;
        let system_prompt = chat::build_system_prompt(Some(&ring_info.0), ring_info.1.as_deref());
        let history = chat::load_history_context(&state.db, Some(&ring_id), &user.token_id, 20).await?;
        let filters = user_row
            .privacy_filters
            .as_deref()
            .map(crate::services::privacy_filter::PrivacyFilters::from_json)
            .unwrap_or_default();
        let filtered_content = crate::services::privacy_filter::apply_filters(&body.content, &filters);

        let pool_t = pool_c.clone();
        let user_t = user_row_tool.clone();

        llm.chat_stream_with_tools(
            system_prompt,
            history,
            filtered_content,
            "group_ring".to_string(),
            tools,
            move |name: String, args: serde_json::Value| {
                let pool = pool_t.clone();
                let user = user_t.clone();
                async move {
                    crate::services::chat::execute_group_tool(&pool, &user, name, args).await
                }
            },
        )
    };
```

This replaces the current `chat::start_chat_stream(...)` call in the normal Group Ring chat path.

- [ ] **Step 2: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 3: Commit**

```bash
git add server/src/routes/chat.rs
git commit -m "feat(workflow): update ring_chat to use tool-calling path"
```

---

### Task 5: Frontend — render `<file_analysis>` and `<knowledge_extraction>` cards

**Files:**
- Modify: `ui/src/components/chat/MessageItem.tsx`

- [ ] **Step 1: Add extraction card parsing and rendering**

Read `ui/src/components/chat/MessageItem.tsx`. Add a function to extract JSON from XML tags and a component to render the analysis card.

Add these utility functions and component inside the file (before the main `MessageItem` export):

```typescript
interface ExtractedConcept {
  label: string
  node_type: string
  tags: string[]
}

interface ExtractionData {
  summary?: string
  concepts: ExtractedConcept[]
  relations: { from: string; to: string; relation: string }[]
  suggested_graph?: string
}

function parseExtraction(text: string, tag: string): ExtractionData | null {
  const re = new RegExp(`<${tag}>\\s*([\\s\\S]*?)\\s*<\\/${tag}>`)
  const match = text.match(re)
  if (!match) return null
  try {
    return JSON.parse(match[1])
  } catch {
    return null
  }
}

function stripExtractionTags(text: string): string {
  return text
    .replace(/<file_analysis>[\s\S]*?<\/file_analysis>/g, '')
    .replace(/<knowledge_extraction>[\s\S]*?<\/knowledge_extraction>/g, '')
    .trim()
}

function ExtractionCard({
  data,
  onAddToGraph,
}: {
  data: ExtractionData
  onAddToGraph: () => void
}) {
  return (
    <div
      style={{
        background: 'var(--bg-input)',
        border: '1px solid var(--border)',
        borderRadius: 6,
        padding: 10,
        marginTop: 8,
      }}
    >
      {data.summary && (
        <div style={{ fontSize: 10, color: 'var(--text-secondary)', marginBottom: 6 }}>
          {data.summary}
        </div>
      )}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4, marginBottom: 6 }}>
        {data.concepts.map((c, i) => (
          <span
            key={i}
            style={{
              fontSize: 9,
              background:
                c.node_type === 'category'
                  ? 'rgba(34,211,238,0.15)'
                  : c.node_type === 'leaf'
                    ? 'rgba(52,211,153,0.15)'
                    : 'rgba(167,139,250,0.15)',
              padding: '2px 6px',
              borderRadius: 3,
              color: 'var(--text-secondary)',
            }}
          >
            {c.label}
          </span>
        ))}
      </div>
      {data.relations.length > 0 && (
        <div style={{ fontSize: 9, color: 'var(--text-dim)', lineHeight: 1.6, marginBottom: 6 }}>
          {data.relations.slice(0, 5).map((r, i) => (
            <div key={i}>
              {r.from} <span style={{ color: 'var(--accent-cyan)' }}>→</span> {r.to}{' '}
              <span style={{ color: 'var(--text-dim)' }}>({r.relation})</span>
            </div>
          ))}
        </div>
      )}
      <button
        onClick={onAddToGraph}
        style={{
          fontSize: 9,
          fontWeight: 700,
          background: 'var(--accent-cyan)',
          color: 'var(--bg-base)',
          border: 'none',
          borderRadius: 3,
          padding: '3px 8px',
          cursor: 'pointer',
        }}
      >
        添加到图谱
      </button>
    </div>
  )
}
```

- [ ] **Step 2: Use extraction card in message rendering**

In the `MessageItem` component, find where message content is rendered. Add extraction detection before rendering:

After parsing the content, check for `<file_analysis>` or `<knowledge_extraction>` tags. If found, render the card component. Strip the XML from the displayed text.

Find the message content rendering section and wrap it:

```typescript
const fileAnalysis = parseExtraction(message.content, 'file_analysis')
const knowledgeExtraction = parseExtraction(message.content, 'knowledge_extraction')
const displayContent = stripExtractionTags(message.content)
const hasExtraction = fileAnalysis || knowledgeExtraction
```

Then in the JSX, after the regular content rendering:

```tsx
{hasExtraction && (
  <>
    {fileAnalysis && (
      <ExtractionCard
        data={fileAnalysis}
        onAddToGraph={() => {
          useGraphStore.getState().createNodesFromExtraction(
            activeRingId,
            fileAnalysis.concepts,
            fileAnalysis.relations,
          )
        }}
      />
    )}
    {knowledgeExtraction && (
      <ExtractionCard
        data={knowledgeExtraction}
        onAddToGraph={() => {
          useGraphStore.getState().createNodesFromExtraction(
            activeRingId,
            knowledgeExtraction.concepts,
            knowledgeExtraction.relations,
          )
        }}
      />
    )}
  </>
)}
```

Note: `activeRingId` must be available in the component. Check how it's currently accessed (likely from `useRingStore`).

- [ ] **Step 3: Verify frontend builds**

Run: `cd ui && npm run build 2>&1 | tail -5`

- [ ] **Step 4: Commit**

```bash
git add ui/src/components/chat/MessageItem.tsx
git commit -m "feat(workflow): render file_analysis and knowledge_extraction cards in chat"
```

---

### Task 6: Add `createNodesFromExtraction()` to graph-store

**Files:**
- Modify: `ui/src/stores/graph-store.ts`

- [ ] **Step 1: Add the function**

Read `ui/src/stores/graph-store.ts`. Add a new function to the store interface and implementation:

In the `GraphState` interface, add:

```typescript
createNodesFromExtraction: (
  ringId: string,
  concepts: { label: string; node_type: string; tags: string[] }[],
  relations: { from: string; to: string; relation: string }[],
) => Promise<void>
```

In the store implementation, add:

```typescript
createNodesFromExtraction: async (ringId, concepts, relations) => {
  const { graph_id, fetchGraph } = get()
  if (!graph_id) {
    const graphs = await api.get<{ graphs: { id: string }[] }>('/rings/' + ringId + '/graphs')
    if (graphs.graphs.length === 0) return
  }

  const labelToId = new Map<string, string>()

  for (const concept of concepts) {
    try {
      const res = await api.post<{ id: string }>(`/rings/${ringId}/graph`, {
        label: concept.label,
        node_type: concept.node_type,
        tags: concept.tags,
      })
      labelToId.set(concept.label, res.id)
    } catch {}
  }

  for (const rel of relations) {
    const sourceId = labelToId.get(rel.from)
    const targetId = labelToId.get(rel.to)
    if (sourceId && targetId) {
      try {
        await api.post(`/rings/${ringId}/graph/edges`, {
          source_id: sourceId,
          target_id: targetId,
          relation: rel.relation,
        })
      } catch {}
    }
  }

  fetchGraph(ringId)
}
```

- [ ] **Step 2: Verify frontend builds**

Run: `cd ui && npm run build 2>&1 | tail -5`

- [ ] **Step 3: Commit**

```bash
git add ui/src/stores/graph-store.ts
git commit -m "feat(workflow): add createNodesFromExtraction to graph-store"
```

---

### Task 7: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add tests**

Read `server/tests/integration.rs` and add:

```rust
#[sqlx::test(migrations = "./migrations")]
async fn test_group_ring_tools_defined(pool: SqlitePool) {
    let tools = crate::services::chat::get_group_ring_tools();
    assert_eq!(tools.len(), 2);

    let names: Vec<&str> = tools.iter().map(|t| t.function.name.as_str()).collect();
    assert!(names.contains(&"file_parse"));
    assert!(names.contains(&"knowledge_extract"));
}
```

- [ ] **Step 2: Run all tests**

Run: `cd server && cargo test --test integration 2>&1 | tail -10`

- [ ] **Step 3: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test(workflow): add group ring tools test"
```

---

### Task 8: Final verification + STATUS.md

**Files:**
- Modify: `docs/STATUS.md`

- [ ] **Step 1: Run full test suite**

Run: `cd server && cargo test --test integration 2>&1 | tail -10`
Run: `cd ui && npm run build 2>&1 | tail -5`

- [ ] **Step 2: Update STATUS.md**

- Update test count
- Change PRD item:

```
| 预设工作流工具（文件解析/知识提取/深度调研） | done（文件解析 + 知识提取，AI tool_calls 驱动，节点推荐 + 确认） | 低 |
```

Add to "本轮完成":

```
- **预设工作流工具** — Group Ring tool_calls 基础设施，file_parse（文件解析 → 结构化提取 → 图谱节点推荐）和 knowledge_extract（文本 → 概念提取 → 图谱节点推荐），AI 自动决定何时调用，`<file_analysis>` / `<knowledge_extraction>` 结构化输出卡片
```

- [ ] **Step 3: Commit**

```bash
git add docs/STATUS.md
git commit -m "docs: update STATUS.md for Preset Workflow Tools"
```
