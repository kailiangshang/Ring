# Deep Blueprint Builder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add AI-guided conversational blueprint building to the BlueprintPanel, where Group Ring co-designs graph structures with the user through multi-turn dialogue, with live D3.js preview and multi-graph support. Also fix the edge creation bug in the quick path.

**Architecture:** Structured output approach — AI outputs `<blueprint>` JSON blocks in SSE chat messages, frontend parses and renders live D3 preview. New blueprint chat route reuses existing SSE streaming. Confirm route creates graphs, nodes, AND edges. Sliding window (15 messages) with `current_blueprint` injection into system prompt for context management.

**Tech Stack:** Rust + Axum (backend SSE), React + TypeScript + Zustand + D3.js (frontend)

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `server/src/prompts.rs` | Add `blueprint` module |
| Modify | `server/src/routes/blueprint.rs` | Add `blueprint_chat`, `blueprint_history`, update `confirm` |
| Modify | `server/src/services/blueprint_service.rs` | Add `ConfirmBlueprintRequest`, `confirm_with_blueprint()` |
| Modify | `server/src/routes/mod.rs` | Register new routes |
| Modify | `ui/src/components/panels/BlueprintPanel.tsx` | Add deep path mode with chat + D3 preview |
| Create | `ui/src/stores/blueprint-store.ts` | Blueprint chat state management |
| Modify | `server/tests/integration.rs` | New tests |

---

### Task 1: Add `blueprint` prompt module to `prompts.rs`

**Files:**
- Modify: `server/src/prompts.rs`

- [ ] **Step 1: Add the blueprint module**

Add a new `pub mod blueprint` block in `server/src/prompts.rs` (after the existing modules like `self_chat`, `group_ring`, etc.):

