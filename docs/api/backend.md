# 后端 API 参考

> **Affects**: [frontend.md](frontend.md) · [api-design.md](../technical/api-design.md) · [data-model.md](../technical/data-model.md)
> **Depends on**: [PRD.md](../product/PRD.md) · [architecture.md](../technical/architecture.md) · [knowledge-graph.md](../technical/knowledge-graph.md)
> **Last verified**: 2026-04-11

---

## 核心模块

> 源码路径：`ring-server/src/`

### Config

#### `Config`
源文件：`config.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `port` | `u16` | 服务器监听端口，默认 7420 |
| `data_dir` | `PathBuf` | 数据目录，默认 `~/.ring/` |
| `release_repo` | `String` | GitHub release 仓库地址 |
| `database_url` | `String` | SQLite 数据库连接 URL |

#### `impl Default for Config`
源文件：`config.rs:11`

- **环境变量**：`RING_PORT`、`RING_DATA_DIR`、`RING_DATABASE_URL`、`RING_RELEASE_REPO`
- **默认端口**：7420

---

### Error

#### `RingError` 枚举
源文件：`error.rs:7`

- `NotFound(String)` — 资源未找到，映射 HTTP 404
- `Unauthorized(String)` — 未授权，映射 HTTP 401
- `Forbidden(String)` — 禁止访问，映射 HTTP 403
- `Conflict(String)` — 冲突，映射 HTTP 409
- `Validation(String)` — 验证错误，映射 HTTP 400
- `Git(...)` — Git 操作错误，内部错误
- `Database(...)` — 数据库错误，内部错误
- `Llm(String)` — LLM 调用错误，内部错误
- `Io(...)` — IO 错误，内部错误
- `Serialization(...)` — 序列化错误，内部错误
- `Internal(String)` — 通用内部错误，映射 HTTP 500（对外隐藏详情）

#### `Result<T>`
源文件：`error.rs:59`

类型别名：`std::result::Result<T, RingError>`

---

### State

#### `AppState`
源文件：`state.rs:13`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库仓库 |
| `graph_store` | `Arc<dyn GraphStore>` | 内存图存储 |
| `search_service` | `Arc<SearchService>` | 搜索服务 |
| `config` | `Arc<Config>` | 全局配置 |
| `llm_provider` | `Arc<dyn LlmProvider>` | LLM 提供者 |
| `ws_hub` | `Arc<WsHub>` | WebSocket Hub |
| `tool_registry` | `Arc<ToolRegistry>` | 工具注册表 |

#### `impl AppState`
源文件：`state.rs:24`

- `async fn rebuild_llm(&self) -> Arc<dyn LlmProvider>` — 根据数据库中的 LLM 配置重建 LLM Provider（openai/ollama/anthropic）

---

### Routes

#### `fn build_router(state: AppState) -> Router`
源文件：`routes.rs:24`

构建完整的 Axum 路由表，包含以下路由组：

**Setup 路由**（`/api/v1/setup`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/status` | `setup::get_status` |
| POST | `/username` | `setup::set_username` |
| POST | `/llm` | `setup::set_llm` |
| POST | `/gitlab` | `setup::set_gitlab` |
| POST | `/complete` | `setup::complete` |

**Ring 路由**（`/api/v1/rings`）
| 方法 | 路径 | Handler |
|------|------|---------|
| POST | `/join` | `member::join_ring` |
| GET | `/` | `ring::list_rings` |
| POST | `/` | `ring::create_ring` |
| GET | `/{ringId}` | `ring::get_ring` |
| PUT | `/{ringId}` | `ring::update_ring` |
| DELETE | `/{ringId}` | `ring::delete_ring` |

**Member 路由**（`/api/v1/rings/{ringId}/members`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/` | `member::list_members` |
| POST | `/invites` | `member::generate_invite` |
| PUT | `/{memberId}/role` | `member::update_role` |
| DELETE | `/{memberId}` | `member::remove_member` |

**Session 路由**（`/api/v1/rings/{ringId}/sessions`）
| 方法 | 路径 | Handler |
|------|------|---------|
| POST | `/` | `session::create_session` |
| GET | `/` | `session::list_sessions` |
| GET | `/{sessionId}` | `session::get_session` |
| DELETE | `/{sessionId}` | `session::delete_session` |
| POST | `/{sessionId}/close` | `session::close_session` |
| POST | `/{sessionId}/leave` | `session::leave_session` |
| PUT | `/{sessionId}/archive-toggle` | `session::toggle_archive` |
| POST | `/{sessionId}/invite` | `session::invite_member` |
| GET | `/{sessionId}/messages` | `session::get_messages` |
| POST | `/{sessionId}/messages` | `session::send_message` |
| POST | `/{sessionId}/materials` | `session::prepare_materials` |
| GET | `/{sessionId}/materials` | `session::get_materials_progress` |

**Conversation 路由**（`/api/v1/rings/{ringId}/conversations`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/` | `conversation::list` |
| POST | `/` | `conversation::create` |
| GET | `/{convId}` | `conversation::get` |
| GET | `/{convId}/messages` | `conversation::get_messages` |
| POST | `/{convId}/messages` | `conversation::send_message` |

**Blueprint 路由**（`/api/v1/rings/{ringId}/blueprint`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/templates` | `blueprint::list_templates` |
| POST | `/chat` | `blueprint::blueprint_chat` |
| POST | `/preview` | `blueprint::preview_blueprint` |
| POST | `/confirm` | `blueprint::confirm_blueprint` |

**Graph 路由**（`/api/v1/rings/{ringId}/graphs`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/` | `graph::list_graphs` |
| GET | `/{graphId}` | `graph::get_graph` |
| POST | `/{graphId}/nodes` | `graph::create_node` |
| GET | `/{graphId}/nodes/{nodeId}` | `graph::get_node` |
| PUT | `/{graphId}/nodes/{nodeId}` | `graph::update_node` |
| DELETE | `/{graphId}/nodes/{nodeId}` | `graph::delete_node` |
| GET | `/{graphId}/nodes/{nodeId}/content` | `graph::get_node_content` |
| POST | `/{graphId}/edges` | `graph::create_edge` |
| DELETE | `/{graphId}/edges/{edgeId}` | `graph::delete_edge` |

**Search 路由**（`/api/v1/rings/{ringId}/search`）
| 方法 | 路径 | Handler |
|------|------|---------|
| POST | `/` | `search::search_nodes` |

**Archive 路由**（`/api/v1/rings/{ringId}/archive`）
| 方法 | 路径 | Handler |
|------|------|---------|
| POST | `/` | `archive::archive` |
| GET | `/queue` | `archive::get_queue` |
| POST | `/{archiveId}/confirm` | `archive::confirm_archive` |

**Git 路由**（`/api/v1/rings/{ringId}/git`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/prs` | `git::list_prs` |
| POST | `/prs/{prId}/merge` | `git::merge_pr` |
| POST | `/prs/{prId}/reject` | `git::reject_pr` |
| GET | `/commits` | `git::get_commit_log` |

**Notification 路由**（`/api/v1/notifications`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/` | `notification::list_notifications` |
| POST | `/{notificationId}` | `notification::mark_read` |

**Settings 路由**（`/api/v1/settings`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/` | `settings::get_settings` |
| PUT | `/` | `settings::update_settings` |

**Skill 路由**（`/api/v1/skills`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/` | `skill::list_skills` |
| POST | `/install` | `skill::install_skill` |
| DELETE | `/{skillId}` | `skill::uninstall_skill` |

**Self 路由**（`/api/v1/self`）
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/metrics` | `self::get_metrics` |
| PUT | `/metrics` | `self::update_metrics` |

**Super Ring & WebSocket**
| 方法 | 路径 | Handler |
|------|------|---------|
| POST | `/api/v1/super-ring/chat` | `ai::super_ring_chat` |
| GET | `/api/v1/ws/{ringId}` | `ws::ws_handler` |

**Public 路由**
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/join` | `install::join_page` |

---

## 数据模型

> 源码路径：`ring-server/src/models/`

### User

