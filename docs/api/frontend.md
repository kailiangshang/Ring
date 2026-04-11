# 前端 API 参考

> **Affects**: [frontend.md](frontend.md) · [api-design.md](../technical/api-design.md)
> **Depends on**: [backend.md](backend.md) · [PRD.md](../product/PRD.md) · [sse-protocol.md](../technical/sse-protocol.md)
> **Last verified**: 2026-04-11

---

## 类型定义

> 源码路径：`ring-frontend/src/types/index.ts`
>
> 所有 TypeScript 接口定义以源码为准。本文档不重复列举，避免与代码不同步。
>
> 关键类型概览（快速参考，详见源码）：

| 类型 | 说明 |
|------|------|
| `User` | 用户（user_id + display_name） |
| `SetupStatus` | 安装向导状态 |
| `LlmConfig` | LLM 配置（provider/model/api_key/base_url） |
| `GitlabConfig` | GitLab 配置（repo_url/auth_type/ssh_key_path） |
| `Ring` | Ring 空间（含 creator_id、gitlab_repo、status 等完整字段） |
| `CreateRingRequest` | 创建 Ring 请求体 |
| `Conversation` | 对话（含 mode、token_count、auto_compact、summary 等字段） |
| `Message` | 消息（含 tool_calls、archived 字段） |
| `SseEvent` / `SseEventType` | SSE 事件（含 code、message_id、token_usage 等字段） |
| `TokenUsage` | token 使用量（prompt/completion/total） |
| `ToolEvent` | 工具调用事件（tool_call / tool_result / archive_suggestion） |
| `GraphDef` | 图谱定义（name/graph_type/categories） |
| `BlueprintTemplate` | 蓝图模板（graphs 为 string 类型，含 is_system、created_by） |
| `GraphPreview` / `GraphPreviewNode` / `GraphPreviewEdge` | 蓝图预览图结构 |
| `PreviewResponse` | 蓝图预览响应 |
| `GraphInfo` | 确认后返回的图基本信息 |
| `ConfirmResponse` | 蓝图确认响应（含 blueprint_id、graphs、status） |
| `GraphNode` / `GraphEdge` / `GraphDetail` | 运行时图谱节点/边/详情 |
| `NodeContent` | 节点 Markdown 内容 |
| `SearchResult` | 搜索结果（node_id/graph_id/label/snippet/rank） |
| `ArchiveRequest` / `ArchiveResponse` | 归档请求/响应 |
| `ArchiveQueueResponse` / `QueueItem` | 归档队列 |
| `PrListItem` | PR 列表项（含 changes: FileChange[]） |
| `FileChange` | 文件变更（file/status/additions/deletions/diff） |
| `CommitLogEntry` | Git 提交日志条目 |
| `InviteToken` | 邀请令牌（含完整字段：id/ring_id/inviter_id/expires_at 等） |
| `Member` | 成员 |
| `SessionListItem` | Session 列表项（id/title/member_count/status 等） |
| `SessionDetail` | Session 详情（含 scenario、members: SessionMemberBrief[]） |
| `SessionMemberBrief` | Session 成员简要信息 |
| `SessionMessage` | Session 消息（含 seq_num） |
| `CreateSessionRequest` | 创建 Session 请求体 |
| `InviteRequest` | 生成邀请请求体 |
| `Notification` | 通知（id/ring_id/type/title/body/is_read 等） |
| `Settings` | 设置（`Record<string, string>`） |

---

## API Client

> 源码路径：`ring-frontend/src/api/client.ts`

### 基础

所有请求自动携带 `X-User-Id` header（从 `localStorage.getItem('ring_user_id')` 获取）。JSON 响应请求自动携带 `Content-Type: application/json`。

SSE 请求（`send_message`、`super_ring_chat`、`blueprint_chat`、`send_session_message`）返回原生 `Promise<Response>`，调用方需自行解析 SSE 流。

---

### Setup

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `get_setup_status` | GET | `/setup/status` | `Promise<SetupStatus>` | 获取设置状态 |
| `set_username(display_name)` | POST | `/setup/username` | `Promise<User>` | 设置用户名 |
| `set_llm(config)` | POST | `/setup/llm` | `Promise<void>` | 保存 LLM 配置 |
| `set_gitlab(config)` | POST | `/setup/gitlab` | `Promise<void>` | 保存 GitLab 配置 |
| `complete_setup` | POST | `/setup/complete` | `Promise<void>` | 完成设置 |

