# Ring 知识图谱设计

> **Affects**: [graphStore (frontend)](../api/frontend.md) · [backend.md](../api/backend.md) · [blueprint (frontend)](../api/frontend.md)
> **Depends on**: [data-model.md](data-model.md) · [PRD.md](../product/PRD.md) · [architecture.md](architecture.md)
> **Last verified**: 2026-04-11

## 1. 图谱模型

### 1.1 核心概念

Ring 的知识图谱由三个层级组成：
```
Ring
├── 图谱 1（如"知识图谱"）
│   ├── 根节点
│   │   ├── 子节点 A
│   │   │   └── 孙节点 A1
│   │   └── 子节点 B
│   └── 边（节点间关系）
│
├── 图谱 2（如"竞品图谱"）
│   └── ...
│
└── 图谱 3（如"事件图谱"）
    └── ...
```

### 1.2 存储引擎：petgraph（进程内嵌）

图谱数据使用 **petgraph**（纯 Rust 图库）在内存中维护，以 **graph.json** 作为持久化格式通过 Git 同步。

**为什么不选 FalkorDB**：
- FalkorDB 是 Redis Module，不能进程内嵌，需要系统预装 Redis + falkordb.so
- `falkordb` crate 是客户端库，不是嵌入式数据库
- Ring 的图规模（上限几百节点）不需要 Cypher 查询语言
- 引入 Redis 依赖增加部署复杂度，违背"本地优先"理念

**petgraph 优势**：
- 纯 Rust，零外部依赖，真正进程内嵌
- 3.3 亿下载，Rust 生态标准图库
- StableGraph 索引在删除后保持稳定（图数据库的关键需求）
- 几百节点规模下操作微秒级完成
- `serde` 支持直接序列化/反序列化

**技术选型**：

| 组件 | 选型 | 版本 | 说明 |
|------|------|------|------|
| 内存图引擎 | petgraph `StableDiGraph` | 0.8.x | 稳定索引的有向图 |
| 持久化格式 | graph.json | — | 单一数据源，Git 同步 |
| 二级索引 | `HashMap<String, Vec<NodeIndex>>` | 标准库 | 按标签/类型快速查找 |
| 并发访问 | `Arc<RwLock<GraphDatabase>>` | 标准库 | 多读单写 |

### 1.3 节点（Node）

| 属性 | 类型 | 描述 |
|------|------|------|
| id | UUID | 唯一标识 |
| graph_id | UUID | 所属图谱 |
| parent_id | UUID? | 父节点（支持层级，null 为根节点） |
| label | String | 显示名称 |
| type | Enum | 节点类型（concept/document/event/person/task） |
| description | String? | 节点描述 |
| markdown_path | String? | 对应的 Markdown 文件路径 |
| metadata | JSON? | 额外元数据（标签、创建时间、references 等） |

### 1.4 边（Edge）

| 属性 | 类型 | 描述 |
|------|------|------|
| id | UUID | 唯一标识 |
| graph_id | UUID | 所属图谱 |
| source_id | UUID | 源节点 |
| target_id | UUID | 目标节点 |
| relation | String | 关系类型 |
| label | String? | 关系描述 |

### 1.5 关系类型预设

| relation | 描述 |
|----------|------|
| contains | 包含（父→子） |
| depends_on | 依赖 |
| related_to | 相关 |
| leads_to | 导致 |
| references | 引用 |
| derived_from | 派生自 |

---

## 2. 内存图引擎设计

### 2.1 核心数据结构

```rust
use petgraph::stable_graph::StableDiGraph;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeData {
    id: String,
    graph_id: String,
    parent_id: Option<String>,
    label: String,
    node_type: String,
    description: Option<String>,
    markdown_path: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EdgeData {
    id: String,
    graph_id: String,
    relation: String,
    label: Option<String>,
}

struct GraphDatabase {
    graph: StableDiGraph<NodeData, EdgeData>,
    node_id_to_index: HashMap<String, petgraph::graph::NodeIndex>,
    edge_id_to_index: HashMap<String, petgraph::graph::EdgeIndex>,
    label_index: HashMap<String, Vec<petgraph::graph::NodeIndex>>,
    graph_id_to_roots: HashMap<String, Vec<petgraph::graph::NodeIndex>>,
}

type SharedGraphDb = Arc<RwLock<GraphDatabase>>;
```

