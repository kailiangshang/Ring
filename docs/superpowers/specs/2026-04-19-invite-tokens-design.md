# Invite Tokens API 设计

> **Affects**: `server/migrations/`, `server/src/models/invite.rs` (新建), `server/src/services/invite.rs` (新建), `server/src/routes/invite.rs` (新建), `server/src/routes/mod.rs`
> **Depends on**: `rings` 表, `users` 表, `members` 表
> **Last verified**: 2026-04-19

## 1. 概述

Ring 邀请系统的数据层和 CRUD API。创建者/管理员可以生成邀请 token、列出 token、撤销 token。token 分两种类型：开放链接（直接加入）和审核链接（需要审批）。

本子功能只做 token 的生成/列出/撤销，不做"加入流程"（加入流程在下一个子功能中实现）。

### 1.1 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| Token 格式 | base64url(random 32 bytes) | PRD 定义，不可猜测 |
| 默认有效期 | 24 小时 | PRD 定义 |
| 撤销方式 | 设置 revoked_at，不删除 | 保留审计记录 |
| 过期处理 | 查询时过滤 | 简单可靠 |
| 权限 | 创建者 + 管理员 | 与 Ring 级权限一致 |

## 2. 数据库

### 2.1 新建 invite_tokens 表

```sql
CREATE TABLE invite_tokens (
    token TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id),
    type TEXT NOT NULL CHECK(type IN ('open', 'audit')),
    role TEXT NOT NULL DEFAULT 'member' CHECK(role IN ('member', 'readonly')),
    max_uses INTEGER NOT NULL DEFAULT 1,
    use_count INTEGER NOT NULL DEFAULT 0,
    max_members INTEGER,
    expires_at TEXT NOT NULL,
    revoked_at TEXT,
    created_by TEXT NOT NULL REFERENCES users(token_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 2.2 字段说明

| 字段 | 说明 |
|------|------|
| `token` | 主键，base64url(random 32 bytes) |
| `ring_id` | 所属 Ring |
| `type` | `open`（直接加入）或 `audit`（需审批） |
| `role` | 加入后分配的角色：`member` 或 `readonly` |
| `max_uses` | 最大使用次数，0 = 不限 |
| `use_count` | 已使用次数 |
| `max_members` | Ring 最大人数上限，NULL = 不限 |
| `expires_at` | 过期时间 |
| `revoked_at` | 撤销时间，NULL = 未撤销 |
| `created_by` | 创建者 token_id |

## 3. API 端点

### 3.1 POST /api/rings/{ring_id}/invite-tokens

生成邀请 token。仅创建者和管理员可用。

**请求体**：
```json
{
  "type": "open",
  "role": "member",
  "max_uses": 0,
  "max_members": null,
  "expires_in_hours": 24
}
```

**响应**：
```json
{
  "token": "a1B2c3D4e5F6...",
  "type": "open",
  "role": "member",
  "max_uses": 0,
  "max_members": null,
  "expires_at": "2026-04-20T12:00:00Z",
  "created_at": "2026-04-19T12:00:00Z"
}
```

**默认值**：
- `type`: `open`
- `role`: `member`
- `max_uses`: `1`
- `expires_in_hours`: `24`

### 3.2 GET /api/rings/{ring_id}/invite-tokens

列出 Ring 的所有邀请 token（排除已过期和已撤销）。

**响应**：
```json
{
  "tokens": [
    {
      "token": "a1B2c3D4e5F6...",
      "type": "open",
      "role": "member",
      "max_uses": 0,
      "use_count": 3,
      "max_members": null,
      "expires_at": "2026-04-20T12:00:00Z",
      "revoked_at": null,
      "created_by": "user-001",
      "created_at": "2026-04-19T12:00:00Z"
    }
  ]
}
```

**查询参数**：
- `include_expired=true` — 包含已过期的 token（默认不包含）
- `include_revoked=true` — 包含已撤销的 token（默认不包含）

### 3.3 DELETE /api/rings/{ring_id}/invite-tokens/{token}

撤销邀请 token。仅创建者和管理员可用。

**响应**：
```json
{
  "ok": true,
  "revoked_at": "2026-04-19T15:00:00Z"
}
```

## 4. Token 生成逻辑

```rust
fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    base64_url::encode(&bytes)
}
```

需要添加 `rand` 和 `base64` 依赖。

## 5. 后端改动

### 5.1 新建文件

| 文件 | 说明 |
|------|------|
| `server/migrations/007_invite_tokens.sql` | 建表 |
| `server/src/models/invite.rs` | 数据模型 + SQL 查询 |
| `server/src/services/invite.rs` | 业务逻辑（生成/列出/撤销） |
| `server/src/routes/invite.rs` | API handler |

### 5.2 修改文件

| 文件 | 改动 |
|------|------|
| `server/src/models/mod.rs` | 添加 `pub mod invite` |
| `server/src/services/mod.rs` | 添加 `pub mod invite` |
| `server/src/routes/mod.rs` | 注册 3 个新路由 |

## 6. 权限检查

每个端点都需要验证：
1. 请求者是 Ring 的创建者或管理员
2. Ring 存在

检查方式：查询 `members` 表，确认 `role IN ('creator', 'admin')`。

## 7. 错误处理

| 场景 | 处理 |
|------|------|
| 非 Ring 成员 | 403 Forbidden |
| 成员但非创建者/管理员 | 403 Forbidden |
| Ring 不存在 | 404 Not Found |
| 撤销不存在的 token | 404 Not Found |
| 撤销已撤销的 token | 200 OK（幂等） |
| expires_in_hours 无效（<=0） | 400 Bad Request |

## 8. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/migrations/007_invite_tokens.sql` | 新建 | 建表 |
| `server/src/models/invite.rs` | 新建 | 模型 + SQL |
| `server/src/models/mod.rs` | 修改 | 添加 mod invite |
| `server/src/services/invite.rs` | 新建 | 业务逻辑 |
| `server/src/services/mod.rs` | 修改 | 添加 mod invite |
| `server/src/routes/invite.rs` | 新建 | 3 个 handler |
| `server/src/routes/mod.rs` | 修改 | 注册路由 |
| `server/Cargo.toml` | 修改 | 添加 rand + base64 依赖 |
| `server/tests/integration.rs` | 修改 | 新增 invite token 测试 |
