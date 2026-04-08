# 数据库层 API 参考

> 源码路径：`ring-server/src/db/`

## Repository Trait

### `trait Repository`
源文件：`db/traits.rs:15`

所有数据库操作的统一接口。所有方法均为 `async`，返回 `Result`。

#### 用户相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_user` | `(NewUser) -> Result<User>` | 创建用户 |
| `get_user` | `(id: &str) -> Result<Option<User>>` | 获取用户 |
| `list_all_users` | `() -> Result<Vec<User>>` | 列出所有用户 |
| `is_setup_completed` | `() -> Result<bool>` | 检查设置是否完成 |
| `complete_setup` | `(user_id: &str) -> Result<()>` | 完成设置 |

#### Ring 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_ring` | `(NewRing) -> Result<Ring>` | 创建 Ring |
| `get_ring` | `(id: &str) -> Result<Option<Ring>>` | 获取 Ring |
| `list_rings_by_user` | `(user_id: &str) -> Result<Vec<Ring>>` | 列出用户的所有 Ring |
| `update_ring` | `(id, name, description) -> Result<Ring>` | 更新 Ring |
| `delete_ring` | `(id: &str) -> Result<()>` | 删除 Ring |
| `update_ring_status` | `(id, status) -> Result<()>` | 更新 Ring 状态 |

#### 邀请相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_invite_token` | `(ring_id, token, token_type, role, inviter_id) -> Result<InviteToken>` | 创建邀请码 |
| `get_invite_token` | `(token: &str) -> Result<Option<InviteToken>>` | 获取邀请码 |
| `count_members_by_ring` | `(ring_id: &str) -> Result<i64>` | 统计成员数量 |

#### 设置相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `get_setting` | `(key: &str) -> Result<Option<String>>` | 获取设置项 |
| `set_setting` | `(key, value) -> Result<()>` | 设置设置项 |

#### Member 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_member` | `(NewMember) -> Result<Member>` | 创建成员 |
| `get_member` | `(id: &str) -> Result<Option<Member>>` | 获取成员 |
| `list_members_by_ring` | `(ring_id: &str) -> Result<Vec<Member>>` | 列出 Ring 成员 |
| `get_member_by_user_and_ring` | `(user_id, ring_id) -> Result<Option<Member>>` | 获取用户在 Ring 中的成员信息 |
| `update_member_role` | `(id, role) -> Result<()>` | 更新成员角色 |
| `delete_member` | `(id: &str) -> Result<()>` | 删除成员 |
| `get_next_token_id` | `(ring_id: &str) -> Result<i64>` | 获取下一个 Token ID |

#### Conversation 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_conversation` | `(ring_id, title, context_mode, created_by) -> Result<Conversation>` | 创建对话 |
| `list_conversations` | `(ring_id: &str) -> Result<Vec<Conversation>>` | 列出对话 |
| `get_conversation` | `(id: &str) -> Result<Option<Conversation>>` | 获取对话 |
| `create_message` | `(conversation_id, role, content, sender_id) -> Result<Message>` | 创建消息 |
| `get_messages` | `(conversation_id, limit, before_id) -> Result<Vec<Message>>` | 获取消息 |

#### Blueprint 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `list_blueprint_templates` | `() -> Result<Vec<BlueprintTemplate>>` | 列出模板 |
| `create_blueprint_template` | `(id, name, description, graphs_json, is_system) -> Result<BlueprintTemplate>` | 创建模板 |

#### 搜索相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `index_node_search` | `(node_id, graph_id, label, content) -> Result<()>` | 索引节点到 FTS |
| `delete_node_search` | `(node_id: &str) -> Result<()>` | 从 FTS 删除节点 |
| `search_nodes_fts` | `(query, graph_ids, limit) -> Result<Vec<SearchResult>>` | FTS 全文搜索 |

#### Archive 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_archive_record` | `(...) -> Result<()>` | 创建归档记录 |
| `list_archive_records_by_ring` | `(ring_id: &str) -> Result<Vec<ArchiveRecord>>` | 列出归档记录 |
| `get_archive_record` | `(id: &str) -> Result<Option<ArchiveRecord>>` | 获取归档记录 |
| `update_archive_pr_status` | `(id, pr_status) -> Result<()>` | 更新 PR 状态 |

#### Notification 相关
| 方法 | 签名 | 说明 |
|------|------|------|
| `create_notification` | `(NewNotification) -> Result<Notification>` | 创建通知 |
| `list_notifications_by_user` | `(user_id, unread_only) -> Result<Vec<Notification>>` | 列出通知 |
| `mark_notification_read` | `(id: &str) -> Result<()>` | 标记已读 |

#### Session 相关
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

## SqliteRepository

### `struct SqliteRepository`
源文件：`db/sqlite/mod.rs:28`

| 字段 | 类型 | 说明 |
|------|------|------|
| `pool` | `SqlitePool` | SQLx 连接池 |
| `jieba` | `Mutex<Option<jieba_rs::Jieba>>` | 中文分词器（懒加载） |

### `impl SqliteRepository`
源文件：`db/sqlite/mod.rs:33`

- `fn new(pool: SqlitePool) -> Self` — 构造函数
- `fn pool(&self) -> &SqlitePool` — 获取连接池引用
- `fn get_jieba(&self) -> jieba_rs::Jieba` — 获取分词器（懒加载初始化）

所有 `Repository` trait 方法均有 `async_trait` 实现，底层调用对应的 `xxx_inner` 私有方法。
