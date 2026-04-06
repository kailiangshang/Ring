# Ring 开发者指南

本文档是设计文档到代码的桥梁。涵盖项目结构、依赖、数据库、错误处理、状态管理、路由设计和构建方式。

---

## 1. Rust 项目结构

```
ring-server/
├── Cargo.toml
├── src/
│   ├── main.rs                    # 入口：启动 Axum、初始化 DB、启动服务
│   ├── config.rs                  # 配置管理（端口、数据目录、LLM 配置）
│   ├── error.rs                   # 统一错误类型
│   ├── state.rs                   # Axum AppState
│   ├── routes.rs                  # 路由注册
│   │
│   ├── db/                        # 数据层
│   │   ├── mod.rs                 # Repository trait + SqliteRepository
│   │   ├── traits/                # 子 trait
│   │   │   ├── ring_repo.rs
│   │   │   ├── member_repo.rs
│   │   │   ├── user_repo.rs
│   │   │   ├── conversation_repo.rs
│   │   │   ├── message_repo.rs
│   │   │   ├── archive_repo.rs
│   │   │   ├── session_repo.rs
│   │   │   ├── session_message_repo.rs
│   │   │   ├── invite_token_repo.rs
│   │   │   ├── notification_repo.rs
│   │   │   ├── settings_repo.rs
│   │   │   └── blueprint_repo.rs
│   │   ├── sqlite/               # SQLite 实现
│   │   │   ├── mod.rs
│   │   │   ├── ring_repo.rs
│   │   │   └── ...
│   │   └── migrations/
│   │       └── 001_initial.sql
│   │
│   ├── graph/                     # 图数据层
│   │   ├── mod.rs                 # GraphStore trait + PetgraphStore
│   │   ├── types.rs               # NodeData, EdgeData, NewNode, etc.
│   │   ├── petgraph_store.rs      # petgraph 实现
│   │   └── sync.rs                # graph.json 导入导出
│   │
│   ├── handlers/                  # HTTP handlers（只做参数解析 + 调 service + 返回响应）
│   │   ├── mod.rs
│   │   ├── setup.rs
│   │   ├── install.rs             # 安装导航页（公共，嵌入 HTML 模板渲染）
│   │   ├── ring.rs
│   │   ├── blueprint.rs
│   │   ├── conversation.rs
│   │   ├── graph.rs
│   │   ├── archive.rs
│   │   ├── git.rs
│   │   ├── member.rs
│   │   ├── session.rs
│   │   ├── notification.rs
│   │   ├── export.rs
│   │   ├── search.rs
│   │   ├── ai.rs
│   │   └── settings.rs
│   │
│   ├── services/                  # 业务逻辑层（所有业务逻辑在此）
│   │   ├── mod.rs
│   │   ├── ring_service.rs
│   │   ├── ai_service.rs
│   │   ├── llm_provider.rs        # LlmProvider trait
│   │   ├── llm_openai.rs          # async-openai 适配
│   │   ├── llm_anthropic.rs       # Anthropic 适配
│   │   ├── context_loader.rs      # .ring/ 文档加载
│   │   ├── graph_service.rs
│   │   ├── git_service.rs
│   │   ├── gitlab_service.rs
│   │   ├── archive_service.rs
│   │   ├── permission_service.rs
│   │   ├── blueprint_service.rs
│   │   ├── session_service.rs
│   │   ├── notification_service.rs
│   │   ├── search_service.rs
│   │   └── export_service.rs
│   │
│   ├── models/                    # 数据模型
│   │   ├── mod.rs
│   │   ├── ring.rs
│   │   ├── user.rs
│   │   ├── member.rs
│   │   ├── graph.rs
│   │   ├── conversation.rs
│   │   ├── message.rs
│   │   ├── session.rs
│   │   ├── archive.rs
│   │   ├── invite.rs
│   │   └── notification.rs
│   │
│   └── ws/                        # WebSocket
│       ├── mod.rs
│       ├── hub.rs
│       └── session_hub.rs
```

