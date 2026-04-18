# Auto Archive 设计

> **Affects**: `server/src/services/session.rs`, `server/src/services/archive_service.rs`, `server/src/models/session.rs`
> **Depends on**: Archive + Git/GitLab 集成（Plan 6 全部完成）、Session 生命周期
> **Last verified**: 2026-04-19

## 1. 概述

Session 关闭时，若 `interaction_mode=auto` 且 `archive_enabled=true`，后端自动触发归档流程：LLM 分析全部对话消息，提取多个归档单元（决策、结论、知识点），每个单元生成独立 Markdown 文件并提交到 Git 仓库。

与手动归档的区别：手动归档是用户在聊天中用 `!save <title>` 触发单个归档；自动归档是关闭 session 时批量提取多个归档单元，无需用户干预。

### 1.1 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 触发时机 | `close_session` 完成后异步 spawn | 不阻塞关闭响应，用户无感知延迟 |
| LLM 输出格式 | JSON 数组 `[{title, content}]` | 结构化，便于逐个处理 |
| 消息上限 | 100 条（超出截断最早的消息） | 控制 token 消耗，避免超长对话 |
| 纯聊天处理 | LLM 返回空数组 `[]` → 不归档 | 无实质内容不应产生空文件 |
| 错误策略 | 失败记 warning 日志，不影响关闭 | 自动归档是增强功能，不应阻断主流程 |
| 归档单元数 | 由 LLM 自行判断（0-N 个） | 不同对话内容密度不同 |
| 前端改动 | 无 | 纯后端功能，用户通过 ArchivePanel 查看结果 |

## 2. 触发条件

`close_session` 中，在 `update_phase("closed")` 成功后检查：

```
ring.interaction_mode == "auto" AND session.archive_enabled == true
```

两个条件同时满足才触发。任一不满足，走原有的关闭流程（无自动归档）。

### 2.1 所需数据

- `ring.interaction_mode`：从 `rings` 表查询（已有 `get_ring_detail` 或直接 query）
- `session.archive_enabled`：`SessionRow.archive_enabled` 字段，已在关闭时加载

## 3. 流程

```
close_session()
  ├── 权限检查（owner）
  ├── update_phase("closed")
  ├── 返回 SessionResponse（不等待归档）
  └── if interaction_mode == "auto" && archive_enabled:
        tokio::spawn(auto_archive_task)
```

### 3.1 auto_archive_task

```
auto_archive_task(state, ring_id, session_id, user_id, is_creator)
  ├── 1. 查询 session_messages（按 seq_num ASC，上限 100）
  ├── 2. 拼接消息文本（格式："[sender_name]: content"）
  ├── 3. 查询 ring 的 creator_id → 从 users 表获取 creator 的 UserRow（含 LLM 配置）
  ├── 4. LLM 分析 → 返回 JSON 数组 [{title, content}]
  ├── 5. 遍历数组，对每个单元调用 archive_content_creator（或 member）
  │     ├── 生成文件名（sanitize_filename）
  │     ├── 写入 Markdown
  │     ├── git add/commit/push
  │     └── 插入 archive_records
  └── 6. 完成（成功/失败均 log）
```

### 3.2 LLM Prompt

System prompt：

```
你是一个知识管理助手。分析以下讨论记录，提取值得长期保存的知识单元。

每个单元包含：
- title: 简短标题（用于文件名，不超过 30 字，不含特殊字符）
- content: Markdown 格式的完整归档内容

归档单元可以是：决策记录、结论总结、知识点、调研发现、方案对比等。
只提取有实质内容的单元。如果讨论内容没有值得归档的，返回空数组。

返回纯 JSON 数组，不要 markdown code block：
[{"title": "...", "content": "..."}]
```

User message：

```
Session 标题: {session.title}
Skill: {session.skill}

讨论记录：
{messages_text}
```

### 3.3 LLM 调用方式

