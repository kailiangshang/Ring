# Ring API Design

Date: 2026-04-17

## 1. Overview

### 1.1 基础约定

- **Base URL**: `http://localhost:7420/api`
- **协议**: REST (CRUD) + WebSocket (实时通信)
- **格式**: JSON，字段名 `snake_case`
- **认证**: 本地优先，请求头 `X-Ring-Token: <token_id>` 标识用户身份
- **ID 策略**: 所有 ID 使用 ULID
- **时间格式**: ISO 8601 (`2026-04-17T08:30:00Z`)
- **错误响应**:
```json
{
  "error": {
    "code": "ring_not_found",
    "message": "Ring xxx does not exist"
  }
}
```
- **HTTP 状态码**: 200 成功, 201 创建, 204 删除成功, 400 参数错误, 403 权限不足, 404 不存在, 409 冲突, 500 内部错误

### 1.2 API 域划分

| 域 | 路径前缀 | 说明 |
|-----|---------|------|
| Setup | `/api/setup` | 初始化向导（身份/LLM/GitLab） |
| Ring | `/api/rings` | Ring CRUD 和管理 |
| Graph | `/api/rings/{ring_id}/graphs` | 图谱节点/边操作 |
| Chat | `/api/rings/{ring_id}/chat` | Group Ring 对话 |
| Archive | `/api/rings/{ring_id}/archive` | 归档流程 |
| Members | `/api/rings/{ring_id}/members` | 成员管理 |
| Invitations | `/api/rings/{ring_id}/invitations` | 邀请链接 |
| PR | `/api/rings/{ring_id}/prs` | PR 审核 |
| Session | `/api/rings/{ring_id}/sessions` | Session 管理 |
| Self | `/api/self` | Self 浮窗对话 |
| Super Ring | `/api/super/chat` | Super Ring 对话 |
| Skills | `/api/skills` | Skill 管理 |
| Export | `/api/rings/{ring_id}/export` | 导出 |
| Notifications | `/api/notifications` | 通知 |
| Config | `/api/config` | 全局配置（LLM/模式） |
| Blueprint | `/api/rings/{ring_id}/blueprint` | 蓝图管理 |

---

## 2. Setup

首次使用向导。Setup 完成后可跳过。

### 2.1 检查 Setup 状态

```
GET /api/setup/status
```

**Response**:
```json
{
  "is_setup": false,
  "step": null
}
```

### 2.2 提交 Setup

```
POST /api/setup
```

**Request**:
```json
{
  "display_name": "Kai",
  "avatar": "🦊",
  "llm_provider": "openai",
  "llm_api_key": "sk-xxx",
  "llm_model": "gpt-4o",
  "llm_base_url": null,
  "gitlab_url": "https://gitlab.company.com",
  "gitlab_token": "glpat-xxx"
}
```

| 字段 | 必需 | 说明 |
|------|------|------|
| `display_name` | 是 | 显示名 |
| `avatar` | 否 | emoji 或 null（用首字母） |
| `llm_provider` | 是 | `openai` / `anthropic` / `ollama` |
| `llm_api_key` | 条件 | ollama 不需要，其他必需 |
| `llm_model` | 否 | 模型名，不填用默认 |
| `llm_base_url` | 否 | 自定义 API 地址（Ollama 等） |
| `gitlab_url` | 是 | GitLab 地址 |
| `gitlab_token` | 是 | GitLab PAT |

**Response**: `201` 用户创建成功
```json
{
  "token_id": "user-001",
  "display_name": "Kai",
  "avatar": "🦊"
}
```

### 2.3 更新 Setup

```
PUT /api/setup
```

同 request body，覆盖更新。

---

## 3. Ring

### 3.1 列出 Rings

```
GET /api/rings
```

**Response**:
```json
{
  "rings": [
    {
      "id": "01JTYXXX",
      "name": "竞品分析组",
      "role": "creator",
      "member_count": 5,
      "node_count": 13,
      "last_activity_at": "2026-04-17T08:00:00Z",
      "has_active_session": false
    }
  ]
}
```

### 3.2 创建 Ring

```
POST /api/rings
```

