# Ring 数据模型

## 0. 存储策略概览

Ring 采用**双存储架构**，均通过 trait 抽象层访问，可替换底层实现：

| 存储层 | 抽象 trait | 当前实现 | 未来可替换为 |
|--------|-----------|---------|------------|
| **关系型数据** | `Repository` + 子 trait | SQLite（`sqlx`） | PostgreSQL、MySQL、TiDB 等 |
| **图数据** | `GraphStore` | petgraph 内存图 + graph.json | Neo4j、Apache AGE 等 |
| **图谱持久化** | Git（graph.json） | GitLab 仓库 | Gitea、Gogs 等 |

**数据流向**：
```
graph.json（Git 同步格式）
  ↕ 导入/导出
petgraph 内存图（本地查询引擎）
  ↕ API 读写
前端（D3.js 可视化）
```

- `graph.json` 是持久化格式，通过 Git 在多台机器间同步
- petgraph 内存图是运行时查询引擎，Ring 启动时从 graph.json 全量导入
- 写入时：内存图操作 → 导出 graph.json → Git commit/push
- 读取时：直接查内存图（微秒级）
- **不再有三方同步问题**（无外部图数据库依赖）

---

## 1. 关系型数据表结构（SQLite 实现）

> **设计说明**：以下表结构基于 SQLite，但业务层通过 `Repository` trait 访问。未来可替换为 PostgreSQL / MySQL 等数据库，只需实现对应的 Repository trait，无需修改业务代码。

### 1.1 rings 表