### 用户数据目录

Ring 在用户机器上的数据布局（产品代码和用户数据完全分离）：

```
~/.ring/
├── data/                          ← 本地数据
│   ├── ring.db                    ← SQLite
│   └── identity.json              ← 全局身份
├── repos/                         ← 用户加入的 Ring（群组）的 Git 仓库
│   ├── ring-竞品分析组/
│   │   ├── .ring-local/
│   │   │   └── identity.json      ← 本地用户身份（不进 Git）
│   │   ├── .ring/
│   │   │   ├── role.md
│   │   │   ├── conventions.md
│   │   │   └── ...
│   │   ├── graphs/
│   │   │   └── knowledge/graph.json
│   │   ├── nodes/
│   │   └── blueprint.json
│   └── ring-项目A/
│       └── ...
└── config.toml                    ← 全局配置
```

### 命名约定

- 代码中 `Ring` = 群组空间（与产品文档一致）
- 产品整体用 `ring-server`（二进制名）
- 产品源码仓库（monorepo）≠ 用户 Ring（群组）的 GitLab 数据仓库
- 全栈 `snake_case`（Rust 函数/变量、TypeScript 函数/变量、JSON 字段、API 路径）

---

## 2. Cargo.toml 依赖清单

```toml
[package]
name = "ring-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = { version = "0.8", features = ["ws", "macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "fs"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }

# Graph
petgraph = { version = "0.8", features = ["serde-1"] }

# Git
git2 = "0.20"

# HTTP client (GitLab API + Anthropic LLM)
reqwest = { version = "0.12", features = ["json", "stream"] }

# LLM
async-openai = "0.34"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# UUID
uuid = { version = "1", features = ["v4", "serde"] }

# Error handling
thiserror = "2"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Chinese tokenization (search)
jieba-rs = "0.8"

# Crypto (credential encryption)
aes-gcm = "0.10"
base64 = "0.22"

# SSE streaming
futures = "0.3"

[dev-dependencies]
tokio = { version = "1", features = ["test-util"] }
```

---

## 3. SQLite 迁移脚本

文件：`ring-server/migrations/001_initial.sql`

