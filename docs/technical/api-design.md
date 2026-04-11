# Ring API 设计

> **Affects**: [frontend.md](../api/frontend.md) · [backend.md](../api/backend.md) · [sse-protocol.md](sse-protocol.md)
> **Depends on**: [PRD.md](../product/PRD.md) · [architecture.md](architecture.md) · [data-model.md](data-model.md)
> **Last verified**: 2026-04-11

## 1. API 概述

- **基础 URL**：`http://{host}:7420/api/v1`
- **认证**：内网环境无需认证，首次启动配置不可变用户名后即可使用
- **格式**：JSON（请求体和响应体）
- **实时**：WebSocket 用于消息推送和状态同步

---

## 1.1 公共页面（无需认证） `[已实现]`

### 1.1.1 安装导航页（去中心化）

```
GET /join?token={inviteToken}
```

> 访问者通过邀请链接访问创建者的 ring-server，获取安装导航页。
> 这是一个独立 HTML 页面（不依赖前端 React），由 ring-server 二进制内嵌的模板动态渲染。
>
> 页面功能：
> 1. 服务端验证 token 有效性，注入 Ring 信息（名称、描述、成员数）
> 2. 客户端 JS 通过 `navigator.platform` / `navigator.userAgent` 检测 OS
> 3. 高亮对应平台下载按钮，下载链接指向 GitHub Releases
> 4. "继续加入"按钮跳转 `http://localhost:7420/join?token=xxx&creator_ip={host}`
>
> **去中心化**：谁分享链接，谁的 ring-server 就服务安装导航页。二进制文件从 GitHub Releases 下载。

**响应**：HTML 页面（`Content-Type: text/html`），服务端注入数据示例：

```html
<script>
window.__RING_JOIN_DATA__ = {
  ring_name: "产品竞品分析组",
  ring_description: "用于竞品分析和市场研究",
  member_count: 5,
  token: "xxx",
  creator_ip: "192.168.1.100",
  downloads: {
    windows: "https://github.com/{owner}/ring/releases/latest/download/ring-server-windows-x86_64.zip",
    linux: "https://github.com/{owner}/ring/releases/latest/download/ring-server-linux-x86_64.tar.gz",
    macos_arm: "https://github.com/{owner}/ring/releases/latest/download/ring-server-macos-arm64.tar.gz",
    macos_intel: "https://github.com/{owner}/ring/releases/latest/download/ring-server-macos-x86_64.tar.gz"
  }
};
</script>
```

### 1.1.2 本地加入页

```
GET /join?token={inviteToken}&creator_ip={creatorIP}
```

> 访问者安装 Ring 后，本地 ring-server 处理加入流程。
> 如果未 Setup → 重定向到 Setup 向导，完成后自动回到 join 流程。

---

## 1.5 首次启动 API `[已实现]`

### 1.5.1 检查初始化状态

```
GET /api/v1/setup/status
```

**响应**：
```json
{
  "setup_completed": false,
  "step": "username"
}
```

### 1.5.2 配置用户信息

```
POST /api/v1/setup/username
```

**请求体**：
```json
{
  "display_name": "王小明"
}
```

> 显示名称可重复、可修改。系统自动生成不可变的 `user_id`（UUID）作为唯一标识。内网 IP 自动检测。

### 1.5.3 配置 LLM

```
POST /api/v1/setup/llm
```

**请求体**：
```json
{
  "provider": "openai",
  "model": "gpt-4",
  "api_key": "sk-xxx",
  "base_url": null
}
```

### 1.5.4 关联 GitLab 仓库

```
POST /api/v1/setup/gitlab
```

**请求体**：
```json
{
  "repo_url": "git@gitlab.company.com:username/ring-workspace.git",
  "auth_type": "ssh_key",
  "ssh_key_path": "~/.ssh/id_rsa",
  "auto_create": false
}
```

> 用户级配置。全局 GitLab 凭证，创建 Ring 时复用，无需重复配置。`auto_create: true` 时自动通过 GitLab API 创建仓库。

