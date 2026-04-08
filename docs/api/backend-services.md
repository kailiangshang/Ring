# 业务服务层 API 参考

> 源码路径：`ring-server/src/services/`

## RingService

### `struct RingService`
源文件：`services/ring_service.rs:21`

| 字段 | 类型 | 说明 |
|------|------|------|
| `repo` | `Arc<dyn Repository>` | 数据库 |
| `data_dir` | `PathBuf` | 数据目录 |

### `impl RingService`
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

### `CreateRingRequest`
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

## MemberService

### `struct MemberService`
源文件：`services/member_service.rs:8`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |

### `impl MemberService`
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

## SessionService

### `struct SessionService`
源文件：`services/session_service.rs:8`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |
| `permission` | `PermissionService` | 权限服务 |

### `impl SessionService`
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

## SearchService

### `struct SearchService`
源文件：`services/search_service.rs:8`

| 字段 | 类型 | 说明 |
|------|------|------|
| `repo` | `Arc<dyn Repository>` | 数据库 |
| `store` | `Arc<dyn GraphStore>` | 图存储 |

### `impl SearchService`
源文件：`services/search_service.rs:14`

- `fn new(repo, store) -> Self` — 构造函数
- `async fn search_nodes(query, graph_ids, limit) -> Result<Vec<SearchResult>>` — FTS 全文搜索
- `async fn index_node(node_id, graph_id, label, content) -> Result<()>` — 索引节点
- `async fn delete_node_index(node_id) -> Result<()>` — 删除索引

---

## ArchiveService

### `struct ArchiveService`
源文件：`services/archive_service.rs:14`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |
| `git_service` | `Arc<GitService>` | Git 服务 |
| `graph_store` | `Arc<dyn GraphStore>` | 图存储 |
| `gitlab_service` | `Option<Arc<GitlabService>>` | GitLab 服务 |

### `impl ArchiveService`
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

## GraphService

### `struct GraphService`
源文件：`services/graph_service.rs:10`

| 字段 | 类型 | 说明 |
|------|------|------|
| `store` | `Arc<dyn GraphStore>` | 图存储 |
| `search_service` | `Arc<SearchService>` | 搜索服务 |
| `data_dir` | `PathBuf` | 数据目录 |

### `impl GraphService`
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

### Markdown 持久化格式
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

## SettingsService

### `struct SettingsService`
源文件：`services/settings_service.rs:6`

| 字段 | 类型 | 说明 |
|------|------|------|
| `repo` | `Arc<dyn Repository>` | 数据库 |

### `impl SettingsService`
源文件：`services/settings_service.rs:10`

- `fn new(repo) -> Self` — 构造函数
- `async fn get_all_settings() -> Result<serde_json::Value>` — 获取所有设置（llm_provider/model/api_key/base_url、privacy_enabled、user_id、display_name）
- `async fn update_settings(settings) -> Result<()>` — 更新设置（仅允许：llm_provider/model/api_key/base_url、privacy_enabled）

---

## NotificationService

### `struct NotificationService`
源文件：`services/notification_service.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |

### `impl NotificationService`
源文件：`services/notification_service.rs:11`

- `fn new(db) -> Self` — 构造函数
- `async fn list_for_user(user_id, unread_only) -> Result<Vec<Notification>>` — 列出通知
- `async fn mark_read(notification_id) -> Result<()>` — 标记已读

---

## PermissionService

### `struct PermissionService`
源文件：`services/permission_service.rs:6`

| 字段 | 类型 | 说明 |
|------|------|------|
| `db` | `Arc<dyn Repository>` | 数据库 |

### `impl PermissionService`
源文件：`services/permission_service.rs:10`

- `fn new(db) -> Self` — 构造函数
- `async fn check_ring_access(ring_id, user_id) -> Result<()>` — 检查 Ring 访问权限（创建者或成员）
- `async fn check_creator_or_admin(ring_id, user_id) -> Result<()>` — 检查创建者或管理员权限
- `async fn check_creator(ring_id, user_id) -> Result<()>` — 检查创建者权限
- `async fn get_member_role(ring_id, user_id) -> Result<Option<String>>` — 获取成员角色
- `async fn is_creator(ring_id, user_id) -> Result<bool>` — 是否创建者
- `async fn is_member(ring_id, user_id) -> Result<bool>` — 是否成员

---

## CredentialService

### `struct CredentialService`
源文件：`services/credential_service.rs:15`

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | `[u8; 32]` | 加密密钥（32 字节） |

### `impl CredentialService`
源文件：`services/credential_service.rs:19`

- `fn new(key) -> Self` — 构造函数
- `fn derive_key_from_password(password) -> [u8; 32]` — PBKDF2-HMAC-SHA256 密钥派生（600,000 轮）
- `fn encrypt(plaintext) -> Result<String>` — AES-256-GCM 加密，输出 Base64（salt + nonce + ciphertext）
- `fn decrypt(encrypted) -> Result<String>` — 解密

---

## WorkflowService

### `Workflow`
源文件：`services/workflow_service.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 工作流 ID |
| `name` | `String` | 名称 |
| `description` | `String` | 描述 |
| `system_prompt` | `String` | 系统提示词 |
| `tool_names` | `Vec<String>` | 启用的工具列表 |

### 预设工作流
源文件：`services/workflow_service.rs:16`

- `meeting_archive` — 会议归档：text_clean + privacy_filter + markdown_gen + search
- `deep_research` — 深度研究：search + web_scrape + text_clean + markdown_gen
- `learning_center` — 学习中心：search + text_clean + markdown_gen

### 函数
- `fn get_workflow(id) -> Option<Workflow>` — 获取工作流
- `fn list_workflows() -> Vec<Workflow>` — 列出所有工作流
- `fn filter_tools_for_workflow(all_tools, tool_names) -> Vec<ToolDefinition>` — 从工具列表中筛选指定工作流需要的工具

---

## TriggerService

### `fn check_archive_suggestion(last_assistant_text) -> Option<LlmEvent>`
源文件：`services/trigger_service.rs:3`

检测对话是否包含归档关键词（总结、归档、记录、要点、会议纪要）。包含则返回 `ArchiveSuggestion` 事件。

### `fn check_empty_graph_guidance(node_count) -> Option<LlmEvent>`
源文件：`services/trigger_service.rs:18`

当图谱节点数 < 3 时，返回 `ArchiveSuggestion` 事件建议。

---

## WsHub

### `struct WsHub`
源文件：`services/ws_hub.rs:12`

| 字段 | 类型 | 说明 |
|------|------|------|
| `channels` | `RwLock<HashMap<String, broadcast::Sender<WsMessage>>>` | Ring ID → 广播频道 |

### `impl WsHub`
源文件：`services/ws_hub.rs:22`

- `fn new() -> Self` — 构造函数
- `async fn subscribe(ring_id) -> broadcast::Receiver<WsMessage>` — 订阅 Ring 消息
- `async fn broadcast(ring_id, msg)` — 广播消息到 Ring

### `WsMessage`
源文件：`services/ws_hub.rs:6`

| 字段 | 类型 | 说明 |
|------|------|------|
| `msg_type` | `String` | 消息类型 |
| `payload` | `serde_json::Value` | 负载数据 |