```sql
-- Ring Server Initial Schema

-- Users
CREATE TABLE IF NOT EXISTS users (
    id           TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    avatar_url   TEXT,
    ip_address   TEXT,
    setup_completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Rings (群组空间)
CREATE TABLE IF NOT EXISTS rings (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    creator_id      TEXT NOT NULL REFERENCES users(id),
    gitlab_repo     TEXT NOT NULL,
    local_path      TEXT NOT NULL,
    next_token_id   INTEGER NOT NULL DEFAULT 2,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Graphs (图谱元数据)
CREATE TABLE IF NOT EXISTS graphs (
    id          TEXT PRIMARY KEY,
    ring_id     TEXT NOT NULL REFERENCES rings(id),
    name        TEXT NOT NULL,
    description TEXT,
    graph_type  TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Members
CREATE TABLE IF NOT EXISTS members (
    id           TEXT PRIMARY KEY,
    ring_id      TEXT NOT NULL REFERENCES rings(id),
    user_id      TEXT NOT NULL,
    token_id     INTEGER NOT NULL,
    display_name TEXT NOT NULL,
    role         TEXT NOT NULL DEFAULT 'member',
    joined_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(ring_id, user_id),
    UNIQUE(ring_id, token_id)
);

-- Invite tokens
CREATE TABLE IF NOT EXISTS invite_tokens (
    id           TEXT PRIMARY KEY,
    ring_id      TEXT NOT NULL REFERENCES rings(id),
    token        TEXT NOT NULL UNIQUE,
    token_type   TEXT NOT NULL DEFAULT 'open',
    role         TEXT NOT NULL DEFAULT 'member',
    inviter_id   TEXT NOT NULL,
    max_uses     INTEGER NOT NULL DEFAULT 1,
    use_count    INTEGER NOT NULL DEFAULT 0,
    max_members  INTEGER,
    expires_at   DATETIME NOT NULL,
    used_at      DATETIME,
    revoked_at   DATETIME,
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Conversations
CREATE TABLE IF NOT EXISTS conversations (
    id              TEXT PRIMARY KEY,
    ring_id         TEXT NOT NULL REFERENCES rings(id),
    title           TEXT,
    mode            TEXT NOT NULL DEFAULT 'chat',
    context_mode    TEXT NOT NULL DEFAULT 'storage',
    token_count     INTEGER NOT NULL DEFAULT 0,
    token_limit     INTEGER NOT NULL DEFAULT 100000,
    auto_compact    BOOLEAN NOT NULL DEFAULT FALSE,
    summary         TEXT,
    compacted_at    DATETIME,
    created_by      TEXT NOT NULL REFERENCES users(id),
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Messages
CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id),
    role            TEXT NOT NULL,
    content         TEXT NOT NULL,
    sender_id       TEXT,
    tool_calls      TEXT,
    archived        BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);

-- Archive records
CREATE TABLE IF NOT EXISTS archive_records (
    id              TEXT PRIMARY KEY,
    ring_id         TEXT NOT NULL REFERENCES rings(id),
    node_id         TEXT,
    conversation_id TEXT REFERENCES conversations(id),
    message_ids     TEXT,
    markdown_path   TEXT NOT NULL,
    archived_by     TEXT NOT NULL,
    git_commit_sha  TEXT,
    pr_status       TEXT,
    pr_url          TEXT,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Blueprint templates
CREATE TABLE IF NOT EXISTS blueprint_templates (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    graphs      TEXT NOT NULL,
    is_system   BOOLEAN NOT NULL DEFAULT FALSE,
    created_by  TEXT,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Sessions
CREATE TABLE IF NOT EXISTS sessions (
    id              TEXT PRIMARY KEY,
    ring_id         TEXT NOT NULL REFERENCES rings(id),
    title           TEXT,
    scenario        TEXT NOT NULL,
    created_by      TEXT NOT NULL,
    archive_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Session members
CREATE TABLE IF NOT EXISTS session_members (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    user_id     TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'participant',
    status      TEXT NOT NULL DEFAULT 'active',
    joined_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    left_at     DATETIME,
    UNIQUE(session_id, user_id)
);

-- Session messages
CREATE TABLE IF NOT EXISTS session_messages (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    sender_id   TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'user',
    content     TEXT NOT NULL,
    seq_num     INTEGER NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_session_messages_seq ON session_messages(session_id, seq_num);

-- Notifications
CREATE TABLE IF NOT EXISTS notifications (
    id          TEXT PRIMARY KEY,
    ring_id     TEXT NOT NULL REFERENCES rings(id),
    user_id     TEXT NOT NULL REFERENCES users(id),
    type        TEXT NOT NULL,
    title       TEXT NOT NULL,
    body        TEXT,
    related_id  TEXT,
    is_read     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, is_read);

-- Settings
CREATE TABLE IF NOT EXISTS settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- FTS5 全文搜索（jieba-rs 预分词后空格拼接插入）
CREATE VIRTUAL TABLE IF NOT EXISTS nodes_search USING fts5(
    node_id,
    graph_id,
    label,
    content,
    tokenize='unicode61'
);
```

---

## 4. 错误类型设计

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RingError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("llm error: {0}")]
    Llm(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Internal(String),
}

impl axum::response::IntoResponse for RingError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            RingError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            RingError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            RingError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
            RingError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            RingError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, RingError>;
```

---

## 5. AppState 设计

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AppState {
    pub db: Arc<dyn Repository>,
    pub graph_store: Arc<RwLock<dyn GraphStore>>,
    pub git_service: Arc<dyn GitService>,
    pub llm_provider: Arc<dyn LlmProvider>,
    pub config: Arc<Config>,
    pub ws_hub: Arc<WsHub>,
    pub session_hub: Arc<SessionHub>,
}
```