### 1.5.5 完成初始化

```
POST /api/v1/setup/complete
```

> 标记 `setup_completed = true`，初始化 Git 仓库结构，进入 Ring Hub。

---

## 2. Ring Hub API `[已实现]`

### 2.1 获取 Ring 列表

```
GET /api/v1/rings
```

**响应**：
```json
{
  "rings": [
    {
      "id": "ring-uuid",
      "name": "产品竞品分析组",
      "member_count": 5,
      "graph_node_count": 23,
      "last_activity_at": "2026-04-05T10:00:00Z",
      "role": "creator"
    }
  ]
}
```

### 2.2 创建 Ring

```
POST /api/v1/rings
```

**请求体**：
```json
{
  "name": "产品竞品分析组",
  "description": "用于竞品分析和市场研究",
  "role_description": "你是一个产品分析专家，帮助团队进行竞品研究",
  "gitlab_repo": "auto_create",
  "namespace": "team"
}
```

> `role_description` 用于初始化 `.ring/role.md`（Group Ring 的角色定义文档）。后续创建者/管理员可直接编辑 `.ring/role.md` 来调整 AI 行为，无需通过 API。
>
> `gitlab_repo`：传入仓库地址（使用已有仓库）或 `"auto_create"`（自动创建新仓库）。
> `namespace`：自动创建时指定 GitLab group 或个人 namespace。不传时默认用用户个人 namespace。
> GitLab 凭证复用全局配置（Setup 时已配置），无需重复传入。

**响应**：
```json
{
  "id": "ring-uuid",
  "name": "产品竞品分析组",
  "status": "blueprint_pending"
}
```

### 2.3 获取 Ring 详情

```
GET /api/v1/rings/{ringId}
```

### 2.4 更新 Ring

```
PUT /api/v1/rings/{ringId}
```

### 2.5 删除 Ring

```
DELETE /api/v1/rings/{ringId}
```

---

## 3. 蓝图 API `[已实现]`

### 3.1 获取蓝图模板列表

```
GET /api/v1/blueprints/templates
```

**响应**：
```json
{
  "templates": [
    {
      "id": "tpl-uuid-1",
      "name": "产品研究",
      "description": "适合产品分析和竞品研究",
      "graphs": [
        {"name": "知识图谱", "type": "knowledge", "categories": ["概念", "方法", "工具"]},
        {"name": "竞品图谱", "type": "competitor", "categories": ["竞品 A", "竞品 B"]},
        {"name": "事件图谱", "type": "event", "categories": ["会议", "决策", "里程碑"]}
      ]
    }
  ]
}
```

### 3.2 蓝图对话

```
POST /api/v1/rings/{ringId}/blueprint/chat
```

**请求体**：
```json
{
  "message": "我需要一个竞品研究的图谱"
}
```

**响应**（流式 SSE）：
```
data: {"type": "text", "content": "我建议创建以下图谱..."}
data: {"type": "blueprint_proposal", "data": {...}}
data: {"type": "done"}
```

### 3.3 预览蓝图

```
POST /api/v1/rings/{ringId}/blueprint/preview
```

**请求体**：
```json
{
  "graphs": [
    {"name": "知识图谱", "type": "knowledge", "categories": ["概念", "方法"]},
    {"name": "竞品图谱", "type": "competitor", "categories": ["竞品 A"]}
  ]
}
```

**响应**：返回图谱预览数据（节点+边），供前端 D3.js 渲染。

### 3.4 确认蓝图

```
POST /api/v1/rings/{ringId}/blueprint/confirm
```

**请求体**：
```json
{
  "graphs": [
    {"name": "知识图谱", "type": "knowledge", "categories": ["概念", "方法", "工具"]},
    {"name": "竞品图谱", "type": "competitor", "categories": ["竞品 A", "竞品 B"]}
  ]
}
```