#### `User`
源文件：`models/user.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 用户唯一 ID |
| `display_name` | `String` | 显示名称 |
| `avatar_url` | `Option<String>` | 头像 URL |
| `ip_address` | `Option<String>` | 注册 IP 地址 |
| `setup_completed` | `bool` | 设置是否完成 |
| `created_at` | `String` | 创建时间（RFC3339） |

#### `NewUser`
源文件：`models/user.rs:13`

| 字段 | 类型 | 说明 |
|------|------|------|
| `display_name` | `String` | 显示名称 |

---

### Ring

#### `Ring`
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

#### `NewRing`
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

### Member

#### `Member`
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

#### `NewMember`
源文件：`models/member.rs:14`

| 字段 | 类型 | 说明 |
|------|------|------|
| `ring_id` | `String` | Ring ID |
| `user_id` | `String` | 用户 ID |
| `display_name` | `String` | 显示名称 |
| `role` | `Option<String>` | 角色（默认 member） |

---

### InviteToken

#### `InviteToken`
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

### Conversation

#### `Conversation`
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

#### `NewConversation`
源文件：`models/conversation.rs:20`

| 字段 | 类型 | 说明 |
|------|------|------|
| `ring_id` | `String` | Ring ID |
| `title` | `Option<String>` | 标题 |
| `context_mode` | `Option<String>` | 上下文模式 |
| `created_by` | `String` | 创建者 |

#### `Message`
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

#### `NewMessage`
源文件：`models/conversation.rs:40`

| 字段 | 类型 | 说明 |
|------|------|------|
| `conversation_id` | `String` | 对话 ID |
| `role` | `String` | 角色 |
| `content` | `String` | 内容 |
| `sender_id` | `Option<String>` | 发送者 ID |

---

### Session

#### `Session`
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

#### `SessionMember`
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

#### `SessionMessage`
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

#### `CreateSessionRequest`
源文件：`models/session_model.rs:38`

| 字段 | 类型 | 说明 |
|------|------|------|
| `title` | `Option<String>` | 标题 |
| `scenario` | `String` | 场景（必填） |
| `archive_enabled` | `Option<bool>` | 是否启用归档 |
| `invite_member_ids` | `Option<Vec<String>>` | 邀请成员 ID 列表 |

#### `SessionDetailResponse`
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

#### `SessionMemberBrief`
源文件：`models/session_model.rs:59`

| 字段 | 类型 | 说明 |
|------|------|------|
| `user_id` | `String` | 用户 ID |
| `role` | `String` | 角色 |
| `status` | `String` | 状态 |

#### `SessionListResponse`
源文件：`models/session_model.rs:66`

| 字段 | 类型 | 说明 |
|------|------|------|
| `sessions` | `Vec<SessionListItem>` | Session 列表 |

#### `SessionListItem`
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

#### `InviteSessionRequest`
源文件：`models/session_model.rs:82`

| 字段 | 类型 | 说明 |
|------|------|------|
| `member_ids` | `Vec<String>` | 成员 ID 列表 |

#### `ArchiveToggleRequest`
源文件：`models/session_model.rs:87`

| 字段 | 类型 | 说明 |
|------|------|------|
| `archive_enabled` | `bool` | 是否启用归档 |

#### `SessionMessagesResponse`
源文件：`models/session_model.rs:92`

| 字段 | 类型 | 说明 |
|------|------|------|
| `messages` | `Vec<SessionMessage>` | 消息列表 |

---

### Blueprint

#### `BlueprintTemplate`
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

#### `NewBlueprintTemplate`
源文件：`models/blueprint.rs:14`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 模板 ID |
| `name` | `String` | 名称 |
| `description` | `Option<String>` | 描述 |
| `graphs` | `String` | 图谱 JSON |
| `is_system` | `bool` | 是否系统模板 |

---

### Graph

#### `CreateNodeRequest`
源文件：`models/graph_model.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `label` | `String` | 节点标签 |
| `node_type` | `String` | 节点类型 |
| `parent_id` | `Option<String>` | 父节点 ID |
| `description` | `Option<String>` | 描述 |

#### `UpdateNodeRequest`
源文件：`models/graph_model.rs:11`

| 字段 | 类型 | 说明 |
|------|------|------|
| `label` | `Option<String>` | 标签 |
| `description` | `Option<String>` | 描述 |
| `node_type` | `Option<String>` | 节点类型 |

#### `CreateEdgeRequest`
源文件：`models/graph_model.rs:18`

| 字段 | 类型 | 说明 |
|------|------|------|
| `source_id` | `String` | 源节点 ID |
| `target_id` | `String` | 目标节点 ID |
| `relation` | `String` | 关系类型 |
| `label` | `Option<String>` | 标签 |

#### `NodeResponse`
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

#### `EdgeResponse`
源文件：`models/graph_model.rs:55`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 边 ID |
| `source_id` | `String` | 源节点 ID |
| `target_id` | `String` | 目标节点 ID |
| `relation` | `String` | 关系类型 |
| `label` | `Option<String>` | 标签 |
| `graph_id` | `String` | 所属图 ID |

#### `GraphDetailResponse`
源文件：`models/graph_model.rs:78`

| 字段 | 类型 | 说明 |
|------|------|------|
| `graph_id` | `String` | 图 ID |
| `nodes` | `Vec<NodeResponse>` | 节点列表 |
| `edges` | `Vec<EdgeResponse>` | 边列表 |

#### `NodeContentResponse`
源文件：`models/graph_model.rs:85`

| 字段 | 类型 | 说明 |
|------|------|------|
| `node_id` | `String` | 节点 ID |
| `label` | `String` | 标签 |
| `markdown_path` | `Option<String>` | Markdown 路径 |
| `content` | `Option<String>` | Markdown 内容 |
| `last_modified` | `String` | 最后修改时间 |

#### `SearchResult`
源文件：`models/graph_model.rs:94`

| 字段 | 类型 | 说明 |
|------|------|------|
| `node_id` | `String` | 节点 ID |
| `graph_id` | `String` | 图 ID |
| `label` | `String` | 标签 |
| `snippet` | `String` | 搜索片段 |
| `rank` | `f64` | 排名分数 |

#### `SearchRequest`
源文件：`models/graph_model.rs:103`

| 字段 | 类型 | 说明 |
|------|------|------|
| `query` | `String` | 搜索词 |
| `graph_ids` | `Option<Vec<String>>` | 限定图 ID 列表 |
| `limit` | `Option<i64>` | 返回数量上限 |

#### `SearchResponse`
源文件：`models/graph_model.rs:110`

| 字段 | 类型 | 说明 |
|------|------|------|
| `results` | `Vec<SearchResult>` | 结果列表 |
| `total` | `usize` | 总数 |

---

### Git / Archive

#### `ArchiveRequest`
源文件：`models/git_model.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `message_ids` | `Vec<String>` | 消息 ID 列表 |
| `conversation_id` | `String` | 对话 ID |
| `graph_id` | `String` | 图 ID |
| `target_node_id` | `Option<String>` | 目标节点 ID |
| `label` | `String` | 归档标题 |

#### `ArchiveResponse`
源文件：`models/git_model.rs:12`

| 字段 | 类型 | 说明 |
|------|------|------|
| `archive_id` | `String` | 归档 ID |
| `markdown_path` | `String` | Markdown 文件路径 |
| `git_status` | `String` | Git 状态（committed/pr_pending） |
| `pr_url` | `Option<String>` | PR URL |
| `queue_position` | `Option<i64>` | 队列位置 |

#### `ArchiveQueueResponse`
源文件：`models/git_model.rs:21`

| 字段 | 类型 | 说明 |
|------|------|------|
| `current_review` | `Option<QueueItem>` | 当前审核项 |
| `queue` | `Vec<QueueItem>` | 队列 |

#### `QueueItem`
源文件：`models/git_model.rs:27`

| 字段 | 类型 | 说明 |
|------|------|------|
| `pr_id` | `i64` | PR ID |
| `author` | `String` | 作者 |
| `title` | `String` | 标题 |
| `position` | `i64` | 队列位置 |

#### `PrResponse`
源文件：`models/git_model.rs:35`

| 字段 | 类型 | 说明 |
|------|------|------|
| `pr_id` | `i64` | PR ID |
| `title` | `String` | 标题 |
| `author` | `String` | 作者 |
| `state` | `String` | 状态 |
| `changes` | `Vec<FileChange>` | 变更文件 |

#### `FileChange`
源文件：`models/git_model.rs:44`

| 字段 | 类型 | 说明 |
|------|------|------|
| `file` | `String` | 文件路径 |
| `status` | `String` | 状态（added/modified/deleted） |
| `additions` | `i64` | 新增行数 |
| `deletions` | `i64` | 删除行数 |
| `diff` | `String` | Diff 内容 |

#### `CommitLogResponse`
源文件：`models/git_model.rs:53`

| 字段 | 类型 | 说明 |
|------|------|------|
| `commits` | `Vec<CommitEntry>` | 提交列表 |

#### `CommitEntry`
源文件：`models/git_model.rs:58`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 提交 SHA |
| `message` | `String` | 提交信息 |
| `author` | `String` | 作者 |
| `date` | `String` | 日期 |

#### `ArchiveRecord`
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

### Notification

#### `Notification`
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

#### `NewNotification`
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

### Tool

#### `ToolDefinition`
源文件：`models/tool_model.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | 工具名称 |
| `description` | `String` | 描述 |
| `parameters` | `serde_json::Value` | JSON Schema 参数定义 |

