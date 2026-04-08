# Handler 层 API 参考

> 源码路径：`ring-server/src/handlers/`

## 通用模式

所有需要认证的 handler 第一个参数都是 `Extension<AuthUser>`，从中获取 `auth_user.user_id`。

所有返回 SSE 的 handler 返回类型为 `SseStream`（`Sse<ReceiverStream<Result<Event, Infallible>>>`）。

SSE 事件格式：`event: message\ndata: {json}\n\n`

---

## Setup Handler

源文件：`handlers/setup.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `get_status` | GET | `/setup/status` | 否 | 返回 setup_completed/step/user_id |
| `set_username` | POST | `/setup/username` | 否 | 创建用户（display_name ≤ 50 字符） |
| `set_llm` | POST | `/setup/llm` | 否 | 保存 LLM 配置到 settings |
| `set_gitlab` | POST | `/setup/gitlab` | 否 | 保存 GitLab 配置到 settings |
| `complete` | POST | `/setup/complete` | 否 | 完成 setup 流程 |

### SetupStatus
| 字段 | 类型 | 说明 |
|------|------|------|
| `setup_completed` | `bool` | 是否完成 |
| `step` | `string` | 当前步骤 |
| `user_id` | `Option<String>` | 用户 ID |

---

## Ring Handler

源文件：`handlers/ring.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_rings` | GET | `/rings` | 是 | 列出当前用户的 Ring |
| `create_ring` | POST | `/rings` | 是 | 创建 Ring，返回 201 |
| `get_ring` | GET | `/rings/{ringId}` | 是 | 获取 Ring 详情 |
| `update_ring` | PUT | `/rings/{ringId}` | 是 | 更新 Ring（name/description） |
| `delete_ring` | DELETE | `/rings/{ringId}` | 是 | 删除 Ring |

---

## Member Handler

源文件：`handlers/member.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_members` | GET | `/rings/{ringId}/members` | 是 | 列出成员 |
| `generate_invite` | POST | `/rings/{ringId}/members/invites` | 是 | 生成邀请码 |
| `update_role` | PUT | `/rings/{ringId}/members/{memberId}/role` | 是 | 更新成员角色 |
| `remove_member` | DELETE | `/rings/{ringId}/members/{memberId}` | 是 | 移除成员 |
| `join_ring` | POST | `/rings/join?token=xxx` | 是 | 通过邀请码加入 Ring |

---

## Session Handler

源文件：`handlers/session.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `create_session` | POST | `/rings/{ringId}/sessions` | 是 | 创建 Session |
| `list_sessions` | GET | `/rings/{ringId}/sessions` | 是 | 列出 Session |
| `get_session` | GET | `/rings/{ringId}/sessions/{sessionId}` | 是 | 获取 Session 详情 |
| `close_session` | POST | `/rings/{ringId}/sessions/{sessionId}/close` | 是 | 关闭 Session |
| `leave_session` | POST | `/rings/{ringId}/sessions/{sessionId}/leave` | 是 | 离开 Session |
| `toggle_archive` | PUT | `/rings/{ringId}/sessions/{sessionId}/archive-toggle` | 是 | 开关归档 |
| `invite_member` | POST | `/rings/{ringId}/sessions/{sessionId}/invite` | 是 | 邀请成员 |
| `delete_session` | DELETE | `/rings/{ringId}/sessions/{sessionId}` | 是 | 删除 Session |
| `get_messages` | GET | `/rings/{ringId}/sessions/{sessionId}/messages` | 是 | 获取消息 |
| `send_message` | POST | `/rings/{ringId}/sessions/{sessionId}/messages` | 是 | **SSE**：发送消息，AI 流式响应 |

---

## Conversation Handler

源文件：`handlers/conversation.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list` | GET | `/rings/{ringId}/conversations` | 是 | 列出对话 |
| `create` | POST | `/rings/{ringId}/conversations` | 是 | 创建对话 |
| `get` | GET | `/rings/{ringId}/conversations/{convId}` | 是 | 获取对话详情 |
| `get_messages` | GET | `/rings/{ringId}/conversations/{convId}/messages` | 是 | 获取消息 |
| `send_message` | POST | `/rings/{ringId}/conversations/{convId}/messages` | 是 | **SSE**：发送消息，AI 流式响应 |

---

## Blueprint Handler

