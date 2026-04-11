# Ring 技术架构与开发者指南

> **Affects**: [data-model.md](data-model.md) · [knowledge-graph.md](knowledge-graph.md) · [api-design.md](api-design.md) · [backend.md](../api/backend.md) · [frontend.md](../api/frontend.md)
> **Depends on**: [PRD.md](../product/PRD.md) · [ai-behavior.md](../product/ai-behavior.md)
> **Last verified**: 2026-04-11

---

## 1. 系统架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                      用户浏览器（React + TS）                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Ring Hub  │  │  对话视图  │  │  图谱视图  │  │  归档视图  │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
│  ┌──────────┐  ┌──────────┐                                │
│  │ D3.js 图谱 │  │ Monaco/CM │                                │
│  └──────────┘  └──────────┘                                │
└────────────────────────┬────────────────────────────────────┘
                         │ HTTP / WebSocket（始终连 localhost）
┌────────────────────────┴────────────────────────────────────┐
│                     后端服务（Rust + Axum）                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Ring 管理  │  │ AI 调度器  │  │  Git 服务  │  │ 工具引擎  │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │ 图谱服务  │  │ 权限服务  │  │ 实时通信  │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
│  ┌──────────┐                                               │
│  │ LLM适配层 │  (async-openai / reqwest)                     │
│  └──────────┘                                               │
└────────────────────────┬────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┬────────────────┐
        │                │                │                │
   ┌────┴────┐    ┌─────┴─────┐   ┌─────┴─────┐   ┌─────┴─────┐
   │ SQLite  │    │ 本地 Git   │   │ 云端 API   │   │ petgraph  │
   │ (本地DB) │    │ (git2)    │   │ (LLM API) │   │ (内存图)  │
   └─────────┘    └─────┬─────┘   └───────────┘   └───────────┘
                        │
                  ┌─────┴─────┐
                  │  GitLab    │
                  │ (公司内网)  │
                  └───────────┘
```

---

## 2. 项目结构

```
ring-server/
├── Cargo.toml
├── migrations/
│   └── 001_initial.sql
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── error.rs
│   ├── state.rs
│   ├── routes.rs
│   │
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── setup.rs
│   │   ├── install.rs             # 安装导航页（嵌入 HTML 模板渲染）
│   │   ├── ring.rs
│   │   ├── chat.rs
│   │   ├── graph.rs
│   │   ├── archive.rs
│   │   ├── git.rs
│   │   ├── member.rs
│   │   ├── session.rs
│   │   ├── notification.rs
│   │   ├── search.rs
│   │   ├── ai.rs
│   │   ├── blueprint.rs
│   │   ├── settings.rs
│   │   ├── sse_helpers.rs
│   │   └── ws.rs
│   │
│   ├── services/
│   │   ├── mod.rs
│   │   ├── ai_service.rs
│   │   ├── archive_service.rs
│   │   ├── context_loader.rs
│   │   ├── credential_service.rs
│   │   ├── git_service.rs
│   │   ├── gitlab_service.rs
│   │   ├── graph_service.rs
│   │   ├── llm_provider.rs        # LlmProvider trait
│   │   ├── llm_openai.rs          # async-openai 适配
│   │   ├── llm_anthropic.rs       # Anthropic 适配
│   │   ├── member_service.rs
│   │   ├── notification_service.rs
│   │   ├── permission_service.rs
│   │   ├── ring_service.rs
│   │   ├── search_service.rs
│   │   ├── session_service.rs
│   │   ├── settings_service.rs
│   │   ├── tool_engine.rs         # ToolRegistry + ToolDispatcher
│   │   ├── trigger_service.rs
│   │   ├── workflow_service.rs
│   │   └── ws_hub.rs
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── ring.rs
│   │   ├── user.rs
│   │   ├── member.rs
│   │   ├── invite.rs
│   │   ├── graph_model.rs
│   │   ├── conversation.rs
│   │   ├── git_model.rs
│   │   ├── blueprint.rs
│   │   ├── notification_model.rs
│   │   ├── session_model.rs
│   │   └── tool_model.rs
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── traits.rs              # 单一 Repository trait（所有方法）
│   │   └── sqlite/
│   │       ├── mod.rs
│   │       ├── user_repo.rs
│   │       ├── ring_repo.rs
│   │       ├── conversation_repo.rs
│   │       ├── member_repo.rs
│   │       ├── session_repo.rs
│   │       ├── settings_repo.rs
│   │       ├── blueprint_repo.rs
│   │       ├── archive_repo.rs
│   │       ├── notification_repo.rs
│   │       ├── search_repo.rs
│   │       └── tests.rs
│   │
│   ├── graph/
│   │   ├── mod.rs
│   │   ├── store_trait.rs         # GraphStore trait
│   │   ├── petgraph_store.rs      # petgraph 实现
│   │   └── types.rs               # NodeData, EdgeData, NewNode, etc.
│   │
│   └── middleware/
│       ├── mod.rs
│       └── auth.rs
```

### 用户数据目录

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

### Cargo.toml 依赖清单

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
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }

# Graph
petgraph = { version = "0.8", features = ["serde-1"] }

# Git
git2 = { version = "0.20", features = ["vendored-openssl"] }

# HTTP client (GitLab API + Anthropic LLM)
reqwest = { version = "0.12", features = ["json", "stream"] }

# LLM
async-openai = { version = "0.34", features = ["chat-completion"] }

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
sha2 = "0.10"
hmac = "0.12"
getrandom = "0.2"

# SSE streaming
futures = "0.3"
http-body-util = "0.1"

# Utilities
async-trait = "0.1"
chrono = "0.4"
tokio-stream = "0.1"

# Tools
scraper = "0.21"
regex = "1"

[dev-dependencies]
tokio = { version = "1", features = ["test-util"] }
tempfile = "3"
```