**Request**:
```json
{
  "name": "竞品分析组",
  "role_description": "你是一个产品分析专家，帮助团队进行竞品研究",
  "gitlab_repo_url": null,
  "gitlab_namespace": null
}
```

| 字段 | 必需 | 说明 |
|------|------|------|
| `name` | 是 | Ring 名称 |
| `role_description` | 是 | Group Ring 角色描述 |
| `gitlab_repo_url` | 否 | 已有仓库地址，不填则自动创建 |
| `gitlab_namespace` | 否 | 自动创建时的 namespace |

**Response**: `201`
```json
{
  "id": "01JTYXXX",
  "name": "竞品分析组",
  "role": "creator",
  "blueprint_status": "pending"
}
```

### 3.3 获取 Ring 详情

```
GET /api/rings/{ring_id}
```

**Response**:
```json
{
  "id": "01JTYXXX",
  "name": "竞品分析组",
  "role": "creator",
  "role_description": "你是一个产品分析专家...",
  "member_count": 5,
  "node_count": 13,
  "blueprint_status": "confirmed",
  "interaction_mode": "normal",
  "skill_permission_mode": "plan",
  "created_at": "2026-04-15T00:00:00Z"
}
```

### 3.4 加入 Ring

```
POST /api/rings/join
```

**Request**:
```json
{
  "token": "base64url-token-xxx",
  "creator_ip": "192.168.1.100",
  "display_name": "Alice"
}
```

**Response**: `200`
```json
{
  "ring_id": "01JTYXXX",
  "ring_name": "竞品分析组",
  "role": "member",
  "token_id": "user-005"
}
```

**审核链接流程**（加入申请）:

**Request**:
```json
{
  "token": "base64url-token-xxx",
  "creator_ip": "192.168.1.100",
  "display_name": "Alice",
  "reason": "我是产品团队的新成员，需要查看竞品资料"
}
```

**Response**: `200`（需要审核时）
```json
{
  "status": "pending_approval",
  "ring_name": "竞品分析组"
}
```

### 3.5 审核加入申请

```
POST /api/rings/{ring_id}/join-requests/{request_id}/approve
POST /api/rings/{ring_id}/join-requests/{request_id}/reject
```

**Reject Request**:
```json
{
  "reason": "目前不需要新增成员"
}
```

### 3.6 列出加入申请

```
GET /api/rings/{ring_id}/join-requests?status=pending
```

### 3.7 安装导航页（创建者托管）

```
GET /ring/join?token=xxx
```

非 API 路径，由创建者 ring-server 提供的 HTML 页面。检测 User-Agent，显示 Ring 信息、下载按钮、"继续加入"链接。

---

## 4. Graph

### 4.1 列出图谱

```
GET /api/rings/{ring_id}/graphs
```

**Response**:
```json
{
  "graphs": [
    {
      "id": "01JTYXXX",
      "name": "竞品分析图谱",
      "node_count": 13,
      "edge_count": 8,
      "updated_at": "2026-04-17T08:00:00Z"
    }
  ]
}
```

### 4.1a 创建图谱

```
POST /api/rings/{ring_id}/graphs
```

**Request**:
```json
{
  "name": "第二图谱"
}
```

**限制**: 每 Ring 最多 3 个图谱。

**Response**: `201`
```json
{
  "id": "01JTYG2",
  "name": "第二图谱",
  "node_count": 0,
  "edge_count": 0
}
```

### 4.1b 删除图谱

```
DELETE /api/rings/{ring_id}/graphs/{graph_id}
```

**限制**: 不能删除 Ring 的最后一个图谱。删除时级联删除所有节点和边。

**Response**: `204`

### 4.2 获取完整图谱

```
GET /api/rings/{ring_id}/graphs/{graph_id}
```

**Response**: 返回完整 `graph.json` 结构（见 PRD 2.3）。

### 4.3 创建节点

```
POST /api/rings/{ring_id}/graphs/{graph_id}/nodes
```

**Request**:
```json
{
  "label": "竞品 A 功能对比",
  "parent_id": "01JTYPARENT",
  "node_type": "leaf",
  "tags": ["竞品", "功能对比"],
  "content": "# 竞品 A 功能对比\n\n详细内容..."
}
```