```rust
pub mod blueprint {
    pub fn system(ring_name: &str, role_description: Option<&str>, current_blueprint: Option<&str>) -> String {
        let mut prompt = format!(
            "你是 {ring_name} 的 Group Ring，正在帮用户设计知识图谱蓝图。\n\n\
            你需要通过对话了解：\n\
            1. 这个 Ring 的核心知识领域\n\
            2. 需要几个图谱（最多 3 个）\n\
            3. 每个图谱的主题和顶层分类节点\n\
            4. 节点之间的关系\n\n\
            每次你提出或调整图谱结构时，必须输出一个 <blueprint> JSON 块：\n\n\
            <blueprint>\n\
            {{\"graphs\": [{{\"name\": \"图谱名\", \"nodes\": [{{\"label\": \"节点名\", \"node_type\": \"category\", \"tags\": []}}], \"edges\": [{{\"from\": \"节点名\", \"to\": \"节点名\", \"relation\": \"related_to\"}}]}}]}}\n\
            </blueprint>\n\n\
            规则：\n\
            - 从了解需求开始，不要一上来就生成图谱\n\
            - 每次调整都输出完整的 blueprint JSON（不是增量）\n\
            - node_type: category / topic / leaf\n\
            - relation: depends_on / related_to / derives_from / contradicts\n\
            - 最多 3 个图谱\n\
            - 简洁对话"
        );
        if let Some(rd) = role_description {
            if !rd.is_empty() {
                prompt.push_str(&format!("\n\nRing 角色定义：\n{rd}"));
            }
        }
        if let Some(bp) = current_blueprint {
            if !bp.is_empty() {
                prompt.push_str(&format!("\n\n## 当前蓝图状态\n<current_blueprint>\n{bp}\n</current_blueprint>\n\n你必须在每次调整时输出完整的 <blueprint> JSON，不是增量。"));
            }
        }
        prompt
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 3: Commit**

```bash
git add server/src/prompts.rs
git commit -m "feat(blueprint): add blueprint prompt module"
```

---

### Task 2: Add `ConfirmBlueprintRequest` and `confirm_with_blueprint()` to blueprint_service

**Files:**
- Modify: `server/src/services/blueprint_service.rs`

- [ ] **Step 1: Add the request struct and confirm function**

Add these after the existing `FromTemplateRequest` struct (after line 37) in `server/src/services/blueprint_service.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct BlueprintGraphInput {
    pub name: String,
    pub nodes: Vec<PreviewNode>,
    pub edges: Vec<PreviewEdge>,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmBlueprintRequest {
    pub blueprint: Option<BlueprintGraphsInput>,
}

#[derive(Debug, Deserialize)]
pub struct BlueprintGraphsInput {
    pub graphs: Vec<BlueprintGraphInput>,
}
```

Add this function after `confirm_blueprint` (after line 91):

```rust
pub async fn confirm_with_blueprint(
    state: &AppState,
    ring_id: &str,
    req: &ConfirmBlueprintRequest,
) -> Result<()> {
    if let Some(ref bp) = req.blueprint {
        for graph_input in &bp.graphs {
            let graph_id = ulid::Ulid::new().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO graphs (id, ring_id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(&graph_id)
            .bind(ring_id)
            .bind(&graph_input.name)
            .bind(&now)
            .bind(&now)
            .execute(&state.db)
            .await?;

            let mut label_to_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            for node in &graph_input.nodes {
                let node_id = ulid::Ulid::new().to_string();
                let tags_json = serde_json::to_string(&node.tags)?;
                let node_now = chrono::Utc::now().to_rfc3339();
                sqlx::query(
                    "INSERT INTO graph_nodes (id, graph_id, ring_id, label, parent_id, node_type, content, tags, markdown_path, metadata, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, NULL, ?5, '', ?6, NULL, '{}', ?7, ?8)",
                )
                .bind(&node_id)
                .bind(&graph_id)
                .bind(ring_id)
                .bind(&node.label)
                .bind(&node.node_type)
                .bind(&tags_json)
                .bind(&node_now)
                .bind(&node_now)
                .execute(&state.db)
                .await?;
                label_to_id.insert(node.label.clone(), node_id);
            }

            for edge in &graph_input.edges {
                let source_id = label_to_id.get(&edge.from);
                let target_id = label_to_id.get(&edge.to);
                if let (Some(src), Some(tgt)) = (source_id, target_id) {
                    let edge_id = ulid::Ulid::new().to_string();
                    let edge_now = chrono::Utc::now().to_rfc3339();
                    sqlx::query(
                        "INSERT INTO graph_edges (id, graph_id, ring_id, source_id, target_id, relation, label, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', ?7)",
                    )
                    .bind(&edge_id)
                    .bind(&graph_id)
                    .bind(ring_id)
                    .bind(src)
                    .bind(tgt)
                    .bind(&edge.relation)
                    .bind(&edge_now)
                    .execute(&state.db)
                    .await?;
                }
            }
        }
    }
    confirm_blueprint(state, ring_id).await
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 3: Commit**

```bash
git add server/src/services/blueprint_service.rs
git commit -m "feat(blueprint): add confirm_with_blueprint that creates graphs, nodes, and edges"
```

---

### Task 3: Add blueprint chat and history routes

**Files:**
- Modify: `server/src/routes/blueprint.rs`

- [ ] **Step 1: Add imports and request structs**

At the top of `server/src/routes/blueprint.rs`, add these imports after the existing ones:

```rust
use async_stream::stream;
use axum::response::sse::{Event, KeepAlive, Sse};
use serde::Deserialize;
use std::convert::Infallible;

use crate::models::message;
use crate::services::llm::SseEvent;
```

Add request structs after imports:

```rust
#[derive(Debug, Deserialize)]
pub struct BlueprintChatRequest {
    pub content: String,
    pub current_blueprint: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BlueprintHistoryQuery {
    pub before: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, serde::Serialize)]
pub struct BlueprintHistoryResponse {
    pub messages: Vec<message::MessageRow>,
    pub has_more: bool,
}
```

- [ ] **Step 2: Add `blueprint_chat` handler**

Add after `confirm_blueprint_handler`:

```rust
pub async fn blueprint_chat(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<BlueprintChatRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(crate::error::RingError::Forbidden(
            "only creator/admin can manage blueprint".into(),
        ));
    }

    let status = sqlx::query_scalar::<_, String>(
        "SELECT blueprint_status FROM rings WHERE id = ?1",
    )
    .bind(&ring_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    if status == "confirmed" {
        return Err(crate::error::RingError::Forbidden(
            "blueprint already confirmed".into(),
        ));
    }

    let user_row = state.get_user_decrypted(&user.token_id).await?;

    let ring_info = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT name, role_description FROM rings WHERE id = ?1",
    )
    .bind(&ring_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?
    .ok_or_else(|| crate::error::RingError::NotFound("ring not found".into()))?;

    let current_bp_str = body
        .current_blueprint
        .as_ref()
        .map(|v| serde_json::to_string_pretty(v).unwrap_or_default());

    let system_prompt = crate::prompts::blueprint::system(
        &ring_info.0,
        ring_info.1.as_deref(),
        current_bp_str.as_deref(),
    );

    let history_messages = message::list_messages(
        &state.db,
        Some(&ring_id),
        &user.token_id,
        None,
        15,
    )
    .await
    .unwrap_or_default()
    .into_iter()
    .rev()
    .filter(|m| m.role == "blueprint" || m.role == "user")
    .map(|m| (m.role, m.content))
    .collect::<Vec<_>>();

    let user_msg_id = ulid::Ulid::new().to_string();
    message::insert_message(
        &state.db,
        &message::NewMessage {
            id: &user_msg_id,
            ring_id: Some(&ring_id),
            user_id: &user.token_id,
            role: "user",
            sender_name: &user.display_name,
            content: &body.content,
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await?;

    let llm = crate::services::llm::LlmClient::from_user(&user_row)?;
    let mut rx = llm.chat_stream(
        system_prompt,
        history_messages,
        body.content.clone(),
        "blueprint".to_string(),
    );

    let pool = state.db.clone();
    let ring_id_c = ring_id.clone();
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
                SseEvent::End { message_id, full_content, token_usage } => {
                    let usage_json = token_usage.as_deref().and_then(|u| serde_json::from_str::<serde_json::Value>(u).ok());
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": usage_json.unwrap_or(serde_json::json!({ "prompt_tokens": 0, "completion_tokens": 0 }))
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    let _ = message::insert_message(
                        &pool,
                        &message::NewMessage {
                            id: &message_id,
                            ring_id: Some(&ring_id_c),
                            user_id: &user_id,
                            role: "blueprint",
                            sender_name: "GROUP RING",
                            content: &full_content,
                            node_refs: &[],
                            tag_refs: &[],
                            token_usage: token_usage.as_deref(),
                        },
                    ).await;

                    let self_dir = crate::services::self_data::get_self_dir(&user_id);
                    let _ = crate::services::self_data::record_tool_usage(&self_dir, "blueprint");
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
```

- [ ] **Step 3: Add `blueprint_history` handler**

```rust
pub async fn blueprint_history(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<BlueprintHistoryQuery>,
) -> Result<Json<BlueprintHistoryResponse>> {
    let _ = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let limit = query.limit + 1;
    let messages = message::list_messages(
        &state.db,
        Some(&ring_id),
        &user.token_id,
        query.before.as_deref(),
        limit,
    )
    .await
    .unwrap_or_default()
    .into_iter()
    .rev()
    .filter(|m| m.role == "blueprint" || m.role == "user")
    .collect::<Vec<_>>();

    let has_more = messages.len() > query.limit as usize;
    let messages = if has_more {
        messages.into_iter().take(query.limit as usize).collect()
    } else {
        messages
    };

    Ok(Json(BlueprintHistoryResponse { messages, has_more }))
}
```

- [ ] **Step 4: Update `confirm_blueprint_handler` to accept optional blueprint body**

Replace the existing `confirm_blueprint_handler` with:

```rust
pub async fn confirm_blueprint_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<crate::services::blueprint_service::ConfirmBlueprintRequest>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(crate::error::RingError::Forbidden(
            "only creator/admin can manage blueprint".into(),
        ));
    }
    crate::services::blueprint_service::confirm_with_blueprint(&state, &ring_id, &body).await?;

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    let _ = crate::services::self_data::record_tool_usage(&self_dir, "blueprint");

    Ok(Json(serde_json::json!({ "status": "confirmed" })))
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 6: Commit**

```bash
git add server/src/routes/blueprint.rs
git commit -m "feat(blueprint): add blueprint chat, history routes, update confirm to accept blueprint JSON"
```

---

### Task 4: Register new routes in `mod.rs`

**Files:**
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Add blueprint chat and history routes**

In `server/src/routes/mod.rs`, find the existing blueprint routes (around lines 214-224):

```rust
        .route(
            "/rings/{ring_id}/blueprint",
            get(blueprint::get_blueprint_handler),
        )
        .route(
            "/rings/{ring_id}/blueprint/from-template",
            post(blueprint::preview_template),
        )
        .route(
            "/rings/{ring_id}/blueprint/confirm",
            post(blueprint::confirm_blueprint_handler),
        )
```

Add after them:

```rust
        .route("/rings/{ring_id}/blueprint/chat", post(blueprint::blueprint_chat))
        .route("/rings/{ring_id}/blueprint/chat/history", get(blueprint::blueprint_history))
```

- [ ] **Step 2: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 3: Commit**

```bash
git add server/src/routes/mod.rs
git commit -m "feat(blueprint): register blueprint chat and history routes"
```

---

### Task 5: Create blueprint store (`ui/src/stores/blueprint-store.ts`)

**Files:**
- Create: `ui/src/stores/blueprint-store.ts`

- [ ] **Step 1: Create the store**

Create `ui/src/stores/blueprint-store.ts`:

```typescript
import { create } from 'zustand'
import { api } from '../services/api'
import { streamChat } from '../services/sse'
import type { SseCallbacks } from '../services/sse'

interface BlueprintMessage {
  id: string
  role: string
  content: string
  token_usage?: { prompt_tokens: number; completion_tokens: number }
}

interface BlueprintGraph {
  name: string
  nodes: { label: string; node_type: string; tags: string[] }[]
  edges: { from: string; to: string; relation: string }[]
}

interface BlueprintState {
  mode: 'quick' | 'deep'
  messages: BlueprintMessage[]
  streaming: boolean
  current_blueprint: { graphs: BlueprintGraph[] } | null
  confirmed: boolean
  streaming_content: string
  abort_controller: AbortController | null
  setMode: (mode: 'quick' | 'deep') => void
  sendMessage: (ringId: string, content: string) => void
  loadHistory: (ringId: string) => Promise<void>
  confirm: (ringId: string) => Promise<void>
  checkStatus: (ringId: string) => Promise<void>
  stopStreaming: () => void
}

function extractBlueprint(text: string): { graphs: BlueprintGraph[] } | null {
  const match = text.match(/<blueprint>\s*([\s\S]*?)\s*<\/blueprint>/)
  if (!match) return null
  try {
    return JSON.parse(match[1])
  } catch {
    return null
  }
}

function stripBlueprintTags(text: string): string {
  return text.replace(/<blueprint>[\s\S]*?<\/blueprint>/g, '').trim()
}

export const useBlueprintStore = create<BlueprintState>((set, get) => ({
  mode: 'quick',
  messages: [],
  streaming: false,
  current_blueprint: null,
  confirmed: false,
  streaming_content: '',
  abort_controller: null,

  setMode: (mode) => set({ mode }),

  sendMessage: (ringId, content) => {
    const state = get()
    if (state.streaming) return

    const userMsg: BlueprintMessage = {
      id: Date.now().toString(),
      role: 'user',
      content,
    }

    const aiMsgId = (Date.now() + 1).toString()
    const aiMsg: BlueprintMessage = {
      id: aiMsgId,
      role: 'blueprint',
      content: '',
    }

    set({
      messages: [...state.messages, userMsg, aiMsg],
      streaming: true,
      streaming_content: '',
    })

    const callbacks: SseCallbacks = {
      onStart: () => {},
      onDelta: (data) => {
        set((s) => {
          const newContent = s.streaming_content + data.content
          const bp = extractBlueprint(newContent)
          const msgs = [...s.messages]
          const last = msgs[msgs.length - 1]
          if (last && last.id === aiMsgId) {
            msgs[msgs.length - 1] = { ...last, content: newContent }
          }
          return {
            messages: msgs,
            streaming_content: newContent,
            current_blueprint: bp ?? s.current_blueprint,
          }
        })
      },
      onEnd: (data) => {
        set((s) => {
          const msgs = [...s.messages]
          const last = msgs[msgs.length - 1]
          if (last && last.id === aiMsgId) {
            msgs[msgs.length - 1] = {
              ...last,
              content: s.streaming_content,
              token_usage: data.usage,
            }
          }
          return { messages: msgs, streaming: false }
        })
      },
      onError: (data) => {
        set((s) => {
          const msgs = [...s.messages]
          const last = msgs[msgs.length - 1]
          if (last && last.id === aiMsgId) {
            msgs[msgs.length - 1] = {
              ...last,
              content: last.content + `\n\nError: ${data.error}`,
            }
          }
          return { messages: msgs, streaming: false }
        })
      },
    }

    const controller = streamChat(
      `/api/rings/${ringId}/blueprint/chat`,
      {
        content,
        current_blueprint: state.current_blueprint,
      },
      callbacks,
    )
    set({ abort_controller: controller })
  },

  loadHistory: async (ringId) => {
    try {
      const res = await api.get<{ messages: BlueprintMessage[]; has_more: boolean }>(
        `/rings/${ringId}/blueprint/chat/history`,
      )
      const bpMsg = [...res.messages].reverse().find((m) => {
        const bp = extractBlueprint(m.content)
        return bp !== null
      })
      const bp = bpMsg ? extractBlueprint(bpMsg.content) : null
      set({ messages: res.messages, current_blueprint: bp })
    } catch {}
  },

  confirm: async (ringId) => {
    const state = get()
    try {
      await api.post(`/rings/${ringId}/blueprint/confirm`, {
        blueprint: state.current_blueprint
          ? { graphs: state.current_blueprint.graphs }
          : null,
      })
      set({ confirmed: true })
    } catch {}
  },

  checkStatus: async (ringId) => {
    try {
      const res = await api.get<{ status: string }>(`/rings/${ringId}/blueprint`)
      if (res.status === 'confirmed') {
        set({ confirmed: true })
      }
    } catch {}
  },

  stopStreaming: () => {
    const state = get()
    state.abort_controller?.abort()
    set({ streaming: false })
  },
}))

export { stripBlueprintTags, extractBlueprint }
export type { BlueprintGraph, BlueprintMessage }
```

- [ ] **Step 2: Verify frontend builds**

Run: `cd ui && npm run build 2>&1 | tail -5`

- [ ] **Step 3: Commit**

```bash
git add ui/src/stores/blueprint-store.ts
git commit -m "feat(blueprint): add blueprint-store for deep path state management"
```

---

### Task 6: Rewrite `BlueprintPanel.tsx` with deep path + D3 preview

**Files:**
- Modify: `ui/src/components/panels/BlueprintPanel.tsx`

This is the largest task. The panel needs to be restructured to support both quick and deep modes.

- [ ] **Step 1: Rewrite BlueprintPanel**

Replace the entire content of `ui/src/components/panels/BlueprintPanel.tsx` with:

```tsx
import { useEffect, useRef, useState } from 'react'
import { useRingStore } from '../../stores/ring-store'
import { useGraphStore } from '../../stores/graph-store'
import {
  useBlueprintStore,
  stripBlueprintTags,
  extractBlueprint,
  type BlueprintGraph,
} from '../../stores/blueprint-store'
import { api } from '../../services/api'
import * as d3 from 'd3'

interface BlueprintPreview {
  nodes: { label: string; node_type: string; tags: string[] }[]
  edges: { from: string; to: string; relation: string }[]
}

const TEMPLATES = [
  { id: 'product-research', name: '竞品分析', desc: '产品分析和竞品调研', icon: '🔍' },
  { id: 'project-management', name: '项目管理', desc: '项目规划和进度跟踪', icon: '📋' },
  { id: 'learning-notes', name: '学习笔记', desc: '知识学习和笔记整理', icon: '📖' },
  { id: 'technical-docs', name: '技术文档', desc: '技术方案设计和文档', icon: '⚙️' },
  { id: 'blank', name: '空白', desc: '从零开始构建图谱', icon: '📝' },
]

function MiniGraphPreview({ graphs }: { graphs: BlueprintGraph[] }) {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current || graphs.length === 0) return
    const container = ref.current
    const width = container.clientWidth || 280
    const height = 160

    d3.select(container).selectAll('*').remove()

    const svg = d3
      .select(container)
      .append('svg')
      .attr('width', width)
      .attr('height', height)

    const graph = graphs[0]
    const labels = graph.nodes.map((n) => n.label)
    const nodeMap = new Map(labels.map((l, i) => [l, i]))

    const nodes = graph.nodes.map((n, i) => ({
      id: i,
      label: n.label,
      node_type: n.node_type,
    }))

    const edges = graph.edges
      .filter((e) => nodeMap.has(e.from) && nodeMap.has(e.to))
      .map((e) => ({
        source: nodeMap.get(e.from)!,
        target: nodeMap.get(e.to)!,
        relation: e.relation,
      }))

    const colorMap: Record<string, string> = {
      category: '#22d3ee',
      topic: '#a78bfa',
      leaf: '#34d399',
    }

    const simulation = d3
      .forceSimulation(nodes as d3.SimulationNodeDatum[])
      .force('link', d3.forceLink(edges as d3.SimulationLinkDatum<d3.SimulationNodeDatum>[]).id((d: any) => d.id).distance(50))
      .force('charge', d3.forceManyBody().strength(-120))
      .force('center', d3.forceCenter(width / 2, height / 2))

    const link = svg
      .append('g')
      .selectAll('line')
      .data(edges)
      .join('line')
      .attr('stroke', '#475569')
      .attr('stroke-width', 1)
      .attr('stroke-opacity', 0.6)

    const node = svg
      .append('g')
      .selectAll('circle')
      .data(nodes)
      .join('circle')
      .attr('r', 6)
      .attr('fill', (d: any) => colorMap[d.node_type] || '#94a3b8')

    const label = svg
      .append('g')
      .selectAll('text')
      .data(nodes)
      .join('text')
      .text((d: any) => d.label)
      .attr('font-size', 7)
      .attr('fill', '#94a3b8')
      .attr('text-anchor', 'middle')
      .attr('dy', -10)

    simulation.on('tick', () => {
      link
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y)
      node.attr('cx', (d: any) => d.x).attr('cy', (d: any) => d.y)
      label.attr('x', (d: any) => d.x).attr('y', (d: any) => d.y)
    })
  }, [graphs])

  return <div ref={ref} style={{ width: '100%', height: 160 }} />
}