---

## 3. 前端架构

### 3.1 技术栈

- **框架**：React + TypeScript
- **图谱可视化**：D3.js（力导向图）
- **代码/Diff 渲染**：Monaco Editor 或 CodeMirror
- **Markdown 渲染**：react-markdown 或类似库
- **状态管理**：Zustand 或 Jotai（轻量级）

### 3.2 页面结构

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
│   │   ├── SuperRingAI.tsx       # Super Ring 对话
│   │   └── Settings.tsx
│   │
│   ├── RingSpace/                # Ring 空间
│   │   ├── Layout.tsx            # 顶部导航 + 左侧面板
│   │   ├── ChatView.tsx          # 对话视图（默认）
│   │   ├── GraphView.tsx         # 图谱可视化（静态查看 + 导出）
│   │   ├── ArchiveView.tsx       # 归档视图（文件树 + PR 队列 + Diff）
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
│   ├── graph/                    # 图谱相关组件
│   │   ├── ForceGraph.tsx        # 力导向图（D3.js）
│   │   ├── NodeTree.tsx          # 节点树导航
│   │   ├── NodeEditor.tsx        # 节点编辑器
│   │   └── BlueprintPreview.tsx  # 蓝图模板预览
│   ├── chat/                     # 对话相关组件
│   │   ├── ChatBubble.tsx
│   │   ├── ChatInput.tsx
│   │   └── ExportButton.tsx      # Export 按钮
│   ├── git/                      # Git 相关组件
│   │   ├── DiffViewer.tsx        # Diff 并排对比（Monaco Diff）
│   │   ├── PRList.tsx            # PR 列表
│   │   ├── PRDetail.tsx          # PR 详情
│   │   └── CommitHistory.tsx     # 提交历史
│   └── common/                   # 通用组件
│       ├── NotificationBell.tsx
│       ├── MemberAvatar.tsx
│       └── TokenProgress.tsx
│
├── services/                     # API 调用层
│   └── api.ts                    # 统一 HTTP client
├── stores/                       # Zustand 状态管理
├── hooks/                        # 自定义 hooks
└── types/                        # TypeScript 类型定义
```

### 3.3 实时通信

- 对话消息通过 WebSocket 实时推送
- AI 流式响应通过 Server-Sent Events (SSE)
- 图谱变更通知通过 WebSocket 广播给所有在线成员
- PR 状态变更通过 WebSocket 推送
- **Session 多人实时讨论**通过创建者后端 WebSocket hub 中转（见下方说明）

#### Session 实时通信架构

```
成员 A 浏览器                    成员 B 浏览器
     │                               │
     │  ws://{sessionOwnerIP}:7420/.. │  ws://{sessionOwnerIP}:7420/..
     │                               │
     └──────────┬────────────────────┘
                │
     ┌──────────┴──────────┐
     │  Session Owner 后端   │
     │   Session WebSocket  │
     │       Hub            │
     │  ┌────────────────┐  │
     │  │ session 频道    │  │
     │  │ - 成员管理      │  │
     │  │ - 消息广播      │  │
     │  │ - 离线缓存      │  │
     │  │ - Session Ring  │  │
     │  └────────────────┘  │
     └──────────┬──────────┘
                │
          SQLite（session_messages）
