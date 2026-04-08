# 前端 API Client 参考

> 源码路径：`ring-frontend/src/api/client.ts`

## 基础

所有请求自动携带 `X-User-Id` header（从 `localStorage.getItem('ring_user_id')` 获取）。JSON 响应请求自动携带 `Content-Type: application/json`。

SSE 请求（`send_message`、`super_ring_chat`、`blueprint_chat`、`send_session_message`）返回原生 `Promise<Response>`，调用方需自行解析 SSE 流。

---

## Setup

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `get_setup_status` | GET | `/setup/status` | `Promise<SetupStatus>` | 获取设置状态 |
| `set_username(display_name)` | POST | `/setup/username` | `Promise<User>` | 设置用户名 |
| `set_llm(config)` | POST | `/setup/llm` | `Promise<void>` | 保存 LLM 配置 |
| `set_gitlab(config)` | POST | `/setup/gitlab` | `Promise<void>` | 保存 GitLab 配置 |
| `complete_setup` | POST | `/setup/complete` | `Promise<void>` | 完成设置 |

---

## Ring

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_rings` | GET | `/rings` | `Promise<RingListItem[]>` | 列出 Ring |
| `create_ring(req)` | POST | `/rings` | `Promise<Ring>` | 创建 Ring |
| `get_ring(id)` | GET | `/rings/{id}` | `Promise<Ring>` | 获取 Ring |
| `delete_ring(id)` | DELETE | `/rings/{id}` | `Promise<void>` | 删除 Ring |

---

## Conversation

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_conversations(ring_id)` | GET | `/rings/{ring_id}/conversations` | `Promise<Conversation[]>` | 列出对话 |
| `create_conversation(ring_id, title)` | POST | `/rings/{ring_id}/conversations` | `Promise<Conversation>` | 创建对话 |
| `get_messages(ring_id, conv_id)` | GET | `/rings/{ring_id}/conversations/{conv_id}/messages` | `Promise<Message[]>` | 获取消息 |
| `send_message(ring_id, conv_id, content)` | POST | `/rings/{ring_id}/conversations/{conv_id}/messages` | `Promise<Response>` | **SSE 发送消息** |

---

## Blueprint

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_blueprint_templates(ring_id)` | GET | `/rings/{ring_id}/blueprint/templates` | `Promise<BlueprintTemplate[]>` | 列出模板 |
| `blueprint_chat(ring_id, message, history)` | POST | `/rings/{ring_id}/blueprint/chat` | `Promise<Response>` | **SSE 蓝图对话** |
| `blueprint_preview(ring_id, graphs)` | POST | `/rings/{ring_id}/blueprint/preview` | `Promise<PreviewResponse>` | 预览蓝图 |
| `blueprint_confirm(ring_id, graphs)` | POST | `/rings/{ring_id}/blueprint/confirm` | `Promise<ConfirmResponse>` | 确认蓝图 |

---

## Graph

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

## Search

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `search_nodes(ring_id, query, graph_ids?)` | POST | `/rings/{ring_id}/search` | `Promise<{ results, total }>` | 搜索节点 |

---

## Archive

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `archive_content(ring_id, req)` | POST | `/rings/{ring_id}/archive` | `Promise<ArchiveResponse>` | 归档内容 |
| `get_archive_queue(ring_id)` | GET | `/rings/{ring_id}/archive/queue` | `Promise<ArchiveQueueResponse>` | 获取队列 |
| `confirm_archive(ring_id, archive_id)` | POST | `/rings/{ring_id}/archive/{archive_id}/confirm` | `Promise<void>` | 确认归档 |

---

## Git

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_prs(ring_id, state?)` | GET | `/rings/{ring_id}/git/prs` | `Promise<PrListItem[]>` | 列出 PR |
| `get_pr_diff(ring_id, pr_id)` | GET | `/rings/{ring_id}/git/prs/{pr_id}/diff` | `Promise<PrDetail>` | 获取 Diff |
| `merge_pr(ring_id, pr_id)` | POST | `/rings/{ring_id}/git/prs/{pr_id}/merge` | `Promise<void>` | 合并 PR |
| `reject_pr(ring_id, pr_id)` | POST | `/rings/{ring_id}/git/prs/{pr_id}/reject` | `Promise<void>` | 拒绝 PR |
| `get_commit_log(ring_id, limit?)` | GET | `/rings/{ring_id}/git/commits` | `Promise<CommitLogEntry[]>` | 获取提交日志 |

---

## Member

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `list_members(ring_id)` | GET | `/rings/{ring_id}/members` | `Promise<Member[]>` | 列出成员 |
| `generate_invite(ring_id, req)` | POST | `/rings/{ring_id}/members/invites` | `Promise<InviteToken>` | 生成邀请 |
| `update_member_role(ring_id, member_id, role)` | PUT | `/rings/{ring_id}/members/{member_id}/role` | `Promise<void>` | 更新角色 |
| `remove_member(ring_id, member_id)` | DELETE | `/rings/{ring_id}/members/{member_id}` | `Promise<void>` | 移除成员 |
| `join_ring(token, display_name)` | POST | `/rings/join?token={token}` | `Promise<Member>` | 加入 Ring |

---

## Session

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `create_session(ring_id, req)` | POST | `/rings/{ring_id}/sessions` | `Promise<SessionData>` | 创建 Session |
| `list_sessions(ring_id, status?)` | GET | `/rings/{ring_id}/sessions` | `Promise<SessionData[]>` | 列出 Session |
| `get_session(ring_id, session_id)` | GET | `/rings/{ring_id}/sessions/{session_id}` | `Promise<SessionData>` | 获取详情 |
| `close_session(ring_id, session_id)` | POST | `/rings/{ring_id}/sessions/{session_id}/close` | `Promise<void>` | 关闭 |
| `leave_session(ring_id, session_id)` | POST | `/rings/{ring_id}/sessions/{session_id}/leave` | `Promise<void>` | 离开 |
| `toggle_session_archive(ring_id, session_id, enabled)` | PUT | `/rings/{ring_id}/sessions/{session_id}/archive-toggle` | `Promise<void>` | 开关归档 |
| `invite_to_session(ring_id, session_id, member_ids)` | POST | `/rings/{ring_id}/sessions/{session_id}/invite` | `Promise<void>` | 邀请成员 |
| `delete_session(ring_id, session_id)` | DELETE | `/rings/{ring_id}/sessions/{session_id}` | `Promise<void>` | 删除 |
| `get_session_messages(ring_id, session_id)` | GET | `/rings/{ring_id}/sessions/{session_id}/messages` | `Promise<Message[]>` | 获取消息 |
| `send_session_message(ring_id, session_id, message)` | POST | `/rings/{ring_id}/sessions/{session_id}/messages` | `Promise<Response>` | **SSE 发送消息** |

---

## Settings

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `get_settings` | GET | `/settings` | `Promise<Record<string, string>>` | 获取设置 |
| `update_settings(settings)` | PUT | `/settings` | `Promise<void>` | 更新设置 |

---

## Super Ring

| 函数 | 方法 | 路径 | 返回类型 | 说明 |
|------|------|------|---------|------|
| `super_ring_chat(message, history)` | POST | `/super-ring/chat` | `Promise<Response>` | **SSE 全局对话** |