**响应**：
```json
{
  "blueprint_id": "bp-uuid",
  "graphs": [
    {"id": "graph-uuid-1", "name": "知识图谱", "type": "knowledge"},
    {"id": "graph-uuid-2", "name": "竞品图谱", "type": "competitor"}
  ],
  "status": "confirmed"
}
```

---

## 4. 对话 API `[已实现]`

### 4.1 获取对话列表

```
GET /api/v1/rings/{ringId}/conversations
```

### 4.2 创建对话

```
POST /api/v1/rings/{ringId}/conversations
```

**请求体**：
```json
{
  "title": "竞品分析讨论",
  "context_mode": "storage"
}
```

> `context_mode` 可选：`storage`（持久会话，消息存 SQLite，支持历史和 compact）或 `ephemeral`（临时会话，消息仅存内存，关闭即丢失）。

### 4.3 发送消息（SSE 流式响应）

```
POST /api/v1/rings/{ringId}/conversations/{convId}/messages
```

**请求体**：
```json
{
  "content": "帮我分析这份竞品报告",
  "attachments": ["file-uuid-1"]
}
```

**响应**（流式 SSE）：
```
data: {"type": "text", "content": "我来帮你分析..."}
data: {"type": "tool_call", "tool": "file_parser", "input": {...}}
data: {"type": "tool_result", "tool": "file_parser", "output": {...}}
data: {"type": "text", "content": "分析完成，发现以下要点..."}
data: {"type": "archive_suggestion", "data": {...}}
data: {"type": "done"}
```

### 4.5 Compact 对话上下文

```
POST /api/v1/rings/{ringId}/conversations/{convId}/compact
```

**响应**：
```json
{
  "conversation_id": "conv-uuid",
  "token_count_before": 98000,
  "token_count_after": 5000,
  "messages_compacted": 45,
  "summary_length": 800
}
```

> 将历史对话压缩为摘要（summary），替换原始消息作为 Group Ring 的上下文输入。触发条件：
> - 手动触发：用户主动调用
> - 自动触发：`auto_compact = true` 且 `token_count > token_limit`

### 4.6 获取对话 Token 统计

```
GET /api/v1/rings/{ringId}/conversations/{convId}/token-stats
```

**响应**：
```json
{
  "conversation_id": "conv-uuid",
  "context_mode": "storage",
  "token_count": 95000,
  "token_limit": 100000,
  "auto_compact": false,
  "usage_percent": 95,
  "warning": "对话上下文已使用 95%，建议 compact"
}
```

```
GET /api/v1/rings/{ringId}/conversations/{convId}/messages?limit=50&before={msgId}
```

---

## 5. 图谱 API `[部分实现]`

### 5.1 获取 Ring 的所有图谱

```
GET /api/v1/rings/{ringId}/graphs
```

### 5.2 获取图谱详情（节点+边）

```
GET /api/v1/rings/{ringId}/graphs/{graphId}
```

### 5.3 创建节点

```
POST /api/v1/rings/{ringId}/graphs/{graphId}/nodes
```

**请求体**：
```json
{
  "label": "竞品 A",
  "type": "concept",
  "parent_id": "parent-node-uuid",
  "description": "竞品 A 的功能分析"
}
```

### 5.4 更新节点

```
PUT /api/v1/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}
```

### 5.5 删除节点

```
DELETE /api/v1/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}
```

### 5.6 创建边

```
POST /api/v1/rings/{ringId}/graphs/{graphId}/edges
```

**请求体**：
```json
{
  "source_id": "node-uuid-1",
  "target_id": "node-uuid-2",
  "relation": "depends_on",
  "label": "依赖"
}
```

### 5.7 删除边

```
DELETE /api/v1/rings/{ringId}/graphs/{graphId}/edges/{edgeId}
```

### 5.8 获取节点内容（Markdown）

```
GET /api/v1/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}/content
```

**响应**：
```json
{
  "node_id": "node-uuid-1",
  "label": "竞品 A",
  "markdown_path": "nodes/competitor-a.md",
  "content": "# 竞品 A\n\n## 功能分析\n...",
  "last_modified": "2026-04-05T10:00:00Z"
}
```