---

### Ring

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_rings` | GET | `/rings` | `Promise<Ring[]>` | 列出 Ring |
| `create_ring(req)` | POST | `/rings` | `Promise<Ring>` | 创建 Ring |
| `get_ring(id)` | GET | `/rings/{id}` | `Promise<Ring>` | 获取 Ring |
| `delete_ring(id)` | DELETE | `/rings/{id}` | `Promise<void>` | 删除 Ring |

---

### Conversation

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_conversations(ring_id)` | GET | `/rings/{ring_id}/conversations` | `Promise<Conversation[]>` | 列出对话 |
| `create_conversation(ring_id, title)` | POST | `/rings/{ring_id}/conversations` | `Promise<Conversation>` | 创建对话 |
| `get_messages(ring_id, conv_id)` | GET | `/rings/{ring_id}/conversations/{conv_id}/messages` | `Promise<Message[]>` | 获取消息 |
| `send_message(ring_id, conv_id, content)` | POST | `/rings/{ring_id}/conversations/{conv_id}/messages` | `Promise<Response>` | **SSE 发送消息** |

---

### Blueprint

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_blueprint_templates(ring_id)` | GET | `/rings/{ring_id}/blueprint/templates` | `Promise<BlueprintTemplate[]>` | 列出模板 |
| `blueprint_chat(ring_id, message, history)` | POST | `/rings/{ring_id}/blueprint/chat` | `Promise<Response>` | **SSE 蓝图对话** |
| `blueprint_preview(ring_id, graphs)` | POST | `/rings/{ring_id}/blueprint/preview` | `Promise<PreviewResponse>` | 预览蓝图 |
| `blueprint_confirm(ring_id, graphs)` | POST | `/rings/{ring_id}/blueprint/confirm` | `Promise<ConfirmResponse>` | 确认蓝图 |

---

### Graph

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_graphs(ring_id)` | GET | `/rings/{ring_id}/graphs` | `Promise<string[]>` | 列出图 ID |
| `get_graph(ring_id, graph_id)` | GET | `/rings/{ring_id}/graphs/{graph_id}` | `Promise<GraphDetail>` | 获取图详情 |
| `create_node(ring_id, graph_id, req)` | POST | `/rings/{ring_id}/graphs/{graph_id}/nodes` | `Promise<GraphNode>` | 创建节点 |
| `update_node(ring_id, graph_id, node_id, req)` | PUT | `/rings/{ring_id}/graphs/{graph_id}/nodes/{node_id}` | `Promise<GraphNode>` | 更新节点 |
| `delete_node(ring_id, graph_id, node_id)` | DELETE | `/rings/{ring_id}/graphs/{graph_id}/nodes/{node_id}` | `Promise<void>` | 删除节点 |
| `get_node_content(ring_id, graph_id, node_id)` | GET | `/rings/{ring_id}/graphs/{graph_id}/nodes/{node_id}/content` | `Promise<NodeContent>` | 获取内容 |
| `create_edge(ring_id, graph_id, req)` | POST | `/rings/{ring_id}/graphs/{graph_id}/edges` | `Promise<GraphEdge>` | 创建边 |
| `delete_edge(ring_id, graph_id, edge_id)` | DELETE | `/rings/{ring_id}/graphs/{graph_id}/edges/{edge_id}` | `Promise<void>` | 删除边 |

---

### Search

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `search_nodes(ring_id, query, graph_ids?)` | POST | `/rings/{ring_id}/search` | `Promise<{ results: SearchResult[], total: number }>` | 搜索节点 |

---

### Archive

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `archive_content(ring_id, req)` | POST | `/rings/{ring_id}/archive` | `Promise<ArchiveResponse>` | 归档内容 |
| `get_archive_queue(ring_id)` | GET | `/rings/{ring_id}/archive/queue` | `Promise<ArchiveQueueResponse>` | 获取队列 |
| `confirm_archive(ring_id, archive_id)` | POST | `/rings/{ring_id}/archive/{archive_id}/confirm` | `Promise<void>` | 确认归档 |