`Config` 中包含 `release_repo` 字段（GitHub 仓库地址，用于拼接下载链接）：

```rust
pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub release_repo: String,  // "https://github.com/{owner}/ring"
    pub llm: LlmConfig,
    pub gitlab: GitlabConfig,
}
```

`graph_store` 用 `RwLock` 是因为 petgraph 是同步库，写操作需要独占锁。其他 service 用 `Arc` 即可（内部自己管理并发）。

---

## 6. API 路由注册

```rust
pub fn build_router(state: AppState) -> Router {
    let public = Router::new()
        .route("/join", get(member::join_page))
        .with_state(Arc::new(state));

    let api_v1 = Router::new()
        .route("/setup/status", get(setup::get_status))
        .route("/setup/username", post(setup::set_username))
        .route("/setup/llm", post(setup::set_llm))
        .route("/setup/gitlab", post(setup::set_gitlab))
        .route("/setup/complete", post(setup::complete))
        .route("/rings", get(ring::list_rings).post(ring::create_ring))
        .route("/rings/{ringId}", get(ring::get_ring).put(ring::update_ring).delete(ring::delete_ring))
        .route("/rings/{ringId}/blueprint/templates", get(blueprint::list_templates))
        .route("/rings/{ringId}/blueprint/chat", post(blueprint::blueprint_chat))
        .route("/rings/{ringId}/blueprint/preview", post(blueprint::preview_blueprint))
        .route("/rings/{ringId}/blueprint/confirm", post(blueprint::confirm_blueprint))
        .route("/rings/{ringId}/conversations", get(conversation::list).post(conversation::create))
        .route("/rings/{ringId}/conversations/{convId}", get(conversation::get))
        .route("/rings/{ringId}/conversations/{convId}/messages", post(conversation::send_message).get(conversation::get_messages))
        .route("/rings/{ringId}/conversations/{convId}/compact", post(conversation::compact))
        .route("/rings/{ringId}/conversations/{convId}/token-stats", get(conversation::token_stats))
        .route("/rings/{ringId}/conversations/{convId}/export", post(conversation::export_conv))
        .route("/rings/{ringId}/graphs", get(graph::list_graphs))
        .route("/rings/{ringId}/graphs/{graphId}", get(graph::get_graph))
        .route("/rings/{ringId}/graphs/{graphId}/nodes", post(graph::create_node))
        .route("/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}", get(graph::get_node).put(graph::update_node).delete(graph::delete_node))
        .route("/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}/content", get(graph::get_node_content))
        .route("/rings/{ringId}/graphs/{graphId}/edges", post(graph::create_edge))
        .route("/rings/{ringId}/graphs/{graphId}/edges/{edgeId}", delete(graph::delete_edge))
        .route("/rings/{ringId}/search", post(search::search))
        .route("/rings/{ringId}/search/global", post(search::global_search))
        .route("/rings/{ringId}/archive", post(archive::archive))
        .route("/rings/{ringId}/archive/{archiveId}/confirm", post(archive::confirm_archive))
        .route("/rings/{ringId}/archive/queue", get(archive::get_queue))
        .route("/rings/{ringId}/prs", get(git::list_prs))
        .route("/rings/{ringId}/prs/{prId}/diff", get(git::get_pr_diff))
        .route("/rings/{ringId}/prs/{prId}/merge", post(git::merge_pr))
        .route("/rings/{ringId}/prs/{prId}/reject", post(git::reject_pr))
        .route("/rings/{ringId}/commits", get(git::list_commits))
        .route("/rings/{ringId}/invites", post(member::create_invite))
        .route("/rings/join", post(member::join_ring))
        .route("/rings/join/apply", post(member::apply_join))
        .route("/rings/{ringId}/join-requests/{requestId}/approve", post(member::approve_request))
        .route("/rings/{ringId}/join-requests/{requestId}/reject", post(member::reject_request))
        .route("/rings/{ringId}/members", get(member::list_members))
        .route("/rings/{ringId}/members/{memberId}/role", put(member::update_role))
        .route("/rings/{ringId}/members/{memberId}", delete(member::remove_member))
        .route("/rings/{ringId}/sessions", get(session::list_sessions).post(session::create_session))
        .route("/rings/{ringId}/sessions/{sessionId}", get(session::get_session).delete(session::delete_session))
        .route("/rings/{ringId}/sessions/{sessionId}/invite", post(session::invite_to_session))
        .route("/rings/{ringId}/sessions/{sessionId}/messages", get(session::get_messages).post(session::send_message))
        .route("/rings/{ringId}/sessions/{sessionId}/archive-toggle", put(session::toggle_archive))
        .route("/rings/{ringId}/sessions/{sessionId}/archive", post(session::archive_session))
        .route("/rings/{ringId}/sessions/{sessionId}/leave", post(session::leave_session))
        .route("/rings/{ringId}/sessions/{sessionId}/close", post(session::close_session))
        .route("/rings/{ringId}/notifications", get(notification::list_notifications))
        .route("/rings/{ringId}/notifications/{notificationId}/read", put(notification::mark_read))
        .route("/rings/{ringId}/exports/graph-image", post(export::export_graph_image))
        .route("/rings/{ringId}/exports/markdown/{nodeId}", get(export::export_markdown))
        .route("/rings/{ringId}/exports/conversation", post(export::export_conversation))
        .route("/rings/{ringId}/exports/report", post(export::export_report))
        .route("/rings/{ringId}/exports/session", post(export::export_session))
        .route("/rings/{ringId}/exports/backup", post(export::export_backup))
        .route("/rings/{ringId}/exports/graph-json/{graphId}", get(export::export_graph_json))
        .route("/super-ring/chat", post(ai::super_ring_chat))
        .route("/super-ring/analyze", post(ai::super_ring_analyze))
        .route("/super-ring/summarize", post(ai::super_ring_summarize))
        .route("/super-ring/merge-suggest", post(ai::super_ring_merge_suggest))
        .route("/settings", get(settings::get_settings).put(settings::update_settings))
        .route("/ws", get(ws::ws_handler))
        .route("/ws/sessions/{sessionId}", get(ws::session_ws_handler))
        .with_state(Arc::new(state));

    Router::new()
        .merge(public)
        .nest("/api/v1", api_v1)
}
```

