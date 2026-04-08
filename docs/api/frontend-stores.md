# 前端 Store API 参考

> 源码路径：`ring-frontend/src/stores/`

## setupStore

> 源文件：`stores/setupStore.ts`

初始化安装流程状态管理，分 3 步：填用户名 → 配 LLM → 配 GitLab → 完成。

### State

```typescript
interface SetupState {
  step: number               // 当前步骤 0/1/2
  error: string | null       // 错误信息
  loading: boolean           // 是否请求中
  user_id: string | null     // 创建的用户 ID
  redirect_home: boolean     // 是否跳转首页
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `set_step` | `(step) => void` | 切换步骤 |
| `submit_username` | `(display_name) => Promise<void>` | 提交用户名，创建用户 |
| `submit_llm` | `(config: LlmConfig) => Promise<void>` | 提交 LLM 配置 |
| `submit_gitlab` | `(config: GitlabConfig) => Promise<void>` | 提交 GitLab 配置 |
| `complete` | `() => Promise<void>` | 完成安装 |
| `reset` | `() => void` | 重置状态 |

---

## superRingStore

> 源文件：`stores/superRingStore.ts`

Super Ring 全局对话状态。消息通过 localStorage 持久化（`ring-super-messages`）。

### State

```typescript
interface SuperRingState {
  messages: Message[]       // 消息列表（持久化）
  is_streaming: boolean    // 是否正在流式响应
  error: string | null      // 错误信息
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `send_message` | `(content) => Promise<void>` | 发送消息，解析 SSE 流（`text` / `error` 事件） |

---

## blueprintStore

> 源文件：`stores/blueprintStore.ts`

蓝图创建流程状态管理，含预览 + 确认两阶段。消息和预览图通过 localStorage 持久化（`ring-blueprint-state`）。

### State

```typescript
interface BlueprintState {
  messages: Message[]            // 对话消息（持久化）
  is_streaming: boolean         // 是否正在流式响应
  error: string | null          // 错误信息
  preview_graphs: GraphDef[] | null  // 待确认的图谱预览（持久化）
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `send_message` | `(ringId, content) => Promise<void>` | 发送消息，解析 SSE 流（`text` / `blueprint_proposal` / `error` 事件） |
| `confirm` | `(ringId) => Promise<void>` | 确认蓝图提案，调用 `blueprint_confirm`，清空预览 |
| `dismiss_preview` | `() => void` | 丢弃预览图 |

---

## sessionChatStore

> 源文件：`stores/sessionChatStore.ts`

Session 协作中的实时对话状态（区别于 sessionStore 管理 Session 列表本身）。

### State

```typescript
interface SessionChatState {
  messages: Message[]       // 消息列表
  is_streaming: boolean    // 是否正在流式响应
  error: string | null      // 错误信息
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_history` | `(ring_id, session_id) => Promise<void>` | 加载历史消息 |
| `send_message` | `(ring_id, session_id, content) => Promise<void>` | 发送消息，解析 SSE 流（`text` / `error` 事件） |
| `reset` | `() => void` | 重置状态 |

---

## chatStore

> 源文件：`stores/chatStore.ts`

基于 Zustand 的对话状态管理。

### State

```typescript
interface ChatState {
  messages: Message[]                   // 消息列表
  tool_events: ToolEvent[]              // 工具调用事件
  is_streaming: boolean                 // 是否正在流式响应
  current_conversation_id: string | null // 当前对话 ID
  error: string | null                  // 错误信息
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `create_conversation` | `(ring_id, title) => Promise<string>` | 创建对话，返回 conv_id |
| `load_history` | `(ring_id, conv_id) => Promise<void>` | 加载历史消息到 messages |
| `send_message` | `(ring_id, content) => Promise<void>` | 发送消息，解析 SSE 流，更新 messages 和 tool_events |
| `reset` | `() => void` | 重置所有状态 |

### SSE 事件处理

`send_message` 解析 SSE 流，处理以下事件类型：
- `text` — 累积到 assistant_content，实时更新 messages 中的 assistant 消息
- `error` — 设置 error 状态
- `tool_call` — 添加 tool_events
- `tool_result` — 添加 tool_events
- `archive_suggestion` — 添加 tool_events

---

## graphStore

> 源文件：`stores/graphStore.ts`

图谱状态管理。

### State

```typescript
interface GraphState {
  graphs: string[]                    // 图 ID 列表
  current_graph_id: string | null     // 当前图 ID
  nodes: GraphNode[]                  // 当前图节点
  edges: GraphEdge[]                  // 当前图边
  selected_node_id: string | null    // 选中节点 ID
  selected_node_content: NodeContent | null  // 选中节点内容
  search_results: SearchResult[]      // 搜索结果
  loading: boolean                    // 是否加载中
  error: string | null                // 错误信息
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_graphs` | `(ring_id) => Promise<void>` | 加载图列表 |
| `select_graph` | `(ring_id, graph_id) => Promise<void>` | 选中图，加载 nodes 和 edges |
| `select_node` | `(ring_id, graph_id, node_id) => Promise<void>` | 选中节点，加载 NodeContent |
| `create_node` | `(ring_id, graph_id, req) => Promise<void>` | 创建节点 |
| `delete_node` | `(ring_id, graph_id, node_id) => Promise<void>` | 删除节点 |
| `create_edge` | `(ring_id, graph_id, req) => Promise<void>` | 创建边 |
| `delete_edge` | `(ring_id, graph_id, edge_id) => Promise<void>` | 删除边 |
| `search_nodes` | `(ring_id, query) => Promise<void>` | 搜索节点 |
| `reset` | `() => void` | 重置状态 |

---

## gitStore

> 源文件：`stores/gitStore.ts`

Git/GitLab 状态管理。

### State

```typescript
interface GitState {
  prs: PrListItem[]               // PR 列表
  current_pr: PrDetail | null     // 当前 PR 详情
  commit_log: CommitLogEntry[]     // 提交日志
  archive_queue: ArchiveQueueResponse | null  // 归档队列
  loading: boolean                // 是否加载中
  error: string | null            // 错误信息
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_prs` | `(ring_id, state?) => Promise<void>` | 加载 PR 列表 |
| `load_pr_detail` | `(ring_id, pr_id) => Promise<void>` | 加载 PR 详情（diff） |
| `merge_pr` | `(ring_id, pr_id) => Promise<void>` | 合并 PR |
| `reject_pr` | `(ring_id, pr_id) => Promise<void>` | 拒绝 PR |
| `load_commit_log` | `(ring_id, limit?) => Promise<void>` | 加载提交日志 |
| `load_archive_queue` | `(ring_id) => Promise<void>` | 加载归档队列 |
| `clear_error` | `() => void` | 清空错误 |

---

## memberStore

> 源文件：`stores/memberStore.ts`

成员状态管理。

### State

```typescript
interface MemberState {
  members: Member[]          // 成员列表
  loading: boolean           // 是否加载中
  error: string | null      // 错误信息
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_members` | `(ring_id) => Promise<void>` | 加载成员列表 |
| `generate_invite` | `(ring_id, req) => Promise<string \| null>` | 生成邀请，返回 token |
| `update_role` | `(ring_id, member_id, role) => Promise<void>` | 更新成员角色 |
| `remove_member` | `(ring_id, member_id) => Promise<void>` | 移除成员 |
| `clear_error` | `() => void` | 清空错误 |

---

## sessionStore

> 源文件：`stores/sessionStore.ts`

Session 列表和生命周期状态管理（不含对话消息，对话消息由 sessionChatStore 管理）。

### State

```typescript
interface SessionState {
  sessions: SessionData[]         // Session 列表
  current_session: SessionData | null  // 当前 Session
  loading: boolean               // 是否加载中
  error: string | null           // 错误信息
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_sessions` | `(ring_id, status?) => Promise<void>` | 加载 Session 列表 |
| `create_session` | `(ring_id, req) => Promise<SessionData \| null>` | 创建 Session |
| `close_session` | `(ring_id, session_id) => Promise<void>` | 关闭 Session |
| `leave_session` | `(ring_id, session_id) => Promise<void>` | 离开 Session |
| `toggle_archive` | `(ring_id, session_id, enabled) => Promise<void>` | 开关归档 |
| `delete_session` | `(ring_id, session_id) => Promise<void>` | 删除 Session |
| `clear_error` | `() => void` | 清空错误 |

---

## settingsStore

> 源文件：`stores/settingsStore.ts`

设置状态管理。

### State

```typescript
interface SettingsState {
  settings: Record<string, string>  // 键值对设置
  loading: boolean                   // 是否加载中
  error: string | null              // 错误信息
}
```

### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_settings` | `() => Promise<void>` | 加载设置 |
| `save_settings` | `(settings) => Promise<void>` | 保存设置并刷新 |
