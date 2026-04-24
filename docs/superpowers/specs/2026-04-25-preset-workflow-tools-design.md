# Preset Workflow Tools Design

> Date: 2026-04-25
> Status: Approved
> Priority: Low

## Goal

Add two preset workflow tools (file_parse, knowledge_extract) to Group Ring via tool_calls. The LLM decides when to invoke them during conversation. Tools run multi-step pipelines on the backend and return structured node proposals for user confirmation.

## Scope

**Included:** file_parse, knowledge_extract, Group Ring tool_calls infrastructure, frontend node proposal rendering.
**Deferred:** deep_research (requires web scraping, separate feature).

## Architecture

**Approach A: Extend Group Ring with tool_calls.** Consistent with Super Ring's existing tool_calls pattern. LLM decides when to call tools during conversation. Backend handles tool execution invisibly and returns results embedded in AI response as structured XML blocks.

## Group Ring Tool Calls

### Tool Registration

New function `get_group_ring_tools()` returns tool definitions:

```json
[
  {
    "type": "function",
    "function": {
      "name": "file_parse",
      "description": "Parse an uploaded file and extract structured knowledge. Recommend graph nodes.",
      "parameters": {
        "type": "object",
        "properties": {
          "file_reference": { "type": "string", "description": "The message_id of the file upload message" },
          "focus": { "type": "string", "description": "Optional focus area" }
        },
        "required": ["file_reference"]
      }
    }
  },
  {
    "type": "function",
    "function": {
      "name": "knowledge_extract",
      "description": "Extract knowledge concepts from text and generate graph node recommendations.",
      "parameters": {
        "type": "object",
        "properties": {
          "content": { "type": "string", "description": "Text or topic to extract from" },
          "target_graph": { "type": "string", "description": "Optional target graph name" }
        },
        "required": ["content"]
      }
    }
  }
]
```

### Execution Flow

1. User sends message in Group Ring chat
2. LLM responds with `tool_calls` (finish_reason: "tool_calls")
3. Backend detects tool_call in LLM response
4. Backend executes the tool pipeline synchronously
5. Backend sends tool result back to LLM as tool message
6. LLM generates final response with structured output
7. Final response streams to frontend as normal SSE

The tool execution happens entirely on the backend. Frontend only sees the final AI response containing structured `<file_analysis>` or `<knowledge_extraction>` blocks.

### Backend Changes

**`server/src/services/llm.rs`:**
- Add `tool_calls` support to `chat_stream` — when LLM returns tool_calls, emit them via a new `SseEvent::ToolCall` variant
- New `chat_with_tools()` function that handles the tool_calls loop: call LLM → detect tool_calls → execute tool → re-call LLM with tool result → stream final response

**`server/src/services/chat.rs`:**
- New `get_group_ring_tools()` function
- New `execute_tool_call()` dispatcher that routes to file_parse or knowledge_extract

**`server/src/services/workflow.rs` (new):**
- `execute_file_parse(state, user, ring_id, file_reference, focus)` — retrieves file text from message, runs LLM extraction, returns structured result
- `execute_knowledge_extract(state, user, ring_id, content, target_graph)` — runs LLM concept extraction, returns structured result

**`server/src/routes/chat.rs`:**
- Update `ring_chat` handler to pass tools to `chat_with_tools()` when ring_id is present

### Prompts

New `workflow` module in `prompts.rs`:

**`file_parse_extraction`:**
```
分析以下文件内容，提取结构化知识。

输出格式：
<file_analysis>
{"summary": "文件摘要", "concepts": [{"label": "概念名", "node_type": "category|topic|leaf", "tags": []}], "relations": [{"from": "概念A", "to": "概念B", "relation": "related_to"}]}
</file_analysis>

规则：
- 提取 3-10 个核心概念作为建议的图谱节点
- node_type: category（顶层分类）/ topic（具体主题）/ leaf（细节）
- relation: depends_on / related_to / derives_from / contradicts
- 每个概念有意义的标签
- 简洁摘要，不超过 3 句
```

**`knowledge_extract_prompt`:**
```
从以下内容中提取知识概念和关系。

输出格式：
<knowledge_extraction>
{"concepts": [{"label": "概念名", "node_type": "category|topic|leaf", "tags": []}], "relations": [{"from": "概念A", "to": "概念B", "relation": "related_to"}], "suggested_graph": "图谱名"}
</knowledge_extraction>

规则：
- 识别核心实体、概念和它们之间的关系
- 生成适合图谱结构的节点和边
- 建议节点类型和标签
- 关系仅限: depends_on / related_to / derives_from / contradicts
```

## Tool: file_parse

### Pipeline

1. Look up `file_reference` message_id in messages table
2. Retrieve the file content (stored as message content for upload messages)
3. Call LLM with `file_parse_extraction` prompt + file content
4. Parse the `<file_analysis>` response
5. Return structured result to the tool_calls loop

### Data

Input: `{ file_reference: "msg_id", focus?: "optional" }`
Output: `{ summary: string, concepts: [...], relations: [...] }`

## Tool: knowledge_extract

### Pipeline

1. Receive content text and optional target_graph
2. Call LLM with `knowledge_extract_prompt` + content
3. Parse the `<knowledge_extraction>` response
4. Return structured result to the tool_calls loop

### Data

Input: `{ content: "text or topic", target_graph?: "name" }`
Output: `{ concepts: [...], relations: [...], suggested_graph: string }`

## Frontend

### Message Rendering

When AI message contains `<file_analysis>` or `<knowledge_extraction>` blocks:
1. Extract JSON from XML tags
2. Strip XML from displayed text
3. Render structured card:
   - Summary text
   - Proposed nodes as colored tags (by node_type)
   - Proposed edges as "A → B (relation)" list
   - "添加到图谱" confirm button
4. On confirm: POST extracted nodes/edges to graph API

### Graph Node Creation

New function in `graph-store.ts`:
- `createNodesFromExtraction(ringId, concepts, relations)` — creates nodes, resolves labels to IDs, creates edges

## Files

| Action | File | Purpose |
|--------|------|---------|
| Modify | `server/src/services/llm.rs` | Add tool_calls handling to chat stream |
| Modify | `server/src/services/chat.rs` | Add `get_group_ring_tools()`, `execute_tool_call()` |
| Create | `server/src/services/workflow.rs` | file_parse and knowledge_extract pipelines |
| Modify | `server/src/prompts.rs` | Add `workflow` module |
| Modify | `server/src/routes/chat.rs` | Pass tools to ring_chat |
| Modify | `ui/src/components/chat/MessageItem.tsx` | Render file_analysis / knowledge_extraction cards |
| Modify | `ui/src/stores/graph-store.ts` | Add `createNodesFromExtraction()` |
| Modify | `server/tests/integration.rs` | Workflow tests |

## Testing

- Unit: `get_group_ring_tools()` returns valid tool definitions
- Unit: workflow prompts generate valid system prompts
- Integration: `execute_file_parse` returns structured extraction
- Integration: `execute_knowledge_extract` returns concepts and relations