---

## 7. 前端路由设计

```
ring-frontend/src/
├── pages/
│   ├── Setup/                    # 首次启动向导
│   │   ├── StepUsername.tsx
│   │   ├── StepLlm.tsx
│   │   ├── StepGitlab.tsx
│   │   └── SetupWizard.tsx
│   │
│   ├── RingHub/                  # Ring Hub 首页
│   │   ├── RingList.tsx
│   │   ├── CreateRing.tsx
│   │   ├── SuperRingChat.tsx
│   │   └── Settings.tsx
│   │
│   ├── RingSpace/                # Ring 空间
│   │   ├── Layout.tsx            # 顶部导航 + 左侧面板
│   │   ├── ChatView.tsx          # 对话视图（默认）
│   │   ├── GraphView.tsx         # 图谱可视化
│   │   ├── ArchiveView.tsx       # 归档视图（PR 队列 + Diff）
│   │   ├── SessionView.tsx       # Session 多人讨论
│   │   ├── BlueprintWizard.tsx   # 蓝图构建向导
│   │   ├── ExportView.tsx        # 导出中心
│   │   └── MemberManager.tsx     # 成员管理
│   │
│   └── Join/                     # 邀请加入页（本地 ring-server 处理）
│       ├── JoinPage.tsx
│       └── InstallGuide.tsx      # 已安装后本地显示的加入引导
│
├── components/
│   ├── graph/
│   │   ├── ForceGraph.tsx        # D3.js 力导向图
│   │   ├── NodeTree.tsx          # 节点树导航
│   │   └── BlueprintPreview.tsx  # 蓝图模板预览
│   ├── chat/
│   │   ├── ChatBubble.tsx
│   │   ├── ChatInput.tsx
│   │   └── ExportButton.tsx
│   ├── git/
│   │   ├── DiffViewer.tsx        # Monaco Diff
│   │   ├── PRList.tsx
│   │   └── PRDetail.tsx
│   └── common/
│       ├── NotificationBell.tsx
│       ├── MemberAvatar.tsx
│       └── TokenProgress.tsx
│
├── services/
│   └── api.ts                    # 统一 HTTP client
├── stores/                       # Zustand 状态管理
├── hooks/
└── types/
```

