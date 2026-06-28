# 归档、Git 与同步测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| AI 归档 | `POST /api/rings/{ring_id}/archive` | title/content | SSE |
| 快速归档 | `/archive/quick` | content/title | archive record |
| 归档列表 | `/archives` | ring_id | archives |
| Review | `/archives/{id}/review` | action、comment | status |
| Diff | `/archives/{id}/diff` | archive_id | diff |
| Repo | `/repo/status/init/git-log/revert` | commit | repo info |
| Sync | `/sync/bundle`、`/rings/sync/import` | bundle | import result |

## 用例

### AGS-01 初始化归档仓库

- 前置条件：Ring 存在，storage_mode 为 local。
- 输入：无。
- 步骤：GET repo status，POST repo init，再 GET status。
- 期望输出：初始化前 `initialized=false`，初始化后为 true。
- 问题记录：

### AGS-02 快速归档 Markdown

- 前置条件：repo 已初始化。
- 输入：复制 `test/test-data.md` 的归档 Markdown。
- 步骤：提交 `/archive/quick`。
- 期望输出：返回 archive record；archives 列表可见；content 能读取 Markdown。
- 问题记录：

### AGS-03 AI 归档流程

- 前置条件：LLM 可用，聊天历史存在。
- 输入：

```json
{"title":"Q3 技术架构选型结论"}
```

- 步骤：调用 `POST /api/rings/{ring_id}/archive`。
- 期望输出：SSE progress 正常；生成待 review 或 committed 记录。
- 问题记录：

### AGS-04 Review merge/reject

- 前置条件：存在待审核 archive。
- 输入：

```json
{"action":"merge","comment":"确认归档"}
```

- 步骤：先测试 merge，再用另一个归档测试 reject。
- 期望输出：状态分别变为 merged/committed 或 rejected；reject 不应写入最终归档。
- 问题记录：

### AGS-05 Diff 与 Git log

- 前置条件：至少两次归档提交。
- 输入：archive_id。
- 步骤：访问 diff 和 git-log。
- 期望输出：diff 展示变更；git-log 有提交记录。
- 问题记录：

### AGS-06 Revert

- 前置条件：有可 revert 的 commit。
- 输入：commit hash。
- 步骤：调用 `/repo/revert`，再查看 content/log。
- 期望输出：生成 revert 结果；内容回退或新增 revert commit。
- 问题记录：

### AGS-07 Sync bundle/import

- 前置条件：源 Ring 有归档/图谱数据。
- 输入：bundle。
- 步骤：GET `/sync/bundle`，在另一个环境或清洁数据中 POST `/rings/sync/import`。
- 期望输出：导入后 Ring 数据可见；冲突策略按 creator-wins 或当前实现生效。
- 问题记录：
