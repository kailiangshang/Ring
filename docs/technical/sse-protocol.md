# Ring SSE 流式协议设计

## 1. 概述

LLM 响应通过 SSE（Server-Sent Events）流式返回前端。适用于 Group Ring 对话、Super Ring 对话、蓝图构建对话和 Session 消息。

### 端点

| 场景 | 端点 | 方法 |
|------|------|------|
| Group Ring 对话 | `/api/v1/rings/{ringId}/conversations/{convId}/messages` | POST |
| 蓝图构建对话 | `/api/v1/rings/{ringId}/blueprint/chat` | POST |
| Super Ring 对话 | `/api/v1/super-ring/chat` | POST |
| Super Ring 分析 | `/api/v1/super-ring/analyze` | POST |
| Super Ring 总结 | `/api/v1/super-ring/summarize` | POST |
| Session 消息 | `/api/v1/rings/{ringId}/sessions/{sessionId}/messages` | POST |

所有端点返回 `Content-Type: text/event-stream`。

---

## 2. SSE Event 格式

每个 SSE event 格式：

```
event: {event_type}
data: {json_payload}

```

`event` 字段区分消息类型，`data` 字段是 JSON。

---

## 3. Event 类型定义

### 3.1 text — AI 文本内容

AI 生成的文本片段（流式分块）。

```json
{
  "type": "text",
  "content": "我来帮你分析"
}
```

前端收到后追加到当前消息气泡。多个 text event 拼接为完整回复。

---

### 3.2 tool_call — 工具调用开始

AI 决定调用一个工具。

```json
{
  "type": "tool_call",
  "tool_call_id": "call_abc123",
  "tool": "file_parser",
  "input": {
    "file_path": "nodes/competitor-a.md"
  }
}
```

前端展示"正在调用 file_parser..."加载状态。

---

### 3.3 tool_result — 工具调用结果

工具执行完成，返回结果。

```json
{
  "type": "tool_result",
  "tool_call_id": "call_abc123",
  "tool": "file_parser",
  "output": {
    "parsed_content": "...",
    "metadata": { "pages": 5 }
  }
}
```

前端展示工具结果摘要。

---

### 3.4 archive_suggestion — 归档推荐

AI 推荐将内容归档到图谱。

```json
{
  "type": "archive_suggestion",
  "data": {
    "summary": "竞品 A 的功能分析",
    "suggested_operations": [
      {
        "operation": "create_node",
        "graph_id": "graph-uuid-1",
        "parent_id": "node-uuid-parent",
        "label": "竞品 A 功能分析",
        "reason": "内容涉及竞品 A 的核心功能对比"
      }
    ],
    "message_ids": ["msg-uuid-1", "msg-uuid-2"]
  }
}
```

前端弹出归档推荐卡片，用户可确认或拒绝。

---

### 3.5 blueprint_proposal — 蓝图方案

蓝图构建阶段，AI 提出图谱结构方案。

```json
{
  "type": "blueprint_proposal",
  "data": {
    "graphs": [
      {
        "name": "知识图谱",
        "type": "knowledge",
        "categories": ["概念", "方法", "工具"]
      },
      {
        "name": "竞品图谱",
        "type": "competitor",
        "categories": ["竞品 A", "竞品 B"]
      }
    ]
  }
}
```

前端渲染蓝图预览（D3.js）。

---

### 3.6 session_message — Session 消息（广播）

Session 内成员发送的消息，通过 WebSocket 广播给所有参与者。

```json
{
  "type": "session_message",
  "sender_id": "user-uuid-1",
  "sender_name": "张三",
  "content": "大家觉得竞品 A 的核心优势是什么？",
  "seq_num": 5
}
```

---

### 3.7 session_ring_response — Session AI 回复（广播）

Session Ring 的回复，广播给所有参与者。

```json
{
  "type": "session_ring_response",
  "content": "根据已有的分析数据...",
  "seq_num": 6
}
```

---

### 3.8 done — 流式响应结束

标志 SSE 流结束。

```json
{
  "type": "done",
  "message_id": "msg-uuid-new",
  "token_usage": {
    "prompt_tokens": 1500,
    "completion_tokens": 800,
    "total_tokens": 2300
  }
}
```

前端收到后关闭 EventSource，更新 token 统计。

---

### 3.9 error — 错误

流式过程中发生错误。

```json
{
  "type": "error",
  "code": "llm_timeout",
  "message": "LLM 响应超时，请重试"
}
```

前端展示错误提示，关闭 EventSource。

---

## 4. 完整流式响应示例

### 4.1 普通对话

```
event: message
data: {"type": "text", "content": "我来帮你"}

event: message
data: {"type": "text", "content": "分析这份报告。"}

event: message
data: {"type": "archive_suggestion", "data": {"summary": "...", "suggested_operations": [...]}}

event: message
data: {"type": "done", "message_id": "msg-uuid", "token_usage": {"prompt_tokens": 1500, "completion_tokens": 200, "total_tokens": 1700}}
```

### 4.2 工具调用

```
event: message
data: {"type": "text", "content": "让我先解析这个文件。"}

event: message
data: {"type": "tool_call", "tool_call_id": "call_1", "tool": "file_parser", "input": {"file_path": "report.pdf"}}

event: message
data: {"type": "tool_result", "tool_call_id": "call_1", "tool": "file_parser", "output": {"parsed_content": "..."}}

event: message
data: {"type": "text", "content": "解析完成。这份报告主要包含以下要点..."}

event: message
data: {"type": "done", "message_id": "msg-uuid", "token_usage": {"prompt_tokens": 3000, "completion_tokens": 500, "total_tokens": 3500}}
```

### 4.3 蓝图构建

```
event: message
data: {"type": "text", "content": "好的，我们来设计图谱结构。这个 Ring 主要用来做什么？"}

（用户回复后）

event: message
data: {"type": "text", "content": "明白了。我建议创建以下图谱维度："}

event: message
data: {"type": "blueprint_proposal", "data": {"graphs": [{"name": "知识图谱", "type": "knowledge", "categories": ["概念", "方法"]}]}}

event: message
data: {"type": "text", "content": "你觉得这个结构合适吗？需要调整哪里？"}

event: message
data: {"type": "done", "message_id": "msg-uuid", "token_usage": {"prompt_tokens": 2000, "completion_tokens": 300, "total_tokens": 2300}}
```

---

## 5. 前端 EventSource 使用

```typescript
const eventSource = new EventSource(`/api/v1/rings/${ringId}/conversations/${convId}/messages`, {
  // POST 请求不支持 EventSource，使用 fetch + ReadableStream 替代
});

// 实际实现：fetch + SSE parser
const response = await fetch(`/api/v1/rings/${ringId}/conversations/${convId}/messages`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ content: userMessage }),
});

const reader = response.body!.getReader();
const decoder = new TextDecoder();

while (true) {
  const { done, value } = await reader.read();
  if (done) break;
  const chunk = decoder.decode(value);
  // 解析 SSE events，按 type 分发处理
  parseSSEEvents(chunk).forEach(event => {
    switch (event.type) {
      case 'text': appendToCurrentMessage(event.content); break;
      case 'tool_call': showToolCallStatus(event); break;
      case 'tool_result': showToolResult(event); break;
      case 'archive_suggestion': showArchiveCard(event.data); break;
      case 'done': finalizeMessage(event); break;
      case 'error': showError(event); break;
    }
  });
}
```

> **注意**：SSE 标准的 `EventSource` API 只支持 GET 请求。Ring 的对话端点是 POST，前端需要用 `fetch` + `ReadableStream` 手动解析 SSE 流。
