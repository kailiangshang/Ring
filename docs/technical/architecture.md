# Ring 技术架构

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

## 2. 前端架构

### 2.1 技术栈

- **框架**：React + TypeScript
- **图谱可视化**：D3.js（力导向图）
- **代码/Diff 渲染**：Monaco Editor 或 CodeMirror
- **Markdown 渲染**：react-markdown 或类似库
- **状态管理**：Zustand 或 Jotai（轻量级）

### 2.2 页面结构

```
src/
├── pages/
│   ├── RingHub/              # Ring Hub 首页
│   │   ├── RingList.tsx      # Ring 卡片列表
│   │   ├── CreateRing.tsx    # 创建 Ring 表单/向导
│   │   └── SuperRingAI.tsx      # Super Ring 对话
│   │
│   ├── RingSpace/            # Ring 空间
│   │   ├── ChatView.tsx      # 对话视图
│   │   ├── GraphView.tsx     # 图谱可视化视图（静态查看 + 导出）
│   │   ├── ArchiveView.tsx   # 归档视图（文件树 + PR 队列 + Diff）
│   │   ├── SessionView.tsx   # Session 多人讨论视图
│   │   ├── ExportView.tsx    # 导出中心（7 种导出选项）
│   │   ├── BlueprintWizard.tsx # 蓝图构建向导
│   │   └── Toolbar.tsx       # 底部工具栏
│   │
│   └── Settings/             # 全局设置
│       ├── LLMConfig.tsx
│       ├── PrivacyRules.tsx
│       └── SuperRingConfig.tsx
│
├── components/
│   ├── graph/                # 图谱相关组件
│   │   ├── ForceGraph.tsx    # 力导向图（D3.js）
│   │   ├── NodeTree.tsx      # 节点树导航
│   │   └── NodeEditor.tsx    # 节点编辑器
│   ├── chat/                 # 对话相关组件
│   │   ├── ChatBubble.tsx
│   │   ├── ChatInput.tsx
│   │   └── ExportButton.tsx  # Export 按钮
│   ├── git/                  # Git 相关组件
│   │   ├── DiffViewer.tsx    # Diff 并排对比
│   │   ├── PRList.tsx        # PR 列表
│   │   ├── PRDetail.tsx      # PR 详情
│   │   └── CommitHistory.tsx # 提交历史
│   └── common/               # 通用组件
│
├── services/                 # API 调用层
├── stores/                   # 状态管理
├── hooks/                    # 自定义 hooks
└── types/                    # TypeScript 类型定义
```

### 2.3 实时通信

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

## 3. 后端架构

### 3.1 技术栈

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

### 3.2 数据库抽象层设计

关系型数据库访问通过 **Repository trait** 抽象，业务层不直接依赖具体数据库实现：

```rust
trait Repository: Send + Sync {
    fn rings(&self) -> &dyn RingRepository;
    fn members(&self) -> &dyn MemberRepository;
    fn conversations(&self) -> &dyn ConversationRepository;
    fn messages(&self) -> &dyn MessageRepository;
    fn archive_records(&self) -> &dyn ArchiveRecordRepository;
    fn settings(&self) -> &dyn SettingsRepository;
    fn invite_tokens(&self) -> &dyn InviteTokenRepository;
    fn blueprints(&self) -> &dyn BlueprintRepository;
    fn sessions(&self) -> &dyn SessionRepository;
    fn session_members(&self) -> &dyn SessionMemberRepository;
    fn session_messages(&self) -> &dyn SessionMessageRepository;
}

// 每个 Repository 有独立的 trait
trait RingRepository {
    async fn create(&self, ring: NewRing) -> Result<Ring>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Ring>>;
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Ring>>;
    async fn update(&self, id: &str, ring: UpdateRing) -> Result<Ring>;
    async fn delete(&self, id: &str) -> Result<()>;
}
```

当前实现：`SqliteRepository`（基于 `sqlx`）。未来可新增 `PostgresRepository`、`MysqlRepository` 等，只需实现 trait，业务层无需改动。

