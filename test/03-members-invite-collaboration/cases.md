# 成员、邀请与协作测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| 成员列表 | `GET /api/rings/{ring_id}/members` | ring_id | members |
| 添加成员 | `POST /api/rings/{ring_id}/members` | user_id | member row |
| 修改角色 | `PUT /api/rings/{ring_id}/members/{target_id}/role` | role | updated member |
| Session 权限 | grant/revoke session | target_id | ok |
| 邀请 token | `/invite-tokens` | type、role、限制 | token |
| 加入申请 | `/join/apply` | invite_token、display_name、message | request_id、status |
| 审批申请 | approve/reject | request_id | status |

## 用例

### MIC-01 Creator 查看成员列表

- 前置条件：Ring 已创建。
- 输入：Creator token。
- 步骤：访问 `GET /api/rings/{ring_id}/members`。
- 期望输出：至少包含 Creator，角色为 `creator`。
- 问题记录：

### MIC-02 添加已有用户为 Member

- 前置条件：数据库中存在 Carol Member 用户。
- 输入：

```json
{"user_id":"<carol-token>"}
```

- 步骤：Creator 提交 `POST /api/rings/{ring_id}/members`。
- 期望输出：HTTP 200，Carol 角色为 `member`；重复添加返回 409。
- 问题记录：

### MIC-03 角色变更

- 前置条件：Carol 已是成员。
- 输入：

```json
{"role":"admin"}
```

- 步骤：Creator 提交 `PUT /api/rings/{ring_id}/members/{carol}/role`。
- 期望输出：Carol 角色更新为 `admin`。
- 问题记录：

### MIC-04 普通成员不能移除 Creator

- 前置条件：Carol 为 member。
- 输入：Carol token。
- 步骤：Carol 访问 `DELETE /api/rings/{ring_id}/members/{creator}`。
- 期望输出：HTTP 403。
- 问题记录：

### MIC-05 Open 邀请加入

- 前置条件：Creator 已登录。
- 输入：

```json
{"type":"open","role":"member","max_uses":3}
```

- 步骤：创建 invite token，访问 `/api/join/info?token=<token>`，再提交 `/api/join`。
- 期望输出：join info 显示 valid；新用户获得 token、ring_id、role。
- 问题记录：

### MIC-06 Audit 邀请申请审批

- 前置条件：创建 audit 类型邀请。
- 输入：

```json
{"invite_token":"<token>","display_name":"Dave Guest","message":"我想参与产品需求评审"}
```

- 步骤：提交 `/api/join/apply`，Creator 查看 join requests，分别测试 approve 和 reject。
- 期望输出：申请为 pending；approve 后获得 token；reject 后状态为 rejected 且带 review_note。
- 问题记录：

### MIC-07 撤销邀请 token

- 前置条件：存在未使用 invite token。
- 输入：token。
- 步骤：Creator 删除 `/api/rings/{ring_id}/invite-tokens/{token}`，再查询 join info。
- 期望输出：删除返回 ok；join info 显示 invalid 或不可加入。
- 问题记录：
