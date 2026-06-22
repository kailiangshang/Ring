# Self 记忆与隐私测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| Identity | `/self/identity` | content | exists、content |
| Style | `/self/style` | content | exists、content |
| Personality | `/self/personality` | content | exists、content |
| Privacy | `/self/privacy` | settings | privacy |
| Memory | `/self/memory/{name}` | content | memory |
| Metrics | `/self/metrics`、heartbeat | view | metrics |
| Export/Reset | `/self/export`、`/self/reset` | 无 | export/reset result |

## 用例

### SMP-01 身份设定 CRUD

- 前置条件：用户已登录。
- 输入：

```json
{"content":"我是 Alice 的私人 AI 助手，关注技术决策、风险和个人偏好。"}
```

- 步骤：PUT `/self/identity`，再 GET。
- 期望输出：`exists=true`，content 一致。
- 问题记录：

### SMP-02 风格和人格设定

- 前置条件：用户已登录。
- 输入：简洁中文、风险优先、避免冗长。
- 步骤：分别更新 `/self/style` 和 `/self/personality`。
- 期望输出：GET 返回持久化内容。
- 问题记录：

### SMP-03 记忆列表与单条记忆

- 前置条件：Self 目录可写。
- 输入：

```json
{"content":"用户偏好：先列风险，再列方案。"}
```

- 步骤：PUT `/self/memory/preferences`，GET list 和 GET 单条。
- 期望输出：列表显示 preferences exists；单条内容一致。
- 问题记录：

### SMP-04 Self Chat 自动记忆提取

- 前置条件：LLM 可用。
- 输入：复制 Self 私人记忆文本。
- 步骤：提交 `/self/chat`，等待后台提取，再查看 memory。
- 期望输出：相关偏好被写入记忆；邮箱等隐私按过滤设置处理。
- 问题记录：

### SMP-05 Metrics heartbeat

- 前置条件：用户已登录。
- 输入：

```json
{"view":"ring_chat"}
```

- 步骤：POST `/self/metrics/heartbeat`，再 GET `/self/metrics`。
- 期望输出：HTTP 204 或成功；停留时间增加。
- 问题记录：

### SMP-06 Export 与 Reset

- 前置条件：已有 identity/memory/metrics 数据。
- 输入：无。
- 步骤：GET `/self/export`，随后在可接受的数据环境中 POST `/self/reset`。
- 期望输出：导出包含 Self 数据；reset 后数据清空或回到默认状态。
- 问题记录：