### 2.2 GraphStore trait

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
    async fn get_root_nodes(&self, graph_id: &str) -> Result<Vec<Node>>;
    async fn search_nodes(&self, graph_id: &str, query: &str) -> Result<Vec<Node>>;
    async fn import_graph_json(&self, graph_id: &str, data: &GraphJson) -> Result<()>;
    async fn export_graph_json(&self, graph_id: &str) -> Result<GraphJson>;
}
```

> **注意**：虽然 petgraph 是同步库，GraphStore trait 方法标记为 `async` 以保持接口一致性。内部实现通过 `tokio::task::spawn_blocking` 包装同步操作。

### 2.3 二级索引

petgraph 不提供内置索引，需要自建：

| 索引 | 用途 | 更新时机 |
|------|------|---------|
| `node_id_to_index` | node_id → petgraph NodeIndex | 节点增删 |
| `edge_id_to_index` | edge_id → petgraph EdgeIndex | 边增删 |
| `label_index` | label → 节点列表（搜索用） | 节点增删改 |
| `graph_id_to_roots` | graph_id → 根节点列表（树导航） | 节点增删 |

### 2.4 并发模型

```rust
// 读操作：RwLock 读锁，可并发
let db = graph_db.read().unwrap();
let node = db.get_node(graph_id, node_id)?;

// 写操作：RwLock 写锁，独占
let mut db = graph_db.write().unwrap();
db.create_node(graph_id, new_node)?;
```

Ring 的图操作频率低（归档时才写），并发压力极小。`Arc<RwLock<>>` 足够。

---

## 3. 数据同步策略

### 3.1 单一数据源原则

```
graph.json（Git 同步格式）  ←→  内存图（petgraph）
    ↑ 持久化                        ↑ 查询
    │                               │
    │  graph.json 导入到内存图       │  直接查内存图
    │  （启动/pull 后触发）           │  （日常查询）
    │
    ↓ 导出
    内存图导出为 graph.json → Git commit/push