```sql
CREATE TABLE rings (
    id          TEXT PRIMARY KEY,          -- UUID
    name        TEXT NOT NULL,             -- Ring 名称（即 Group Ring 名称）
    description TEXT,                      -- Ring 描述
    creator_id  TEXT NOT NULL REFERENCES users(id), -- 创建者用户 ID
    gitlab_repo TEXT NOT NULL,             -- GitLab 仓库地址（每个 Group Ring 一个独立仓库）
    local_path  TEXT NOT NULL,             -- 本地 Git 仓库路径
    next_token_id INTEGER NOT NULL DEFAULT 2, -- 下一个分配的 token_id（创建者为 #1）
    status      TEXT NOT NULL DEFAULT 'active', -- active（Ring 始终为 active，无归档冻结）
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

> **AI 行为驱动**：Group Ring 的行为不再由 `ai_prompt` 字段驱动，而是由仓库中 `.ring/` 目录下的持久化 MD 文档体系驱动。`role.md` 是角色定义（替代原 `ai_prompt`），随 Ring 一起 Git 版本管理。

### 1.2 graphs 表（元数据）

> **注意**：图谱的节点和边数据存储在 petgraph 内存图中，此表仅记录图谱的元信息。

```sql
CREATE TABLE graphs (
    id          TEXT PRIMARY KEY,          -- UUID
    ring_id     TEXT NOT NULL REFERENCES rings(id),
    name        TEXT NOT NULL,             -- 图谱名称（如"知识图谱"）
    description TEXT,                      -- 图谱描述
    graph_type  TEXT NOT NULL,             -- 图谱类型（knowledge / event / person / task / ...）
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### ~~1.3 graph_nodes 表（已移至 petgraph 内存图）~~

> 节点数据现在存储在 petgraph 内存图中，不再使用 SQLite 表。见第 2 节。

### ~~1.4 graph_edges 表（已移至 petgraph 内存图）~~

> 边数据现在存储在 petgraph 内存图中，不再使用 SQLite 表。见第 2 节。

### 1.5 members 表

```sql
CREATE TABLE members (
    id          TEXT PRIMARY KEY,          -- UUID
    ring_id     TEXT NOT NULL REFERENCES rings(id),
    user_id     TEXT NOT NULL,             -- 用户 ID（UUID，不可变）
    token_id    INTEGER NOT NULL,          -- Ring 内自增长唯一标识（#1, #2, #3...）
    display_name TEXT NOT NULL,            -- 显示名称（可重复，可修改）
    role        TEXT NOT NULL DEFAULT 'member', -- creator / admin / member / readonly
    joined_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(ring_id, user_id),
    UNIQUE(ring_id, token_id)
);
```

> **token_id**：Ring 内自增长的唯一标识，创建者为 #1，后续成员依次递增。用于在 Ring 内区分同名成员。开放链接加入时自动分配，审核链接加入时审批通过后分配。

### 1.6 invite_tokens 表

```sql
CREATE TABLE invite_tokens (
    id          TEXT PRIMARY KEY,          -- UUID
    ring_id     TEXT NOT NULL REFERENCES rings(id),
    token       TEXT NOT NULL UNIQUE,      -- 一次性邀请 token（格式：base64url(random 32 bytes)）
    token_type  TEXT NOT NULL DEFAULT 'open', -- open（开放链接）/ audit（审核链接）
    role        TEXT NOT NULL DEFAULT 'member', -- 被邀请者的初始角色
    inviter_id  TEXT NOT NULL,             -- 邀请人 ID（仅创建者可生成）
    max_uses    INTEGER NOT NULL DEFAULT 1, -- 最大使用次数（开放链接可设为 0 表示不限）
    use_count   INTEGER NOT NULL DEFAULT 0, -- 已使用次数
    max_members INTEGER,                   -- Ring 最大人数上限（可选，防止链接泄露）
    expires_at  DATETIME NOT NULL,         -- 过期时间（默认创建后 24 小时）
    used_at     DATETIME,                  -- 最后一次使用时间
    revoked_at  DATETIME,                  -- 撤销时间（创建者可撤销未用完的 token）
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

> **两种邀请链接**：
> - **开放链接（open）**：点链接直接加入，token_id 自增长分配，加入时同步其他成员名称。
> - **审核链接（audit）**：点链接提交申请（输入 display_name）→ 创建者审批 → 通过后分配 token_id 并加入。
>
> **Token 一次性规则**：开放链接可设 `max_uses = 0`（不限次数）或指定次数；审核链接每个 token 只能发起一次申请。

### 1.7 conversations 表

> **两种会话模式**：
> - **storage 模式（持久会话）**：消息存入 SQLite，支持历史回溯、归档、compact。Group Ring 上下文从 messages 表重建。
> - **ephemeral 模式（临时会话）**：消息仅存在内存中，关闭对话后丢失。不消耗存储，适合快速问答。
>
> **上下文管理（仅 storage 模式）**：系统追踪每个会话的 LLM token 消耗。超过阈值时提醒用户 compact（或自动 compact）。Compact = Group Ring 将历史对话压缩为摘要，替换原始消息作为上下文输入，减少 token 消耗。

```sql
CREATE TABLE conversations (
    id          TEXT PRIMARY KEY,          -- UUID
    ring_id     TEXT NOT NULL REFERENCES rings(id),
    title       TEXT,                      -- 对话标题
    mode        TEXT NOT NULL DEFAULT 'chat', -- chat / archive / auto
    context_mode TEXT NOT NULL DEFAULT 'storage', -- storage（持久）/ ephemeral（临时）
    token_count INTEGER NOT NULL DEFAULT 0, -- 当前会话累计消耗的 LLM token 数
    token_limit INTEGER NOT NULL DEFAULT 100000, -- compact 阈值（可配置，默认 100k tokens）
    auto_compact BOOLEAN NOT NULL DEFAULT FALSE, -- 是否自动 compact（达到阈值后自动压缩）
    summary     TEXT,                      -- compact 后的对话摘要（替换原始消息作为上下文）
    compacted_at DATETIME,                 -- 最近一次 compact 时间
    created_by  TEXT NOT NULL REFERENCES users(id),
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 1.8 messages 表

```sql
CREATE TABLE messages (
    id              TEXT PRIMARY KEY,      -- UUID
    conversation_id TEXT NOT NULL REFERENCES conversations(id),
    role            TEXT NOT NULL,         -- user / assistant / system / tool
    content         TEXT NOT NULL,         -- 消息内容
    sender_id       TEXT,                  -- 发送者用户 ID（user 消息）
    tool_calls      TEXT,                  -- JSON：工具调用记录
    archived        BOOLEAN NOT NULL DEFAULT FALSE, -- 是否已被归档
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 1.9 archive_records 表

```sql
CREATE TABLE archive_records (
    id              TEXT PRIMARY KEY,      -- UUID
    ring_id         TEXT NOT NULL REFERENCES rings(id),
    node_id         TEXT,                  -- petgraph 内存图中的节点 ID
    conversation_id TEXT REFERENCES conversations(id),
    message_ids     TEXT,                  -- JSON 数组：被归档的消息 ID 列表
    markdown_path   TEXT NOT NULL,         -- 生成的 Markdown 文件路径
    archived_by     TEXT NOT NULL,         -- 归档操作者用户 ID
    git_commit_sha  TEXT,                  -- 对应的 Git commit SHA
    pr_status       TEXT,                  -- none / pending / merged / rejected
    pr_url          TEXT,                  -- GitLab MR URL
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 1.10 blueprint_templates 表

```sql
CREATE TABLE blueprint_templates (
    id          TEXT PRIMARY KEY,          -- UUID
    name        TEXT NOT NULL,             -- 模板名称
    description TEXT,                      -- 模板描述
    graphs      TEXT NOT NULL,             -- JSON：图谱配置 [{name, type, categories: [...]}]
    is_system   BOOLEAN NOT NULL DEFAULT FALSE, -- 是否系统预设
    created_by  TEXT,                      -- 创建者用户 ID（自定义模板）
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 1.11 sessions 表

> **Session** 是 Ring 内的多人实时讨论会话，区别于 Ring 级邀请（仓库同步）。Session 级邀请仅限 Ring 内已有成员加入，消息通过 Session owner 的后端 WebSocket 中转。

```sql
CREATE TABLE sessions (
    id              TEXT PRIMARY KEY,          -- UUID
    ring_id         TEXT NOT NULL REFERENCES rings(id),
    title           TEXT,                      -- Session 标题
    scenario        TEXT NOT NULL,             -- 预设场景（必选）：deep_research / meeting_archive / learning_center
    created_by      TEXT NOT NULL,             -- Session 创建者用户 ID
    archive_enabled BOOLEAN NOT NULL DEFAULT FALSE, -- 是否开启归档能力（创建者可动态切换）
    status          TEXT NOT NULL DEFAULT 'active', -- active / closed / deleted
    -- active: 进行中
    -- closed: 创建者关闭但保留记录
    -- deleted: 创建者已删除
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

> **预设场景（scenario）**：创建 Session 时必须选择一个预设场景，决定 Session Ring 的行为和能力。当前支持：
> - `discussion`（自由讨论）：多人自由聊天，Session Ring 作为基础讨论助手
> - `deep_research`（深度调研）：跨源聚合 + 报告生成
> - `meeting_archive`（会议归档）：结构化提取 + 归档推荐
> - `learning_center`（学习中心）：知识解读 + 概念提取
>
> 不开放自定义场景，保留未来扩展能力。
>
> **并发限制**：同一个 Ring 同一时刻只能有一个活跃 Session。创建新 Session 前需关闭或删除现有 Session。
>
> **Session 暂停机制**：Session owner 离线时 Session 自动暂停，所有参与者无法发消息。Owner 重连后自动恢复。不设临时接管机制，避免消息合并复杂度。消息始终存储在 Session owner 后端的 SQLite 中。

### 1.12 session_members 表

```sql
CREATE TABLE session_members (
    id          TEXT PRIMARY KEY,          -- UUID
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    user_id     TEXT NOT NULL,             -- 用户 ID
    role        TEXT NOT NULL DEFAULT 'participant', -- owner / participant
    -- owner: session 创建者，拥有归档开关、邀请、删除等权限
    -- participant: 普通参与者
    status      TEXT NOT NULL DEFAULT 'active', -- active / left / kicked
    joined_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    left_at     DATETIME,                  -- 离开时间
    UNIQUE(session_id, user_id)
);
```

### 1.13 session_messages 表

> Session 消息独立于 conversations/messages 表，存储在 Session owner 后端的 SQLite 中。支持离线消息缓存（成员重连后补发）。Session Ring 为共享 AI 实例，回复广播给所有 session 成员。

```sql
CREATE TABLE session_messages (
    id              TEXT PRIMARY KEY,          -- UUID
    session_id      TEXT NOT NULL REFERENCES sessions(id),
    sender_id       TEXT NOT NULL,             -- 发送者用户 ID
    role            TEXT NOT NULL DEFAULT 'user', -- user / assistant / system
    content         TEXT NOT NULL,             -- 消息内容
    seq_num         INTEGER NOT NULL,          -- 消息序列号（用于离线补发，按 session 递增）
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

> **离线补发机制**：成员重连时，发送本地已收到的最大 `seq_num`，创建者后端返回 `seq_num > 该值` 的所有未读消息。

### 1.14 users 表

> **首次启动流程**：用户安装后首次启动后端，浏览器打开 `http://localhost:7420`，进入初始化页面。用户配置显示名称（`display_name`，可重复、可修改）。系统自动生成不可变的 `user_id`（UUID）作为唯一标识。配置完成后进入 Ring Hub。

```sql
CREATE TABLE users (
    id           TEXT PRIMARY KEY,          -- UUID（首次启动自动生成，不可变，全局唯一标识）
    display_name TEXT NOT NULL,             -- 显示名称（可重复，可修改）
    avatar_url   TEXT,                      -- 头像 URL
    ip_address   TEXT,                      -- 内网 IP（自动检测）
    setup_completed BOOLEAN NOT NULL DEFAULT FALSE, -- 是否完成首次配置
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

> **身份标识策略**：`user_id`（UUID）是系统唯一标识，不可变。`display_name` 可重复（公司可能有两个"张伟"），可修改。在 Ring 内通过 `token_id`（Ring 内自增长 ID）区分成员。

### 1.12 settings 表

```sql
CREATE TABLE settings (
    key         TEXT PRIMARY KEY,          -- 设置键
    value       TEXT NOT NULL,             -- 设置值（JSON）
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### 1.15 notifications 表

```sql
CREATE TABLE notifications (
    id          TEXT PRIMARY KEY,          -- UUID
    ring_id     TEXT NOT NULL REFERENCES rings(id),
    user_id     TEXT NOT NULL REFERENCES users(id),
    type        TEXT NOT NULL,             -- pr_created / pr_merged / pr_rejected / member_joined / member_removed / role_changed / session_invite
    title       TEXT NOT NULL,             -- 通知标题
    body        TEXT,                      -- 通知正文
    related_id  TEXT,                      -- 关联资源 ID（PR ID / member ID / session ID 等）
    is_read     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

> **离线通知**：用户离线期间产生的通知缓存到此表，下次启动前端时展示未读通知列表。

---

## 2. 图数据（petgraph 内存图）

> **设计说明**：图数据通过 `GraphStore` trait 抽象访问。当前实现基于 petgraph（纯 Rust 图库）内存图，以 graph.json 持久化。Ring 启动时从 graph.json 全量导入到内存，写入时导出 graph.json + Git commit。未来可替换为 Neo4j 等外部图数据库，只需实现 `GraphStore` trait。

### 2.1 核心数据结构

```rust
use petgraph::stable_graph::StableDiGraph;

// 内存图引擎
struct GraphDatabase {
    graph: StableDiGraph<NodeData, EdgeData>,
    node_id_to_index: HashMap<String, NodeIndex>,   // UUID → petgraph 索引
    edge_id_to_index: HashMap<String, EdgeIndex>,   // UUID → petgraph 索引
    label_index: HashMap<String, Vec<NodeIndex>>,    // 标签 → 节点列表（搜索用）
    graph_id_to_roots: HashMap<String, Vec<NodeIndex>>, // graph_id → 根节点列表
}
```

> **为什么选 petgraph**：FalkorDB 是 Redis Module，不能进程内嵌，需要系统预装 Redis。petgraph 是纯 Rust，零外部依赖，真正进程内嵌。Ring 图规模（上限几百节点）不需要 Cypher 查询，petgraph API 直接操作即可，微秒级完成。

### 2.2 节点数据（NodeData）

节点数据作为 petgraph 的节点权重（Node Weight）存储：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeData {
    id: String,                    // UUID
    graph_id: String,              // 所属图谱 UUID
    parent_id: Option<String>,     // 父节点 UUID
    label: String,                 // 显示名称（如"竞品分析"）
    node_type: String,             // 节点类型（concept/document/event/person/task）
    description: Option<String>,
    markdown_path: Option<String>, // 对应的 Markdown 文件路径
    metadata: Option<serde_json::Value>, // 额外元数据（tags, references 等）
    created_at: String,
    created_by: String,
}
```

### 2.3 边数据（EdgeData）

边数据作为 petgraph 的边权重（Edge Weight）存储：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EdgeData {
    id: String,           // UUID
    graph_id: String,     // 所属图谱 UUID
    relation: String,     // 关系类型（contains/depends_on/related_to/...）
    label: Option<String>, // 关系描述
}
```

### 2.4 查询场景

| 查询场景 | 实现方式 |
|---------|---------|
| 查找节点的所有子节点 | `neighbors_directed(node_idx, Outgoing)` + 过滤 parent_id |
| 查找两个节点之间的关系 | `edges_connecting(a_idx, b_idx)` |
| 按标签搜索节点 | `label_index` 二级索引直接查 HashMap |
| 查找某个概念的所有关联节点 | `neighbors(node_idx)` 双向遍历 |
| 统计图谱规模 | `graph.node_count()` / `graph.edge_count()` |
| 查找根节点 | `graph_id_to_roots` 索引直接查 |

---

## 3. graph.json 与内存图同步

graph.json 是图谱数据在 Git 仓库中的持久化格式。petgraph 内存图是运行时查询引擎。

### 3.1 同步策略

- **graph.json 是持久化格式**：通过 Git 在多台机器间同步
- **内存图是查询引擎**：Ring 运行时所有图操作直接查内存图
- **写入流程**：内存图操作 → 导出 graph.json → Git commit/push
- **读取流程**：直接查内存图（微秒级）
- **同步流程**：git pull 后检测 graph.json 变更 → 全量重新导入内存图 + 重建索引
- **启动流程**：从 graph.json 全量导入到内存图 + 重建索引

### 3.2 graph.json 结构

每个图谱的 `graph.json` 文件存储在 Git 仓库中：

```json
{
  "version": 1,
  "id": "graph-uuid",
  "name": "知识图谱",
  "type": "knowledge",
  "nodes": [
    {
      "id": "node-uuid-1",
      "label": "竞品分析",
      "type": "concept",
      "parent_id": null,
      "markdown_path": "nodes/competitor-analysis.md",
      "metadata": {
        "created_at": "2026-04-05T10:00:00Z",
        "created_by": "user-uuid",
        "tags": ["产品", "竞品"],
        "references": [
          {"type": "code", "url": "https://gitlab.company.com/team/api/blob/main/src/service.rs#L42", "label": "src/service.rs:42"},
          {"type": "ticket", "url": "https://jira.company.com/browse/PROJ-123", "label": "PROJ-123"},
          {"type": "doc", "url": "https://wiki.company.com/pages/decision-001", "label": "架构决策记录"}
        ]
      }
    },
    {
      "id": "node-uuid-2",
      "label": "竞品 A",
      "type": "concept",
      "parent_id": "node-uuid-1",
      "markdown_path": "nodes/competitor-a.md",
      "metadata": {
        "created_at": "2026-04-05T10:30:00Z",
        "created_by": "user-uuid"
      }
    }
  ],
  "edges": [
    {
      "id": "edge-uuid-1",
      "source": "node-uuid-1",
      "target": "node-uuid-2",
      "relation": "contains",
      "label": "包含"
    }
  ]
}
```

---

## 3. Git 仓库目录结构

> **每个 Ring（Group Ring）一个独立仓库**。用户身份信息存储在本地 `.ring-local/identity.json`（不进 Git），跨设备时重新 Setup 即可。加入多个 Ring = clone 多个仓库。

```
ring-{name}/
├── .ring-local/              # 本地配置（.gitignore 排除，不进 Git）
│   └── identity.json         # 当前用户的本地身份（仅本机）
├── blueprint.json            # 蓝图配置
├── .ring/                    # Group Ring 的"大脑"（可长期进化的 AI 上下文文档）
│   ├── role.md               # 角色定义（替代 ai_prompt，创建者/管理员可编辑）
│   ├── conventions.md        # 团队约定和术语（创建者/管理员可编辑）
│   ├── archive-patterns.md   # 归档模式（AI 自动积累）
│   ├── corrections.md        # 修正记录（AI 自动积累）
│   ├── knowledge-summary.md  # 定期知识总结（AI 自动生成）
│   └── active-context.md     # 当前活跃上下文（AI 动态维护）
├── graphs/
│   ├── knowledge/
│   │   └── graph.json
│   ├── event/
│   │   └── graph.json
│   └── competitor/
│       └── graph.json
├── nodes/
│   ├── competitor-analysis.md
│   ├── competitor-a.md
│   └── ...
└── assets/
    └── images/
```

### 3.1 .ring/ 文档体系（Group Ring 自成长核心）

Group Ring 不是靠一个 `ai_prompt` 字段驱动的，而是通过 `.ring/` 目录下的 6 个持久化文档实现可进化行为。这些文档随 Git 版本管理，跟随 Ring 一起成长。

**写入权限**：只有创建者和管理员可以写入 `.ring/` 文档（直接 commit），成员完全只读。

| 文档 | 用途 | 维护者 | 加载策略 |
|------|------|--------|---------|
| `role.md` | 角色定义：Group Ring 的身份、行为准则、擅长的领域 | 创建者/管理员（可编辑） | **始终加载** |
| `conventions.md` | 团队约定：术语表、命名规范、分类偏好、组织习惯 | 创建者/管理员（可编辑） | **始终加载** |
| `archive-patterns.md` | 归档模式：AI 学到的归档偏好（粒度、风格、更新 vs 新建） | 创建者/管理员的 Group Ring 自动积累 | 按需加载 |
| `corrections.md` | 修正记录：用户对 AI 的修正，AI 从中学习避免重复犯错 | 创建者/管理员的 Group Ring 自动积累 | 按需加载 |
| `knowledge-summary.md` | 知识总结：定期生成的 Ring 知识全貌，避免每次全量扫描图谱 | 创建者/管理员的 Group Ring 自动生成 | 按需加载 |
| `active-context.md` | 活跃上下文：当前最相关的上下文片段，每次对话动态维护 | 创建者/管理员的 Group Ring 动态维护 | **始终加载** |

> **成员的 Group Ring 对 `.ring/` 完全只读**，不写入不 PR。成员想修改约定或角色 → 向创建者/管理员提出，由他们编辑。创建者和管理员的并发写入通过后端序列化队列保证。

**按需加载机制**：
- `role.md` + `conventions.md` + `active-context.md` 始终加载到 Group Ring 的上下文（核心层）
- `archive-patterns.md` 在归档模式下加载
- `corrections.md` 在用户修正行为后加载
- `knowledge-summary.md` 在需要全局理解 Ring 知识时加载
- 所有 .ring/ 文档在 Git 中版本管理，随 Ring 同步

**identity.json（本地身份，不进 Git）**：
```json
{
  "version": 1,
  "user_id": "user-uuid",
  "display_name": "王小明",
  "avatar_url": null,
  "device_id": "device-uuid",
  "updated_at": "2026-04-06T10:00:00Z"
}
```

> **本地身份，不跨设备同步**：`identity.json` 存储在 `.ring-local/` 目录（.gitignore 排除），不进 Git 仓库。跨设备时用户在新设备重新 Setup（输入 display_name），系统自动生成新 user_id。内网环境下换设备频率低，重新配置可接受。
>
> **为什么不用共享仓库存身份**：Git 仓库是多成员共享的，如果 hub.json 进仓库，成员 B clone 后会读到成员 A 的身份，产生覆盖冲突。

**成员加入 Ring**：被邀请人 clone 该 Ring 的 GitLab 仓库到本地。

---

## 4. blueprint.json 结构

```json
{
  "version": 1,
  "ring_id": "ring-uuid",
  "ring_name": "产品竞品分析组",
  "template_id": "template-uuid",
  "template_name": "产品研究",
  "graphs": [
    {
      "id": "graph-uuid-1",
      "name": "知识图谱",
      "type": "knowledge",
      "categories": ["概念", "方法", "工具"]
    },
    {
      "id": "graph-uuid-2",
      "name": "竞品图谱",
      "type": "competitor",
      "categories": ["竞品 A", "竞品 B", "竞品 C"]
    },
    {
      "id": "graph-uuid-3",
      "name": "事件图谱",
      "type": "event",
      "categories": ["会议", "决策", "里程碑"]
    }
  ],
  "confirmed_at": "2026-04-05T12:00:00Z",
  "confirmed_by": "user-uuid"
}
```