#### `ToolCallRequest`
源文件：`models/tool_model.rs:10`

| 字段 | 类型 | 说明 |
|------|------|------|
| `tool_call_id` | `String` | 调用 ID |
| `tool_name` | `String` | 工具名称 |
| `input` | `serde_json::Value` | 输入参数 |

#### `ToolResultRecord`
源文件：`models/tool_model.rs:17`

| 字段 | 类型 | 说明 |
|------|------|------|
| `tool_call_id` | `String` | 调用 ID |
| `tool_name` | `String` | 工具名称 |
| `output` | `serde_json::Value` | 输出结果 |
| `success` | `bool` | 是否成功 |

#### `ToolExecution`
源文件：`models/tool_model.rs:25`

| 字段 | 类型 | 说明 |
|------|------|------|
| `call` | `ToolCallRequest` | 调用请求 |
| `result` | `Option<ToolResultRecord>` | 执行结果 |

---

## 数据库层

> 源码路径：`ring-server/src/db/`

### Repository Trait

#### `trait Repository`
源文件：`db/traits.rs:15`

所有数据库操作的统一接口。所有方法均为 `async`，返回 `Result`。

##### 用户相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_user` | `(NewUser) -> Result<User>` | 创建用户 |
| `get_user` | `(id: &str) -> Result<Option<User>>` | 获取用户 |
| `list_all_users` | `() -> Result<Vec<User>>` | 列出所有用户 |
| `is_setup_completed` | `() -> Result<bool>` | 检查设置是否完成 |
| `complete_setup` | `(user_id: &str) -> Result<()>` | 完成设置 |

##### Ring 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_ring` | `(NewRing) -> Result<Ring>` | 创建 Ring |
| `get_ring` | `(id: &str) -> Result<Option<Ring>>` | 获取 Ring |
| `list_rings_by_user` | `(user_id: &str) -> Result<Vec<Ring>>` | 列出用户的所有 Ring |
| `update_ring` | `(id, name, description) -> Result<Ring>` | 更新 Ring |
| `delete_ring` | `(id: &str) -> Result<()>` | 删除 Ring |
| `update_ring_status` | `(id, status) -> Result<()>` | 更新 Ring 状态 |

##### 邀请相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_invite_token` | `(ring_id, token, token_type, role, inviter_id) -> Result<InviteToken>` | 创建邀请码 |
| `get_invite_token` | `(token: &str) -> Result<Option<InviteToken>>` | 获取邀请码 |
| `count_members_by_ring` | `(ring_id: &str) -> Result<i64>` | 统计成员数量 |

##### 设置相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `get_setting` | `(key: &str) -> Result<Option<String>>` | 获取设置项 |
| `set_setting` | `(key, value) -> Result<()>` | 设置设置项 |

##### Member 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_member` | `(NewMember) -> Result<Member>` | 创建成员 |
| `get_member` | `(id: &str) -> Result<Option<Member>>` | 获取成员 |
| `list_members_by_ring` | `(ring_id: &str) -> Result<Vec<Member>>` | 列出 Ring 成员 |
| `get_member_by_user_and_ring` | `(user_id, ring_id) -> Result<Option<Member>>` | 获取用户在 Ring 中的成员信息 |
| `update_member_role` | `(id, role) -> Result<()>` | 更新成员角色 |
| `delete_member` | `(id: &str) -> Result<()>` | 删除成员 |
| `get_next_token_id` | `(ring_id: &str) -> Result<i64>` | 获取下一个 Token ID |

##### Conversation 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_conversation` | `(ring_id, title, context_mode, created_by) -> Result<Conversation>` | 创建对话 |
| `list_conversations` | `(ring_id: &str) -> Result<Vec<Conversation>>` | 列出对话 |
| `get_conversation` | `(id: &str) -> Result<Option<Conversation>>` | 获取对话 |
| `create_message` | `(conversation_id, role, content, sender_id) -> Result<Message>` | 创建消息 |
| `get_messages` | `(conversation_id, limit, before_id) -> Result<Vec<Message>>` | 获取消息 |

##### Blueprint 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `list_blueprint_templates` | `() -> Result<Vec<BlueprintTemplate>>` | 列出模板 |
| `create_blueprint_template` | `(id, name, description, graphs_json, is_system) -> Result<BlueprintTemplate>` | 创建模板 |

##### 搜索相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `index_node_search` | `(node_id, graph_id, label, content) -> Result<()>` | 索引节点到 FTS |
| `delete_node_search` | `(node_id: &str) -> Result<()>` | 从 FTS 删除节点 |
| `search_nodes_fts` | `(query, graph_ids, limit) -> Result<Vec<SearchResult>>` | FTS 全文搜索 |

##### Archive 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_archive_record` | `(...) -> Result<()>` | 创建归档记录 |
| `list_archive_records_by_ring` | `(ring_id: &str) -> Result<Vec<ArchiveRecord>>` | 列出归档记录 |
| `get_archive_record` | `(id: &str) -> Result<Option<ArchiveRecord>>` | 获取归档记录 |
| `update_archive_pr_status` | `(id, pr_status) -> Result<()>` | 更新 PR 状态 |

##### Notification 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_notification` | `(NewNotification) -> Result<Notification>` | 创建通知 |
| `list_notifications_by_user` | `(user_id, unread_only) -> Result<Vec<Notification>>` | 列出通知 |
| `mark_notification_read` | `(id: &str) -> Result<()>` | 标记已读 |

##### Session 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_session` | `(ring_id, title, scenario, created_by, archive_enabled) -> Result<Session>` | 创建 Session |
| `get_session` | `(id: &str) -> Result<Option<Session>>` | 获取 Session |
| `list_sessions_by_ring` | `(ring_id, status) -> Result<Vec<Session>>` | 列出 Session |
| `update_session_status` | `(id, status) -> Result<()>` | 更新状态 |
| `update_session_archive` | `(id, enabled) -> Result<()>` | 更新归档开关 |
| `delete_session` | `(id: &str) -> Result<()>` | 删除 Session |
| `create_session_member` | `(session_id, user_id, role) -> Result<SessionMember>` | 添加 Session 成员 |
| `list_session_members` | `(session_id: &str) -> Result<Vec<SessionMember>>` | 列出成员 |
| `leave_session_member` | `(session_id, user_id) -> Result<()>` | 成员离开 |
| `create_session_message` | `(session_id, sender_id, role, content, seq_num) -> Result<SessionMessage>` | 创建消息 |
| `get_session_messages` | `(session_id, after_seq, limit) -> Result<Vec<SessionMessage>>` | 获取消息 |

---

### SqliteRepository

#### `struct SqliteRepository`
源文件：`db/sqlite/mod.rs:28`

| 字段 | 类型 | 说明 |
|------|------|------|
| `pool` | `SqlitePool` | SQLx 连接池 |
| `jieba` | `Mutex<Option<jieba_rs::Jieba>>` | 中文分词器（懒加载） |

#### `impl SqliteRepository`
源文件：`db/sqlite/mod.rs:33`

- `fn new(pool: SqlitePool) -> Self` — 构造函数
- `fn pool(&self) -> &SqlitePool` — 获取连接池引用
- `fn get_jieba(&self) -> jieba_rs::Jieba` — 获取分词器（懒加载初始化）

所有 `Repository` trait 方法均有 `async_trait` 实现，底层调用对应的 `xxx_inner` 私有方法。

---

## 服务层

> 源码路径：`ring-server/src/services/`

### RingService

#### `struct RingService`
源文件：`services/ring_service.rs:21`

| 字段 | 类型 | 说明 |
|------|------|------|
| `repo` | `Arc<dyn Repository>` | 数据库 |
| `data_dir` | `PathBuf` | 数据目录 |

#### `impl RingService`
源文件：`services/ring_service.rs:26`

- `fn new(repo, data_dir) -> Self` — 构造函数
- `async fn create_ring(req: CreateRingRequest) -> Result<Ring>` — 创建 Ring
  - 校验 name 非空且 ≤ 100 字符
  - 自动创建/查找用户
  - 调用 `repo.create_ring`
  - 创建本地目录结构 `repos/ring-{name}/nodes`
  - 写入 `graph.json`（空图）
- `async fn get_ring(id) -> Result<Ring>` — 获取 Ring
- `async fn list_rings(user_id) -> Result<Vec<Ring>>` — 列出用户的所有 Ring
- `async fn update_ring(id, name, description) -> Result<Ring>` — 更新 Ring
- `async fn delete_ring(id) -> Result<()>` — 删除 Ring