不使用 `chat_stream`（流式），而是直接调用 `client.chat().create()` 获取完整响应，因为：

1. 自动归档是后台任务，不需要流式输出
2. 需要完整 JSON 才能解析
3. 避免 SSE channel 的额外复杂度

在 `LlmClient` 中新增 `chat_complete` 方法，返回 `String`（完整响应文本）。

### 3.4 消息拼接格式

```
[Alice]: 我们决定用 Rust + Axum 作为后端框架
[Bob]: 同意，性能和类型安全都不错
[Alice]: 前端用 React + TypeScript
[Session Ring]: 总结：团队确认了技术选型方案
```

`sender_name` 来自 `SessionMessageRow.sender_name`，`content` 来自 `SessionMessageRow.content`。

## 4. 数据变更

### 4.1 新增查询

在 `models/session.rs` 中新增：

```rust
pub async fn get_all_messages_ordered(
    pool: &SqlitePool,
    session_id: &str,
    limit: i64,
) -> Result<Vec<SessionMessageRow>>
```

查询：
```sql
SELECT * FROM session_messages
WHERE session_id = ?1
ORDER BY seq_num DESC
LIMIT ?2
```

注意：DESC + LIMIT 取最新的 N 条，然后在 Rust 中 reverse 得到时间正序。这样保证截断的是最早的消息。

### 4.2 LlmClient 新增方法

在 `services/llm.rs` 中新增：

```rust
pub async fn chat_complete(
    self,
    system_prompt: String,
    user_message: String,
) -> Result<String>
```

使用 `client.chat().create()` 一次性获取完整响应。

## 5. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/src/services/session.rs` | 修改 `close_session` | 关闭后 spawn 自动归档任务 |
| `server/src/services/archive_service.rs` | 新增 `auto_archive_session` | 自动归档业务逻辑 |
| `server/src/services/llm.rs` | 新增 `chat_complete` | 非流式 LLM 调用 |
| `server/src/models/session.rs` | 新增 `get_all_messages_ordered` | 查询 session 全部消息 |

无前端改动，无数据库 migration。

## 6. 错误处理

| 场景 | 处理 |
|------|------|
| LLM 请求失败 | log warning，停止归档 |
| LLM 返回非法 JSON | log warning + 原始响应，停止归档 |
| LLM 返回空数组 `[]` | 正常，无需归档，静默结束 |
| git 操作失败（某个单元） | log warning，跳过该单元，继续处理下一个 |
| 所有单元都失败 | log warning，不产生任何归档记录 |
| ring 没有 Git 仓库 | log warning，跳过归档 |

所有错误只记日志，不影响 session 关闭的成功响应。

## 7. 边界情况

| 情况 | 处理 |
|------|------|
| Session 消息数为 0 | 不调用 LLM，直接结束 |
| 消息超过 100 条 | 取最新 100 条（截断最早） |
| 文件名冲突 | `sanitize_filename` 含日期+标题，加上时间戳后缀避免冲突 |
| 同一 session 多次关闭 | `close_session` 已检查 `phase == "closed"` 返回错误 |
| Reopen 后再关闭 | 再次触发自动归档（生成新的归档文件） |
| 并发归档 | 每次关闭 spawn 独立 task，git 操作有 pull 确保最新 |

## 8. 日志

使用 `tracing` 宏记录关键节点：

- `info!("auto_archive started: session={session_id}, ring={ring_id}")`
- `info!("auto_archive extracted {} units", units.len())`
- `info!("auto_archive completed: session={session_id}, {} files created", success_count)`
- `warn!("auto_archive LLM failed: {error}")`
- `warn!("auto_archive unit failed: title={title}, error={error}")`

## 9. 不在范围内

- 前端自动归档进度通知（可后续通过 WebSocket 推送）
- 自动归档取消机制（spawn 后无法取消）
- 归档单元的 graph node 创建（后续迭代）
- 归档内容去重（后续迭代）
