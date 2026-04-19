# 审核链接 + 审批流程 设计

> **Affects**: `server/migrations/`, `server/src/models/invite.rs`, `server/src/services/invite.rs`, `server/src/routes/invite.rs`, `server/src/routes/mod.rs`
> **Depends on**: `invite_tokens` 表, open link join flow（已实现）
> **Last verified**: 2026-04-19

## 1. 概述

审核链接（audit）的加入流程后端 API。申请人在创建者 ring-server 提交申请 → 创建者审批 → 批准后自动加入。申请人通过轮询查询审批状态。

本子功能只做后端 API，不含前端页面。

### 1.1 架构

```
申请人                创建者 ring-server              创建者
  │                        │                           │
  │ POST /api/join/apply   │                           │
  │───────────────────────>│                           │
  │ {request_id, status}   │                           │
  │<───────────────────────│                           │
  │                        │                           │
  │ GET /api/join/apply/status?id=xxx（轮询）          │
  │───────────────────────>│                           │
  │ {status: "pending"}    │                           │
  │<───────────────────────│                           │
  │                        │  GET /api/rings/{id}/join-requests
  │                        │<──────────────────────────│
  │                        │  [{pending requests}]     │
  │                        │──────────────────────────>│
  │                        │                           │
  │                        │  POST .../approve         │
  │                        │<──────────────────────────│
  │                        │  {token_id, ring_id}      │
  │                        │──────────────────────────>│
  │                        │                           │
  │ GET /api/join/apply/status?id=xxx                  │
  │───────────────────────>│                           │
  │ {status:"approved"}    │                           │
  │<───────────────────────│                           │
  │                        │                           │
  │ POST /api/join/local（clone repo）                 │
  │───────────────────────>│（加入者本地后端）           │
```

### 1.2 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 通知机制 | 轮询 | P2P 架构无双向连接，轮询最简单 |
| 批准时自动加入 | 是 | 创建 user + member，与 open join 一致 |
| 拒绝可附理由 | 是 | PRD 4.4.5 要求 |
| 申请 ID 格式 | `req-{ULID}` | 与 `user-{ULID}` 一致 |
| 同一 token 重复申请 | 允许（不同 display_name） | 每次 apply 创建新 request |

## 2. 数据库

### 2.1 新建 join_requests 表

```sql
CREATE TABLE join_requests (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id),
    invite_token TEXT NOT NULL REFERENCES invite_tokens(token),
    display_name TEXT NOT NULL,
    message TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'approved', 'rejected')),
    reviewer_id TEXT REFERENCES users(token_id),
    review_note TEXT,
    reviewed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 2.2 字段说明

| 字段 | 说明 |
|------|------|
| `id` | 主键，`req-{ULID}` |
| `ring_id` | 所属 Ring |
| `invite_token` | 关联的 invite token |
| `display_name` | 申请人名字 |
| `message` | 申请理由 |
| `status` | `pending` / `approved` / `rejected` |
| `reviewer_id` | 审批人 token_id |
| `review_note` | 审批备注（拒绝理由等） |
| `reviewed_at` | 审批时间 |

## 3. API 端点

### 3.1 POST /api/join/apply

公开端点。提交加入申请。

注意：Rust 代码中使用 `ApplyBody` 作为请求结构体名（避免与 open join 的 `JoinRequest` 冲突）。

**请求体**：
```json
{
  "invite_token": "a1B2c3D4...",
  "display_name": "Bob",
  "message": "我是后端团队的，想加入讨论"
}
```

**响应**：
```json
{
  "request_id": "req-01JW...",
  "status": "pending",
  "ring_name": "Backend Team"
}
```

**逻辑**：
1. 验证 invite token 存在、未过期、未撤销
2. 检查 `type == 'audit'`
3. 创建 join_request（status = pending）
4. 返回 request_id

### 3.2 GET /api/join/apply/status?id=xxx

公开端点。查询申请状态。供申请人轮询。

**响应**：
```json
{
  "request_id": "req-01JW...",
  "status": "approved",
  "ring_name": "Backend Team",
  "ring_id": "01JW...",
  "role": "member",
  "review_note": null,
  "token_id": "user-01JW..."
}
```

`token_id` 仅在 `status == "approved"` 时返回。申请人拿到 `token_id` 后调用本地 `/api/join/local` 完成 clone。

### 3.3 GET /api/rings/{ring_id}/join-requests

需要 X-Ring-Token（creator/admin）。列出待审批申请。

**响应**：
```json
{
  "requests": [
    {
      "id": "req-01JW...",
      "display_name": "Bob",
      "message": "我是后端团队的",
      "status": "pending",
      "created_at": "2026-04-19T12:00:00Z"
    }
  ]
}
```

**查询参数**：
- `status=pending` — 默认只看 pending
- `status=all` — 看所有

### 3.4 POST /api/rings/{ring_id}/join-requests/{request_id}/approve

需要 X-Ring-Token（creator/admin）。批准申请。

**响应**：
```json
{
  "ok": true,
  "token_id": "user-01JW...",
  "ring_name": "Backend Team",
  "role": "member"
}
```

**逻辑**：
1. 查找 request → 不存在返回 404
2. 检查 status == pending → 否则 409
3. 检查 invite token 仍然有效（未过期/撤销）
4. 检查 max_uses、max_members 约束
5. 创建 user（is_creator=false）+ member
6. 更新 request status = approved
7. 递增 use_count
8. 返回结果

### 3.5 POST /api/rings/{ring_id}/join-requests/{request_id}/reject

需要 X-Ring-Token（creator/admin）。拒绝申请。

**请求体**（可选）：
```json
{
  "note": "当前团队已满"
}
```

**响应**：
```json
{
  "ok": true
}
```

## 4. 错误处理

| 场景 | HTTP 状态码 | 端点 |
|------|------------|------|
| invite token 不存在 | 404 | /api/join/apply |
| invite token 过期/已撤销 | 410 | /api/join/apply |
| invite token 不是 audit 类型 | 400 | /api/join/apply |
| display_name 为空 | 400 | /api/join/apply |
| request 不存在 | 404 | /status, /approve, /reject |
| request 不是 pending | 409 | /approve, /reject |
| 非 creator/admin | 403 | list, approve, reject |
| token 已失效（审批时） | 410 | /approve |
| max_uses 已用完 | 403 | /approve |
| Ring 成员已满 | 403 | /approve |

## 5. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/migrations/008_join_requests.sql` | 新建 | 建表 |
| `server/src/models/invite.rs` | 修改 | 新增 JoinRequestRow + SQL 查询 |
| `server/src/services/invite.rs` | 修改 | 新增 apply/approve/reject/list 服务 |
| `server/src/routes/invite.rs` | 修改 | 新增 5 个 handler |
| `server/src/routes/mod.rs` | 修改 | 注册 5 个新路由 |
| `server/tests/integration.rs` | 修改 | 新增审批流程测试 |