### 前端路由表

| 路径 | 页面 | 条件 |
|------|------|------|
| `/setup` | SetupWizard | 未 setup |
| `/` | RingHub | 已 setup |
| `/ring/{ringId}` | RingSpace.Layout | 已 setup + Ring 成员 |
| `/ring/{ringId}/blueprint` | BlueprintWizard | Ring 创建者 |
| `/join?token=xxx` | JoinPage（本地）| 持有有效 token |

> `/join` 页面有两种来源：
> 1. **远程**：`http://{creatorIP}:7420/join?token=xxx` → 创建者 ring-server 返回独立 HTML 安装导航页（不经过 React）
> 2. **本地**：`http://localhost:7420/join?token=xxx&creator_ip={IP}` → 本地 ring-server 返回 React 前端的 JoinPage

---

## 8. 构建与分发

### 开发

```bash
# 后端
cd ring-server && cargo run

# 前端
cd ring-frontend && npm run dev
```

### 构建

```bash
# 后端
cd ring-server && cargo build --release

# 前端
cd ring-frontend && npm run build
```

### 分发

单个二进制文件。前端静态资源由 Axum 通过 `tower-http::ServeDir` 服务 `ring-frontend/dist/` 目录。

用户下载 `ring-server` 可执行文件后运行，浏览器打开 `http://localhost:7420` 进入 Setup 向导。

### 去中心化安装导航页

安装导航页嵌入二进制文件（`include_str!` 编译期嵌入），由分享邀请链接的用户的 ring-server 动态提供。

**工作流程**：
1. 创建者分享邀请链接 `http://{creatorIP}:7420/join?token=xxx`
2. 访问者打开链接 → 创建者的 ring-server 服务安装导航页
3. 页面通过 User-Agent 检测访问者操作系统，高亮对应下载按钮
4. 下载链接指向 GitHub Releases（`https://github.com/{owner}/ring/releases/latest/download/{filename}`）
5. 访问者安装后点击"继续加入" → 跳转 `http://localhost:7420/join?token=xxx&creator_ip={creatorIP}`

**安装导航页特点**：
- 独立 HTML 页面（不依赖前端 React 构建）
- 服务端注入 Ring 信息（名称、描述、成员数）
- 客户端 JS 检测 OS 并自动高亮对应平台
- 三步引导：下载 → 安装 → 继续

### 平台支持

| 平台 | 二进制文件名 | 架构 |
|------|-------------|------|
| Windows | `ring-server-windows-x86_64.zip` | x86_64 (MSVC) |
| Linux / WSL | `ring-server-linux-x86_64.tar.gz` | x86_64 (GNU) |
| macOS (Apple Silicon) | `ring-server-macos-arm64.tar.gz` | aarch64 |
| macOS (Intel) | `ring-server-macos-x86_64.tar.gz` | x86_64 |

### CI/CD（GitHub Actions）

```yaml
# .github/workflows/release.yml
# 触发条件：tag v*
# 四个平台并行构建 → 上传到 GitHub Releases
jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            ext: .zip
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            ext: .tar.gz
          - target: aarch64-apple-darwin
            os: macos-latest
            ext: .tar.gz
          - target: x86_64-apple-darwin
            os: macos-13
            ext: .tar.gz
```

不做安装包（.dmg / .AppImage / .deb / .msi），只提供裸 binary。
