# 导出、上传与配置测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| Chat 导出 | `/export/chat` | ring_id | Markdown |
| Graph 导出 | `/export/graph?format=` | json/svg/png | 文件/JSON |
| Backup 导出 | `/export/backup` | ring_id | tar.gz 或错误 |
| Node 导出 | `/export/node` | node_id | Markdown |
| Upload | `/upload`、`/upload/parse` | file | parsed content |
| LLM/GitLab 配置 | `/config/*` | config | status |
| Privacy Filters | `/config/privacy_filters` | filters | filters |

## 用例

### EUC-01 导出 Ring 聊天 Markdown

- 前置条件：Ring 有聊天记录。
- 输入：无。
- 步骤：GET `/api/rings/{ring_id}/export/chat`。
- 期望输出：HTTP 200，Content-Type 可读，内容包含聊天文本。
- 问题记录：

### EUC-02 导出图谱 JSON/SVG/PNG

- 前置条件：Ring 有节点和边。
- 输入：`format=json`、`format=svg`、`format=png`。
- 步骤：分别请求导出。
- 期望输出：JSON 返回 nodes/edges；SVG/PNG 返回可打开文件或明确错误。
- 问题记录：

### EUC-03 节点导出与路径穿越

- 前置条件：存在 node_id。
- 输入：合法 node_id 和 `../../../etc/passwd`。
- 步骤：分别访问 `/export/node?node_id=...`。
- 期望输出：合法节点返回 Markdown；非法路径返回 404/400，不能泄露文件。
- 问题记录：

### EUC-04 上传文本文件到 Ring

- 前置条件：准备 `architecture-notes.txt`。
- 输入文件内容：

```text
认证系统负责登录、Token 轮换和权限校验。Session 协作依赖成员角色和参与者列表。
```

- 步骤：上传到 `/api/rings/{ring_id}/upload`。
- 期望输出：系统消息包含文件名和内容摘要；搜索可查到关键词。
- 问题记录：

### EUC-05 上传限制

- 前置条件：准备超大文件和 `.exe` 文件名。
- 输入：超过 10MB 文件、`evil.exe`。
- 步骤：上传或调用解析。
- 期望输出：被拒绝，返回 400。
- 问题记录：

### EUC-06 LLM 配置测试

- 前置条件：用户已登录。
- 输入：

```json
{"provider":"openai","model":"gpt-4o","api_key":"sk-test"}
```

- 步骤：PUT `/config/llm`，GET 验证，POST `/config/llm/test`。
- 期望输出：配置保存；test 成功或返回明确外部服务错误。
- 问题记录：

### EUC-07 隐私过滤配置

- 前置条件：用户已登录。
- 输入：手机号、邮箱、API key 过滤规则。
- 步骤：PUT `/config/privacy_filters`，再提交隐私过滤样例到聊天。
- 期望输出：规则持久化；聊天发送内容被脱敏。
- 问题记录：
