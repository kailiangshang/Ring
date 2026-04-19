# Super Ring Tool Framework + 跨 Ring 查询设计

> **Affects**: `server/src/services/super_chat.rs`, `server/src/services/llm.rs`, `server/src/routes/super_chat.rs`, `ui/src/stores/chat-store.ts`
> **Depends on**: Super Ring 基础聊天（已完成）、`chat_complete`（已完成）、`rings`/`members`/`archive_records` 表、`~/.ring/rings/` 文件系统
> **Last verified**: 2026-04-19

## 1. 概述

建立 Super Ring 的 Tool 框架。Super Ring = LLM + Tools。LLM 通过原生 function calling 决定何时调用 tool，后端执行 tool 并将结果喂回 LLM 继续回复。

MVP 包含 2 个查询 tool：
- `query_rings` — 列出用户所有 Ring 的摘要（名称、成员数、归档数）
- `query_ring_detail` — 读取指定 Ring 的图谱节点 + 最近归档内容（截断）

### 1.1 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| Tool 机制 | async-openai 原生 function calling | 业界标准，稳定可靠 |
| 流式策略 | 混合：非流式检测 tool call → 执行 → 流式回复最终答案 | 兼顾 tool calling 和流式体验 |
| Tool 定义 | 代码中硬编码（Vec<ChatCompletionTool>） | MVP 阶段，后续可从文件加载 |
| Tool 执行 | 同步在 handler 中执行 | 简单可靠 |
| 前端体验 | 等待时显示"分析中..."，tool 执行完毕后流式输出 | |

## 2. 流程

```
POST /api/super/chat
  ├── 1. insert user message
  ├── 2. get_system_prompt() + build_ring_summary() → system prompt
  ├── 3. load_history_context(limit=20)
  ├── 4. 第一轮：chat_complete（非流式，带 tools 定义）
  │     ├── LLM 返回普通文本 → 直接作为完整回复
  │     │     → 插入 assistant message → 返回 SSE（单条 message_end）
  │     └── LLM 返回 tool_calls → 执行 tool → 将结果喂回
  │           → 第二轮：chat_stream（流式，不带 tools）
  │           → 插入 assistant message → 返回 SSE 流
  └── 5. 前端展示
```

### 2.1 第一轮：非流式 + Tools

使用 `chat_complete` 的变体，支持传入 `tools` 参数。LLM 可能：
- 直接回复文本（不需要 tool）
- 返回 `tool_calls`（需要调用 tool）

### 2.2 Tool 执行

后端收到 `tool_calls` 后：
1. 遍历每个 tool call
2. 根据 `function.name` 路由到对应执行函数
3. 收集所有 tool 结果
4. 将结果作为 `tool` role 消息追加到消息历史

### 2.3 第二轮：流式回复

将包含 tool 结果的完整消息历史发给 `chat_stream`，流式返回最终答案。

## 3. Tool 定义

### 3.1 query_rings

列出用户所有 Ring 的摘要信息。

**参数：** 无

**返回格式：**
```
## 用户的所有 Ring

### 产品讨论组
- 成员: 5 人
- 归档: 12 篇
- 最近归档: 技术选型决策, Q2 规划, 新成员指南

### 设计团队
- 成员: 3 人
- 归档: 5 篇
- 最近归档: 新版 UI 设计稿
```

**实现：**
```sql
SELECT r.id, r.name,
       (SELECT COUNT(*) FROM members m WHERE m.ring_id = r.id) as member_count
FROM rings r
JOIN members mem ON mem.ring_id = r.id AND mem.user_id = ?1
ORDER BY r.created_at;

SELECT title FROM archive_records
WHERE ring_id = ?1 AND status IN ('pushed', 'committed')
ORDER BY created_at DESC LIMIT 3;
```

### 3.2 query_ring_detail

读取指定 Ring 的图谱节点和最近归档内容。

**参数：**
```json
{
  "ring_name": "产品讨论组"
}
```

**截断限制：**
- 图谱节点：最多 50 个，只取 label + description
- 归档文件：最近 3 篇，每篇前 500 字

**返回格式：**
```
## Ring: 产品讨论组

### 图谱节点（共 23 个，显示前 20 个）
- Rust: 后端开发语言
- Axum: Web 框架
- React: 前端框架
...

### 最近归档

#### 技术选型决策
我们决定用 Rust + Axum 作为后端框架...

#### Q2 规划
Q2 重点是完成 Session 功能...
```