#### `CreateRingRequest`
源文件：`services/ring_service.rs:11`

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | `String` | Ring 名称 |
| `description` | `Option<String>` | 描述 |
| `role_description` | `String` | 角色描述 |
| `creator_id` | `String` | 创建者 ID |
| `gitlab_repo` | `String` | GitLab 仓库 |
| `namespace` | `Option<String>` | GitLab namespace |

---

### MemberService

#### `struct MemberService`
源文件：`services/member_service.rs:8`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |

#### `impl MemberService`
源文件：`services/member_service.rs:12`

- `fn new(db) -> Self` — 构造函数
- `async fn generate_invite(ring_id, inviter_id, token_type, role, max_uses, max_members, expires_in_seconds) -> Result<InviteToken>` — 生成邀请码
  - 仅创建者可邀请
  - 检查成员上限
  - 生成 UUID 作为 token
- `async fn join_ring(token, user_id, display_name) -> Result<Member>` — 加入 Ring
  - 校验 token 有效性、过期、撤销、次数
  - 检查是否已是成员
  - 创建 Member 记录
- `async fn list_members(ring_id) -> Result<Vec<Member>>` — 列出成员
- `async fn update_role(ring_id, member_id, new_role, caller_id) -> Result<()>` — 更新角色（仅创建者可操作）
- `async fn remove_member(ring_id, member_id, caller_id) -> Result<()>` — 移除成员（不能移除创建者）

---

### SessionService

#### `struct SessionService`
源文件：`services/session_service.rs:8`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |
| `permission` | `PermissionService` | 权限服务 |

#### `impl SessionService`
源文件：`services/session_service.rs:13`

- `fn new(db) -> Self` — 构造函数（内建 PermissionService）
- `async fn create_session(ring_id, req, user_id) -> Result<SessionDetailResponse>` — 创建 Session
  - 校验用户 Ring 权限
  - 检查是否已有 active session
  - 创建 Session + SessionMember（owner）
  - 邀请指定成员
- `async fn list_sessions(ring_id, status) -> Result<SessionListResponse>` — 列出 Session
- `async fn get_session_detail(ring_id, session_id) -> Result<SessionDetailResponse>` — 获取详情
- `async fn invite_member(ring_id, session_id, member_ids, caller_id) -> Result<Vec<SessionMemberBrief>>` — 邀请成员（仅 owner）
- `async fn close_session(ring_id, session_id, caller_id) -> Result<()>` — 关闭 Session（仅 owner）
- `async fn leave_session(ring_id, session_id, user_id) -> Result<()>` — 离开 Session（owner 不可，需用 close）
- `async fn toggle_archive(ring_id, session_id, enabled, caller_id) -> Result<()>` — 开关归档（仅 owner）
- `async fn delete_session(ring_id, session_id, caller_id) -> Result<()>` — 删除 Session（仅 owner）
- `async fn get_messages(ring_id, session_id, after_seq, limit) -> Result<SessionMessagesResponse>` — 获取消息

---

### SearchService

#### `struct SearchService`
源文件：`services/search_service.rs:8`

| 字段 | 类型 | 说明 |
|------|------|------|
| `repo` | `Arc<dyn Repository>` | 数据库 |
| `store` | `Arc<dyn GraphStore>` | 图存储 |

#### `impl SearchService`
源文件：`services/search_service.rs:14`

- `fn new(repo, store) -> Self` — 构造函数
- `async fn search_nodes(query, graph_ids, limit) -> Result<Vec<SearchResult>>` — FTS 全文搜索
- `async fn index_node(node_id, graph_id, label, content) -> Result<()>` — 索引节点
- `async fn delete_node_index(node_id) -> Result<()>` — 删除索引

---

### ArchiveService

#### `struct ArchiveService`
源文件：`services/archive_service.rs:14`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |
| `git_service` | `Arc<GitService>` | Git 服务 |
| `graph_store` | `Arc<dyn GraphStore>` | 图存储 |
| `gitlab_service` | `Option<Arc<GitlabService>>` | GitLab 服务 |

#### `impl ArchiveService`
源文件：`services/archive_service.rs:21`

- `fn new(db, git_service, graph_store, gitlab_service) -> Self` — 构造函数
- `async fn archive(ring_id, request, archived_by, is_creator) -> Result<ArchiveResponse>` — 归档内容
  - 收集消息内容，拼接为 Markdown
  - 写入 `nodes/{slug}.md`
  - 导出 graph.json
  - **创建者**：直接 git add + commit
  - **成员**：创建分支 → commit → 生成 MR 链接
- `async fn get_queue(ring_id) -> Result<ArchiveQueueResponse>` — 获取待审核队列
- `async fn confirm_archive(ring_id, archive_id) -> Result<()>` — 确认归档
- `async fn list_prs(ring_id, state) -> Result<Vec<PrResponse>>` — 列出 PR
- `async fn merge_pr(ring_id, archive_id) -> Result<()>` — 合并 PR
- `async fn reject_pr(ring_id, archive_id) -> Result<()>` — 拒绝 PR
- `async fn get_commit_log(ring_id, limit) -> Result<CommitLogResponse>` — 获取提交日志

---

### GraphService

#### `struct GraphService`
源文件：`services/graph_service.rs:10`

| 字段 | 类型 | 说明 |
|------|------|------|
| `store` | `Arc<dyn GraphStore>` | 图存储 |
| `search_service` | `Arc<SearchService>` | 搜索服务 |
| `data_dir` | `PathBuf` | 数据目录 |

#### `impl GraphService`
源文件：`services/graph_service.rs:16`

- `fn new(store, search_service, data_dir) -> Self` — 构造函数
- `async fn create_node(graph_id, req) -> Result<NodeData>` — 创建节点 + 写 Markdown + 索引 + 持久化
- `async fn get_node(graph_id, node_id) -> Result<Option<NodeData>>` — 获取节点
- `async fn update_node(graph_id, node_id, req) -> Result<NodeData>` — 更新节点 + 更新 Markdown + 更新索引
- `async fn delete_node(graph_id, node_id) -> Result<()>` — 删除节点 + 删除 Markdown + 删除索引 + 持久化
- `async fn get_children(graph_id, parent_id) -> Result<Vec<NodeData>>` — 获取子节点
- `async fn get_root_nodes(graph_id) -> Result<Vec<NodeData>>` — 获取根节点
- `async fn get_neighbors(graph_id, node_id) -> Result<Vec<(NodeData, EdgeData)>>` — 获取邻居
- `async fn get_node_content(graph_id, node_id) -> Result<NodeContentResponse>` — 获取节点 Markdown 内容
- `async fn list_graphs(ring_id) -> Result<Vec<String>>` — 列出图 ID

#### Markdown 持久化格式
源文件：`services/graph_service.rs:41`

```markdown
---
node_id: {id}
type: {node_type}
labels: ["{label}"]
created_at: {created_at}
updated_at: {updated_at}
---

{description}
```

---

### SettingsService

#### `struct SettingsService`
源文件：`services/settings_service.rs:6`

| 字段 | 类型 | 说明 |
|------|------|------|
| `repo` | `Arc<dyn Repository>` | 数据库 |

#### `impl SettingsService`
源文件：`services/settings_service.rs:10`

- `fn new(repo) -> Self` — 构造函数
- `async fn get_all_settings() -> Result<serde_json::Value>` — 获取所有设置（llm_provider/model/api_key/base_url、privacy_enabled、user_id、display_name）
- `async fn update_settings(settings) -> Result<()>` — 更新设置（仅允许：llm_provider/model/api_key/base_url、privacy_enabled）

---

### NotificationService

#### `struct NotificationService`
源文件：`services/notification_service.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |

#### `impl NotificationService`
源文件：`services/notification_service.rs:11`

- `fn new(db) -> Self` — 构造函数
- `async fn list_for_user(user_id, unread_only) -> Result<Vec<Notification>>` — 列出通知
- `async fn mark_read(notification_id) -> Result<()>` — 标记已读

---

### PermissionService

#### `struct PermissionService`
源文件：`services/permission_service.rs:6`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |

#### `impl PermissionService`
源文件：`services/permission_service.rs:10`

- `fn new(db) -> Self` — 构造函数
- `async fn check_ring_access(ring_id, user_id) -> Result<()>` — 检查 Ring 访问权限（创建者或成员）
- `async fn check_creator_or_admin(ring_id, user_id) -> Result<()>` — 检查创建者或管理员权限
- `async fn check_creator(ring_id, user_id) -> Result<()>` — 检查创建者权限
- `async fn get_member_role(ring_id, user_id) -> Result<Option<String>>` — 获取成员角色
- `async fn is_creator(ring_id, user_id) -> Result<bool>` — 是否创建者
- `async fn is_member(ring_id, user_id) -> Result<bool>` — 是否成员

---

### CredentialService

#### `struct CredentialService`
源文件：`services/credential_service.rs:15`

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `[u8; 32]` | 加密密钥（32 字节） |

#### `impl CredentialService`
源文件：`services/credential_service.rs:19`

- `fn new(key) -> Self` — 构造函数
- `fn derive_key_from_password(password) -> [u8; 32]` — PBKDF2-HMAC-SHA256 密钥派生（600,000 轮）
- `fn encrypt(plaintext) -> Result<String>` — AES-256-GCM 加密，输出 Base64（salt + nonce + ciphertext）
- `fn decrypt(encrypted) -> Result<String>` — 解密

---

### WorkflowService

#### `Workflow`
源文件：`services/workflow_service.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 工作流 ID |
| `name` | `String` | 名称 |
| `description` | `String` | 描述 |
| `system_prompt` | `String` | 系统提示词 |
| `tool_names` | `Vec<String>` | 启用的工具列表 |