**Response**: `201`
```json
{
  "id": "01JTYNEW",
  "label": "竞品 A 功能对比",
  "parent_id": "01JTYPARENT",
  "markdown_path": "nodes/竞品分析/竞品A功能对比.md",
  "node_type": "leaf",
  "tags": ["竞品", "功能对比"],
  "created_at": "2026-04-17T08:30:00Z",
  "updated_at": "2026-04-17T08:30:00Z"
}
```

### 4.4 更新节点

```
PUT /api/rings/{ring_id}/graphs/{graph_id}/nodes/{node_id}
```

**Request**:
```json
{
  "label": "竞品 A 功能对比 v2",
  "tags": ["竞品", "功能对比", "更新"],
  "content": "# 更新后的内容..."
}
```

### 4.5 删除节点

```
DELETE /api/rings/{ring_id}/graphs/{graph_id}/nodes/{node_id}
```

### 4.6 创建边

```
POST /api/rings/{ring_id}/graphs/{graph_id}/edges
```

**Request**:
```json
{
  "source_id": "01JTYSRC",
  "target_id": "01JTYTGT",
  "relation": "depends_on",
  "label": "依赖"
}
```

### 4.7 删除边

```
DELETE /api/rings/{ring_id}/graphs/{graph_id}/edges/{edge_id}
```

### 4.8 搜索节点/标签

```
GET /api/rings/{ring_id}/graphs/{graph_id}/search?q=竞品&type=all
```

**Query params**:

| 参数 | 说明 |
|------|------|
| `q` | 搜索关键词（模糊匹配 label 和 tag） |
| `type` | `all` / `node` / `tag` |

**Response**:
```json
{
  "nodes": [...],
  "tags": ["竞品", "功能对比"]
}
```

---

## 5. Chat — Group Ring 对话

### 5.1 发送消息

```
POST /api/rings/{ring_id}/chat
```

**Request**:
```json
{
  "content": "帮我看看 #竞品分析 里最近的内容",
  "mentions": ["ring"],
  "node_refs": ["01JTYXXX"],
  "tag_refs": ["竞品分析"]
}
```

前端解析 `@`、`#` 后，将结构化数据一并提交。

| 字段 | 说明 |
|------|------|
| `content` | 原始输入文本 |
| `mentions` | `@` 寻址列表：`self` / `ring` / `super` / `username` |
| `node_refs` | `#` 引用的节点 ID 列表 |
| `tag_refs` | `#` 引用的标签列表 |

**Response**: `200`（AI 回复通过 SSE 流式返回）

```
Content-Type: text/event-stream

event: message_start
data: {"message_id": "01JTYMSG", "role": "group_ring"}

event: delta
data: {"content": "根据"}

event: delta
data: {"content": "#竞品分析"}

event: delta
data: {"content": " 节点的内容..."}

event: message_end
data: {"message_id": "01JTYMSG", "usage": {"prompt_tokens": 500, "completion_tokens": 200}}
```

### 5.2 获取对话历史

```
GET /api/rings/{ring_id}/chat/history?before={message_id}&limit=50
```

**Response**:
```json
{
  "messages": [
    {
      "id": "01JTYMSG",
      "role": "user",
      "sender_name": "Kai",
      "content": "帮我看看 #竞品分析 里最近的内容",
      "node_refs": ["01JTYXXX"],
      "tag_refs": ["竞品分析"],
      "created_at": "2026-04-17T08:30:00Z"
    },
    {
      "id": "01JTYMSG2",
      "role": "group_ring",
      "sender_name": "GROUP RING",
      "content": "根据 #竞品分析 节点的内容...",
      "created_at": "2026-04-17T08:30:05Z"
    }
  ],
  "has_more": true
}
```

### 5.3 压缩上下文

```
POST /api/rings/{ring_id}/chat/compact
```

**Response**:
```json
{
  "status": "compacting",
  "message": "正在压缩对话上下文..."
}
```

### 5.4 切换会话模式

```
PUT /api/rings/{ring_id}/chat/session-mode
```

**Request**:
```json
{
  "mode": "ephemeral"
}
```

`mode` 取值：`storage`（持久） / `ephemeral`（临时）