### 5.9 搜索节点

```
POST /api/v1/rings/{ringId}/search
```

**请求体**：
```json
{
  "query": "定价策略",
  "graph_ids": ["graph-uuid-1"],
  "search_type": "semantic",
  "time_range": {
    "from": "2026-03-01T00:00:00Z",
    "to": "2026-04-06T00:00:00Z"
  },
  "limit": 20
}
```

> `search_type` 可选：`semantic`（向量语义搜索，默认）、`keyword`（关键词匹配）、`hybrid`（语义 + 关键词混合）。
>
> `time_range` 可选：按节点创建/更新时间过滤。
>
> `graph_ids` 可选：指定搜索哪些图谱，不传则搜索所有图谱。

**响应**：
```json
{
  "results": [
    {
      "node_id": "node-uuid-1",
      "graph_id": "graph-uuid-1",
      "label": "竞品定价策略",
      "snippet": "...定价策略主要分为三种模式...",
      "score": 0.92,
      "created_at": "2026-04-01T10:00:00Z",
      "updated_at": "2026-04-05T14:30:00Z"
    }
  ],
  "total": 5,
  "search_type": "semantic"
}
```

### 5.10 全局搜索（跨图谱） `⚠️ 计划中`

```
POST /api/v1/rings/{ringId}/search/global
```

**请求体**：
```json
{
  "query": "本周关键决策",
  "time_range": {
    "from": "2026-04-01T00:00:00Z",
    "to": null
  },
  "include_conversations": true,
  "limit": 30
}
```

> 跨所有图谱搜索节点 + 可选搜索对话历史。返回混合结果（节点 + 对话片段）。

---

## 6. 归档 API `[已实现]`

### 6.1 标记归档

```
POST /api/v1/rings/{ringId}/archive
```

> **归档前自动 pull**：后端在处理归档请求前先执行 `git pull`，确保本地图谱状态最新，再由 Group Ring 推荐挂载位置。

**请求体**：
```json
{
  "message_ids": ["msg-uuid-1", "msg-uuid-2"],
  "conversation_id": "conv-uuid",
  "graph_id": "graph-uuid-1",
  "target_node_id": "node-uuid-1",
  "label": "竞品 A 功能分析"
}
```

**响应**：
```json
{
  "archive_id": "archive-uuid",
  "markdown_path": "nodes/competitor-a-func.md",
  "git_status": "committed",
  "pr_url": null,
  "queue_position": null
}
```

> **PR 审核队列**：成员归档时 `git_status` 为 `pr_pending`，`queue_position` 显示在审核队列中的位置。PR 按提交顺序逐个审核（串行队列），冲突时打回给提交成员。

### 6.1.1 获取 PR 审核队列

```
GET /api/v1/rings/{ringId}/archive/queue
```

**响应**：
```json
{
  "current_review": {"pr_id": 3, "author": "李四", "title": "新增节点：竞品分析"},
  "queue": [
    {"pr_id": 4, "author": "王五", "title": "更新节点：产品定位", "position": 1},
    {"pr_id": 5, "author": "赵六", "title": "新增节点：定价策略", "position": 2}
  ]
}
```

### 6.2 确认归档推荐

```
POST /api/v1/rings/{ringId}/archive/{archiveId}/confirm
```

### 6.3 Export 对话片段

```
POST /api/v1/rings/{ringId}/conversations/{convId}/export
```

**请求体**：
```json
{
  "message_ids": ["msg-uuid-1", "msg-uuid-2"]
}
```

**响应**：返回 AI 归档推荐。

---

## 7. Git API `[已实现]`

### 7.1 获取 PR 列表

```
GET /api/v1/rings/{ringId}/prs?status=opened
```

### 7.2 获取 PR Diff

```
GET /api/v1/rings/{ringId}/prs/{prId}/diff
```

