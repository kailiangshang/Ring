# Deep Blueprint Builder Design

> Date: 2026-04-25
> Status: Approved
> Priority: Medium

## Goal

Implement the PRD's "深度路径" for blueprint construction: an AI-guided conversational experience where Group Ring collaborates with the user to design their knowledge graph structure through unlimited dialogue rounds, with live D3.js preview and multi-graph support. Also fix the edge creation bug in the existing quick path.

## Background

The current blueprint system only supports the "快速路径" (quick path): user selects a preset template → preview → confirm. The PRD requires a "深度路径" (deep path) where AI and user co-create graph structures through dialogue.

**Existing bug:** The quick path confirm handler only creates nodes, not edges.

## Architecture

**Approach A: Structured Output.** AI outputs `<blueprint>` JSON blocks in chat messages. Frontend parses them and renders live D3 preview. Reuses existing SSE chat infrastructure.

## Entry Point

`BlueprintPanel` shows two modes:
- **快速路径**: "从模板选择" (existing template selection)
- **深度路径**: "AI 协作设计" (new AI chat mode)

Deep path shows a split layout: chat area + D3 preview.

## Backend

### 1. Blueprint Prompt (`prompts.rs`)

New `blueprint` module:

```
system(ring_name, role_description):
  "你是 {ring_name} 的 Group Ring，正在帮用户设计知识图谱蓝图。

  你需要通过对话了解：
  1. 这个 Ring 的核心知识领域
  2. 需要几个图谱（最多 3 个）
  3. 每个图谱的主题和顶层分类节点
  4. 节点之间的关系

  每次你提出或调整图谱结构时，必须输出一个 <blueprint> JSON 块：

  <blueprint>
  {\"graphs\": [{\"name\": \"图谱名\", \"nodes\": [{\"label\": \"节点名\", \"node_type\": \"category\", \"tags\": []}], \"edges\": [{\"from\": \"节点名\", \"to\": \"节点名\", \"relation\": \"related_to\"}]}]}
  </blueprint>

  规则：
  - 从了解需求开始，不要一上来就生成图谱
  - 每次调整都输出完整的 blueprint JSON（不是增量）
  - node_type: category / topic / leaf
  - relation: depends_on / related_to / derives_from / contradicts
  - 最多 3 个图谱
  - 简洁对话"
```

If `role_description` is provided, append it as additional context about the Ring's purpose.

### 2. Blueprint Chat Route

```
POST /rings/{ring_id}/blueprint/chat
Body: { content: string }
Response: SSE stream (same format as ring_chat)
```

Handler:
- Auth: creator/admin only
- Check `blueprint_status != "confirmed"` (can't modify confirmed blueprint)
- Build system prompt using `prompts::blueprint::system(ring_name, role_description)`
- Load blueprint chat history (messages with role="blueprint") as context
- Stream via LLM, save messages to `messages` table with `role="blueprint"`
- Return SSE events

### 3. Blueprint History Route

```
GET /rings/{ring_id}/blueprint/chat/history
Query: ?before=ulid&limit=50
Response: { messages: [...], has_more: bool }
```

Loads messages with `role="blueprint"` for this ring.

### 4. Updated Confirm Route

```
POST /rings/{ring_id}/blueprint/confirm
Body (optional): { blueprint: { graphs: [...] } }
Response: { ok: true }
```

If `blueprint` body is provided:
1. For each graph: create a `GraphRow` via graph model
2. For each node: create a `GraphNodeRow` (resolve label to generated ULID)
3. For each edge: resolve `from`/`to` labels to node IDs, create `GraphEdgeRow`
4. Set `blueprint_status = "confirmed"`

If no body (backward compat with quick path): set `blueprint_status = "confirmed"` only.

### 5. Quick Path Fix

Update `preview_template` to return a `BlueprintPreview` that includes edges. Frontend passes the full preview (nodes + edges) to confirm endpoint.

## Blueprint JSON Format

```json
{
  "graphs": [
    {
      "name": "竞品分析",
      "nodes": [
        { "label": "竞品概览", "node_type": "category", "tags": [] },
        { "label": "功能对比", "node_type": "topic", "tags": ["核心"] }
      ],
      "edges": [
        { "from": "竞品概览", "to": "功能对比", "relation": "related_to" }
      ]
    }
  ]
}
```

Nodes are identified by label within a graph. The confirm handler maps labels to ULIDs.

## Frontend

### 1. Blueprint Store (`ui/src/stores/blueprint-store.ts`)

New Zustand store:
- `mode`: "quick" | "deep"
- `messages`: blueprint chat messages
- `streaming`: boolean
- `current_blueprint`: parsed blueprint JSON (from last `<blueprint>` block)
- `confirmed`: boolean
- `send_message(content)`: POST to blueprint chat, parse SSE stream
- `load_history()`: load blueprint chat history
- `confirm()`: POST confirm with current_blueprint

### 2. BlueprintPanel Refactor

```
BlueprintPanel
├── Mode selector: [从模板选择] [AI 协作设计]
├── QuickPath (existing template cards + preview)
└── DeepPath
    ├── Chat area (messages + input, reuses message rendering patterns)
    ├── D3 preview (mini force graph of current_blueprint)
    └── [确认蓝图] button
```

### 3. Blueprint Message Parsing

When a blueprint chat message contains `<blueprint>...</blueprint>`:
1. Extract JSON between tags
2. Parse into `current_blueprint` state
3. Strip the `<blueprint>` block from displayed message text
4. Show a small "图谱已更新" indicator in the message

### 4. D3 Preview

Mini force-directed graph showing:
- Nodes as labeled circles (colored by node_type)
- Edges as links with relation labels
- Auto-layout, no user interaction needed
- Reuses D3 patterns from GraphPanel but simplified (no zoom controls, no drag)

## Session Recovery

When user returns to a Ring with `blueprint_status = "pending"` and blueprint chat messages exist:
- Load history from `GET /rings/{ring_id}/blueprint/chat/history`
- Re-parse last `<blueprint>` block from messages
- Resume the conversation

## Testing

- Unit: blueprint prompt generates valid system prompt
- Integration: `POST /rings/{ring_id}/blueprint/chat` streams and saves messages
- Integration: `POST /rings/{ring_id}/blueprint/confirm` creates graphs, nodes, and edges
- Integration: edge creation from quick path template also works
- Frontend: `<blueprint>` JSON extraction from message text

## Files to Create/Modify

| Action | File | Purpose |
|--------|------|---------|
| Modify | `server/src/prompts.rs` | Add `blueprint` module |
| Modify | `server/src/routes/blueprint.rs` | Add `blueprint_chat`, `blueprint_history`, update `confirm` |
| Modify | `server/src/routes/mod.rs` | Register new routes |
| Modify | `server/src/services/blueprint_service.rs` | Add `confirm_with_blueprint()` |
| Create | `ui/src/stores/blueprint-store.ts` | Blueprint state management |
| Modify | `ui/src/components/panels/BlueprintPanel.tsx` | Add deep path mode + D3 preview |
| Modify | `server/tests/integration.rs` | New tests |