---

## 6. Archive

### 6.1 触发归档

```
POST /api/rings/{ring_id}/archive
```

**Request**:
```json
{
  "message_ids": ["01JTYMSG", "01JTYMSG2"],
  "content": "归档的文本内容（可由前端从消息拼接）"
}
```

**Response**: AI 分析后返回推荐
```json
{
  "status": "recommending",
  "recommendations": [
    {
      "action": "create_node",
      "graph_id": "01JTYG1",
      "parent_id": "01JTYPARENT",
      "label": "竞品 A 功能对比",
      "preview": "# 预览 Markdown 内容..."
    }
  ]
}
```

### 6.2 确认归档

```
POST /api/rings/{ring_id}/archive/confirm
```

**Request**:
```json
{
  "archive_id": "01JTYARCH",
  "accepted": true,
  "modifications": {
    "parent_id": "01JTYOTHER",
    "label": "修改后的节点名"
  }
}
```

**Response**:
```json
{
  "status": "archived",
  "node_id": "01JTYNEW",
  "git_action": "commit",
  "pr_id": null
}
```

或（成员提交 PR）：
```json
{
  "status": "archived",
  "node_id": "01JTYNEW",
  "git_action": "pr",
  "pr_id": "01JTYPR"
}
```

### 6.3 获取归档历史

```
GET /api/rings/{ring_id}/archive/history?limit=20
```

---

## 7. Members

### 7.1 列出成员

```
GET /api/rings/{ring_id}/members
```

**Response**:
```json
{
  "members": [
    {
      "token_id": "user-001",
      "display_name": "Kai",
      "avatar": "🦊",
      "role": "creator",
      "joined_at": "2026-04-15T00:00:00Z",
      "online": true
    }
  ]
}
```

### 7.2 变更角色

```
PUT /api/rings/{ring_id}/members/{token_id}/role
```

**Request**:
```json
{
  "role": "admin"
}
```

`role` 取值：`admin` / `member` / `readonly`

### 7.3 移除成员

```
DELETE /api/rings/{ring_id}/members/{token_id}
```

**Request**（如果该成员是 session owner）:
```json
{
  "session_owner_successor": "user-003"
}
```

### 7.4 授权成员创建 Session

```
POST /api/rings/{ring_id}/members/{token_id}/grant-session
POST /api/rings/{ring_id}/members/{token_id}/revoke-session
```

**权限**: 仅 creator/admin 可授予，仅 creator 可撤销。

**Response**: `200`
```json
{
  "status": "granted"
}
```

### 7.5 Session 所有权转移

```
POST /api/rings/{ring_id}/sessions/{session_id}/transfer-ownership
```

**Request**:
```json
{
  "new_owner_id": "user-003"
}
```

**权限**: 仅 creator。新 owner 必须是该 session 的参与者。

**Response**: `200`
```json
{
  "status": "transferred",
  "session_id": "01JTYSESS",
  "new_owner": "user-003"
}
```

### 7.6 移除成员（Session ownership 保护）

```
DELETE /api/rings/{ring_id}/members/{token_id}
```

如果目标成员拥有活动的 session，返回 `409`：
```json
{
  "error": {
    "code": "has_active_sessions",
    "message": "Cannot remove member: owns sessions [ses_xxx, ses_yyy]"
  }
}
```

---

## 8. Invitations

### 8.1 创建邀请链接

```
POST /api/rings/{ring_id}/invitations
```

**Request**:
```json
{
  "type": "open",
  "role": "member",
  "max_uses": 10,
  "expires_in_hours": 24,
  "max_members": null
}
```

| 字段 | 说明 |
|------|------|
| `type` | `open`（直接加入）/ `audit`（需审核） |
| `role` | 被邀请人角色：`admin` / `member` / `readonly` |
| `max_uses` | 最大使用次数，`null` 不限 |
| `expires_in_hours` | 有效期（小时） |
| `max_members` | Ring 人数上限，`null` 不限 |

**Response**: `201`
```json
{
  "id": "01JTYINV",
  "token": "base64url-random-32bytes",
  "join_url": "http://192.168.1.100:7420/ring/join?token=base64url-random-32bytes",
  "expires_at": "2026-04-18T08:00:00Z"
}
```