---

### Git

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_prs(ring_id, state?)` | GET | `/rings/{ring_id}/git/prs` | `Promise<PrListItem[]>` | 列出 PR |
| `get_pr_diff(ring_id, pr_id)` | GET | `/rings/{ring_id}/git/prs/{pr_id}/diff` | `Promise<PrListItem>` | 获取 PR Diff |
| `merge_pr(ring_id, pr_id)` | POST | `/rings/{ring_id}/git/prs/{pr_id}/merge` | `Promise<void>` | 合并 PR |
| `reject_pr(ring_id, pr_id)` | POST | `/rings/{ring_id}/git/prs/{pr_id}/reject` | `Promise<void>` | 拒绝 PR |
| `get_commit_log(ring_id, limit?)` | GET | `/rings/{ring_id}/git/commits` | `Promise<CommitLogEntry[]>` | 获取提交日志 |

---

### Member

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_members(ring_id)` | GET | `/rings/{ring_id}/members` | `Promise<Member[]>` | 列出成员 |
| `generate_invite(ring_id, req)` | POST | `/rings/{ring_id}/members/invites` | `Promise<InviteToken>` | 生成邀请 |
| `update_member_role(ring_id, member_id, role)` | PUT | `/rings/{ring_id}/members/{member_id}/role` | `Promise<void>` | 更新角色 |
| `remove_member(ring_id, member_id)` | DELETE | `/rings/{ring_id}/members/{member_id}` | `Promise<void>` | 移除成员 |
| `join_ring(token, display_name)` | POST | `/rings/join?token={token}` | `Promise<Member>` | 加入 Ring |

---

### Session

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `create_session(ring_id, req)` | POST | `/rings/{ring_id}/sessions` | `Promise<SessionDetail>` | 创建 Session |
| `list_sessions(ring_id, status?)` | GET | `/rings/{ring_id}/sessions` | `Promise<SessionListItem[]>` | 列出 Session |
| `get_session(ring_id, session_id)` | GET | `/rings/{ring_id}/sessions/{session_id}` | `Promise<SessionDetail>` | 获取详情 |
| `close_session(ring_id, session_id)` | POST | `/rings/{ring_id}/sessions/{session_id}/close` | `Promise<void>` | 关闭 |
| `leave_session(ring_id, session_id)` | POST | `/rings/{ring_id}/sessions/{session_id}/leave` | `Promise<void>` | 离开 |
| `toggle_session_archive(ring_id, session_id, enabled)` | PUT | `/rings/{ring_id}/sessions/{session_id}/archive-toggle` | `Promise<void>` | 开关归档 |
| `invite_to_session(ring_id, session_id, member_ids)` | POST | `/rings/{ring_id}/sessions/{session_id}/invite` | `Promise<{ invited: string[] }>` | 邀请成员 |
| `delete_session(ring_id, session_id)` | DELETE | `/rings/{ring_id}/sessions/{session_id}` | `Promise<void>` | 删除 |
| `get_session_messages(ring_id, session_id)` | GET | `/rings/{ring_id}/sessions/{session_id}/messages` | `Promise<SessionMessage[]>` | 获取消息 |
| `send_session_message(ring_id, session_id, message)` | POST | `/rings/{ring_id}/sessions/{session_id}/messages` | `Promise<Response>` | **SSE 发送消息** |

---

### Settings

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `get_settings` | GET | `/settings` | `Promise<Settings>` | 获取设置 |
| `update_settings(settings)` | PUT | `/settings` | `Promise<{ ok: boolean }>` | 更新设置 |

---

