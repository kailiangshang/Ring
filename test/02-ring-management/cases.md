# Ring 管理测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| 创建 Ring | `POST /api/rings` | name、role_description、storage_mode | id、name、role |
| Ring 列表 | `GET /api/rings` | token | rings 数组 |
| Ring 详情 | `GET /api/rings/{ring_id}` | ring_id | Ring detail |
| 删除 Ring | `DELETE /api/rings/{ring_id}` | ring_id | ok |
| 模式设置 | `GET/PUT /api/rings/{ring_id}/mode` | interaction_mode、auto_archive | mode 状态 |

## 用例

### RM-01 创建本地 Ring

- 前置条件：Creator 已登录。
- 输入：

```json
{"name":"技术架构讨论","role_description":"你是技术架构组长，负责记录架构决策。","storage_mode":"local"}
```

- 步骤：提交 `POST /api/rings`。
- 期望输出：HTTP 201，返回 `role=creator`，列表中可见该 Ring。
- 问题记录：

### RM-02 获取 Ring 详情

- 前置条件：RM-01 已创建 Ring。
- 输入：`ring_id`。
- 步骤：访问 `GET /api/rings/{ring_id}`。
- 期望输出：HTTP 200，名称和角色描述正确。
- 问题记录：

### RM-03 修改交互模式和自动归档

- 前置条件：Ring 存在。
- 输入：

```json
{"interaction_mode":"auto","auto_archive":true}
```

- 步骤：提交 `PUT /api/rings/{ring_id}/mode`，再 GET 验证。
- 期望输出：返回 `interaction_mode=auto`，`auto_archive=true`。
- 问题记录：

### RM-04 非成员访问 Ring

- 前置条件：准备一个未加入 Ring 的用户 token。
- 输入：非成员 token。
- 步骤：访问 `GET /api/rings/{ring_id}`。
- 期望输出：HTTP 403 或 404，不能泄露敏感内容。
- 问题记录：

### RM-05 删除 Ring

- 前置条件：使用专门测试 Ring，避免删除有效数据。
- 输入：`ring_id`。
- 步骤：Creator 访问 `DELETE /api/rings/{ring_id}`。
- 期望输出：HTTP 200，返回 `ok=true`，列表不再显示。
- 问题记录：
