# Session 协作测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| 创建 Session | `POST /api/rings/{ring_id}/sessions` | title、skill、archivable | session |
| 参与者 | `/participants` | target ids | participants |
| 材料准备 | `/material-prep` | item_type、title、content | material |
| 开始讨论 | `/start` | 无 | phase=discussion |
| 总结 | `/summarize` | 无 | SSE、summary |
| 关闭/重开 | `/close`、`/reopen` | 无 | phase |
| 所有权转移 | `/transfer-ownership` | new_owner_id | status |

## 用例

### SC-01 创建 Decision Session

- 前置条件：Ring 存在。
- 输入：

```json
{"title":"Q3 技术架构选型讨论","skill":"decision","archivable":true}
```

- 步骤：Creator 提交创建 Session。
- 期望输出：HTTP 201，owner 为 Creator，phase 为 `material_prep` 或当前实现默认阶段。
- 问题记录：

### SC-02 单 Ring 活跃 Session 限制

- 前置条件：已有一个未关闭 Session。
- 输入：再次创建另一个 Session。
- 步骤：提交 `POST /sessions`。
- 期望输出：HTTP 409 或明确冲突提示。
- 问题记录：

### SC-03 添加和编辑材料

- 前置条件：Session 处于 material_prep。
- 输入：

```json
{"item_type":"text","title":"技术选型背景","content":"团队需要在 Rust Axum、Node.js NestJS、Python FastAPI 中选择后端框架。"}
```

- 步骤：创建材料，随后更新标题和内容。
- 期望输出：材料列表展示最新内容。
- 问题记录：

### SC-04 邀请参与者并移除

- 前置条件：Carol 已是 Ring 成员。
- 输入：Carol token_id。
- 步骤：邀请 Carol 参加 Session，再移除。
- 期望输出：participants 列表先包含 Carol，移除后不包含。
- 问题记录：

### SC-05 开始讨论并获取消息

- 前置条件：Session 已有材料。
- 输入：无。
- 步骤：调用 `/start`，通过 WebSocket 或 UI 发送 Session 讨论文本，再 GET messages。
- 期望输出：phase=discussion；消息按 seq 递增返回。
- 问题记录：

### SC-06 总结并关闭

- 前置条件：Session 处于 discussion 且有消息。
- 输入：无。
- 步骤：Owner 调用 `/summarize`。
- 期望输出：SSE 正常；summary 写入；phase 最终变为 closed。
- 问题记录：

### SC-07 转移所有权

- 前置条件：Carol 是 Session participant。
- 输入：

```json
{"new_owner_id":"<carol-token>"}
```

- 步骤：Creator 调用 transfer ownership。
- 期望输出：返回 `status=transferred`；Carol 成为 owner。
- 问题记录：