### 8.2 列出邀请链接

```
GET /api/rings/{ring_id}/invitations
```

### 8.3 撤销邀请链接

```
DELETE /api/rings/{ring_id}/invitations/{invitation_id}
```

---

## 9. PR 审核队列

### 9.1 列出 PR

```
GET /api/rings/{ring_id}/prs?status=open
```

**Response**:
```json
{
  "prs": [
    {
      "id": "01JTYPR",
      "title": "添加竞品 A 功能对比节点",
      "author": {
        "token_id": "user-003",
        "display_name": "Alice"
      },
      "status": "open",
      "queue_position": 1,
      "created_at": "2026-04-17T08:00:00Z",
      "changes": {
        "files_added": 1,
        "files_modified": 1,
        "nodes_added": ["01JTYNEW"],
        "graph_json_modified": true
      }
    }
  ]
}
```

### 9.2 获取 PR Diff

```
GET /api/rings/{ring_id}/prs/{pr_id}/diff
```

**Response**:
```json
{
  "pr_id": "01JTYPR",
  "diff": [
    {
      "file": "nodes/竞品分析/竞品A功能对比.md",
      "status": "added",
      "content": "--- /dev/null\n+++ b/nodes/..."
    },
    {
      "file": "graphs/main.json",
      "status": "modified",
      "content": "--- a/graphs/main.json\n+++ b/graphs/main.json\n..."
    }
  ]
}
```

### 9.3 审批 PR

```
POST /api/rings/{ring_id}/prs/{pr_id}/approve
POST /api/rings/{ring_id}/prs/{pr_id}/reject
```

**Reject Request**:
```json
{
  "reason": "与已有节点 XX 冲突，请 pull 最新后重新提交"
}
```

---

## 10. Session

### 10.1 创建 Session

```
POST /api/rings/{ring_id}/sessions
```

**Request**:
```json
{
  "title": "竞品 A 深度讨论",
  "description": "讨论竞品 A 的最新功能更新",
  "skill": "decision",
  "archivable": true,
  "invitees": ["user-002", "user-003"]
}
```

| 字段 | 说明 |
|------|------|
| `skill` | 场景类型：`decision` / `research` / `review` / `retrospective` / `knowledge_sharing` / `discussion` |
| `archivable` | 是否开启归档能力 |
| `invitees` | 受邀 Ring 成员 token_id 列表 |

**Response**: `201`
```json
{
  "id": "01JTYSESS",
  "title": "竞品 A 深度讨论",
  "skill": "decision",
  "phase": "material_prep",
  "owner": "user-001",
  "participants": ["user-001", "user-002", "user-003"],
  "created_at": "2026-04-17T08:00:00Z"
}
```

### 10.2 列出 Sessions

```
GET /api/rings/{ring_id}/sessions?status=active
```

`status` 取值：`active` / `closed` / `all`

### 10.3 获取 Session 详情

```
GET /api/rings/{ring_id}/sessions/{session_id}
```

### 10.4 关闭 / 重新打开 / 删除 Session

```
POST /api/rings/{ring_id}/sessions/{session_id}/close
POST /api/rings/{ring_id}/sessions/{session_id}/reopen
DELETE /api/rings/{ring_id}/sessions/{session_id}
```

### 10.5 获取 Session 历史消息

```
GET /api/rings/{ring_id}/sessions/{session_id}/messages?after_seq=42&limit=50
```

**Response**:
```json
{
  "messages": [
    {
      "seq_num": 43,
      "sender": "user-002",
      "sender_name": "Alice",
      "content": "我觉得竞品 A 的定价策略有变化",
      "created_at": "2026-04-17T08:05:00Z"
    }
  ],
  "has_more": false
}
```

### 10.6 Session 邀请/移除成员

```
POST /api/rings/{ring_id}/sessions/{session_id}/participants
DELETE /api/rings/{ring_id}/sessions/{session_id}/participants/{token_id}
```

**Invite Request**:
```json
{
  "token_ids": ["user-004"]
}
```

### 10.7 Session 归档开关

```
PUT /api/rings/{ring_id}/sessions/{session_id}/archive-toggle
```