```

**消息流转**：
1. 成员发消息 → Session owner 后端接收 → 存入 session_messages（带 seq_num）→ 广播给所有 session 成员
2. Session Ring 响应 → Session owner 后端的 Session Ring 实例生成 → 广播给所有成员
3. 成员离线重连 → 发送 `after_seq` → Session owner 后端返回缺失消息（离线补发）
4. 归档操作 → 仅 session owner 触发 → 走标准归档流程（graph.json + Git）
5. Session owner 离线 → Session 暂停（参与者无法发消息）→ Session owner 重连后自动恢复

---

## 4. 后端架构

### 4.1 模块组织

后端采用严格分层架构：**handlers 不写业务逻辑**，handler 只做参数解析 → 调 service → 返回响应，所有业务逻辑在 services 层。

#### 数据库抽象层

关系型数据库访问通过 **Repository trait** 抽象，业务层不直接依赖具体数据库实现：

```rust
#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    async fn create_user(&self, new_user: NewUser) -> Result<User>;
    async fn get_user(&self, id: &str) -> Result<Option<User>>;
    async fn list_all_users(&self) -> Result<Vec<User>>;
    async fn is_setup_completed(&self) -> Result<bool>;
    async fn complete_setup(&self, user_id: &str) -> Result<()>;
    async fn create_ring(&self, new_ring: NewRing) -> Result<Ring>;
    async fn get_ring(&self, id: &str) -> Result<Option<Ring>>;
    async fn list_rings_by_user(&self, user_id: &str) -> Result<Vec<Ring>>;
    async fn update_ring(&self, id: &str, name: Option<String>, description: Option<String>) -> Result<Ring>;
    async fn delete_ring(&self, id: &str) -> Result<()>;
    async fn create_invite_token(/* ... */) -> Result<InviteToken>;
    async fn get_invite_token(&self, token: &str) -> Result<Option<InviteToken>>;
    async fn get_setting(&self, key: &str) -> Result<Option<String>>;
    async fn set_setting(&self, key: &str, value: &str) -> Result<()>;
    async fn create_conversation(/* ... */) -> Result<Conversation>;
    async fn list_conversations(&self, ring_id: &str) -> Result<Vec<Conversation>>;
    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>>;
    async fn create_message(/* ... */) -> Result<Message>;
    async fn get_messages(&self, conversation_id: &str, limit: i64, before_id: Option<&str>) -> Result<Vec<Message>>;
    async fn create_member(&self, new_member: NewMember) -> Result<Member>;
    async fn get_member(&self, id: &str) -> Result<Option<Member>>;
    async fn list_members_by_ring(&self, ring_id: &str) -> Result<Vec<Member>>;
    async fn create_session(/* ... */) -> Result<Session>;
    async fn create_session_member(/* ... */) -> Result<SessionMember>;
    async fn create_session_message(/* ... */) -> Result<SessionMessage>;
    async fn create_archive_record(/* ... */) -> Result<()>;
    async fn create_notification(&self, n: NewNotification) -> Result<Notification>;
    async fn search_nodes_fts(&self, query: &str, graph_ids: Option<Vec<String>>, limit: i64) -> Result<Vec<SearchResult>>;
}
```

所有方法定义在单个 `Repository` trait 上，不拆分子 trait。当前实现：`SqliteRepository`（基于 `sqlx`）。未来可新增 `PostgresRepository`、`MysqlRepository` 等，只需实现 trait，业务层无需改动。

#### 图数据抽象层

图数据通过 **GraphStore trait** 抽象访问，当前基于 petgraph 内存图实现：

```rust
#[async_trait::async_trait]
pub trait GraphStore: Send + Sync {
    async fn create_node(&self, graph_id: &str, input: NewNode) -> Result<NodeData>;
    async fn get_node(&self, graph_id: &str, node_id: &str) -> Result<Option<NodeData>>;
    async fn update_node(&self, graph_id: &str, node_id: &str, label: Option<String>, description: Option<String>, node_type: Option<String>) -> Result<NodeData>;
    async fn delete_node(&self, graph_id: &str, node_id: &str) -> Result<()>;
    async fn create_edge(&self, graph_id: &str, input: NewEdge) -> Result<EdgeData>;
    async fn delete_edge(&self, graph_id: &str, edge_id: &str) -> Result<()>;
    async fn get_children(&self, graph_id: &str, parent_id: &str) -> Result<Vec<NodeData>>;
    async fn list_graph_ids(&self) -> Vec<String>;
    async fn import_graph_json(&self, graph_id: &str, data: &GraphJson) -> Result<()>;
    async fn export_graph_json(&self, graph_id: &str) -> Result<GraphJson>;
}
```

当前实现：`PetgraphStore`（基于 `petgraph::stable_graph::StableDiGraph` + 二级索引 `node_id_to_index` / `graph_id_to_nodes`）。图数据存内存，graph.json 持久化到 Git 仓库。Ring 启动时从 graph.json 全量导入，写入时同步导出。

### 4.2 核心服务

#### 技术栈

- **语言**：Rust
- **Web 框架**：Axum
- **关系型数据库**：SQLite（通过 Repository 抽象层）— 存储 Ring 元数据、成员、消息等。抽象层设计允许未来替换为 PostgreSQL / MySQL 等
- **图数据引擎**：petgraph `StableDiGraph`（进程内嵌，纯 Rust）— 内存图 + graph.json 持久化
- **Git 操作**：`git2` crate
- **HTTP 客户端**：`reqwest`（调用 GitLab API 和 LLM API）
- **LLM 客户端**：`async-openai`（OpenAI + Ollama）+ 自建 Anthropic 适配层
- **序列化**：`serde` + `serde_json`
- **实时通信**：Axum WebSocket 支持
- **中文分词**：`jieba-rs`（FTS5 关键词搜索）

#### AI 调度器（三层架构 + 自成长设计）

AI 调度器管理三个层级的 AI 实例，严格分层：Super Ring → Group Ring → Session Ring。

**自成长设计理念**：Ring 不为特定角色硬编码功能。AI prompt 是角色适配层，蓝图是知识结构适配层，行为反馈是持续优化层。同一个 Ring 框架，通过不同 prompt 和蓝图配置，适配行政、技术、管理等不同场景。

```
AI 调度器
├── Super Ring（Hub 级，全局唯一）
│   ├── 定位: 全局助手 + 跨 Ring 协调者
│   ├── system prompt: 全局助手，负责 Ring Hub 层面的引导和管理
│   ├── 数据访问: 按需只读访问本机所有 Ring 内容（图谱、归档 Markdown、元数据）
│   ├── 核心能力:
│   │   ├── Ring 管理: 创建/配置 Ring，使用引导
│   │   ├── 跨 Ring 分析: 分析多个 Ring 的数据，发现关联
│   │   ├── 跨 Ring 问答: 跨 Ring 搜索和回答问题
│   │   ├── 跨 Ring 总结: 汇总多个 Ring 的核心内容
│   │   └── 跨 Ring 合并: 发现可合并的知识点，推荐合并方案
│   └── LLM 后端: 可配置
│
├── Group Ring（Ring 级，每个 Ring 一个）
│   ├── 定位: 群组专属 AI
│   ├── 行为驱动: .ring/ 持久化文档体系（非单一 ai_prompt）
│   │   ├── 写入权限: 只有创建者和管理员可写入（直接 commit），成员完全只读
│   │   ├── 核心层（始终加载）:
│   │   │   ├── role.md — 角色定义（用户可编辑）
│   │   │   ├── conventions.md — 团队约定（用户可编辑）
│   │   │   └── active-context.md — 当前活跃上下文（AI 动态维护）
│   │   ├── 扩展层（按需加载）:
│   │   │   ├── archive-patterns.md — 归档偏好（归档模式时加载）
│   │   │   ├── corrections.md — 修正记录（用户修正后加载）
│   │   │   └── knowledge-summary.md — 知识总结（需要全局理解时加载）
│   │   └── 所有文档在 Git 中版本管理，随 Ring 同步进化
│   ├── 数据访问: 读写本 Ring 图谱和归档
│   ├── 自成长: 用户行为反馈写入 .ring/ 文档，AI 持续优化行为
│   └── LLM 后端: 继承全局配置
│
└── Session Ring（Session 级，每个活跃 Session 一个）
    ├── 定位: 多人讨论场景的专属 AI
    ├── system prompt: 继承 Group Ring prompt + session 场景上下文
    ├── 数据访问: 继承 Group Ring 的数据访问（只读图谱 + 归档）
    ├── 触发条件: 仅在多人 Session 激活时存在
    ├── 核心特点:
    │   ├── 必须以预设场景启动（深度调研/会议归档/学习中心）
    │   ├── 所有 session 成员共享，回复广播
    │   ├── 按场景提供定向能力（如深度调研 = 跨源聚合 + 报告生成）
    │   └── 不开放自定义逻辑，保留扩展能力
    ├── LLM 后端: 继承全局配置
    └── 生命周期: Session 关闭后销毁
