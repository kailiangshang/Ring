# 数据模型 API 参考

> 源码路径：`ring-server/src/models/`

## User

### `User`
源文件：`models/user.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 用户唯一 ID |
| `display_name` | `String` | 显示名称 |
| `avatar_url` | `Option<String>` | 头像 URL |
| `ip_address` | `Option<String>` | 注册 IP 地址 |
| `setup_completed` | `bool` | 设置是否完成 |
| `created_at` | `String` | 创建时间（RFC3339） |

### `NewUser`
源文件：`models/user.rs:13`

| 字段 | 类型 | 说明 |
|------|------|------|
| `display_name` | `String` | 显示名称 |

---

## Ring

### `Ring`
源文件：`models/ring.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | Ring 唯一 ID |
| `name` | `String` | Ring 名称 |
| `description` | `Option<String>` | 描述 |
| `creator_id` | `String` | 创建者用户 ID |
| `gitlab_repo` | `String` | GitLab 仓库地址 |
| `local_path` | `String` | 本地仓库路径 |
| `next_token_id` | `i64` | 下一个 Token ID |
| `status` | `String` | 状态（active 等） |
| `created_at` | `String` | 创建时间 |
| `updated_at` | `String` | 更新时间 |

### `NewRing`
源文件：`models/ring.rs:17`

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | Ring 名称 |
| `description` | `Option<String>` | 描述 |
| `creator_id` | `String` | 创建者 ID |
| `gitlab_repo` | `String` | GitLab 仓库（`auto_create` 表示自动创建） |
| `namespace` | `Option<String>` | GitLab namespace |
| `role_description` | `String` | 角色描述 |

---

## Member

### `Member`
源文件：`models/member.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 成员 ID |
| `ring_id` | `String` | 所属 Ring ID |
| `user_id` | `String` | 用户 ID |
| `token_id` | `i64` | Token ID（群组内序号） |
| `display_name` | `String` | 显示名称 |
| `role` | `String` | 角色（creator/admin/member） |
| `joined_at` | `String` | 加入时间 |

### `NewMember`
源文件：`models/member.rs:14`

| 字段 | 类型 | 说明 |
|------|------|------|
| `ring_id` | `String` | Ring ID |
| `user_id` | `String` | 用户 ID |
| `display_name` | `String` | 显示名称 |
| `role` | `Option<String>` | 角色（默认 member） |

---

## InviteToken

### `InviteToken`
源文件：`models/invite.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 邀请 ID |
| `ring_id` | `String` | Ring ID |
| `token` | `String` | 邀请码 |
| `token_type` | `String` | 类型（open 等） |
| `role` | `String` | 授予的角色 |
| `inviter_id` | `String` | 邀请人 ID |
| `max_uses` | `i64` | 最大使用次数 |
| `use_count` | `i64` | 已使用次数 |
| `max_members` | `Option<i64>` | 最大成员数 |
| `expires_at` | `String` | 过期时间 |
| `used_at` | `Option<String>` | 使用时间 |
| `revoked_at` | `Option<String>` | 撤销时间 |
| `created_at` | `String` | 创建时间 |

---

## Conversation

### `Conversation`
源文件：`models/conversation.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 对话 ID |
| `ring_id` | `String` | Ring ID |
| `title` | `Option<String>` | 标题 |
| `mode` | `String` | 模式 |
| `context_mode` | `String` | 上下文模式 |
| `token_count` | `i64` | Token 计数 |
| `token_limit` | `i64` | Token 上限 |
| `auto_compact` | `bool` | 是否自动压缩 |
| `summary` | `Option<String>` | 摘要 |
| `compacted_at` | `Option<String>` | 上次压缩时间 |
| `created_by` | `String` | 创建者 |
| `created_at` | `String` | 创建时间 |
| `updated_at` | `String` | 更新时间 |

### `NewConversation`
源文件：`models/conversation.rs:20`

| 字段 | 类型 | 说明 |
|------|------|------|
| `ring_id` | `String` | Ring ID |
| `title` | `Option<String>` | 标题 |
| `context_mode` | `Option<String>` | 上下文模式 |
| `created_by` | `String` | 创建者 |

### `Message`
源文件：`models/conversation.rs:28`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 消息 ID |
| `conversation_id` | `String` | 所属对话 |
| `role` | `String` | 角色（user/assistant） |
| `content` | `String` | 内容 |
| `sender_id` | `Option<String>` | 发送者 ID |
| `tool_calls` | `Option<String>` | 工具调用 JSON |
| `archived` | `bool` | 是否已归档 |
| `created_at` | `String` | 创建时间 |

### `NewMessage`
源文件：`models/conversation.rs:40`

| 字段 | 类型 | 说明 |
|------|------|------|
| `conversation_id` | `String` | 对话 ID |
| `role` | `String` | 角色 |
| `content` | `String` | 内容 |
| `sender_id` | `Option<String>` | 发送者 ID |

---

## Session

### `Session`
源文件：`models/session_model.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | Session ID |
| `ring_id` | `String` | Ring ID |
| `title` | `Option<String>` | 标题 |
| `scenario` | `String` | 场景（discussion/deep_research/meeting_archive/learning_center） |
| `created_by` | `String` | 创建者 |
| `archive_enabled` | `bool` | 是否启用归档 |
| `status` | `String` | 状态（active/closed） |
| `created_at` | `String` | 创建时间 |
| `updated_at` | `String` | 更新时间 |

