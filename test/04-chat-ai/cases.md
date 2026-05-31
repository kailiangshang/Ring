# Chat AI 测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| Group Chat | `POST /api/rings/{ring_id}/chat` | content、node_refs、tag_refs、ephemeral | SSE |
| Group History | `GET /api/rings/{ring_id}/chat/history` | before、limit | messages、has_more |
| Self Chat | `POST /api/self/chat` | content | SSE |
| Super Chat | `POST /api/super/chat` | content | SSE |
| Compact | `/compact` | token | summary、removed_count |
| 删除消息 | `DELETE /messages/{message_id}` | message_id | 204 |

## 用例

### CA-01 Group Ring 基础对话

- 前置条件：Ring 已创建，LLM 配置可用。
- 输入：复制 `test/test-data.md` 中 Group Ring 文本。
- 步骤：提交 `POST /api/rings/{ring_id}/chat`。
- 期望输出：SSE 包含 `message_start`、`delta`、`message_end`；历史中出现 user 和 group_ring 消息。
- 问题记录：

### CA-02 归档意图识别

- 前置条件：Ring 已创建。
- 输入：复制归档意图文本。
- 步骤：提交 Group Chat。
- 期望输出：SSE 提示检测到归档意图；归档流程被触发；不应阻塞普通聊天。
- 问题记录：

### CA-03 Chat History 分页

- 前置条件：已产生超过 3 条聊天消息。
- 输入：`limit=2`。
- 步骤：访问 `GET /api/rings/{ring_id}/chat/history?limit=2`。
- 期望输出：最多 2 条消息；如还有更多则 `has_more=true`。
- 问题记录：

### CA-04 Self Chat 与 Group 隔离

- 前置条件：已完成 Group Chat。
- 输入：复制 Self 私人记忆文本。
- 步骤：提交 `POST /api/self/chat`，再分别查看 Self 和 Group 历史。
- 期望输出：Self 消息只出现在 Self 历史；Group 历史不包含私人文本。
- 问题记录：

### CA-05 Super Chat 跨 Ring 入口

- 前置条件：至少有两个 Ring。
- 输入：复制 Super Ring 跨 Ring 查询文本。
- 步骤：提交 `POST /api/super/chat`。
- 期望输出：SSE 正常；Super 历史记录消息；回答能引用多个 Ring 的上下文或说明无结果。
- 问题记录：

### CA-06 删除自己的消息

- 前置条件：已产生消息并拿到 message_id。
- 输入：message_id。
- 步骤：调用对应删除接口。
- 期望输出：HTTP 204；历史不再返回该消息。
- 问题记录：

### CA-07 隐私过滤

- 前置条件：配置隐私过滤规则。
- 输入：隐私过滤样例。
- 步骤：提交 Group/Self Chat。
- 期望输出：发送给 LLM 的内容应被脱敏；界面/历史按产品约定显示，若明文泄露需记录。
- 问题记录：