```

**层级调用规则**：
- Super Ring 可向下查询 Group Ring 的数据（只读），不直接调用 Group Ring 实例
- Group Ring 独立运行，不知道 Super Ring 和 Session Ring 的存在
- Session Ring 继承 Group Ring 的 prompt 和数据，但独立运行
- 三层之间无直接函数调用，通过共享数据层（SQLite + 内存图 + Git）间接交互

#### Git 服务

Git 服务封装所有 Git 操作，对上层透明：

```
Git 服务
├── 本地操作（git2 crate）
│   ├── clone / pull / push
│   ├── commit / checkout
│   └── diff / log
│
├── GitLab API（reqwest）
│   ├── 创建 MR
│   ├── 合并 / 关闭 MR
│   ├── 获取 MR 列表和 Diff
│   └── 创建仓库
│
├── 仓库管理
│   ├── 初始化仓库结构
│   ├── 管理本地路径
│   └── 凭证管理（SSH key / PAT）
│
└── 图谱同步（graph.json ↔ 内存图）
    ├── 写入时：内存图操作 → 导出 graph.json → git commit
    ├── 读取时：直接查内存图
    └── 同步时：git pull → 全量导入 graph.json → 重建索引
```

#### 图谱服务

图谱服务通过 petgraph 内存图管理知识图谱数据：

```
图谱服务
├── petgraph StableDiGraph（有向图，稳定索引）
├── 二级索引（node_id / label / graph_id → NodeIndex 映射）
├── 并发控制（Arc<RwLock<GraphDatabase>>，多读单写）
└── graph.json 导入/导出
    ├── 从 graph.json 全量导入（启动时 / git pull 后）
    ├── 从内存图导出 graph.json（写入操作后）
    └── 几百节点规模下导入/导出 < 1ms
