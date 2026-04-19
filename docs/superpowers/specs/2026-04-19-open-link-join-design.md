# 开放链接加入流程 设计

> **Affects**: `server/src/models/invite.rs`, `server/src/services/invite.rs`, `server/src/routes/invite.rs`, `server/src/routes/mod.rs`
> **Depends on**: `invite_tokens` 表（已实现）, `users` 表, `members` 表, `rings` 表
> **Last verified**: 2026-04-19

## 1. 概述

开放链接（open）的加入流程后端 API。包含两层端点：

- **创建者 ring-server 的公开端点**（无需 X-Ring-Token）：验证 invite token、完成加入（在创建者 DB 创建 user + member）
- **加入者本地 ring-server 的代理端点**：调用创建者公开端点 + clone 仓库

本子功能只做后端 API，不含前端页面和安装导航页。

### 1.1 架构

```
加入者浏览器          加入者 ring-server         创建者 ring-server
     │                      │                        │
     │  POST /api/join/local│                        │
     │─────────────────────>│                        │
     │                      │  GET /api/join/info    │
     │                      │  ?token=xxx            │
     │                      │───────────────────────>│
     │                      │  {ring_name, members}  │
     │                      │<───────────────────────│
     │                      │                        │
     │                      │  POST /api/join        │
     │                      │  {token, display_name} │
     │                      │───────────────────────>│
     │                      │  {token_id, role}      │
     │                      │<───────────────────────│
     │                      │                        │
     │                      │  git clone repo        │
     │                      │──────┐                 │
     │                      │<─────┘                 │
     │  { success }         │                        │
     │<─────────────────────│                        │
```

### 1.2 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 创建者端点是否需要 auth | 不需要，invite token 即认证 | 加入者还没有创建者系统的账户 |
| is_creator | 新建 user 时设为 false | 只有 setup 创建的第一个用户是 creator |
| LLM 配置 | 加入者用自己本地的 | 不需要在创建者 DB 存 LLM key |
| use_count 递增时机 | POST /api/join 成功时 | 防止 token 滥用 |
| max_uses=0 含义 | 不限次数 | PRD 定义 |

## 2. API 端点

### 2.1 GET /api/join/info?token=xxx

公开端点，无需 X-Ring-Token。创建者 ring-server 提供。

验证 invite token 并返回 Ring 基本信息。

**响应（成功）**：
```json
{
  "valid": true,
  "ring_id": "01JW...",
  "ring_name": "Backend Team",
  "member_count": 5,
  "role": "member",
  "token_type": "open"
}
```

**响应（无效/过期/已撤销）**：
```json
{
  "valid": false,
  "reason": "token expired"
}
```

### 2.2 POST /api/join

公开端点，无需 X-Ring-Token。创建者 ring-server 提供。

在创建者 DB 中创建 user + member，完成加入。

**请求体**：
```json
{
  "invite_token": "a1B2c3D4...",
  "display_name": "Bob"
}
```

**响应**：
```json
{
  "token_id": "user-01JW...",
  "ring_id": "01JW...",
  "ring_name": "Backend Team",
  "role": "member",
  "gitlab_repo_url": "https://gitlab.test.com/team/repo.git"
}
```

### 2.3 POST /api/join/local

需要 X-Ring-Token（加入者自己的 token）。加入者本地 ring-server 提供。

代理调用创建者的 `/api/join/info` 和 `/api/join`，然后 clone 仓库。

**请求体**：
```json
{
  "invite_token": "a1B2c3D4...",
  "creator_ip": "192.168.1.100"
}
```

**响应**：
```json
{
  "ok": true,
  "ring_id": "01JW...",
  "ring_name": "Backend Team",
  "role": "member"
}
```

## 3. 数据库操作

### 3.1 invite_tokens 表新增查询

| 函数 | 说明 |
|------|------|
| `find_valid_token(pool, token)` | 按 token 查找，返回 InviteTokenRow（不检查有效性） |
| `increment_use_count(pool, token)` | `UPDATE invite_tokens SET use_count = use_count + 1 WHERE token = ?` |
| `get_member_count(pool, ring_id)` | `SELECT COUNT(*) FROM members WHERE ring_id = ?` |

### 3.2 加入逻辑（POST /api/join）

1. 查找 token → 不存在返回 404
2. 检查 `type == 'open'` → 否则 400
3. 检查 `revoked_at IS NULL` → 已撤销 410
4. 检查 `expires_at > now` → 已过期 410
5. 检查 `max_uses == 0 || use_count < max_uses` → 已用完 403
6. 检查 `max_members`：`get_member_count(ring_id) < max_members` → 已满 403
7. 检查新 token_id 是否已是该 Ring 成员 → 已在则 409（防止重复调用）
8. 生成 `token_id = "user-{ULID}"`
9. 创建 user（`is_creator = false`，LLM 字段填空默认值）
10. 创建 member（`role = token.role`）
11. `increment_use_count`
12. 返回结果

### 3.3 创建者 DB 中的 user 记录

加入者在创建者 DB 中只需要最小信息：

```sql
INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, gitlab_url, gitlab_token)
VALUES (?1, ?2, NULL, 0, 'openai', NULL, 'gpt-4o', '', '')
```

LLM/GitLab 配置使用空默认值。加入者实际使用自己本地的配置。

## 4. 本地代理端点逻辑（POST /api/join/local）

1. 验证加入者 X-Ring-Token
2. 从请求体获取 `invite_token` 和 `creator_ip`
3. 构建创建者 URL：`http://{creator_ip}:7420/api/join/info?token={invite_token}`
4. 调用创建者 `/api/join/info` → 无效则返回错误
5. 获取加入者 display_name（从本地 DB）
6. 调用创建者 `POST /api/join`，传 `invite_token` + `display_name`
7. 如果响应中包含 `gitlab_repo_url`，异步 clone 仓库到本地
8. 返回成功

### 4.1 HTTP 客户端

使用已有的 `reqwest` crate（已在 Cargo.toml 中）。

## 5. 错误处理

| 场景 | HTTP 状态码 | 端点 |
|------|------------|------|
| token 不存在 | 404 Not Found | /api/join/info, /api/join |
| token 过期 | 410 Gone | /api/join/info, /api/join |
| token 已撤销 | 410 Gone | /api/join/info, /api/join |
| token 不是 open 类型 | 400 Bad Request | /api/join |
| max_uses 已用完 | 403 Forbidden | /api/join |
| Ring 成员已满（max_members） | 403 Forbidden | /api/join |
| 已经是该 Ring 成员 | 409 Conflict | /api/join |
| display_name 为空 | 400 Bad Request | /api/join |
| 创建者服务不可达 | 502 Bad Gateway | /api/join/local |
| creator_ip 缺失 | 400 Bad Request | /api/join/local |

### 5.1 新增 RingError 变体

```rust
#[error("Gone: {0}")]
Gone(String),
```

在 `IntoResponse` 中映射 `Gone` → `StatusCode::GONE`。

## 6. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/src/error.rs` | 修改 | 新增 `Gone` 变体 |
| `server/src/models/invite.rs` | 修改 | 新增 `find_valid_token`、`increment_use_count`、`get_member_count` |
| `server/src/services/invite.rs` | 修改 | 新增 `verify_join_token`、`execute_join`、`local_join` |
| `server/src/routes/invite.rs` | 修改 | 新增 3 个 handler |
| `server/src/routes/mod.rs` | 修改 | 注册 3 个新路由 |
| `server/tests/integration.rs` | 修改 | 新增 join 相关测试 |