### Notification

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_notifications` | GET | `/notifications` | `Promise<Notification[]>` | 列出通知 |
| `mark_notification_read(notification_id)` | POST | `/notifications/{notification_id}` | `Promise<void>` | 标记已读 |

---

### Super Ring

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `super_ring_chat(message, history)` | POST | `/super-ring/chat` | `Promise<Response>` | **SSE 全局对话** |

---

## 页面和组件

> 源码路径：`ring-frontend/src/pages/`、`ring-frontend/src/components/`

### 页面路由

源文件：`App.tsx`

| 路径 | 组件 | 说明 |
|------|------|------|
| `/setup` | `SetupWizard` | 初始化向导（用户名 → LLM → GitLab → 完成） |
| `/hub` | `RingHub` | Ring 列表 + 创建 |
| `/ring/:ringId` | `RingSpace` | Ring 空间主页（重定向到 chat） |
| `/ring/:ringId/chat` | `ChatView` | 对话视图 |
| `/ring/:ringId/graph` | `GraphView` | 图谱视图 |
| `/ring/:ringId/blueprint` | `BlueprintWizard` | 蓝图构建向导 |
| `/ring/:ringId/members` | `MemberList` | 成员管理 |
| `/ring/:ringId/sessions` | `SessionView` | Session 列表 |
| `/ring/:ringId/prs` | `PrList` | PR 列表 |
| `/ring/:ringId/prs/:prId` | `PrDetail` | PR 详情 + Diff |
| `/settings` | `SettingsPage` | 设置页面 |

---

### Setup Wizard

源文件：`pages/Setup/SetupWizard.tsx`

步骤流程：
1. `StepUsername` — 输入用户名
2. `StepLlm` — 配置 LLM（provider/model/api_key/base_url）
3. `StepGitlab` — 配置 GitLab
4. 完成 → 跳转到 `/hub`

Props：children 渲染当前步骤组件。

#### StepUsername

| Props | 类型 | 说明 |
|-------|------|------|
| `onNext` | `(name: string) => void` | 下一步 |

#### StepLlm

| Props | 类型 | 说明 |
|-------|------|------|
| `onNext` | `(config: LlmConfig) => void` | 下一步 |
| `onBack` | `() => void` | 上一步 |

#### StepGitlab

| Props | 类型 | 说明 |
|-------|------|------|
| `onComplete` | `(config: GitlabConfig) => void` | 完成 |
| `onBack` | `() => void` | 上一步 |

---

### RingHub

源文件：`pages/RingHub/RingHub.tsx`

包含 `RingList` + `CreateRing` + `SuperRingChat`。

#### RingList

源文件：`pages/RingHub/RingList.tsx`

Props：渲染 Ring 列表（卡片），点击进入 RingSpace。

#### CreateRing

源文件：`pages/RingHub/CreateRing.tsx`

Props：创建 Ring 表单。

#### SuperRingChat

源文件：`pages/RingHub/SuperRingChat.tsx`

Props：Super Ring 全局对话界面（调用 `super_ring_chat` API，解析 SSE）。

---

### ChatView

源文件：`pages/RingSpace/ChatView.tsx`

Props：`ringId: string`

包含对话列表 + ChatInput。

**ChatInput**

| Props | 类型 | 说明 |
|-------|------|------|
| `onSend` | `(content: string) => void` | 发送消息 |

**ChatBubble**

| Props | 类型 | 说明 |
|-------|------|------|
| `message` | `Message` | 消息数据 |
| `isStreaming?` | `boolean` | 是否正在流式输出 |

**ToolCallBubble**

| Props | 类型 | 说明 |
|-------|------|------|
| `event` | `ToolEvent` | 工具调用事件 |

**ToolResultBubble**

| Props | 类型 | 说明 |
|-------|------|------|
| `event` | `ToolEvent` | 工具结果事件 |

**ArchiveSuggestion**

| Props | 类型 | 说明 |
|-------|------|------|
| `data` | `unknown` | 归档建议数据 |

---

### GraphView

源文件：`pages/RingSpace/GraphView.tsx`

Props：`ringId: string`

D3.js 力导向图渲染 + `NodeTree` 侧边导航。

**ForceGraph**

源文件：`components/graph/ForceGraph.tsx`

| Props | 类型 | 说明 |
|-------|------|------|
| `nodes` | `GraphNode[]` | 节点列表 |
| `edges` | `GraphEdge[]` | 边列表 |
| `onNodeClick` | `(node) => void` | 节点点击 |

**NodeTree**

源文件：`components/graph/NodeTree.tsx`

| Props | 类型 | 说明 |
|-------|------|------|
| `nodes` | `GraphNode[]` | 节点列表 |
| `onSelect` | `(node) => void` | 选中节点 |

---

### BlueprintWizard

源文件：`pages/RingSpace/BlueprintWizard.tsx`

Props：`ringId: string`

多步骤蓝图构建流程：选择模板 → 多轮对话 → 预览 → 确认。

---

### PrList / PrDetail

源文件：`pages/RingSpace/PrList.tsx`、`PrDetail.tsx`

**DiffView**

源文件：`components/git/DiffView.tsx`

| Props | 类型 | 说明 |
|-------|------|------|
| `diff` | `FileChange[]` | 文件变更列表 |

---

### MemberList

源文件：`components/member/MemberList.tsx`

| Props | 类型 | 说明 |
|-------|------|------|
| `ring_id` | `string` | Ring ID |
| `members` | `Member[]` | 成员列表 |

---

### SessionView

源文件：`components/session/SessionView.tsx`

| Props | 类型 | 说明 |
|-------|------|------|
| `ring_id` | `string` | Ring ID |
| `session_id` | `string` | Session ID |

---

### SettingsPage

源文件：`pages/Settings/SettingsPage.tsx`

LLM 配置表单（provider/model/api_key/base_url）+ 隐私设置。

---

### Toolbar

源文件：`components/toolbar/Toolbar.tsx`

顶部工具栏，包含：导航链接、操作按钮。

---

## 状态管理 (Stores)

> 源码路径：`ring-frontend/src/stores/`

### setupStore

> 源文件：`stores/setupStore.ts`

初始化安装流程状态管理，分 3 步：填用户名 → 配 LLM → 配 GitLab → 完成。

#### State

```typescript
interface SetupState {
  step: number               // 当前步骤 0/1/2
  error: string | null       // 错误信息
  loading: boolean           // 是否请求中
  user_id: string | null     // 创建的用户 ID
  redirect_home: boolean     // 是否跳转首页
}
```

#### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `set_step` | `(step) => void` | 切换步骤 |
| `submit_username` | `(display_name) => Promise<void>` | 提交用户名，创建用户 |
| `submit_llm` | `(config: LlmConfig) => Promise<void>` | 提交 LLM 配置 |
| `submit_gitlab` | `(config: GitlabConfig) => Promise<void>` | 提交 GitLab 配置 |
| `complete` | `() => Promise<void>` | 完成安装 |
| `reset` | `() => void` | 重置状态 |

---

### superRingStore

> 源文件：`stores/superRingStore.ts`

Super Ring 全局对话状态。消息通过 localStorage 持久化（`ring-super-messages`）。

#### State

```typescript
interface SuperRingState {
  messages: Message[]       // 消息列表（持久化）
  is_streaming: boolean    // 是否正在流式响应
  error: string | null      // 错误信息
}
```

#### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `send_message` | `(content) => Promise<void>` | 发送消息，解析 SSE 流（`text` / `error` 事件） |

---

### blueprintStore

> 源文件：`stores/blueprintStore.ts`

蓝图创建流程状态管理，含预览 + 确认两阶段。消息和预览图通过 localStorage 持久化（`ring-blueprint-state`）。

#### State

```typescript
interface BlueprintState {
  messages: Message[]            // 对话消息（持久化）
  is_streaming: boolean         // 是否正在流式响应
  error: string | null          // 错误信息
  preview_graphs: GraphPreview[] | null  // 待确认的图谱预览（持久化）
}
```

#### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `send_message` | `(ringId, content) => Promise<void>` | 发送消息，解析 SSE 流（`text` / `blueprint_proposal` / `error` 事件） |
| `confirm` | `(ringId) => Promise<void>` | 确认蓝图提案，调用 `blueprint_confirm`，清空预览 |
| `dismiss_preview` | `() => void` | 丢弃预览图 |

---

### sessionChatStore

> 源文件：`stores/sessionChatStore.ts`

Session 协作中的实时对话状态（区别于 sessionStore 管理 Session 列表本身）。

#### State

```typescript
interface SessionChatState {
  messages: Message[]       // 消息列表
  is_streaming: boolean    // 是否正在流式响应
  error: string | null      // 错误信息
}
```

#### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_history` | `(ring_id, session_id) => Promise<void>` | 加载历史消息 |
| `send_message` | `(ring_id, session_id, content) => Promise<void>` | 发送消息，解析 SSE 流（`text` / `error` 事件） |
| `reset` | `() => void` | 重置状态 |