```

### 4.3 AppState

```rust
use std::sync::Arc;

use crate::config::Config;
use crate::db::traits::Repository;
use crate::graph::store_trait::GraphStore;
use crate::services::ai_service::AiService;
use crate::services::llm_provider::LlmProvider;
use crate::services::notification_service::NotificationService;
use crate::services::search_service::SearchService;
use crate::services::settings_service::SettingsService;
use crate::services::tool_engine::ToolRegistry;
use crate::services::ws_hub::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<dyn Repository>,
    pub graph_store: Arc<dyn GraphStore>,
    pub ai_service: Arc<AiService>,
    pub llm_provider: Arc<dyn LlmProvider>,
    pub tool_registry: Arc<ToolRegistry>,
    pub search_service: Arc<SearchService>,
    pub settings_service: Arc<SettingsService>,
    pub notification_service: Arc<NotificationService>,
    pub ws_hub: Arc<WsHub>,
}
```

`Config`：

```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub release_repo: String,
    pub database_url: String,
}
```

`graph_store` 不用 `RwLock` 包裹——`GraphStore` trait 内部管理并发。`ToolRegistry` 管理 LLM tool 调用注册与分发。`WsHub` 管理 WebSocket 连接广播。`AiService` 统一调度三层 AI 实例（Super Ring / Group Ring / Session Ring）。`SearchService` 提供 FTS5 全文搜索能力。

### 4.4 错误处理

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

    #[error("database error")]
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

### 4.5 LLM 适配层

LLM 调用通过统一的 `LlmProvider` trait 抽象，屏蔽不同 LLM 供应商的 API 差异：

```
                    ┌───────────────────┐
                    │   业务层（AI 调度器） │
                    └─────────┬─────────┘
                              │
                    ┌─────────┴─────────┐
                    │  LlmProvider trait │
                    └─────────┬─────────┘
                              │
          ┌───────────────────┼───────────────────┐
          │                   │                   │
  ┌───────┴────────┐ ┌───────┴────────┐ ┌───────┴────────┐
  │ OpenAI Adapter │ │ Anthropic      │ │ Ollama Adapter │
  │ async-openai   │ │ Adapter        │ │ async-openai   │
  │                │ │ reqwest        │ │ (改 base URL)  │
  └────────────────┘ └────────────────┘ └────────────────┘