export function BlueprintPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const mode = useBlueprintStore((s) => s.mode)
  const setMode = useBlueprintStore((s) => s.setMode)
  const messages = useBlueprintStore((s) => s.messages)
  const streaming = useBlueprintStore((s) => s.streaming)
  const current_blueprint = useBlueprintStore((s) => s.current_blueprint)
  const confirmed = useBlueprintStore((s) => s.confirmed)
  const sendMessage = useBlueprintStore((s) => s.sendMessage)
  const confirm = useBlueprintStore((s) => s.confirm)
  const loadHistory = useBlueprintStore((s) => s.loadHistory)
  const checkStatus = useBlueprintStore((s) => s.checkStatus)
  const stopStreaming = useBlueprintStore((s) => s.stopStreaming)
  const fetchGraph = useGraphStore((s) => s.fetchGraph)

  const [input, setInput] = useState('')
  const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null)
  const [preview, setPreview] = useState<BlueprintPreview | null>(null)
  const [loading, setLoading] = useState(false)
  const chatEndRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (active_ring_id) {
      checkStatus(active_ring_id)
      if (mode === 'deep') loadHistory(active_ring_id)
    }
  }, [active_ring_id, mode])

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  if (!active_ring_id) return null

  if (confirmed) {
    return (
      <div style={{ padding: 20, textAlign: 'center' }}>
        <div style={{ fontSize: 48, marginBottom: 16 }}>&#10003;</div>
        <h2 style={{ color: 'var(--accent-green)', marginBottom: 8 }}>Blueprint Confirmed</h2>
        <p style={{ color: 'var(--text-secondary)', fontSize: 12 }}>
          Your ring blueprint has been set up. You can now start building your knowledge graph.
        </p>
      </div>
    )
  }

  const handleQuickConfirm = async () => {
    if (!active_ring_id || !preview) return
    setLoading(true)
    try {
      const blueprintData = preview.nodes.length > 0
        ? {
            graphs: [{
              name: 'Main',
              nodes: preview.nodes,
              edges: preview.edges,
            }],
          }
        : null
      await api.post(`/rings/${active_ring_id}/blueprint/confirm`, { blueprint: blueprintData })
      fetchGraph(active_ring_id)
      useBlueprintStore.setState({ confirmed: true })
    } catch {}
    setLoading(false)
  }

  const handleDeepConfirm = async () => {
    if (!active_ring_id) return
    setLoading(true)
    await confirm(active_ring_id)
    fetchGraph(active_ring_id)
    setLoading(false)
  }

  const handleSend = () => {
    const trimmed = input.trim()
    if (!trimmed || !active_ring_id) return
    sendMessage(active_ring_id, trimmed)
    setInput('')
  }

  return (
    <div style={{ padding: '12px 16px', height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ display: 'flex', gap: 8, marginBottom: 12 }}>
        <button
          onClick={() => setMode('quick')}
          style={{
            flex: 1, padding: '6px 8px', fontSize: 10, fontWeight: 700,
            background: mode === 'quick' ? 'var(--bg-hover)' : 'var(--bg-input)',
            border: `1px solid ${mode === 'quick' ? 'var(--accent-cyan)' : 'var(--border)'}`,
            borderRadius: 4, color: mode === 'quick' ? 'var(--accent-cyan)' : 'var(--text-dim)',
            cursor: 'pointer',
          }}
        >
          从模板选择
        </button>
        <button
          onClick={() => setMode('deep')}
          style={{
            flex: 1, padding: '6px 8px', fontSize: 10, fontWeight: 700,
            background: mode === 'deep' ? 'var(--bg-hover)' : 'var(--bg-input)',
            border: `1px solid ${mode === 'deep' ? 'var(--accent-cyan)' : 'var(--border)'}`,
            borderRadius: 4, color: mode === 'deep' ? 'var(--accent-cyan)' : 'var(--text-dim)',
            cursor: 'pointer',
          }}
        >
          AI 协作设计
        </button>
      </div>

      {mode === 'quick' && (
        <div style={{ flex: 1, overflow: 'auto' }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6, marginBottom: 12 }}>
            {TEMPLATES.map((t) => (
              <button
                key={t.id}
                onClick={async () => {
                  setSelectedTemplate(t.id)
                  try {
                    const res = await api.post<{ preview: BlueprintPreview }>(
                      `/rings/${active_ring_id}/blueprint/from-template`,
                      { template: t.id },
                    )
                    setPreview(res.preview)
                  } catch {}
                }}
                style={{
                  padding: '8px 10px',
                  background: selectedTemplate === t.id ? 'var(--bg-hover)' : 'var(--bg-input)',
                  border: `1px solid ${selectedTemplate === t.id ? 'var(--accent-cyan)' : 'var(--border)'}`,
                  borderRadius: 4, cursor: 'pointer', textAlign: 'left',
                }}
              >
                <div style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-primary)' }}>
                  {t.icon} {t.name}
                </div>
                <div style={{ fontSize: 9, color: 'var(--text-dim)', marginTop: 2 }}>{t.desc}</div>
              </button>
            ))}
          </div>

          {preview && (
            <div style={{ background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: 10, marginBottom: 10 }}>
              <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4 }}>预览: {preview.nodes.length} 节点 · {preview.edges.length} 边</div>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                {preview.nodes.map((n, i) => (
                  <span key={i} style={{ fontSize: 9, background: 'var(--bg-hover)', padding: '2px 5px', borderRadius: 2, color: 'var(--text-secondary)' }}>
                    {n.label}
                  </span>
                ))}
              </div>
            </div>
          )}

          {selectedTemplate && (
            <button
              onClick={handleQuickConfirm}
              disabled={loading}
              style={{
                width: '100%', padding: '8px', background: 'var(--accent-cyan)',
                color: 'var(--bg-base)', border: 'none', borderRadius: 4,
                fontSize: 11, fontWeight: 700, cursor: loading ? 'default' : 'pointer',
              }}
            >
              {loading ? '创建中...' : '确认并应用蓝图'}
            </button>
          )}
        </div>
      )}

      {mode === 'deep' && (
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
          <div style={{ flex: 1, overflowY: 'auto', marginBottom: 8, minHeight: 0 }}>
            {messages.length === 0 && (
              <div style={{ fontSize: 10, color: 'var(--text-dim)', padding: '20px 0', textAlign: 'center' }}>
                描述你的知识领域，AI 会帮你设计图谱结构
              </div>
            )}
            {messages.map((msg, i) => (
              <div
                key={msg.id || i}
                style={{
                  marginBottom: 6,
                  display: 'flex',
                  justifyContent: msg.role === 'user' ? 'flex-end' : 'flex-start',
                }}
              >
                <div
                  style={{
                    maxWidth: '85%',
                    padding: '6px 8px',
                    borderRadius: 4,
                    fontSize: 10,
                    lineHeight: 1.5,
                    background: msg.role === 'user' ? 'var(--bg-hover)' : 'var(--bg-input)',
                    color: 'var(--text-secondary)',
                    border: `1px solid ${msg.role === 'user' ? 'var(--accent-cyan)' : 'var(--border)'}`,
                  }}
                >
                  {msg.role === 'blueprint' ? stripBlueprintTags(msg.content) || '(图谱已更新)' : msg.content}
                </div>
              </div>
            ))}
            <div ref={chatEndRef} />
          </div>

          {current_blueprint && current_blueprint.graphs.length > 0 && (
            <div style={{ borderTop: '1px solid var(--border)', paddingTop: 6, marginBottom: 6 }}>
              <div style={{ fontSize: 9, color: 'var(--text-dim)', marginBottom: 4 }}>
                当前蓝图: {current_blueprint.graphs.length} 个图谱
              </div>
              <MiniGraphPreview graphs={current_blueprint.graphs} />
            </div>
          )}

          {current_blueprint && current_blueprint.graphs.length > 0 && (
            <button
              onClick={handleDeepConfirm}
              disabled={loading}
              style={{
                width: '100%', padding: '6px', marginBottom: 6,
                background: 'var(--accent-green)', color: 'var(--bg-base)',
                border: 'none', borderRadius: 4, fontSize: 10, fontWeight: 700,
                cursor: loading ? 'default' : 'pointer',
              }}
            >
              {loading ? '创建中...' : '确认蓝图'}
            </button>
          )}

          <div style={{ display: 'flex', gap: 6 }}>
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  handleSend()
                }
              }}
              placeholder="描述你的知识领域..."
              style={{
                flex: 1, background: 'var(--bg-input)', border: '1px solid var(--border)',
                borderRadius: 4, padding: '6px 8px', color: 'var(--text-primary)',
                fontSize: 10, resize: 'none', fontFamily: 'inherit', outline: 'none',
                minHeight: 28, maxHeight: 60,
              }}
            />
            {streaming ? (
              <button
                onClick={stopStreaming}
                style={{
                  padding: '4px 8px', background: 'var(--accent-amber)', border: 'none',
                  borderRadius: 4, fontSize: 9, fontWeight: 700, cursor: 'pointer',
                  color: 'var(--bg-base)',
                }}
              >
                STOP
              </button>
            ) : (
              <button
                onClick={handleSend}
                disabled={!input.trim()}
                style={{
                  padding: '4px 8px', background: 'var(--accent-cyan)', border: 'none',
                  borderRadius: 4, fontSize: 9, fontWeight: 700, cursor: 'pointer',
                  color: 'var(--bg-base)', opacity: input.trim() ? 1 : 0.5,
                }}
              >
                SEND
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Verify frontend builds**

Run: `cd ui && npm run build 2>&1 | tail -5`

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/panels/BlueprintPanel.tsx
git commit -m "feat(blueprint): add deep path with AI chat, D3 preview, quick path edge fix"
```

---

### Task 7: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add blueprint chat integration test**

Read the existing test file to understand the test infrastructure pattern. Add:

```rust
#[sqlx::test(migrations = "./migrations")]
async fn test_blueprint_chat_requires_creator(pool: SqlitePool) {
    let (state, _) = setup_state(pool).await;
    let creator = create_test_user(&state).await;
    let ring_id = create_test_ring(&state, &creator.token_id).await;

    let member_token = format!("member_{}", ulid::Ulid::new());
    sqlx::query("INSERT INTO users (token_id, display_name) VALUES (?1, ?2)")
        .bind(&member_token)
        .bind("Member")
        .execute(&state.db)
        .await
        .unwrap();
    sqlx::query("INSERT INTO ring_members (ring_id, user_id, role) VALUES (?1, ?2, 'member')")
        .bind(&ring_id)
        .bind(&member_token)
        .execute(&state.db)
        .await
        .unwrap();

    let body = serde_json::json!({"content": "hello", "current_blueprint": null});
    let req = TestRequest::post()
        .uri(&format!("/api/rings/{ring_id}/blueprint/chat"))
        .json(&body)
        .auth(&member_token);
    let resp = req.send_with_state(state.clone()).await;
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_blueprint_confirm_with_graphs(pool: SqlitePool) {
    let (state, _) = setup_state(pool).await;
    let user = create_test_user(&state).await;
    let ring_id = create_test_ring(&state, &user.token_id).await;

    let body = serde_json::json!({
        "blueprint": {
            "graphs": [{
                "name": "Test Graph",
                "nodes": [
                    {"label": "Root", "node_type": "category", "tags": []},
                    {"label": "Child", "node_type": "topic", "tags": ["test"]}
                ],
                "edges": [
                    {"from": "Root", "to": "Child", "relation": "related_to"}
                ]
            }]
        }
    });

    let req = TestRequest::post()
        .uri(&format!("/api/rings/{ring_id}/blueprint/confirm"))
        .json(&body)
        .auth(&user.token_id);
    let resp = req.send_with_state(state.clone()).await;
    assert_eq!(resp.status(), 200);

    let graphs: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, name FROM graphs WHERE ring_id = ?1",
    )
    .bind(&ring_id)
    .fetch_all(&state.db)
    .await
    .unwrap();
    assert_eq!(graphs.len(), 1);
    assert_eq!(graphs[0].1, "Test Graph");

    let nodes: Vec<(String, String)> = sqlx::query_as(
        "SELECT label, node_type FROM graph_nodes WHERE ring_id = ?1",
    )
    .bind(&ring_id)
    .fetch_all(&state.db)
    .await
    .unwrap();
    assert_eq!(nodes.len(), 2);

    let edges: Vec<(String, String)> = sqlx::query_as(
        "SELECT source_id, target_id FROM graph_edges WHERE ring_id = ?1",
    )
    .bind(&ring_id)
    .fetch_all(&state.db)
    .await
    .unwrap();
    assert_eq!(edges.len(), 1);
}
```

Note: Adjust test helper function names (`create_test_ring`, `setup_state`, `create_test_user`, `TestRequest`) to match the existing patterns in the test file.

- [ ] **Step 2: Run all tests**

Run: `cd server && cargo test --test integration 2>&1 | tail -10`

- [ ] **Step 3: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test(blueprint): add integration tests for blueprint chat and confirm with graphs"
```

---

### Task 8: Final verification + STATUS.md update

**Files:**
- Modify: `docs/STATUS.md`

- [ ] **Step 1: Run full test suite**

Run: `cd server && cargo test --test integration 2>&1 | tail -10`
Expected: all tests pass

Run: `cd ui && npm run build 2>&1 | tail -5`
Expected: build succeeds

- [ ] **Step 2: Update STATUS.md**

In `docs/STATUS.md`:
- Update test count
- Change the PRD missing item row:

```
| 深度蓝图构建器（AI 对话式） | done（AI 引导多轮对话 + D3 实时预览 + 多图谱 + 边创建修复） | 中 |
```

Add to the "本轮完成" section:

```
- **深度蓝图构建器** — AI 引导的多轮对话式蓝图设计，`<blueprint>` JSON 结构化输出，D3.js 实时预览，多图谱支持，滑动窗口 + current_blueprint 注入上下文管理，快速路径边创建 bug 修复
```

- [ ] **Step 3: Commit**

```bash
git add docs/STATUS.md
git commit -m "docs: update STATUS.md for Deep Blueprint Builder completion"
```