图数据通过 **GraphStore trait** 抽象访问，当前基于 petgraph 内存图实现：

```rust
trait GraphStore: Send + Sync {
    async fn create_node(&self, graph_id: &str, node: NewNode) -> Result<Node>;
    async fn get_node(&self, graph_id: &str, node_id: &str) -> Result<Option<Node>>;
    async fn update_node(&self, graph_id: &str, node_id: &str, update: UpdateNode) -> Result<Node>;
    async fn delete_node(&self, graph_id: &str, node_id: &str) -> Result<()>;
    async fn create_edge(&self, graph_id: &str, edge: NewEdge) -> Result<Edge>;
    async fn delete_edge(&self, graph_id: &str, edge_id: &str) -> Result<()>;
    async fn get_children(&self, graph_id: &str, parent_id: &str) -> Result<Vec<Node>>;
    async fn get_neighbors(&self, graph_id: &str, node_id: &str) -> Result<Vec<(Node, Edge)>>;
    async fn import_graph_json(&self, graph_id: &str, data: &GraphJson) -> Result<()>;
    async fn export_graph_json(&self, graph_id: &str) -> Result<GraphJson>;
}
```

当前实现：`PetgraphStore`（基于 `petgraph::stable_graph::StableDiGraph` + 二级索引）。图数据存内存，graph.json 持久化到 Git 仓库。Ring 启动时从 graph.json 全量导入，写入时同步导出。

### 3.3 模块划分

```
src/
├── main.rs                   # 入口，启动 Axum 服务
├── config.rs                 # 配置管理
│
├── handlers/                 # HTTP 请求处理器
│   ├── install.rs            # 安装导航页（公共 HTML，嵌入二进制）
│   ├── ring.rs               # Ring CRUD
│   ├── chat.rs               # 对话相关
│   ├── graph.rs              # 图谱操作
│   ├── archive.rs            # 归档操作
│   ├── git.rs                # Git 操作（PR/Diff/合并）
│   ├── member.rs             # 成员管理
│   ├── session.rs            # Session 多人讨论管理
│   ├── notification.rs       # 通知相关（WebSocket 推送 + 离线缓存）
│   ├── export.rs             # 导出相关（图谱、Markdown、对话、报告等）
│   ├── ai.rs                 # AI 对话
│   └── settings.rs           # 全局设置
│
├── services/                 # 业务逻辑层
│   ├── ring_service.rs       # Ring 管理
│   ├── ai_service.rs         # AI 调度器（Super Ring + Group Ring + Session Ring）
│   ├── context_loader.rs     # .ring/ 文档按需加载（核心层始终加载，扩展层按场景加载）
│   ├── llm_provider.rs       # LLM 适配层（async-openai + Anthropic 适配）
│   ├── llm_openai.rs         # OpenAI / Ollama 适配（async-openai）
│   └── llm_anthropic.rs      # Anthropic 适配（reqwest + 自定义序列化）
│   ├── graph_service.rs      # 图谱管理（petgraph 内存图 + graph.json 导入导出）
│   ├── git_service.rs        # Git 操作封装（git2 + 凭证管理）
│   ├── gitlab_service.rs     # GitLab API 集成
│   ├── archive_service.rs    # 归档流程
│   ├── permission_service.rs # 权限校验
│   ├── blueprint_service.rs  # 蓝图构建
│   ├── sync_service.rs       # graph.json ↔ petgraph 内存图同步
│   ├── session_service.rs    # Session 多人讨论管理（创建、邀请、归档开关）
│   ├── notification_service.rs # 通知服务（WebSocket 推送 + 离线缓存）
│   ├── export_service.rs     # 导出服务（7 种格式：图谱、Markdown、对话、报告、Session、备份、JSON）
│   └── tool_engine.rs        # 原子工具引擎
│
├── tools/                    # 原子工具实现
│   ├── file_parser.rs        # 文件解析（PDF/Markdown/Docx）
│   ├── text_cleaner.rs       # 文本清洗
│   ├── extractor.rs          # 结构化提取（LLM 调用）
│   ├── search.rs             # 全文搜索
│   ├── web_scraper.rs        # 网页爬取
│   ├── graph_generator.rs    # 知识图谱生成
│   ├── markdown_gen.rs       # Markdown 生成
│   ├── privacy_filter.rs     # 隐私过滤
│   └── git_ops.rs            # Git 操作
│
├── models/                   # 数据模型
│   ├── ring.rs
│   ├── graph.rs
│   ├── member.rs
│   ├── message.rs
│   ├── session.rs            # Session / SessionMember / SessionMessage
│   └── archive.rs
│
├── db/                       # 数据库抽象层 + 实现
│   ├── mod.rs                # Repository trait 定义
│   ├── traits/               # 各子 Repository trait
│   │   ├── ring_repo.rs
│   │   ├── member_repo.rs
│   │   ├── message_repo.rs
│   │   ├── archive_repo.rs
│   │   ├── settings_repo.rs
│   │   ├── invite_repo.rs
│   │   └── blueprint_repo.rs
│   ├── sqlite/               # SQLite 实现（当前默认）
│   │   ├── mod.rs            # SqliteRepository 实现
│   │   ├── ring_repo.rs
│   │   ├── member_repo.rs
│   │   └── ...
│   ├── migrations/           # SQLite 迁移脚本
│   └── graph/                # 图数据存储
│       ├── mod.rs            # GraphStore trait 定义
│       ├── petgraph_store.rs # petgraph 内存图实现（当前默认）
│       └── sync.rs           # graph.json ↔ 内存图同步
│
└── ws/                       # WebSocket 处理
    ├── mod.rs
    ├── hub.rs                # WebSocket 连接管理（Ring 级通知）
    └── session_hub.rs        # Session WebSocket hub（多人讨论消息中转、离线缓存）
```