**Request**:
```json
{
  "enabled": true
}
```

### 10.8 触发 Session 总结

```
POST /api/rings/{ring_id}/sessions/{session_id}/summarize
```

**Response**（SSE 流式）: AI 生成总结报告。

### 10.9 材料准备进度

```
GET /api/rings/{ring_id}/sessions/{session_id}/material-prep
```

**Response**:
```json
{
  "phase": "material_prep",
  "progress": "collecting",
  "items": [
    {
      "type": "document",
      "title": "竞品 A 最新财报",
      "status": "collected"
    },
    {
      "type": "graph_node",
      "title": "#竞品分析 相关节点",
      "status": "analyzing"
    }
  ],
  "highlights": ["user-001 标记了竞品 A 定价数据"]
}
```

### 10.10 标记材料重点

```
POST /api/rings/{ring_id}/sessions/{session_id}/material-prep/highlights
```

**Request**:
```json
{
  "item_index": 0,
  "note": "重点关注定价数据"
}
```

---

## 11. WebSocket

统一 WebSocket 端点，通过 `type` 字段区分消息类型。

### 11.1 连接

```
WS /api/ws?token=<user_token_id>
```

### 11.2 消息类型

#### 发送消息

```json
{
  "type": "session_message",
  "session_id": "01JTYSESS",
  "content": "我觉得竞品 A 的定价策略有变化"
}
```

#### 接收广播消息

```json
{
  "type": "session_message",
  "session_id": "01JTYSESS",
  "seq_num": 44,
  "sender": "user-002",
  "sender_name": "Alice",
  "content": "我觉得竞品 A 的定价策略有变化",
  "created_at": "2026-04-17T08:05:01Z"
}
```

#### 离线补发

```json
{
  "type": "session_catchup",
  "session_id": "01JTYSESS",
  "messages": [...]
}
```

#### Session 暂停/恢复

```json
{
  "type": "session_paused",
  "session_id": "01JTYSESS",
  "reason": "owner_offline"
}
```

```json
{
  "type": "session_resumed",
  "session_id": "01JTYSESS"
}
```

#### 通知推送

```json
{
  "type": "notification",
  "notification": {
    "id": "01JTYNOTIF",
    "category": "pr",
    "title": "新 PR 待审核",
    "body": "Alice 提交了归档 PR",
    "ring_id": "01JTYRING",
    "created_at": "2026-04-17T08:10:00Z"
  }
}
```

#### 成员被踢

```json
{
  "type": "session_member_kicked",
  "session_id": "01JTYSESS"
}
```

#### AI 流式回复（Session 中）

```json
{
  "type": "session_ai_delta",
  "session_id": "01JTYSESS",
  "message_id": "01JTYMSG",
  "content": "根据材料分析..."
}
```

```json
{
  "type": "session_ai_end",
  "session_id": "01JTYSESS",
  "message_id": "01JTYMSG"
}
```

---

## 12. Self

### 12.1 发送消息给 Self

```
POST /api/self/chat
```

**Request**:
```json
{
  "content": "帮我总结一下最近的工作"
}
```

**Response**: SSE 流式，同 Group Ring 格式。

### 12.2 获取 Self 对话历史

```
GET /api/self/chat/history?before={message_id}&limit=50
```

### 12.3 获取 Self 设置

```
GET /api/self/settings
```

**Response**:
```json
{
  "identity": "我是你的个人AI助手...",
  "style": "友好、简洁",
  "llm_config": {
    "provider": "openai",
    "model": "gpt-4o"
  },
  "autonomy_level": "suggest"
}
```

### 12.4 更新 Self 设置

```
PUT /api/self/settings
```

**Request**:
```json
{
  "identity": "更新后的身份描述",
  "style": "专业、详细",
  "autonomy_level": "auto"
}
```

---

## 13. Super Ring

### 13.1 发送消息给 Super Ring

```
POST /api/super/chat
```

**Request**:
```json
{
  "content": "帮我对比一下三个 Ring 里关于 AI 的讨论",
  "context_ring_ids": ["01JTYR1", "01JTYR2", "01JTYR3"]
}
```

**Response**: SSE 流式。

