# 后端核心 API 参考

> 源码路径：`ring-server/src/`

## Config

### `Config`
源文件：`config.rs:3`

| 字段 | 类型 | 说明 |
|------|------|------|
| `port` | `u16` | 服务器监听端口，默认 7420 |
| `data_dir` | `PathBuf` | 数据目录，默认 `~/.ring/` |
| `release_repo` | `String` | GitHub release 仓库地址 |
| `database_url` | `String` | SQLite 数据库连接 URL |

### `impl Default for Config`
源文件：`config.rs:11`

- **环境变量**：`RING_PORT`、`RING_DATA_DIR`、`RING_DATABASE_URL`、`RING_RELEASE_REPO`
- **默认端口**：7420

---

## Error

### `RingError` 枚举
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

### `Result<T>`
源文件：`error.rs:59`

类型别名：`std::result::Result<T, RingError>`

---

## State

### `AppState`
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

### `impl AppState`
源文件：`state.rs:24`

- `async fn rebuild_llm(&self) -> Arc<dyn LlmProvider>` — 根据数据库中的 LLM 配置重建 LLM Provider（openai/ollama/anthropic）

---

## Routes

### `fn build_router(state: AppState) -> Router`
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

**Super Ring & WebSocket**
| 方法 | 路径 | Handler |
|------|------|---------|
| POST | `/api/v1/super-ring/chat` | `ai::super_ring_chat` |
| GET | `/api/v1/ws/{ringId}` | `ws::ws_handler` |

**Public 路由**
| 方法 | 路径 | Handler |
|------|------|---------|
| GET | `/join` | `install::join_page` |