#### 预设工作流
源文件：`services/workflow_service.rs:16`

- `meeting_archive` — 会议归档：text_clean + privacy_filter + markdown_gen + search
- `deep_research` — 深度研究：search + web_scrape + text_clean + markdown_gen
- `learning_center` — 学习中心：search + text_clean + markdown_gen

#### 函数
- `fn get_workflow(id) -> Option<Workflow>` — 获取工作流
- `fn list_workflows() -> Vec<Workflow>` — 列出所有工作流
- `fn filter_tools_for_workflow(all_tools, tool_names) -> Vec<ToolDefinition>` — 从工具列表中筛选指定工作流需要的工具

---

### TriggerService

#### `fn check_archive_suggestion(last_assistant_text) -> Option<LlmEvent>`
源文件：`services/trigger_service.rs:3`

检测对话是否包含归档关键词（总结、归档、记录、要点、会议纪要）。包含则返回 `ArchiveSuggestion` 事件。

#### `fn check_empty_graph_guidance(node_count) -> Option<LlmEvent>`
源文件：`services/trigger_service.rs:18`

当图谱节点数 < 3 时，返回 `ArchiveSuggestion` 事件建议。

---

### WsHub

#### `struct WsHub`
源文件：`services/ws_hub.rs:12`

| 字段 | 类型 | 说明 |
|------|------|------|
| `channels` | `RwLock<HashMap<String, broadcast::Sender<WsMessage>>>` | Ring ID → 广播频道 |

#### `impl WsHub`
源文件：`services/ws_hub.rs:22`

- `fn new() -> Self` — 构造函数
- `async fn subscribe(ring_id) -> broadcast::Receiver<WsMessage>` — 订阅 Ring 消息
- `async fn broadcast(ring_id, msg)` — 广播消息到 Ring

#### `WsMessage`
源文件：`services/ws_hub.rs:6`

| 字段 | 类型 | 说明 |
|------|------|------|
| `msg_type` | `String` | 消息类型 |
| `payload` | `serde_json::Value` | 负载数据 |

---

## AI 服务

> 源码路径：`ring-server/src/services/ai_service.rs`、`context_loader.rs`

### AiService

#### `struct AiService`
源文件：`services/ai_service.rs:15`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |
| `llm` | `Arc<dyn LlmProvider>` | LLM |
| `tool_dispatcher` | `Arc<ToolDispatcher>` | 工具调度器 |

#### `impl AiService`
源文件：`services/ai_service.rs:21`

- `fn new(db, llm, tool_dispatcher) -> Self` — 构造函数
- `async fn super_ring_chat(user_id, message, history) -> Result<SseStream>` — 全局超级助手对话，在 prompt 中注入用户 Ring 列表
- `async fn group_ring_chat(ring_id, conv_id, message) -> Result<SseStream>` — 群组助手对话，自动保存用户消息
- `async fn blueprint_chat(ring_id, message, history) -> Result<SseStream>` — 蓝图构建对话
- `async fn session_chat(ring_id, session_id, sender_id, ring_name, scenario, message) -> Result<SseStream>` — Session 协作对话
- `async fn chat_with_tools(messages, tools) -> Result<SseStream>` — 工具调用循环（最多 5 轮）

#### Token 预算控制
所有对话方法都会：
1. 构建 system prompt（通过 `context_loader`）
2. 计算 `estimate_tokens(context)`
3. 从 history 中截取不超过 100,000 - system_tokens 的内容
4. `truncate_llm_messages` / `truncate_messages`：从后往前保留，总字符数不超过 budget_chars

---

### Context Loader

#### `fn build_super_ring_prompt() -> String`
源文件：`services/context_loader.rs:1`

构建 Super Ring 系统提示词。包含：Ring 管理引导、跨 Ring 分析、跨 Ring 问答、新用户引导。严格禁止虚构 Ring 数据。

#### `fn build_group_ring_prompt(ring_name, role_md, conventions_md, active_context_md) -> String`
源文件：`services/context_loader.rs:26`

构建 Group Ring 系统提示词。包含：角色定义、团队约定、当前活跃上下文。

#### `fn build_blueprint_prompt(role_md) -> String`
源文件：`services/context_loader.rs:53`

构建蓝图构建器提示词。核心原则：图谱节点必须对应 `.ring/` 目录下的 Markdown 文档。交互流程：追问 → 确认维度 → 说明模板 → mermaid 预览 → 用户确认。

#### `fn build_session_prompt(ring_name, scenario) -> String`
源文件：`services/context_loader.rs:95`

构建 Session 助手提示词。根据 scenario（discussion/deep_research/meeting_archive/learning_center）调整行为。

---

### 辅助函数

#### `fn estimate_tokens(text: &str) -> usize`
源文件：`services/ai_service.rs:316`

粗略估算：`text.len() / 3`

#### `fn truncate_messages(messages: &[Message], budget_tokens) -> Vec<Message>`
源文件：`services/ai_service.rs:322`

从后往前保留消息，总字符数不超过 `budget_tokens * 3`。

#### `fn truncate_llm_messages(messages: &[LlmMessage], budget_tokens) -> Vec<LlmMessage>`
源文件：`services/ai_service.rs:340`

同 `truncate_messages`，但作用于 `LlmMessage`。

---

## 图谱引擎

> 源码路径：`ring-server/src/graph/`

### GraphStore Trait

#### `trait GraphStore`
源文件：`graph/store_trait.rs:5`

内存图存储的统一接口，所有方法均为 `async`。

| 方法 | 签名 | 说明 |
|------|------|------|
| `create_node` | `(graph_id, NewNode) -> Result<NodeData>` | 创建节点 |
| `get_node` | `(graph_id, node_id) -> Result<Option<NodeData>>` | 获取节点 |
| `update_node` | `(graph_id, node_id, label, description, node_type) -> Result<NodeData>` | 更新节点 |
| `delete_node` | `(graph_id, node_id) -> Result<()>` | 删除节点（包含子孙节点） |
| `create_edge` | `(graph_id, NewEdge) -> Result<EdgeData>` | 创建边 |
| `delete_edge` | `(graph_id, edge_id) -> Result<()>` | 删除边 |
| `get_children` | `(graph_id, parent_id) -> Result<Vec<NodeData>>` | 获取子节点 |
| `list_graph_ids` | `() -> Vec<String>` | 列出所有图 ID |
| `export_graph_json` | `(graph_id) -> Result<GraphJson>` | 导出图为 JSON |
| `import_graph_json` | `(graph_id, data) -> Result<()>` | 从 JSON 导入图 |

---

### PetgraphStore

#### `struct PetgraphStore`
源文件：`graph/petgraph_store.rs:15`

基于 `petgraph::stable_graph::StableDiGraph` 的内存图实现。

| 字段 | 类型 | 说明 |
|------|------|------|
| `inner` | `Arc<RwLock<GraphInner>>` | 内部状态 |

#### `struct GraphInner`
源文件：`graph/petgraph_store.rs:19`

| 字段 | 类型 | 说明 |
|------|------|------|
| `graph` | `StableDiGraph<NodeData, EdgeData>` | petgraph 有向图 |
| `node_id_to_index` | `HashMap<String, NodeIndex>` | 节点 ID → 索引映射 |
| `graph_id_to_nodes` | `HashMap<String, Vec<NodeIndex>>` | 图 ID → 节点索引列表 |

#### `impl PetgraphStore`
源文件：`graph/petgraph_store.rs:31`