```

**三家差异**：

| 差异点 | OpenAI | Anthropic | Ollama |
|--------|--------|-----------|--------|
| 工具定义参数名 | `parameters` | `input_schema` | `parameters`（兼容 OpenAI） |
| 工具结果消息 | `role: "tool"` | `role: "user"` + content block | `role: "tool"` |
| System message | 消息列表中 `role: "system"` | 请求顶层 `system` 参数 | 同 OpenAI |
| 流式响应 | SSE `text/event-stream` | SSE `event: xxx` | 同 OpenAI |

**策略**：
- OpenAI 和 Ollama 共用 `async-openai` crate（Ollama 暴露 OpenAI 兼容端点 `/v1/chat/completions`）
- Anthropic 使用 `reqwest` + 自定义序列化（现有 `anthropic-sdk` crate 不成熟）
- 统一 `ToolDefinition`（JSON Schema）在适配层按供应商转换字段名

### 4.6 工具引擎

工具引擎管理原子工具的调用和组合：

```
工具引擎
├── 工具注册表（注册所有原子工具）
├── 工具调度器（根据 AI 指令调用工具）
├── 工具编排器（组合多个工具成工作流）
└── 权限检查（根据当前模式决定是否允许执行）
```

---

## 5. 数据存储

详细数据模型见 [data-model.md](data-model.md)。

### SQLite 迁移脚本

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

## 6. API 路由表

```rust
pub fn build_router(state: AppState) -> Router {
    let setup_routes = Router::new()
        .route("/status", get(setup::get_status))
        .route("/username", post(setup::set_username))
        .route("/llm", post(setup::set_llm))
        .route("/gitlab", post(setup::set_gitlab))
        .route("/complete", post(setup::complete));

    let ring_routes = Router::new()
        .route("/join", post(member::join_ring))
        .route("/", get(ring::list_rings).post(ring::create_ring))
        .route(
            "/{ringId}",
            get(ring::get_ring)
                .put(ring::update_ring)
                .delete(ring::delete_ring),
        );

    let member_routes = Router::new()
        .route("/", get(member::list_members))
        .route("/invites", post(member::generate_invite))
        .route("/{memberId}/role", put(member::update_role))
        .route("/{memberId}", delete(member::remove_member));

    let session_routes = Router::new()
        .route("/", post(session::create_session).get(session::list_sessions))
        .route("/{sessionId}", get(session::get_session).delete(session::delete_session))
        .route("/{sessionId}/close", post(session::close_session))
        .route("/{sessionId}/leave", post(session::leave_session))
        .route("/{sessionId}/archive-toggle", put(session::toggle_archive))
        .route("/{sessionId}/invite", post(session::invite_member))
        .route("/{sessionId}/messages", get(session::get_messages));

    let conversation_routes = Router::new()
        .route("/", get(conversation::list).post(conversation::create))
        .route("/{convId}", get(conversation::get))
        .route("/{convId}/messages", get(conversation::get_messages).post(conversation::send_message));

    let blueprint_routes = Router::new()
        .route("/templates", get(blueprint::list_templates))
        .route("/chat", post(blueprint::blueprint_chat))
        .route("/preview", post(blueprint::preview_blueprint))
        .route("/confirm", post(blueprint::confirm_blueprint));

    let graph_routes = Router::new()
        .route("/", get(graph::list_graphs))
        .route("/{graphId}", get(graph::get_graph))
        .route("/{graphId}/nodes", post(graph::create_node))
        .route("/{graphId}/nodes/{nodeId}", get(graph::get_node).put(graph::update_node).delete(graph::delete_node))
        .route("/{graphId}/nodes/{nodeId}/content", get(graph::get_node_content))
        .route("/{graphId}/edges", post(graph::create_edge))
        .route("/{graphId}/edges/{edgeId}", delete(graph::delete_edge));

    let search_routes = Router::new().route("/", post(search::search_nodes));

    let archive_routes = Router::new()
        .route("/", post(archive::archive))
        .route("/queue", get(archive::get_queue))
        .route("/{archiveId}/confirm", post(archive::confirm_archive));

    let git_routes = Router::new()
        .route("/prs", get(git::list_prs))
        .route("/prs/{prId}/merge", post(git::merge_pr))
        .route("/prs/{prId}/reject", post(git::reject_pr))
        .route("/commits", get(git::get_commit_log));

    let notification_routes = Router::new()
        .route("/", get(notification::list_notifications))
        .route("/{notificationId}", post(notification::mark_read));

    let settings_routes = Router::new()
        .route("/", get(settings::get_settings).put(settings::update_settings));

    let protected = Router::new()
        .nest("/api/v1/rings", ring_routes)
        .nest("/api/v1/rings/{ringId}/members", member_routes)
        .nest("/api/v1/rings/{ringId}/sessions", session_routes)
        .nest("/api/v1/rings/{ringId}/conversations", conversation_routes)
        .nest("/api/v1/rings/{ringId}/blueprint", blueprint_routes)
        .nest("/api/v1/rings/{ringId}/graphs", graph_routes)
        .nest("/api/v1/rings/{ringId}/search", search_routes)
        .nest("/api/v1/rings/{ringId}/archive", archive_routes)
        .nest("/api/v1/rings/{ringId}/git", git_routes)
        .nest("/api/v1/notifications", notification_routes)
        .nest("/api/v1/settings", settings_routes)
        .route("/api/v1/super-ring/chat", post(ai::super_ring_chat))
        .route("/api/v1/ws/{ringId}", get(ws::ws_handler))
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .nest("/api/v1/setup", setup_routes)
        .route("/join", get(install::join_page))
        .merge(protected)
        .with_state(state)
        .layer(CorsLayer::permissive())
}
```

### 路由分层说明

- `/api/v1/setup/*` — 公开路由，无需认证
- `/join` — 公开路由，安装导航页
- 其他所有 `/api/v1/*` 路由受 `auth_middleware` 保护

---

## 7. 前端路由

### 前端路由表

| 路径 | 页面 | 条件 |
|------|------|------|
| `/setup` | SetupWizard | 未 setup |
| `/` | RingHub | 已 setup |
| `/ring/:ringId` | ChatView | 已 setup + Ring 成员 |
| `/ring/:ringId/blueprint` | BlueprintWizard | Ring 创建者 |
| `/ring/:ringId/graph` | GraphView | 已 setup + Ring 成员 |
| `/ring/:ringId/prs` | PrList | 已 setup + Ring 成员 |
| `/ring/:ringId/prs/:prId` | PrDetail | 已 setup + Ring 成员 |
| `/ring/:ringId/members` | MemberList | 已 setup + Ring 成员 |
| `/ring/:ringId/sessions` | SessionView | 已 setup + Ring 成员 |
| `/super-ring` | SuperRingChat | 已 setup |
| `/settings` | SettingsPage | 已 setup |

> `/join` 页面由后端 `handlers::install::join_page` 处理：
> 1. **远程**：`http://{creatorIP}:7420/join?token=xxx` → 创建者 ring-server 返回独立 HTML 安装导航页（`include_str!` 嵌入，不经过 React）
> 2. **本地**：`http://localhost:7420/join?token=xxx&creator_ip={IP}` → 本地 ring-server 同样返回安装导航页，已安装用户可继续加入流程

---

## 8. 构建、测试与部署

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

### 测试

```bash
cargo test                    # Rust 单元 + 集成测试
cd ring-frontend && npm test  # 前端测试
```

### 分发

单个二进制文件。前端静态资源由 Axum 通过 `tower-http::ServeDir` 服务 `ring-frontend/dist/` 目录。

用户下载 `ring-server` 可执行文件后运行，浏览器打开 `http://localhost:7420` 进入 Setup 向导。

### 网络拓扑

```
用户 A 的机器（创建者）
├── Ring 后端服务（Axum，端口 7420）
├── AppState { config, db, graph_store, ai_service, llm_provider, tool_registry, search_service, settings_service, notification_service, ws_hub }
├── SQLite 数据库（本地，Ring 元数据）
├── petgraph 内存图（Ring 启动时从 graph.json 加载）
├── Git 本地仓库（每个 Ring 一个独立仓库）
└── 对外暴露：http://192.168.x.x:7420

用户 B 的机器（成员）
├── Ring 后端服务（Axum，端口 7420）
├── AppState（同创建者结构）
├── SQLite 数据库（本地，Ring 元数据）
├── petgraph 内存图（从 GitLab clone 后加载 graph.json）
├── Git 本地仓库（clone 自 GitLab，每个 Ring 一个）
└── 对外暴露：http://192.168.y.y:7420

公司 GitLab
└── ring-{name} 仓库（归档内容）

云端 LLM API
├── OpenAI
├── Anthropic
└── Ollama（可选，本地部署）
```

### 前端连接拓扑

**前端始终连 `localhost:7420`（本地后端）**，不直接连创建者后端。

```
成员浏览器 → localhost:7420（成员本地后端）
  ├── 读操作（查图谱、看归档、对话）→ 本地 SQLite + petgraph 内存图
  ├── 写操作（归档）→ 本地 git pull → 本地 Git 操作 → GitLab
  ├── WebSocket 实时通知 → 直连创建者后端 ws://{creatorIP}:7420/ws
  └── Session WebSocket → 直连 Session owner 后端 ws://{ownerIP}:7420/ws/sessions/{id}
```

好处：无 CORS 问题，读操作零延迟，写操作走 GitLab PR。

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