### `SessionMember`
源文件：`models/session_model.rs:17`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | ID |
| `session_id` | `String` | Session ID |
| `user_id` | `String` | 用户 ID |
| `role` | `String` | 角色（owner/participant） |
| `status` | `String` | 状态 |
| `joined_at` | `String` | 加入时间 |
| `left_at` | `Option<String>` | 离开时间 |

### `SessionMessage`
源文件：`models/session_model.rs:27`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 消息 ID |
| `session_id` | `String` | Session ID |
| `sender_id` | `String` | 发送者 ID |
| `role` | `String` | 角色 |
| `content` | `String` | 内容 |
| `seq_num` | `i64` | 序列号 |
| `created_at` | `String` | 创建时间 |

### `CreateSessionRequest`
源文件：`models/session_model.rs:38`

| 字段 | 类型 | 说明 |
|------|------|------|
| `title` | `Option<String>` | 标题 |
| `scenario` | `String` | 场景（必填） |
| `archive_enabled` | `Option<bool>` | 是否启用归档 |
| `invite_member_ids` | `Option<Vec<String>>` | 邀请成员 ID 列表 |

### `SessionDetailResponse`
源文件：`models/session_model.rs:46`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | Session ID |
| `ring_id` | `String` | Ring ID |
| `title` | `Option<String>` | 标题 |
| `scenario` | `String` | 场景 |
| `created_by` | `String` | 创建者 |
| `archive_enabled` | `bool` | 是否启用归档 |
| `status` | `String` | 状态 |
| `members` | `Vec<SessionMemberBrief>` | 成员列表 |
| `created_at` | `String` | 创建时间 |

### `SessionMemberBrief`
源文件：`models/session_model.rs:59`

| 字段 | 类型 | 说明 |
|------|------|------|
| `user_id` | `String` | 用户 ID |
| `role` | `String` | 角色 |
| `status` | `String` | 状态 |

### `SessionListResponse`
源文件：`models/session_model.rs:66`

| 字段 | 类型 | 说明 |
|------|------|------|
| `sessions` | `Vec<SessionListItem>` | Session 列表 |

### `SessionListItem`
源文件：`models/session_model.rs:71`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | Session ID |
| `title` | `Option<String>` | 标题 |
| `created_by` | `String` | 创建者 |
| `member_count` | `i64` | 成员数量 |
| `archive_enabled` | `bool` | 是否启用归档 |
| `status` | `String` | 状态 |
| `created_at` | `String` | 创建时间 |

### `InviteSessionRequest`
源文件：`models/session_model.rs:82`

| 字段 | 类型 | 说明 |
|------|------|------|
| `member_ids` | `Vec<String>` | 成员 ID 列表 |

### `ArchiveToggleRequest`
源文件：`models/session_model.rs:87`

| 字段 | 类型 | 说明 |
|------|------|------|
| `archive_enabled` | `bool` | 是否启用归档 |

### `SessionMessagesResponse`
源文件：`models/session_model.rs:92`

| 字段 | 类型 | 说明 |
|------|------|------|
| `messages` | `Vec<SessionMessage>` | 消息列表 |

---

## Blueprint