- `fn new() -> Self` — 创建空图存储
- `async fn create_node(graph_id, input) -> Result<NodeData>` — 创建节点，自动生成 UUID，parent_id 决定层次
- `async fn get_node(graph_id, node_id) -> Result<Option<NodeData>>` — 获取节点，校验 graph_id
- `async fn update_node(...) -> Result<NodeData>` — 更新节点字段，校验 graph_id，更新 updated_at
- `async fn delete_node(graph_id, node_id) -> Result<()>` — **级联删除**：递归删除所有子孙节点及关联边
- `async fn create_edge(graph_id, input) -> Result<EdgeData>` — 创建边，校验源/目标节点存在
- `async fn delete_edge(graph_id, edge_id) -> Result<()>` — 删除边
- `async fn get_children(graph_id, parent_id) -> Result<Vec<NodeData>>` — 获取直接子节点
- `async fn list_graph_ids() -> Vec<String>` — 列出有节点的图 ID
- `async fn export_graph_json(graph_id) -> Result<GraphJson>` — 导出指定图的节点和边
- `async fn import_graph_json(graph_id, data) -> Result<()>` — **替换导入**：先删除旧图数据，再导入新数据

---

### 类型定义

#### `NodeData`
源文件：`graph/types.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 节点 ID（UUID） |
| `label` | `String` | 标签/名称 |
| `node_type` | `String` | 节点类型（concept/category 等） |
| `parent_id` | `Option<String>` | 父节点 ID（null 表示根节点） |
| `description` | `Option<String>` | 描述 |
| `graph_id` | `String` | 所属图 ID |
| `markdown_path` | `Option<String>` | Markdown 文件路径（格式：`nodes/{id}.md`） |
| `created_at` | `String` | 创建时间（RFC3339） |
| `updated_at` | `String` | 更新时间（RFC3339） |

#### `EdgeData`
源文件：`graph/types.rs:16`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 边 ID（UUID） |
| `source_id` | `String` | 源节点 ID |
| `target_id` | `String` | 目标节点 ID |
| `relation` | `String` | 关系类型 |
| `label` | `Option<String>` | 标签 |
| `graph_id` | `String` | 所属图 ID |

#### `NewNode`
源文件：`graph/types.rs:26`

创建节点时的输入结构。

| 字段 | 类型 | 说明 |
|------|------|------|
| `label` | `String` | 标签 |
| `node_type` | `String` | 节点类型 |
| `parent_id` | `Option<String>` | 父节点 ID |
| `description` | `Option<String>` | 描述 |

#### `NewEdge`
源文件：`graph/types.rs:34`

创建边时的输入结构。

| 字段 | 类型 | 说明 |
|------|------|------|
| `source_id` | `String` | 源节点 ID |
| `target_id` | `String` | 目标节点 ID |
| `relation` | `String` | 关系类型 |
| `label` | `Option<String>` | 标签 |

#### `GraphJson`
源文件：`graph/types.rs:42`

图序列化格式，用于持久化（graph.json）。

| 字段 | 类型 | 说明 |
|------|------|------|
| `nodes` | `Vec<NodeData>` | 节点列表 |
| `edges` | `Vec<EdgeData>` | 边列表 |

---

## LLM 适配层

> 源码路径：`ring-server/src/services/llm_provider.rs`、`llm_openai.rs`、`llm_anthropic.rs`

### LlmProvider Trait

#### `trait LlmProvider`
源文件：`services/llm_provider.rs:53`

| 方法 | 签名 | 说明 |
|------|------|------|
| `chat_stream` | `(messages, tools) -> Result<Pin<Box<dyn Stream<Item=LlmEvent> + Send>>>` | 发起流式对话 |

---

### 数据结构

#### `LlmMessage`
源文件：`services/llm_provider.rs:10`

| 字段 | 类型 | 说明 |
|------|------|------|
| `role` | `String` | 角色（system/user/assistant/tool） |
| `content` | `String` | 内容 |

#### `TokenUsage`
源文件：`services/llm_provider.rs:16`

| 字段 | 类型 | 说明 |
|------|------|------|
| `prompt_tokens` | `u32` | 提示 Token 数 |
| `completion_tokens` | `u32` | 完成 Token 数 |
| `total_tokens` | `u32` | 总 Token 数 |

#### `LlmEvent` 枚举
源文件：`services/llm_provider.rs:24`

流式事件类型，`serde` tag 格式：

- `Text { content: String }` — 文本片段
- `ToolCall { tool_call_id, tool, input }` — 工具调用请求
- `ToolResult { tool_call_id, tool, output }` — 工具执行结果
- `ArchiveSuggestion { data }` — 归档建议
- `BlueprintProposal { data }` — 蓝图提案
- `Error { code, message }` — 错误
- `Done { message_id, token_usage }` — 结束信号

---

### OpenAiProvider

#### `struct OpenAiProvider`
源文件：`services/llm_openai.rs:22`

支持 OpenAI API 和 Ollama（通过自定义 base_url）。

| 字段 | 类型 | 说明 |
|------|------|------|
| `api_key` | `String` | API Key |
| `model` | `String` | 模型名称（如 gpt-4o、llama3） |
| `base_url` | `Option<String>` | 自定义 Base URL（Ollama 用） |

#### `impl OpenAiProvider`
源文件：`services/llm_openai.rs:28`

- `fn new(api_key, model, base_url) -> Self` — 构造函数
- `fn build_client() -> OpenAIClient<OpenAIConfig>` — 构建客户端
- `async fn chat_stream(messages, tools) -> Result<...>` — 流式对话，通过 `async-openai` 调用 OpenAI/Ollama API

#### `convert_messages(messages: &[LlmMessage]) -> Vec<ChatCompletionRequestMessage>`
源文件：`services/llm_openai.rs:46`

将 `LlmMessage` 转换为 OpenAI 格式。system → SystemMessage，assistant → AssistantMessage，tool → ToolMessage，user/其他 → UserMessage。

---

### AnthropicProvider

#### `struct AnthropicProvider`
源文件：`services/llm_anthropic.rs:12`

支持 Anthropic Claude API。

| 字段 | 类型 | 说明 |
|------|------|------|
| `api_key` | `String` | API Key |
| `model` | `String` | 模型名称（如 claude-3-5-sonnet） |
| `base_url` | `Option<String>` | 自定义 Base URL |

#### `impl AnthropicProvider`
源文件：`services/llm_anthropic.rs:18`

- `fn new(api_key, model, base_url) -> Self` — 构造函数
- `fn base_url() -> &str` — 返回 base URL，默认 `https://api.anthropic.com`
- `async fn chat_stream(messages, tools) -> Result<...>` — 流式对话，通过 `reqwest` 发送 SSE 请求

#### `convert_messages(messages: &[LlmMessage]) -> (Option<String>, Vec<Value>)`
源文件：`services/llm_anthropic.rs:34`

将 `LlmMessage` 转换为 Anthropic 格式。多个 system 消息合并为一个 system，role 保持不变。

#### `parse_sse_event(data: &str) -> Option<AnthropicStreamEvent>`
源文件：`services/llm_anthropic.rs:74`

解析 Anthropic SSE 事件。处理 `content_block_delta`（text_delta/input_json_delta）、`content_block_start`（tool_use）、`content_block_stop`、`message_stop`、`error` 类型。

---

### MockLlmProvider

#### `struct MockLlmProvider`
源文件：`services/llm_provider.rs:62`

用于测试的模拟 LLM Provider。

| 字段 | 类型 | 说明 |
|------|------|------|
| `events` | `Vec<LlmEvent>` | 预设事件列表 |

- `fn new(events: Vec<LlmEvent>) -> Self` — 构造函数
- `async fn chat_stream(...) -> Result<...>` — 返回预设事件的流

---

## 工具引擎

> 源码路径：`ring-server/src/services/tool_engine/`

### Tool Trait

#### `trait Tool`
源文件：`services/tool_engine/mod.rs:13`

| 方法 | 签名 | 说明 |
|------|------|------|
| `definition` | `() -> ToolDefinition` | 返回工具定义（名称、描述、参数 Schema） |
| `execute` | `(input) -> Result<serde_json::Value>` | 执行工具，返回 JSON 结果 |

---

### ToolRegistry