**响应**：
```json
{
  "pr_id": 3,
  "title": "新增节点：竞品分析",
  "author": "张三",
  "changes": [
    {
      "file": "graphs/knowledge/graph.json",
      "status": "modified",
      "additions": 15,
      "deletions": 0,
      "diff": "@@ -10,6 +10,21 @@\n+    {\n+      \"id\": \"node-uuid-new\",\n..."
    },
    {
      "file": "nodes/competitor-analysis.md",
      "status": "added",
      "additions": 50,
      "deletions": 0,
      "diff": "+# 竞品分析\n+主要竞品：A、B、C\n..."
    }
  ]
}
```

### 7.3 合并 PR

```
POST /api/v1/rings/{ringId}/prs/{prId}/merge
```

### 7.4 拒绝 PR

```
POST /api/v1/rings/{ringId}/prs/{prId}/reject
```

### 7.5 获取提交历史

```
GET /api/v1/rings/{ringId}/commits?limit=20
```

---

## 8. 成员 API `[已实现]`

### 8.1 生成邀请链接

```
POST /api/v1/rings/{ringId}/invites
```

**请求体**：
```json
{
  "token_type": "open",
  "role": "member",
  "max_uses": 0,
  "max_members": 50,
  "expires_in": 86400
}
```

> `token_type`：`open`（开放链接，直接加入）或 `audit`（审核链接，需创建者审批）。
> `max_uses`：0 表示不限次数（仅 open 类型有效），默认 1。
> `max_members`：Ring 最大人数上限（可选），防止链接泄露。

### 8.2 加入 Ring

**开放链接加入**：
```
POST /api/v1/rings/join?token={inviteToken}
```

**请求体**（新用户首次加入）：
```json
{
  "display_name": "李四"
}
```

> 引导页 URL 格式：`http://{creatorIP}:7420/join?token=xxx`（由创建者的 ring-server 服务安装导航页）。
> 安装完成后点击"继续加入"跳转 `http://localhost:7420/join?token=xxx&creator_ip={creatorIP}`，本地后端处理加入流程。
> 未 Setup 时先走 Setup 向导，完成后自动回到 join。
> 加入时自动从 GitLab clone 仓库，分配 token_id，同步其他成员名称。

**审核链接申请加入**：
```
POST /api/v1/rings/join/apply?token={inviteToken}
```

**请求体**：
```json
{
  "display_name": "王五",
  "reason": "我是产品组的新成员"
}
```

> 提交申请后等待创建者审批。创建者批准后分配 token_id 并通知申请人。

### 8.2.1 审核申请审批

```
POST /api/v1/rings/{ringId}/join-requests/{requestId}/approve
```

> 创建者批准加入申请，自动分配 token_id，通知申请人。

```
POST /api/v1/rings/{ringId}/join-requests/{requestId}/reject
```

**请求体**：
```json
{
  "reason": "请联系张三获取正确的邀请链接"
}
```

### 8.3 获取成员列表

```
GET /api/v1/rings/{ringId}/members
```

### 8.4 更新成员角色

```
PUT /api/v1/rings/{ringId}/members/{memberId}/role
```

### 8.5 移除成员

```
DELETE /api/v1/rings/{ringId}/members/{memberId}
```

---

## 9. Session API（多人讨论会话） `[已实现]`

> Session 是 Ring 内的多人实时讨论空间。区别于 Ring 级邀请（仓库同步），Session 级邀请仅限 Ring 内已有成员，消息通过 Session owner 的后端 WebSocket 中转，共享一个 Session Ring 实例。

### 9.1 创建 Session

```
POST /api/v1/rings/{ringId}/sessions
```

**请求体**：
```json
{
  "title": "竞品 A 深度讨论",
  "scenario": "deep_research",
  "archive_enabled": false,
  "invite_member_ids": ["user-uuid-2", "user-uuid-3"]
}
```