### `BlueprintTemplate`
源文件：`models/blueprint.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 模板 ID |
| `name` | `String` | 名称 |
| `description` | `Option<String>` | 描述 |
| `graphs` | `String` | 图谱 JSON |
| `is_system` | `bool` | 是否系统模板 |
| `created_by` | `Option<String>` | 创建者 |
| `created_at` | `String` | 创建时间 |

### `NewBlueprintTemplate`
源文件：`models/blueprint.rs:14`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 模板 ID |
| `name` | `String` | 名称 |
| `description` | `Option<String>` | 描述 |
| `graphs` | `String` | 图谱 JSON |
| `is_system` | `bool` | 是否系统模板 |

---

## Graph

### `CreateNodeRequest`
源文件：`models/graph_model.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `label` | `String` | 节点标签 |
| `node_type` | `String` | 节点类型 |
| `parent_id` | `Option<String>` | 父节点 ID |
| `description` | `Option<String>` | 描述 |

### `UpdateNodeRequest`
源文件：`models/graph_model.rs:11`

| 字段 | 类型 | 说明 |
|------|------|------|
| `label` | `Option<String>` | 标签 |
| `description` | `Option<String>` | 描述 |
| `node_type` | `Option<String>` | 节点类型 |

### `CreateEdgeRequest`
源文件：`models/graph_model.rs:18`

| 字段 | 类型 | 说明 |
|------|------|------|
| `source_id` | `String` | 源节点 ID |
| `target_id` | `String` | 目标节点 ID |
| `relation` | `String` | 关系类型 |
| `label` | `Option<String>` | 标签 |

### `NodeResponse`
源文件：`models/graph_model.rs:26`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 节点 ID |
| `label` | `String` | 标签 |
| `node_type` | `String` | 节点类型 |
| `parent_id` | `Option<String>` | 父节点 ID |
| `description` | `Option<String>` | 描述 |
| `graph_id` | `String` | 所属图 ID |
| `markdown_path` | `Option<String>` | Markdown 文件路径 |
| `created_at` | `String` | 创建时间 |
| `updated_at` | `String` | 更新时间 |

### `EdgeResponse`
源文件：`models/graph_model.rs:55`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 边 ID |
| `source_id` | `String` | 源节点 ID |
| `target_id` | `String` | 目标节点 ID |
| `relation` | `String` | 关系类型 |
| `label` | `Option<String>` | 标签 |
| `graph_id` | `String` | 所属图 ID |

### `GraphDetailResponse`
源文件：`models/graph_model.rs:78`

| 字段 | 类型 | 说明 |
|------|------|------|
| `graph_id` | `String` | 图 ID |
| `nodes` | `Vec<NodeResponse>` | 节点列表 |
| `edges` | `Vec<EdgeResponse>` | 边列表 |

### `NodeContentResponse`
源文件：`models/graph_model.rs:85`

| 字段 | 类型 | 说明 |
|------|------|------|
| `node_id` | `String` | 节点 ID |
| `label` | `String` | 标签 |
| `markdown_path` | `Option<String>` | Markdown 路径 |
| `content` | `Option<String>` | Markdown 内容 |
| `last_modified` | `String` | 最后修改时间 |

### `SearchResult`
源文件：`models/graph_model.rs:94`

| 字段 | 类型 | 说明 |
|------|------|------|
| `node_id` | `String` | 节点 ID |
| `graph_id` | `String` | 图 ID |
| `label` | `String` | 标签 |
| `snippet` | `String` | 搜索片段 |
| `rank` | `f64` | 排名分数 |

### `SearchRequest`
源文件：`models/graph_model.rs:103`

| 字段 | 类型 | 说明 |
|------|------|------|
| `query` | `String` | 搜索词 |
| `graph_ids` | `Option<Vec<String>>` | 限定图 ID 列表 |
| `limit` | `Option<i64>` | 返回数量上限 |

### `SearchResponse`
源文件：`models/graph_model.rs:110`

| 字段 | 类型 | 说明 |
|------|------|------|
| `results` | `Vec<SearchResult>` | 结果列表 |
| `total` | `usize` | 总数 |

---

## Git / Archive