#### `struct ToolRegistry`
源文件：`services/tool_engine/registry.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `tools` | `HashMap<String, Arc<dyn Tool>>` | 工具注册表 |

#### `impl ToolRegistry`
源文件：`services/tool_engine/registry.rs:11`

- `fn new() -> Self` — 构造函数
- `fn register(tool: Arc<dyn Tool>)` — 注册工具（按 definition().name 索引）
- `fn get(name) -> Option<Arc<dyn Tool>>` — 获取工具
- `fn definitions() -> Vec<ToolDefinition>` — 获取所有工具定义

---

### ToolDispatcher

#### `struct ToolDispatcher`
源文件：`services/tool_engine/dispatcher.rs:6`

| 字段 | 类型 | 说明 |
|------|------|------|
| `registry` | `Arc<ToolRegistry>` | 工具注册表 |

#### `impl ToolDispatcher`
源文件：`services/tool_engine/dispatcher.rs:10`

- `fn new(registry) -> Self` — 构造函数
- `fn definitions() -> Vec<ToolDefinition>` — 获取所有工具定义
- `async fn dispatch(call) -> ToolResultRecord` — 调度工具执行
  - 查找工具并调用 `execute(input)`
  - 成功：`success: true` + `output`
  - 失败：`success: false` + 错误 JSON
  - 工具不存在：返回 unknown tool 错误

---

### 工具实现

#### SearchTool
源文件：`services/tool_engine/tools/search_tool.rs`

| 字段 | 类型 | 说明 |
|------|------|------|
| `repo` | `Arc<dyn Repository>` | 数据库 |

- **工具名**：`search`
- **描述**：Full-text search knowledge graph nodes
- **参数**：`{ query: string, graph_ids?: string[], limit?: integer }`
- **返回**：`{ results: SearchResult[] }`

---

#### TextCleanTool
源文件：`services/tool_engine/tools/text_clean_tool.rs`

- **工具名**：`text_clean`
- **描述**：Clean and normalize text by stripping extra whitespace and normalizing unicode
- **参数**：`{ text: string }`
- **返回**：`{ cleaned_text: string }`

---

#### WebScrapeTool
源文件：`services/tool_engine/tools/web_scrape_tool.rs`

| 字段 | 类型 | 说明 |
|------|------|------|
| `client` | `Client` | HTTP 客户端 |

- **工具名**：`web_scrape`
- **描述**：Fetch a web page and extract its title and text content
- **参数**：`{ url: string }`
- **返回**：`{ title: string, text: string }`（提取 p、h1-h6、li、td 标签内容）

---

#### MarkdownGenTool
源文件：`services/tool_engine/tools/markdown_gen_tool.rs`

- **工具名**：`markdown_gen`
- **描述**：Generate formatted markdown from a title and sections
- **参数**：
  ```json
  {
    "title": "string",
    "sections": [{ "heading": "string", "body": "string" }]
  }
  ```
- **返回**：`{ markdown: string }`（格式：`# {title}\n\n## {heading}\n\n{body}`）

---

#### PrivacyFilterTool
源文件：`services/tool_engine/tools/privacy_filter_tool.rs`

| 字段 | 类型 | 说明 |
|------|------|------|
| `email_re` | `Regex` | 邮箱正则 |
| `phone_re` | `Regex` | 手机号正则（中国 1[3-9] 开头） |
| `id_card_re` | `Regex` | 身份证号正则（18位） |

- **工具名**：`privacy_filter`
- **描述**：Redact PII (email, phone, ID card) from text
- **参数**：`{ text: string }`
- **返回**：`{ filtered_text: string, redactions_count: number }`

---

### 预注册工具（main.rs 中）

源文件：`main.rs:70-76`

在 `AppState` 构建时注册 5 个工具：

```rust
registry.register(Arc::new(SearchTool::new(db.clone())));
registry.register(Arc::new(TextCleanTool::new()));
registry.register(Arc::new(WebScrapeTool::new()));
registry.register(Arc::new(MarkdownGenTool::new()));
registry.register(Arc::new(PrivacyFilterTool::new()));
```

---

## Git 服务

> 源码路径：`ring-server/src/services/git_service.rs`、`gitlab_service.rs`

### GitService

#### `struct GitService`
源文件：`services/git_service.rs:29`

纯 `git2` 封装，所有操作通过 `tokio::task::spawn_blocking` 在阻塞线程池执行。

#### 公开方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `fn new() -> Self` | 构造函数 |
| `async fn init_repo(path) -> Result<()>` | 在指定路径初始化 git 仓库 |
| `async fn clone_repo(url, to_path) -> Result<()>` | 克隆仓库 |
| `async fn add_all(repo_path) -> Result<()>` | `git add .` 所有文件 |
| `async fn commit(repo_path, message) -> Result<String>` | 提交，返回 commit SHA |
| `async fn create_branch(repo_path, name) -> Result<()>` | 创建分支 |
| `async fn get_current_branch(repo_path) -> Result<String>` | 获取当前分支名 |
| `async fn get_diff(repo_path, from, to) -> Result<DiffResult>` | 获取两个 commit 间的 diff |
| `async fn get_log(repo_path, limit) -> Result<Vec<CommitInfo>>` | 获取提交日志（按时间排序） |
| `async fn has_changes(repo_path) -> Result<bool>` | 是否有未提交变更 |
| `async fn status_files(repo_path) -> Result<Vec<String>>` | 获取变更文件列表 |

---

### GitlabService

#### `struct GitlabService`
源文件：`services/gitlab_service.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `base_url` | `String` | GitLab 实例 URL |
| `token` | `String` | Private Token |
| `client` | `reqwest::Client` | HTTP 客户端 |

#### 公开方法

| 方法 | 签名 | 说明 |
|------|------|------|
| `fn new(base_url, token) -> Self` | 构造函数（自动去除 URL 尾部斜杠） |
| `async fn create_repo(name, namespace) -> Result<CreateRepoResponse>` | 创建项目 |
| `async fn create_mr(project_id, source_branch, target_branch, title) -> Result<MergeRequestInfo>` | 创建 MR |
| `async fn merge_mr(project_id, mr_iid) -> Result<MergeRequestInfo>` | 合并 MR |
| `async fn close_mr(project_id, mr_iid) -> Result<MergeRequestInfo>` | 关闭 MR |
| `async fn list_mrs(project_id, state) -> Result<Vec<MergeRequestInfo>>` | 列出 MR |
| `async fn get_mr_diff(project_id, mr_iid) -> Result<Vec<MrDiff>>` | 获取 MR diff |
| `fn get_repo_url(project_path) -> Result<String>` | 拼接 git 仓库 URL |

---

### 数据结构

#### `PullResult`
源文件：`services/git_service.rs:5`

| 字段 | 类型 | 说明 |
|------|------|------|
| `had_changes` | `bool` | 是否有变更 |
| `changed_files` | `Vec<String>` | 变更文件列表 |

#### `CommitInfo`
源文件：`services/git_service.rs:10`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | Commit SHA |
| `message` | `String` | 提交信息 |
| `author` | `String` | 作者名称 |
| `timestamp` | `String` | 时间戳 |

#### `DiffResult`
源文件：`services/git_service.rs:17`

| 字段 | 类型 | 说明 |
|------|------|------|
| `files` | `Vec<FileDiff>` | 变更文件列表 |

#### `FileDiff`
源文件：`services/git_service.rs:21`

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `String` | 文件路径 |
| `status` | `String` | 状态（added/modified/deleted/renamed/unknown） |
| `additions` | `i64` | 新增行数 |
| `deletions` | `i64` | 删除行数 |
| `content` | `String` | diff 内容摘要（old → new） |

#### `CreateRepoResponse`
源文件：`services/git_model.rs:14`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 项目 ID |
| `url` | `String` | HTTP 克隆 URL |
| `ssh_url` | `String` | SSH 克隆 URL |

#### `MergeRequestInfo`
源文件：`services/gitlab_service.rs:23`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | MR ID |
| `iid` | `i64` | MR 编号 |
| `title` | `String` | 标题 |
| `author` | `MergeRequestAuthor` | 作者信息 |
| `state` | `String` | 状态 |
| `web_url` | `String` | Web URL |

#### `MergeRequestAuthor`
源文件：`services/gitlab_service.rs:34`

| 字段 | 类型 | 说明 |
|------|------|------|
| `username` | `String` | 用户名 |

#### `MrDiff`
源文件：`services/gitlab_service.rs:45`

| 字段 | 类型 | 说明 |
|------|------|------|
| `old_path` | `String` | 旧文件路径 |
| `new_path` | `String` | 新文件路径 |
| `diff` | `String` | unified diff |

---

## Handler 层

> 源码路径：`ring-server/src/handlers/`

### 通用模式

所有需要认证的 handler 第一个参数都是 `Extension<AuthUser>`，从中获取 `auth_user.user_id`。

所有返回 SSE 的 handler 返回类型为 `SseStream`（`Sse<ReceiverStream<Result<Event, Infallible>>>`）。

SSE 事件格式：`event: message\ndata: {json}\n\n`

---

### Setup Handler

源文件：`handlers/setup.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `get_status` | GET | `/setup/status` | 否 | 返回 setup_completed/step/user_id |
| `set_username` | POST | `/setup/username` | 否 | 创建用户（display_name ≤ 50 字符） |
| `set_llm` | POST | `/setup/llm` | 否 | 保存 LLM 配置到 settings |
| `set_gitlab` | POST | `/setup/gitlab` | 否 | 保存 GitLab 配置到 settings |
| `complete` | POST | `/setup/complete` | 否 | 完成 setup 流程 |