---

### chatStore

> 源文件：`stores/chatStore.ts`

基于 Zustand 的对话状态管理。

#### State

```typescript
interface ChatState {
  messages: Message[]                   // 消息列表
  tool_events: ToolEvent[]              // 工具调用事件
  is_streaming: boolean                 // 是否正在流式响应
  current_conversation_id: string | null // 当前对话 ID
  error: string | null                  // 错误信息
}
```

#### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `create_conversation` | `(ring_id, title) => Promise<string>` | 创建对话，返回 conv_id |
| `load_history` | `(ring_id, conv_id) => Promise<void>` | 加载历史消息到 messages |
| `send_message` | `(ring_id, content) => Promise<void>` | 发送消息，解析 SSE 流，更新 messages 和 tool_events |
| `reset` | `() => void` | 重置所有状态 |

#### SSE 事件处理

`send_message` 解析 SSE 流，处理以下事件类型：
- `text` — 累积到 assistant_content，实时更新 messages 中的 assistant 消息
- `error` — 设置 error 状态
- `tool_call` — 添加 tool_events
- `tool_result` — 添加 tool_events
- `archive_suggestion` — 添加 tool_events

---

### graphStore

> 源文件：`stores/graphStore.ts`

图谱状态管理。

#### State

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

#### Actions

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