---

## 4. 核心服务交互

### 4.1 AI 调度器（三层架构 + 自成长设计）

AI 调度器管理三个层级的 AI 实例，严格分层：Super Ring → Group Ring → Session Ring。

**自成长设计理念**：Ring 不为特定角色硬编码功能。AI prompt 是角色适配层，蓝图是知识结构适配层，行为反馈是持续优化层。同一个 Ring 框架，通过不同 prompt 和蓝图配置，适配行政、技术、管理等不同场景。

```
AI 调度器

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

### 4.2 LLM 适配层

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

### 4.2 工具引擎（原 4.2）

工具引擎管理原子工具的调用和组合：

```
工具引擎
├── 工具注册表（注册所有原子工具）
├── 工具调度器（根据 AI 指令调用工具）
├── 工具编排器（组合多个工具成工作流）
└── 权限检查（根据当前模式决定是否允许执行）
```

### 4.3 Git 服务

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

### 4.4 图谱服务

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

---

## 5. 部署架构

### 5.1 网络拓扑

```
用户 A 的机器（创建者）
├── Ring 后端服务（Axum，端口 7420）
├── SQLite 数据库（本地，Ring 元数据）
├── petgraph 内存图（Ring 启动时从 graph.json 加载）
├── Git 本地仓库（每个 Ring 一个独立仓库）
└── 对外暴露：http://192.168.x.x:7420

用户 B 的机器（成员）
├── Ring 后端服务（Axum，端口 7420）
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

### 5.2 前端连接拓扑

**前端始终连 `localhost:7420`（本地后端）**，不直接连创建者后端。

```
成员浏览器 → localhost:7420（成员本地后端）
  ├── 读操作（查图谱、看归档、对话）→ 本地 SQLite + petgraph 内存图
  ├── 写操作（归档）→ 本地 git pull → 本地 Git 操作 → GitLab
  ├── WebSocket 实时通知 → 直连创建者后端 ws://{creatorIP}:7420/ws
  └── Session WebSocket → 直连 Session owner 后端 ws://{ownerIP}:7420/ws/sessions/{id}
```

好处：无 CORS 问题，读操作零延迟，写操作走 GitLab PR。