### 13.2 获取 Super Ring 对话历史

```
GET /api/super/chat/history?before={message_id}&limit=50
```

### 13.3 跨 Ring 查询

```
POST /api/super/cross-ring-query
```

**Request**:
```json
{
  "query": "AI 相关讨论",
  "ring_ids": ["01JTYR1", "01JTYR2"]
}
```

**Response**: `200` 查询结果。

### 13.4 跨 Ring 分析

```
POST /api/super/cross-ring-analysis
```

**Request**:
```json
{
  "analysis_type": "compare",
  "ring_ids": ["01JTYR1", "01JTYR2", "01JTYR3"]
}
```

**Response**: `200` 分析报告。

---

## 14. Skills

### 14.1 列出已安装 Skills

```
GET /api/skills
```

**Response**:
```json
{
  "skills": [
    {
      "name": "decision",
      "description": "团队决策：收集材料 → 讨论 → 决策结论 + 行动项",
      "source": "builtin",
      "installed_at": null
    },
    {
      "name": "custom-skill",
      "description": "自定义 Skill",
      "source": "user",
      "installed_at": "2026-04-17T08:00:00Z"
    }
  ]
}
```

### 14.2 安装 Skill

```
POST /api/skills/install
```

**Request**:
```json
{
  "name": "custom-skill",
  "source_url": "https://example.com/skills/custom-skill"
}
```

### 14.3 卸载 Skill

```
DELETE /api/skills/{skill_name}
```

---

## 15. Export

### 15.1 触发导出

```
POST /api/rings/{ring_id}/export
```

**Request**:
```json
{
  "type": "graph",
  "format": "svg",
  "graph_id": "01JTYG1",
  "node_ids": null
}
```

| type | format 选项 | 额外参数 |
|------|------------|---------|
| `graph` | `png` / `svg` / `pdf` | `graph_id` |
| `md` | `md` | `node_id` |
| `chat` | `md` / `pdf` | `message_ids` (可选，null 导出全部) |
| `report` | `md` / `pdf` | `node_ids`, `topic` |
| `session` | `md` / `pdf` | `session_id` |
| `backup` | `tar.gz` | 无 |
| `json` | `json` | `graph_id` |

**Response**:
```json
{
  "export_id": "01JTYEXP",
  "status": "processing"
}
```

### 15.2 下载导出文件

```
GET /api/rings/{ring_id}/export/{export_id}/download
```

返回二进制文件流，浏览器直接下载。

### 15.3 生成 AI 报告（流式）

```
GET /api/rings/{ring_id}/export/report?node_ids={id1,id2}&topic={topic}
```

**Query params**:

| 参数 | 说明 |
|------|------|
| `node_ids` | 逗号分隔的节点 ID 列表 |
| `topic` | 报告主题 |

**Response**: SSE 流式生成 Markdown 报告。

---

## 16. Notifications

### 16.1 列出通知

```
GET /api/notifications?unread=true&limit=20
```

**Response**:
```json
{
  "notifications": [
    {
      "id": "01JTYNOTIF",
      "category": "pr",
      "title": "新 PR 待审核",
      "body": "Alice 提交了归档 PR",
      "ring_id": "01JTYRING",
      "read": false,
      "created_at": "2026-04-17T08:10:00Z"
    }
  ],
  "unread_count": 3
}
```

### 16.2 标记已读

```
PUT /api/notifications/{id}/read
PUT /api/notifications/read-all
```

---

## 17. Config — 全局配置

### 17.1 获取 LLM 配置

```
GET /api/config/llm
```

**Response**:
```json
{
  "provider": "openai",
  "model": "gpt-4o",
  "api_key_set": true,
  "base_url": null
}
```

### 17.2 更新 LLM 配置

```
PUT /api/config/llm
```

**Request**:
```json
{
  "provider": "anthropic",
  "model": "claude-sonnet-4-20250514",
  "api_key": "sk-ant-xxx",
  "base_url": null
}
```

### 17.3 获取 Ring 模式

```
GET /api/rings/{ring_id}/mode
```

**Response**:
```json
{
  "interaction_mode": "normal",
  "skill_permission_mode": "plan"
}
```