### gitStore

> 源文件：`stores/gitStore.ts`

Git/GitLab 状态管理。

#### State

```typescript
interface GitState {
  prs: PrListItem[]               // PR 列表
  current_pr: PrListItem | null   // 当前 PR（含 changes）
  commit_log: CommitLogEntry[]     // 提交日志
  archive_queue: ArchiveQueueResponse | null  // 归档队列
  loading: boolean                // 是否加载中
  error: string | null            // 错误信息
}
```

#### Actions

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

### memberStore

> 源文件：`stores/memberStore.ts`

成员状态管理。

#### State

```typescript
interface MemberState {
  members: Member[]          // 成员列表
  loading: boolean           // 是否加载中
  error: string | null      // 错误信息
}
```

#### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_members` | `(ring_id) => Promise<void>` | 加载成员列表 |
| `generate_invite` | `(ring_id, req) => Promise<string \| null>` | 生成邀请，返回 token |
| `update_role` | `(ring_id, member_id, role) => Promise<void>` | 更新成员角色 |
| `remove_member` | `(ring_id, member_id) => Promise<void>` | 移除成员 |
| `clear_error` | `() => void` | 清空错误 |

---

### sessionStore

> 源文件：`stores/sessionStore.ts`

Session 列表和生命周期状态管理（不含对话消息，对话消息由 sessionChatStore 管理）。

#### State

```typescript
interface SessionState {
  sessions: SessionListItem[]            // Session 列表
  current_session: SessionDetail | null  // 当前 Session
  loading: boolean                       // 是否加载中
  error: string | null                   // 错误信息
}
```

#### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_sessions` | `(ring_id, status?) => Promise<void>` | 加载 Session 列表 |
| `create_session` | `(ring_id, req) => Promise<SessionDetail \| null>` | 创建 Session |
| `close_session` | `(ring_id, session_id) => Promise<void>` | 关闭 Session |
| `leave_session` | `(ring_id, session_id) => Promise<void>` | 离开 Session |
| `toggle_archive` | `(ring_id, session_id, enabled) => Promise<void>` | 开关归档 |
| `delete_session` | `(ring_id, session_id) => Promise<void>` | 删除 Session |
| `clear_error` | `() => void` | 清空错误 |

---

### settingsStore

> 源文件：`stores/settingsStore.ts`

设置状态管理。

#### State

```typescript
interface SettingsState {
  settings: Settings     // 键值对设置（Record<string, string>）
  loading: boolean       // 是否加载中
  error: string | null  // 错误信息
}
```

#### Actions

| Action | 签名 | 说明 |
|--------|------|------|
| `load_settings` | `() => Promise<void>` | 加载设置 |
| `save_settings` | `(settings) => Promise<void>` | 保存设置并刷新 |
