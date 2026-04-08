# Graph 存储 API 参考

> 源码路径：`ring-server/src/graph/`

## GraphStore Trait

### `trait GraphStore`
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

## PetgraphStore

### `struct PetgraphStore`
源文件：`graph/petgraph_store.rs:15`

基于 `petgraph::stable_graph::StableDiGraph` 的内存图实现。

| 字段 | 类型 | 说明 |
|------|------|------|
| `inner` | `Arc<RwLock<GraphInner>>` | 内部状态 |

### `struct GraphInner`
源文件：`graph/petgraph_store.rs:19`

| 字段 | 类型 | 说明 |
|------|------|------|
| `graph` | `StableDiGraph<NodeData, EdgeData>` | petgraph 有向图 |
| `node_id_to_index` | `HashMap<String, NodeIndex>` | 节点 ID → 索引映射 |
| `graph_id_to_nodes` | `HashMap<String, Vec<NodeIndex>>` | 图 ID → 节点索引列表 |

### `impl PetgraphStore`
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

## 类型定义

### `NodeData`
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

### `EdgeData`
源文件：`graph/types.rs:16`

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `String` | 边 ID（UUID） |
| `source_id` | `String` | 源节点 ID |
| `target_id` | `String` | 目标节点 ID |
| `relation` | `String` | 关系类型 |
| `label` | `Option<String>` | 标签 |
| `graph_id` | `String` | 所属图 ID |

### `NewNode`
源文件：`graph/types.rs:26`

创建节点时的输入结构。

| 字段 | 类型 | 说明 |
|------|------|------|
| `label` | `String` | 标签 |
| `node_type` | `String` | 节点类型 |
| `parent_id` | `Option<String>` | 父节点 ID |
| `description` | `Option<String>` | 描述 |

### `NewEdge`
源文件：`graph/types.rs:34`

创建边时的输入结构。

| 字段 | 类型 | 说明 |
|------|------|------|
| `source_id` | `String` | 源节点 ID |
| `target_id` | `String` | 目标节点 ID |
| `relation` | `String` | 关系类型 |
| `label` | `Option<String>` | 标签 |

### `GraphJson`
源文件：`graph/types.rs:42`

图序列化格式，用于持久化（graph.json）。

| 字段 | 类型 | 说明 |
|------|------|------|
| `nodes` | `Vec<NodeData>` | 节点列表 |
| `edges` | `Vec<EdgeData>` | 边列表 |
