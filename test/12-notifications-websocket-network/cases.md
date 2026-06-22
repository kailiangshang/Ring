# 通知、WebSocket 与网络测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| 通知列表 | `GET /api/notifications` | token | notifications |
| 未读数 | `/notifications/unread-count` | token | count |
| 标记已读 | `/notifications/{id}/read` | notification_id | ok |
| 全部已读 | `/notifications/read-all` | token | ok |
| 删除通知 | `DELETE /notifications/{id}` | notification_id | ok |
| WebSocket | `GET /api/ws` | session message | realtime messages |
| 网络信息 | `GET /api/network/info` | 无 | host/network info |

## 用例

### NWN-01 通知列表和未读数

- 前置条件：触发成员加入、Session 关闭或归档事件。
- 输入：Creator token。
- 步骤：GET notifications 和 unread-count。
- 期望输出：列表包含相关事件；未读数与列表状态一致。
- 问题记录：

### NWN-02 标记单条通知已读

- 前置条件：存在未读通知。
- 输入：notification_id。
- 步骤：POST `/notifications/{id}/read`，再 GET unread-count。
- 期望输出：该通知 read 状态更新；未读数减少。
- 问题记录：

### NWN-03 全部已读和删除通知

- 前置条件：存在多条通知。
- 输入：无和 notification_id。
- 步骤：POST read-all，再删除一条通知。
- 期望输出：未读数为 0；删除后列表不再包含该通知。
- 问题记录：

### NWN-04 WebSocket 建连

- 前置条件：Session 处于 discussion，准备 Creator 和 Member 两个客户端。
- 输入：`X-Ring-Token` 或当前实现支持的认证方式。
- 步骤：两个客户端连接 `/api/ws`。
- 期望输出：连接成功；无权限用户连接失败或无法订阅 Session。
- 问题记录：

### NWN-05 WebSocket Session 消息广播

- 前置条件：NWN-04 已连接。
- 输入：

```json
{"type":"message","session_id":"<session-id>","content":"我支持 Rust Axum，但需要补充团队培训计划。"}
```

- 步骤：Creator 发送消息，Member 观察接收；再由 Member 发送。
- 期望输出：参与者实时收到消息；消息可通过 `/sessions/{id}/messages` 查询。
- 问题记录：

### NWN-06 WebSocket 速率限制

- 前置条件：WebSocket 可连接。
- 输入：短时间连续连接或发送超过限制。
- 步骤：一分钟内发起超过 10 次连接/请求。
- 期望输出：触发 429 或连接被拒绝，并有明确错误。
- 问题记录：

### NWN-07 Network info

- 前置条件：服务运行。
- 输入：无。
- 步骤：GET `/api/network/info`。
- 期望输出：返回可用于本地加入/发现的网络信息；不泄露敏感 token。
- 问题记录：