```

- **graph.json 是持久化格式**，通过 Git 在多台机器间同步
- **内存图是查询引擎**，Ring 运行时所有图操作直接查内存
- **不再有三方同步问题**（之前的 FalkorDB 引入了第三方）

### 3.2 同步触发时机

| 事件 | 操作 |
|------|------|
| Ring 启动 | 全量导入：读取所有 graph.json → 初始化内存图 + 二级索引 |
| git pull（拉取远端更新） | 检测 graph.json 变更 → 增量更新内存图 |
| 创建/更新节点（写入操作） | 内存图操作 → 导出 graph.json → git add + commit |
| 删除节点 | 内存图删除 → 导出 graph.json → git add + commit |

### 3.3 启动时全量导入

```
1. 读取 Git 仓库中所有 graphs/*/graph.json
2. 清空内存图
3. 解析 JSON，逐个添加节点和边到 StableDiGraph
4. 重建二级索引（node_id_to_index, label_index, graph_id_to_roots）
```

### 3.4 增量同步（git pull 后）

```
1. git pull 后，检测 graph.json 文件是否有变更（对比 SHA）
2. 有变更 → 全量重新导入该图谱（简单可靠，几百节点耗时 < 1ms）
3. 重建该图谱的二级索引
```

> **不做增量 diff 导入**。几百节点的 graph.json 解析 + 导入在 1ms 内完成，全量替换比增量 diff 更简单可靠。

---

## 4. 多图谱规则

### 4.1 独立性

- 每个 Ring 可维护最多 N 个独立图谱（默认上限 3，可配置）
- 每个图谱有独立的 graph.json 和节点树
- 图谱之间没有显式跨图谱边

### 4.2 隐式关联

多个图谱的节点可以引用同一个 Markdown 文件，形成跨图谱的隐式关联：

```
图谱 A 的节点 X → nodes/shared-topic.md
图谱 B 的节点 Y → nodes/shared-topic.md

结果：X 和 Y 通过共享同一个 .md 文件建立了关联
```

### 4.3 资源控制

- 图谱数量上限默认 3，过高时系统提醒资源消耗
- 每个图谱的节点数量不做硬性限制，但前端可视化有性能优化（超过 100 节点时启用分层渲染）

---

## 5. 搜索方案

### 5.1 MVP 阶段：关键词搜索

| 组件 | 选型 | 说明 |
|------|------|------|
| 全文搜索引擎 | SQLite FTS5 | 内置，无需额外依赖 |
| 中文分词 | jieba-rs 0.8.x | Rust 标准中文分词库 |
| 分词策略 | 预分词后空格拼接插入 FTS5 | unicode61 tokenizer 按空格切分 |

```
归档内容 → jieba-rs 分词 → 空格拼接 → 插入 FTS5 虚拟表
搜索 query → jieba-rs 分词 → FTS5 MATCH 查询 → BM25 排序
```

### 5.2 后期增强：语义搜索

| 组件 | 选型 | 说明 |
|------|------|------|
| Embedding 模型 | fastembed 5.x（BGEM3） | 支持 44 种模型，BGEM3 原生支持中文 |
| 向量索引 | hnsw_rs 0.3.x | C++ hnswlib 的 Rust binding，生产级 |
| 首次使用 | 自动下载模型（~100-200MB） | 之后本地推理，无需调 API |

```
归档内容 → fastembed 生成 embedding → 存入 hnsw_rs 索引
搜索 query → fastembed 生成 embedding → hnsw_rs 近邻搜索 → 合并 FTS5 结果
```

---

## 6. 蓝图构建

### 6.1 蓝图模板

系统预设蓝图模板，用户可基于模板定制或完全自定义：

| 模板 | 图谱组成 | 适用场景 |
|------|---------|---------|
| 产品研究 | 知识图谱、竞品图谱、事件图谱 | 产品分析、竞品研究 |
| 项目管理 | 任务图谱、决策图谱、人物图谱 | 项目管理、团队协作 |
| 学习笔记 | 知识图谱、概念图谱 | 个人/团队学习 |
| 技术文档 | 架构图谱、API 图谱、变更图谱 | 技术团队文档管理 |
| 空白 | 无预设 | 完全自定义 |

### 6.2 构建流程

**快速路径**：
1. 用户浏览预设模板卡片 → 预览图谱结构（D3.js 可视化）→ "使用此模板" → 确认

**深度路径**：
1. 用户点"自定义"或"从零开始"
2. Group Ring 蓝图 prompt + role.md 叠加引导
3. 多轮对话确认图谱结构
4. 展示可视化预览
5. 用户点击"确认蓝图"
6. 系统创建 graph.json + 初始化 Git 仓库结构 + 推送初始 commit

---

## 7. 图谱生长

### 7.1 生长触发

| 事件 | 生长方式 |
|------|---------|
| 用户上传文件 | AI 解析内容 → 建议创建节点 |
| 对话中讨论新主题 | AI 识别新概念 → 建议创建节点 |
| 使用工具（会议归档/学习中心/深度调研） | 工具产出 → 建议归档到节点 |
| 手动创建 | 用户直接在图谱视图中添加节点 |

### 7.2 节点推荐逻辑

AI 分析内容后，按以下逻辑推荐节点操作：

```
1. 分析内容的主题和关键概念
2. 匹配蓝图中最近的图谱大类
3. 判断是否属于已有节点：
   a. 如果匹配已有节点的主题 → 建议挂载到该节点（更新 Markdown）
   b. 如果是新的子主题 → 建议在大类下创建新节点
   c. 如果是全新的顶层概念 → 建议新增顶层大类（需要用户确认）
4. 生成推荐描述和可视化预览
5. 等待用户确认（手动模式）或自动执行（Auto 模式）
```

---

## 8. 图谱可视化

### 8.1 视图类型

| 视图 | 描述 | 使用场景 |
|------|------|---------|
| 节点树 | 左侧导航，层级展示 | 快速导航和定位 |
| 力导向图 | D3.js 力导向布局 | 全局视图，发现关系 |
| 蓝图预览 | 简化的结构图 | 蓝图确认阶段 |

### 8.2 交互

- 点击节点 → 展开子节点 + 显示对应的 Markdown 内容
- 拖拽节点 → 调整布局
- 缩放/平移 → 大图谱浏览
- 搜索节点 → 按名称/标签搜索

### 8.3 性能优化

- 节点数 < 50：完整渲染
- 节点数 50-200：启用节点聚合（同类节点合并显示）
- 节点数 > 200：启用分层渲染（只渲染当前层级 + 父层级）
- 懒加载子节点（展开时才请求）