**实现：**
1. 根据 `ring_name` 模糊匹配 `rings` 表获取 `ring_id`
2. 读取 `~/.ring/rings/<ring-id>/graph.json`，解析取前 50 节点
3. 读取 `~/.ring/rings/<ring-id>/archives/` 最近 3 个 .md 文件，截取前 500 字

## 4. 后端改动

### 4.1 LlmClient 新增方法

在 `services/llm.rs` 新增 `chat_complete_with_tools`：

```rust
pub async fn chat_complete_with_tools(
    self,
    system_prompt: String,
    history: Vec<(String, String)>,
    user_message: String,
    tools: Vec<async_openai::types::ChatCompletionTool>,
) -> Result<ChatCompleteWithToolsResult>

pub enum ChatCompleteWithToolsResult {
    Message { content: String },
    ToolCalls { tool_calls: Vec<async_openai::types::ChatCompletionMessageToolCall> },
}
```

### 4.2 Tool 执行框架

在 `services/super_chat.rs` 新增：

```rust
pub fn get_super_tools() -> Vec<ChatCompletionTool>
// 返回 query_rings + query_ring_detail 的 tool 定义

pub async fn execute_tool(
    pool: &SqlitePool,
    rings_dir: &Path,
    user_id: &str,
    tool_name: &str,
    arguments: &str,
) -> Result<String>
// 根据 tool_name 路由到 query_rings 或 query_ring_detail
```

### 4.3 修改 start_super_chat

改为混合模式：
1. 构建 system prompt（含 ring 摘要）
2. 加载历史
3. 调用 `chat_complete_with_tools`
4. 如果返回 `Message` → 直接作为 SSE 单条消息返回
5. 如果返回 `ToolCalls` → 执行 tools → 构建新历史 → 调用 `chat_stream`

### 4.4 SSE 协议扩展

新增 SSE 事件类型：

| 事件 | 数据 | 说明 |
|------|------|------|
| `tool_call` | `{"name": "query_rings", "args": {}}` | 前端展示"正在查询 Ring 数据..." |
| `tool_result` | `{"name": "query_rings", "result": "..."}` | 前端展示查询完成 |

流程中 SSE 事件序列：

**不需要 tool 时：**
```
message_start → delta×N → message_end
```

**需要 tool 时：**
```
message_start → delta("让我查看一下...") → tool_call → tool_result → delta×N → message_end
```

或者更简单：tool 阶段不在 SSE 中体现，前端只看到一段等待后流式输出。

### 4.5 简化方案

MVP 阶段，tool 执行阶段不在 SSE 中体现。流程：

1. 前端发送 POST `/api/super/chat`
2. 后端执行第一轮（非流式 + tools），可能执行 tool
3. 后端执行第二轮（流式），返回 SSE
4. 前端体验：等几秒后开始流式输出

这样 SSE 协议不变，只是等待时间稍长（多了 tool 执行时间）。

## 5. 前端改动

无。SSE 协议不变（`message_start` → `delta` → `message_end`），前端无感知。

## 6. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/src/services/llm.rs` | 修改 | 新增 `chat_complete_with_tools` + `ChatCompleteWithToolsResult` |
| `server/src/services/super_chat.rs` | 修改 | 新增 `get_super_tools` + `execute_tool` + `build_ring_summary`，修改 `start_super_chat` 为混合模式 |
| `server/src/routes/super_chat.rs` | 修改 | handler 处理两种返回（直接消息 / tool call 后流式） |

无新端点，无前端改动，无 migration。

## 7. 错误处理

| 场景 | 处理 |
|------|------|
| 第一轮 LLM 调用失败 | 返回 SSE error |
| tool 执行失败 | 将错误信息作为 tool result 喂回 LLM，让 LLM 告知用户 |
| graph.json 不存在 | 返回"该 Ring 暂无图谱数据" |
| ring_name 匹配不到 | 返回"未找到该 Ring" |
| 第二轮 LLM 调用失败 | 返回 SSE error |

## 8. 后续可添加的 Tools

| Tool | 功能 |
|------|------|
| `create_ring` | 创建新 Ring |
| `manage_skills` | 安装/卸载 Skill |
| `query_user_preferences` | 读取用户偏好 |
| `update_user_preferences` | 更新用户偏好 |