**响应**：
```json
{
  "id": "session-uuid",
  "ring_id": "ring-uuid",
  "title": "竞品 A 深度讨论",
  "scenario": "deep_research",
  "scenario_display_name": "深度调研",
  "created_by": "user-uuid-1",
  "archive_enabled": false,
  "status": "active",
  "members": [
    {"user_id": "user-uuid-1", "role": "owner", "status": "active"},
    {"user_id": "user-uuid-2", "role": "participant", "status": "active"},
    {"user_id": "user-uuid-3", "role": "participant", "status": "active"}
  ],
  "created_at": "2026-04-06T10:00:00Z"
}
```

> `scenario` 为必填字段，可选值：`discussion`（自由讨论）、`deep_research`（深度调研）、`meeting_archive`（会议归档）、`learning_center`（学习中心）。未来可扩展更多预设场景。
>
> **并发限制**：同一个 Ring 同一时刻只能有一个活跃 Session。如果已有活跃 Session，创建新 Session 会返回 `409 Conflict`。
>
> **Session 暂停机制**：Session owner 离线时 Session 自动暂停（`session_paused` 事件推送给参与者），所有参与者无法发消息。Owner 重连后自动恢复（`session_resumed` 事件推送）。不设临时接管机制。

### 9.2 邀请成员加入 Session

```
POST /api/v1/rings/{ringId}/sessions/{sessionId}/invite
```

**请求体**：
```json
{
  "member_ids": ["user-uuid-4"]
}
```

**响应**：
```json
{
  "invited": [
    {"user_id": "user-uuid-4", "status": "active"}
  ]
}
```

> 前提：被邀请者必须是该 Ring 的已有成员（任意角色）。

### 9.3 获取 Session 列表

```
GET /api/v1/rings/{ringId}/sessions?status=active
```

**响应**：
```json
{
  "sessions": [
    {
      "id": "session-uuid",
      "title": "竞品 A 深度讨论",
      "created_by": "user-uuid-1",
      "member_count": 3,
      "archive_enabled": false,
      "status": "active",
      "created_at": "2026-04-06T10:00:00Z"
    }
  ]
}
```

### 9.4 获取 Session 详情

```
GET /api/v1/rings/{ringId}/sessions/{sessionId}
```

### 9.5 获取 Session 消息历史

```
GET /api/v1/rings/{ringId}/sessions/{sessionId}/messages?after_seq={lastSeqNum}&limit=50
```

**响应**：
```json
{
  "messages": [
    {
      "id": "msg-uuid",
      "sender_id": "user-uuid-1",
      "role": "user",
      "content": "大家觉得竞品 A 的核心优势是什么？",
      "seq_num": 1,
      "created_at": "2026-04-06T10:01:00Z"
    },
    {
      "id": "msg-uuid-2",
      "sender_id": null,
      "role": "assistant",
      "content": "根据已有的竞品分析数据...",
      "seq_num": 2,
      "created_at": "2026-04-06T10:01:05Z"
    }
  ]
}
```

> 离线重连时：客户端发送 `after_seq` 为本地最后收到的 `seq_num`，服务端返回该值之后的所有消息。

### 9.6 发送 Session 消息（SSE 流式响应）

```
POST /api/v1/rings/{ringId}/sessions/{sessionId}/messages
```

**请求体**：
```json
{
  "content": "大家觉得竞品 A 的核心优势是什么？"
}
```

**响应**（流式 SSE，同对话 API）：
```
data: {"type": "session_message", "sender_id": "user-uuid-1", "content": "...", "seq_num": 1}
data: {"type": "session_ring_response", "content": "根据已有的竞品分析数据...", "seq_num": 2}
data: {"type": "done"}
```

> Group Ring 回复对所有 session 成员可见（通过 WebSocket 广播）。

### 9.7 切换归档开关

```
PUT /api/v1/rings/{ringId}/sessions/{sessionId}/archive-toggle
```

**请求体**：
```json
{
  "archive_enabled": true
}
```

> 仅 session owner 可调用。开启后，session 内的对话内容可被触发归档流程（同 Ring 归档流程，仅 session owner 可触发）。