源文件：`handlers/blueprint.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_templates` | GET | `/rings/{ringId}/blueprint/templates` | 是 | 列出蓝图模板 |
| `blueprint_chat` | POST | `/rings/{ringId}/blueprint/chat` | 是 | **SSE**：蓝图构建对话 |
| `preview_blueprint` | POST | `/rings/{ringId}/blueprint/preview` | 是 | 生成蓝图预览（mermaid） |
| `confirm_blueprint` | POST | `/rings/{ringId}/blueprint/confirm` | 是 | 确认蓝图，创建图谱节点，激活 Ring |

---

## Graph Handler

源文件：`handlers/graph.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_graphs` | GET | `/rings/{ringId}/graphs` | 是 | 列出图 ID |
| `get_graph` | GET | `/rings/{ringId}/graphs/{graphId}` | 是 | 获取图详情（节点+边） |
| `create_node` | POST | `/rings/{ringId}/graphs/{graphId}/nodes` | 是 | 创建节点，返回 201 |
| `get_node` | GET | `/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}` | 是 | 获取节点 |
| `update_node` | PUT | `/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}` | 是 | 更新节点 |
| `delete_node` | DELETE | `/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}` | 是 | 删除节点（级联） |
| `get_node_content` | GET | `/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}/content` | 是 | 获取 Markdown 内容 |
| `create_edge` | POST | `/rings/{ringId}/graphs/{graphId}/edges` | 是 | 创建边 |
| `delete_edge` | DELETE | `/rings/{ringId}/graphs/{graphId}/edges/{edgeId}` | 是 | 删除边 |

---

## Search Handler

源文件：`handlers/search.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `search_nodes` | POST | `/rings/{ringId}/search` | 是 | FTS 全文搜索节点 |

---

## Archive Handler

源文件：`handlers/archive.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `archive` | POST | `/rings/{ringId}/archive` | 是 | 归档对话消息到 Markdown |
| `get_queue` | GET | `/rings/{ringId}/archive/queue` | 是 | 获取待审核队列 |
| `confirm_archive` | POST | `/rings/{ringId}/archive/{archiveId}/confirm` | 是 | 确认归档 |

---

## Git Handler

源文件：`handlers/git.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_prs` | GET | `/rings/{ringId}/git/prs` | 是 | 列出 MR |
| `merge_pr` | POST | `/rings/{ringId}/git/prs/{prId}/merge` | 是 | 合并 MR |
| `reject_pr` | POST | `/rings/{ringId}/git/prs/{prId}/reject` | 是 | 拒绝 MR |
| `get_commit_log` | GET | `/rings/{ringId}/git/commits` | 是 | 获取提交日志 |

---

## Notification Handler

源文件：`handlers/notification.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_notifications` | GET | `/notifications` | 是 | 列出通知 |
| `mark_read` | POST | `/notifications/{notificationId}` | 是 | 标记已读 |

---

## Settings Handler

源文件：`handlers/settings.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `get_settings` | GET | `/settings` | 是 | 获取所有设置 |
| `update_settings` | PUT | `/settings` | 是 | 更新设置 |

---

## AI Handler

源文件：`handlers/ai.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `super_ring_chat` | POST | `/super-ring/chat` | 是 | **SSE**：全局超级助手对话 |

---

## WebSocket Handler

源文件：`handlers/ws.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `ws_handler` | GET | `/ws/{ringId}` | 是（通过 query/cookie） | WebSocket 升级，处理 Ring 内实时消息 |

**流程**：升级 → 订阅 ring_id 的 WsHub → 转发收到的消息到客户端

---

## Install Handler

源文件：`handlers/install.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `join_page` | GET | `/join` | 否 | 公开安装引导页面（显示 Ring 信息、下载链接） |

---

## SSE Helpers

源文件：`handlers/sse_helpers.rs`

### `type SseStream`
`Sse<ReceiverStream<Result<Event, Infallible>>>`

### `fn spawn_sse_stream(llm_stream) -> SseStream`
将 `LlmEvent` 流转换为 SSE 响应，事件名为 `message`。

### `fn spawn_sse_stream_with_callback(llm_stream, on_complete) -> SseStream`
同上，但追加 `on_complete` 回调：流结束后收集所有 `LlmEvent::Text`，调用 `on_complete(content_string)`。