### 17.4 设置 Ring 模式

```
PUT /api/rings/{ring_id}/mode
```

**Request**:
```json
{
  "interaction_mode": "auto",
  "skill_permission_mode": "plan"
}
```

---

## 18. Blueprint

### 18.1 获取蓝图

```
GET /api/rings/{ring_id}/blueprint
```

**Response**:
```json
{
  "status": "confirmed",
  "graphs": [
    {
      "id": "01JTYG1",
      "name": "竞品分析图谱",
      "structure": { }
    }
  ],
  "template": "product-research",
  "created_at": "2026-04-15T00:00:00Z"
}
```

### 18.2 从模板创建蓝图

```
POST /api/rings/{ring_id}/blueprint/from-template
```

**Request**:
```json
{
  "template": "product-research"
}
```

可用模板：`product-research` / `project-management` / `learning-notes` / `technical-docs` / `blank`

**Response**: 返回预览图谱结构
```json
{
  "preview": {
    "graphs": [
      {
        "name": "竞品分析图谱",
        "nodes": [...],
        "edges": [...]
      }
    ]
  }
}
```

### 18.3 确认蓝图

```
POST /api/rings/{ring_id}/blueprint/confirm
```

### 18.4 自定义蓝图（对话式）

通过 Group Ring 对话构建，使用 Chat API（`POST /api/rings/{ring_id}/chat`），AI 自动识别蓝图构建意图。

---

## 19. .group/ 文档

### 19.1 获取 .group/ 文档

```
GET /api/rings/{ring_id}/group-docs/{doc_name}
```

`doc_name` 取值：`role` / `conventions` / `active-context` / `archive-patterns` / `corrections` / `knowledge-summary`

### 19.2 更新 .group/ 文档

```
PUT /api/rings/{ring_id}/group-docs/{doc_name}
```

**Request**:
```json
{
  "content": "# Role\n你是一个产品分析专家..."
}
```

仅创建者/管理员可写 `role` 和 `conventions`。其余 4 个文档由 AI 自动维护（通过后端内部调用）。

---

## 20. API 与 CLI 命令映射

| CLI 命令 | API 调用 |
|----------|---------|
| `@self <msg>` | `POST /api/self/chat` |
| `@ring <msg>` | `POST /api/rings/{id}/chat` (mention=ring) |
| `@super <msg>` | `POST /api/super/chat` |
| `#<node>` | 前端补全 → `GET /api/rings/{id}/graphs/{gid}/search` |
| `!graph` | 前端 UI 动作（打开面板），无 API |
| `!save` | `POST /api/rings/{id}/archive` |
| `!export <type>` | `POST /api/rings/{id}/export` |
| `!auto` | `PUT /api/rings/{id}/mode` |
| `!compact` | `POST /api/rings/{id}/chat/compact` |
| `!ephemeral` | `PUT /api/rings/{id}/chat/session-mode` |
| `!session new` | `POST /api/rings/{id}/sessions` |
| `!session close` | `POST /api/rings/{id}/sessions/{sid}/close` |
| `!session reopen <id>` | `POST /api/rings/{id}/sessions/{sid}/reopen` |
| `!invite` | `POST /api/rings/{id}/invitations` |
| `!members` | `GET /api/rings/{id}/members` |
| `!pr` | `GET /api/rings/{id}/prs` |
| `!pr approve <id>` | `POST /api/rings/{id}/prs/{pid}/approve` |
| `!pr reject <id>` | `POST /api/rings/{id}/prs/{pid}/reject` |
| `%skill list` | `GET /api/skills` |
| `%skill install <n>` | `POST /api/skills/install` |
| `%skill remove <n>` | `DELETE /api/skills/{name}` |
| `%role` | `GET /api/rings/{id}/group-docs/role` |
| `%conventions` | `GET /api/rings/{id}/group-docs/conventions` |
| `%blueprint` | `GET /api/rings/{id}/blueprint` |
| `%mode auto` | `PUT /api/rings/{id}/mode` |
| `%mode skill <m>` | `PUT /api/rings/{id}/mode` |
| `%llm` | `GET /api/config/llm` |
| `%llm set <p>` | `PUT /api/config/llm` |