### `ArchiveRequest`
源文件：`models/git_model.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `message_ids` | `Vec<String>` | 消息 ID 列表 |
| `conversation_id` | `String` | 对话 ID |
| `graph_id` | `String` | 图 ID |
| `target_node_id` | `Option<String>` | 目标节点 ID |
| `label` | `String` | 归档标题 |

### `ArchiveResponse`
源文件：`models/git_model.rs:12`

| 字段 | 类型 | 说明 |
|------|------|------|
| `archive_id` | `String` | 归档 ID |
| `markdown_path` | `String` | Markdown 文件路径 |
| `git_status` | `String` | Git 状态（committed/pr_pending） |
| `pr_url` | `Option<String>` | PR URL |
| `queue_position` | `Option<i64>` | 队列位置 |

### `ArchiveQueueResponse`
源文件：`models/git_model.rs:21`

| 字段 | 类型 | 说明 |
|------|------|------|
| `current_review` | `Option<QueueItem>` | 当前审核项 |
| `queue` | `Vec<QueueItem>` | 队列 |

### `QueueItem`
源文件：`models/git_model.rs:27`

| 字段 | 类型 | 说明 |
|------|------|------|
| `pr_id` | `i64` | PR ID |
| `author` | `String` | 作者 |
| `title` | `String` | 标题 |
| `position` | `i64` | 队列位置 |

### `PrResponse`
源文件：`models/git_model.rs:35`

| 字段 | 类型 | 说明 |
|------|------|------|
| `pr_id` | `i64` | PR ID |
| `title` | `String` | 标题 |
| `author` | `String` | 作者 |
| `state` | `String` | 状态 |
| `changes` | `Vec<FileChange>` | 变更文件 |

### `FileChange`
源文件：`models/git_model.rs:44`

| 字段 | 类型 | 说明 |
|------|------|------|
| `file` | `String` | 文件路径 |
| `status` | `String` | 状态（added/modified/deleted） |
| `additions` | `i64` | 新增行数 |
| `deletions` | `i64` | 删除行数 |
| `diff` | `String` | Diff 内容 |

### `CommitLogResponse`
源文件：`models/git_model.rs:53`

| 字段 | 类型 | 说明 |
|------|------|------|
| `commits` | `Vec<CommitEntry>` | 提交列表 |

### `CommitEntry`
源文件：`models/git_model.rs:58`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 提交 SHA |
| `message` | `String` | 提交信息 |
| `author` | `String` | 作者 |
| `date` | `String` | 日期 |

### `ArchiveRecord`
源文件：`models/git_model.rs:66`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | ID |
| `ring_id` | `String` | Ring ID |
| `node_id` | `Option<String>` | 节点 ID |
| `conversation_id` | `Option<String>` | 对话 ID |
| `message_ids` | `Option<String>` | 消息 ID JSON |
| `markdown_path` | `String` | Markdown 路径 |
| `archived_by` | `String` | 归档人 |
| `git_commit_sha` | `Option<String>` | Git 提交 SHA |
| `pr_status` | `Option<String>` | PR 状态 |
| `pr_url` | `Option<String>` | PR URL |
| `created_at` | `String` | 创建时间 |

---

## Notification

### `Notification`
源文件：`models/notification_model.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 通知 ID |
| `ring_id` | `String` | Ring ID |
| `user_id` | `String` | 目标用户 ID |
| `type` | `String` | 类型 |
| `title` | `String` | 标题 |
| `body` | `Option<String>` | 内容 |
| `related_id` | `Option<String>` | 关联 ID |
| `is_read` | `bool` | 是否已读 |
| `created_at` | `String` | 创建时间 |

### `NewNotification`
源文件：`models/notification_model.rs:16`

| 字段 | 类型 | 说明 |
|------|------|------|
| `ring_id` | `String` | Ring ID |
| `user_id` | `String` | 用户 ID |
| `type` | `String` | 类型 |
| `title` | `String` | 标题 |
| `body` | `Option<String>` | 内容 |
| `related_id` | `Option<String>` | 关联 ID |

---

## Tool

### `ToolDefinition`
源文件：`models/tool_model.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 工具名称 |
| `description` | `String` | 描述 |
| `parameters` | `serde_json::Value` | JSON Schema 参数定义 |

### `ToolCallRequest`
源文件：`models/tool_model.rs:10`

| 字段 | 类型 | 说明 |
|------|------|------|
| `tool_call_id` | `String` | 调用 ID |
| `tool_name` | `String` | 工具名称 |
| `input` | `serde_json::Value` | 输入参数 |

### `ToolResultRecord`
源文件：`models/tool_model.rs:17`

| 字段 | 类型 | 说明 |
|------|------|------|
| `tool_call_id` | `String` | 调用 ID |
| `tool_name` | `String` | 工具名称 |
| `output` | `serde_json::Value` | 输出结果 |
| `success` | `bool` | 是否成功 |

### `ToolExecution`
源文件：`models/tool_model.rs:25`

| 字段 | 类型 | 说明 |
|------|------|------|
| `call` | `ToolCallRequest` | 调用请求 |
| `result` | `Option<ToolResultRecord>` | 执行结果 |