### 9.8 Session 归档

```
POST /api/v1/rings/{ringId}/sessions/{sessionId}/archive
```

**请求体**：
```json
{
  "message_ids": ["msg-uuid-1", "msg-uuid-2"],
  "graph_id": "graph-uuid-1",
  "target_node_id": "node-uuid-1",
  "label": "竞品 A 核心优势讨论"
}
```

> 前提：`archive_enabled` 为 true。仅 session owner 可触发。归档流程同 Ring 归档（AI 分析 → 推荐图谱操作 → 确认 → Git commit/PR）。

### 9.9 离开 Session

```
POST /api/v1/rings/{ringId}/sessions/{sessionId}/leave
```

### 9.10 关闭 Session

```
POST /api/v1/rings/{ringId}/sessions/{sessionId}/close
```

> 仅 session owner。关闭后 session 状态变为 `closed`，消息记录保留在 SQLite。成员不可再发消息。

### 9.11 删除 Session

```
DELETE /api/v1/rings/{ringId}/sessions/{sessionId}
```

> 仅 session owner。删除后 session 及所有消息记录从 SQLite 清除，不可恢复。

---

## 10. Super Ring API（全局助手） `[部分实现]`

> Super Ring 是 Ring Hub 级的全局 AI 助手，具备跨 Ring 分析、问答、总结、合并能力。按需只读访问本机所有 Ring 内容。

### 10.1 Super Ring 对话

```
POST /api/v1/super-ring/chat
```

**请求体**：
```json
{
  "message": "帮我创建一个竞品分析的 Ring"
}
```

**响应**：流式 SSE，同对话 API。

### 10.2 跨 Ring 分析 `⚠️ 计划中`

```
POST /api/v1/super-ring/analyze
```

**请求体**：
```json
{
  "ring_ids": ["ring-uuid-1", "ring-uuid-2"],
  "query": "对比两个团队对竞品 A 的分析结论"
}
```

**响应**：流式 SSE，Super Ring 按需读取相关 Ring 的图谱和归档内容进行分析。

### 10.3 跨 Ring 总结 `⚠️ 计划中`

```
POST /api/v1/super-ring/summarize
```

**请求体**：
```json
{
  "ring_ids": ["ring-uuid-1", "ring-uuid-2", "ring-uuid-3"],
  "topic": "本周各团队的关键决策"
}
```

### 10.4 跨 Ring 合并推荐 `⚠️ 计划中`

```
POST /api/v1/super-ring/merge-suggest
```

**请求体**：
```json
{
  "source_ring_id": "ring-uuid-1",
  "target_ring_id": "ring-uuid-2"
}
```

**响应**：
```json
{
  "suggestions": [
    {
      "source_node": {"ring_id": "ring-uuid-1", "node_id": "node-1", "label": "竞品 A 功能分析"},
      "target_node": {"ring_id": "ring-uuid-2", "node_id": "node-5", "label": "竞品 A 市场策略"},
      "reason": "两个节点都分析了竞品 A，内容互补",
      "merge_type": "link"
    }
  ]
}
```

> Super Ring 只生成推荐，合并操作由用户在具体 Ring 中执行。

---

## 11. Export API（导出中心） `[计划中]`

> 提供七种导出选项，支持 Ring 内容的多种输出形式。

### 11.1 导出图谱为图片 `⚠️ 计划中`

```
POST /api/v1/rings/{ringId}/exports/graph-image
```

**请求体**：
```json
{
  "graph_id": "graph-uuid-1",
  "format": "svg",
  "options": {
    "layout": "force",
    "show_labels": true,
    "max_depth": null
  }
}
```

> `format` 可选：`png`、`svg`、`pdf`。返回对应格式的二进制文件。

### 11.2 导出单篇 Markdown `⚠️ 计划中`

```
GET /api/v1/rings/{ringId}/exports/markdown/{nodeId}
```

> 返回该节点对应的 Markdown 文件内容，以 `attachment` 形式下载。