#### SetupStatus
| 字段 | 类型 | 说明 |
|------|------|------|
| `setup_completed` | `bool` | 是否完成 |
| `step` | `string` | 当前步骤 |
| `user_id` | `Option<String>` | 用户 ID |

---

### Ring Handler

源文件：`handlers/ring.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_rings` | GET | `/rings` | 是 | 列出当前用户的 Ring |
| `create_ring` | POST | `/rings` | 是 | 创建 Ring，返回 201 |
| `get_ring` | GET | `/rings/{ringId}` | 是 | 获取 Ring 详情 |
| `update_ring` | PUT | `/rings/{ringId}` | 是 | 更新 Ring（name/description） |
| `delete_ring` | DELETE | `/rings/{ringId}` | 是 | 删除 Ring |

---

### Member Handler

源文件：`handlers/member.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_members` | GET | `/rings/{ringId}/members` | 是 | 列出成员 |
| `generate_invite` | POST | `/rings/{ringId}/members/invites` | 是 | 生成邀请码 |
| `update_role` | PUT | `/rings/{ringId}/members/{memberId}/role` | 是 | 更新成员角色 |
| `remove_member` | DELETE | `/rings/{ringId}/members/{memberId}` | 是 | 移除成员 |
| `join_ring` | POST | `/rings/join?token=xxx` | 是 | 通过邀请码加入 Ring |

---

### Session Handler

源文件：`handlers/session.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `create_session` | POST | `/rings/{ringId}/sessions` | 是 | 创建 Session |
| `list_sessions` | GET | `/rings/{ringId}/sessions` | 是 | 列出 Session |
| `get_session` | GET | `/rings/{ringId}/sessions/{sessionId}` | 是 | 获取 Session 详情 |
| `close_session` | POST | `/rings/{ringId}/sessions/{sessionId}/close` | 是 | 关闭 Session |
| `leave_session` | POST | `/rings/{ringId}/sessions/{sessionId}/leave` | 是 | 离开 Session |
| `toggle_archive` | PUT | `/rings/{ringId}/sessions/{sessionId}/archive-toggle` | 是 | 开关归档 |
| `invite_member` | POST | `/rings/{ringId}/sessions/{sessionId}/invite` | 是 | 邀请成员 |
| `delete_session` | DELETE | `/rings/{ringId}/sessions/{sessionId}` | 是 | 删除 Session |
| `get_messages` | GET | `/rings/{ringId}/sessions/{sessionId}/messages` | 是 | 获取消息 |
| `send_message` | POST | `/rings/{ringId}/sessions/{sessionId}/messages` | 是 | **SSE**：发送消息，AI 流式响应 |

---

### Conversation Handler

源文件：`handlers/conversation.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list` | GET | `/rings/{ringId}/conversations` | 是 | 列出对话 |
| `create` | POST | `/rings/{ringId}/conversations` | 是 | 创建对话 |
| `get` | GET | `/rings/{ringId}/conversations/{convId}` | 是 | 获取对话详情 |
| `get_messages` | GET | `/rings/{ringId}/conversations/{convId}/messages` | 是 | 获取消息 |
| `send_message` | POST | `/rings/{ringId}/conversations/{convId}/messages` | 是 | **SSE**：发送消息，AI 流式响应 |

---

### Blueprint Handler

源文件：`handlers/blueprint.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_templates` | GET | `/rings/{ringId}/blueprint/templates` | 是 | 列出蓝图模板 |
| `blueprint_chat` | POST | `/rings/{ringId}/blueprint/chat` | 是 | **SSE**：蓝图构建对话 |
| `preview_blueprint` | POST | `/rings/{ringId}/blueprint/preview` | 是 | 生成蓝图预览（mermaid） |
| `confirm_blueprint` | POST | `/rings/{ringId}/blueprint/confirm` | 是 | 确认蓝图，创建图谱节点，激活 Ring |

---

### Graph Handler

源文件：`handlers/graph.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_graphs` | GET | `/rings/{ringId}/graphs` | 是 | 列出图 ID |
| `get_graph` | GET | `/rings/{ringId}/graphs/{graphId}` | 是 | 获取图详情（节点+边） |
| `create_node` | POST | `/rings/{ringId}/graphs/{graphId}/nodes` | 是 | 创建节点，返回 201 |
| `get_node` | GET | `/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}` | 是 | 获取节点 |
| `update_node` | PUT | `/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}` | 是 | 更新节点 |
| `delete_node` | DELETE | `/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}` | 是 | 删除节点（级联） |
| `get_node_content` | GET | `/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}/content` | 是 | 获取 Markdown 内容 |
| `create_edge` | POST | `/rings/{ringId}/graphs/{graphId}/edges` | 是 | 创建边 |
| `delete_edge` | DELETE | `/rings/{ringId}/graphs/{graphId}/edges/{edgeId}` | 是 | 删除边 |

---

### Search Handler

源文件：`handlers/search.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `search_nodes` | POST | `/rings/{ringId}/search` | 是 | FTS 全文搜索节点 |

---

### Archive Handler

源文件：`handlers/archive.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `archive` | POST | `/rings/{ringId}/archive` | 是 | 归档对话消息到 Markdown |
| `get_queue` | GET | `/rings/{ringId}/archive/queue` | 是 | 获取待审核队列 |
| `confirm_archive` | POST | `/rings/{ringId}/archive/{archiveId}/confirm` | 是 | 确认归档 |

---

### Git Handler

源文件：`handlers/git.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_prs` | GET | `/rings/{ringId}/git/prs` | 是 | 列出 MR |
| `merge_pr` | POST | `/rings/{ringId}/git/prs/{prId}/merge` | 是 | 合并 MR |
| `reject_pr` | POST | `/rings/{ringId}/git/prs/{prId}/reject` | 是 | 拒绝 MR |
| `get_commit_log` | GET | `/rings/{ringId}/git/commits` | 是 | 获取提交日志 |

---

### Notification Handler

源文件：`handlers/notification.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `list_notifications` | GET | `/notifications` | 是 | 列出通知 |
| `mark_read` | POST | `/notifications/{notificationId}` | 是 | 标记已读 |

---

### Settings Handler

源文件：`handlers/settings.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `get_settings` | GET | `/settings` | 是 | 获取所有设置 |
| `update_settings` | PUT | `/settings` | 是 | 更新设置 |

---

### AI Handler

源文件：`handlers/ai.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `super_ring_chat` | POST | `/super-ring/chat` | 是 | **SSE**：全局超级助手对话 |

---

### WebSocket Handler

源文件：`handlers/ws.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `ws_handler` | GET | `/ws/{ringId}` | 是（通过 query/cookie） | WebSocket 升级，处理 Ring 内实时消息 |

**流程**：升级 → 订阅 ring_id 的 WsHub → 转发收到的消息到客户端

---

### Install Handler

源文件：`handlers/install.rs`

| Handler | 方法 | 路径 | 认证 | 说明 |
|---------|------|------|------|------|
| `join_page` | GET | `/join` | 否 | 公开安装引导页面（显示 Ring 信息、下载链接） |

---

### SSE Helpers

源文件：`handlers/sse_helpers.rs`

#### `type SseStream`
`Sse<ReceiverStream<Result<Event, Infallible>>>`

#### `fn spawn_sse_stream(llm_stream) -> SseStream`
将 `LlmEvent` 流转换为 SSE 响应，事件名为 `message`。

#### `fn spawn_sse_stream_with_callback(llm_stream, on_complete) -> SseStream`
同上，但追加 `on_complete` 回调：流结束后收集所有 `LlmEvent::Text`，调用 `on_complete(content_string)`。

---

## 中间件

> 源码路径：`ring-server/src/middleware/auth.rs`

### AuthUser

#### `struct AuthUser`
源文件：`middleware/auth.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `user_id` | `String` | 用户 ID |

通过 Axum Extension 机制传递，handler 中通过 `Extension<AuthUser>` 提取。

---

### auth_middleware

#### `async fn auth_middleware(request: Request, next: Next) -> Response`
源文件：`middleware/auth.rs:12`

**认证流程**：
1. 从请求头 `X-User-Id` 获取用户 ID
2. 如果存在：插入 `AuthUser` extension → 执行后续 handler
3. 如果不存在：返回 HTTP 401 Unauthorized，`{ "error": "missing X-User-Id header" }`

**使用方式**（在 `routes.rs` 中）：
```rust
.layer(middleware::from_fn(auth_middleware))
```