### 11.3 导出对话记录 `⚠️ 计划中`

```
POST /api/v1/rings/{ringId}/exports/conversation
```

**请求体**：
```json
{
  "conversation_id": "conv-uuid",
  "format": "markdown",
  "include_ai_responses": true
}
```

> `format` 可选：`markdown`、`pdf`。

### 11.4 导出 AI 结构化报告 `⚠️ 计划中`

```
POST /api/v1/rings/{ringId}/exports/report
```

**请求体**：
```json
{
  "topic": "竞品 A 综合分析",
  "node_ids": ["node-uuid-1", "node-uuid-2"],
  "graph_id": "graph-uuid-1",
  "format": "pdf"
}
```

> Group Ring 基于指定节点的内容，生成结构化报告。`format` 可选：`markdown`、`pdf`。

### 11.5 导出 Session 讨论记录 `⚠️ 计划中`

```
POST /api/v1/rings/{ringId}/exports/session
```

**请求体**：
```json
{
  "session_id": "session-uuid",
  "format": "markdown"
}
```

> `format` 可选：`markdown`、`pdf`。

### 11.6 导出整 Ring 数据备份 `⚠️ 计划中`

```
POST /api/v1/rings/{ringId}/exports/backup
```

**响应**：返回一个 `.tar.gz` 压缩包，包含：
- 所有 graph.json 文件
- 所有 nodes/ 目录下的 Markdown 文件
- blueprint.json
- SQLite 数据库快照（元数据、成员、对话历史）
- assets/ 目录

### 11.7 导出 graph.json 原始数据 `⚠️ 计划中`

```
GET /api/v1/rings/{ringId}/exports/graph-json/{graphId}
```

> 直接返回该图谱的 graph.json 文件内容。

---

## 12. 设置 API `[已实现]`

### 12.1 获取设置

```
GET /api/v1/settings
```

### 12.2 更新设置

```
PUT /api/v1/settings
```

**请求体**：
```json
{
  "llm": {
    "provider": "openai",
    "model": "gpt-4",
    "api_key": "sk-xxx"
  },
  "privacy": {
    "enabled": true,
    "rules": ["email", "phone", "id_card"]
  }
}
```

---

## 13. WebSocket `[已实现]`

### 13.1 连接

```
ws://{host}:7420/api/v1/ws?ringId={ringId}
```

### 13.2 消息类型

| 类型 | 方向 | 描述 |
|------|------|------|
| `chat_message` | 客户端 → 服务端 | 发送对话消息 |
| `chat_response` | 服务端 → 客户端 | AI 响应（流式） |
| `graph_update` | 服务端 → 客户端 | 图谱变更通知 |
| `pr_notification` | 服务端 → 客户端 | PR 状态变更通知 |
| `member_update` | 服务端 → 客户端 | 成员变更通知 |
| `mode_change` | 客户端 → 服务端 | 切换交互模式（chat/archive/auto） |
| `mode_changed` | 服务端 → 客户端 | 模式切换确认 |

### 13.3 Session WebSocket 消息类型

> Session 消息通过 Session owner 的后端 WebSocket hub 中转。成员连接到 Session owner 后端的 session WebSocket 频道。

| 类型 | 方向 | 描述 |
|------|------|------|
| `session_join` | 客户端 → 服务端 | 加入 session 频道 |
| `session_message` | 双向 | Session 内成员消息（广播给所有参与者） |
| `session_ring_response` | 服务端 → 客户端 | Group Ring 回复（广播给所有参与者） |
| `session_member_joined` | 服务端 → 客户端 | 新成员加入通知 |
| `session_member_left` | 服务端 → 客户端 | 成员离开通知 |
| `session_archive_toggled` | 服务端 → 客户端 | 归档开关变更通知 |
| `session_closed` | 服务端 → 客户端 | Session 关闭通知 |

**Session WebSocket 连接**：
```
ws://{sessionOwnerIP}:7420/api/v1/ws/sessions/{sessionId}?token={ringMemberToken}
```
