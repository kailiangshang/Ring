# 知识图谱测试用例

## 功能输入输出

| 功能 | 入口 | 输入 | 输出 |
| --- | --- | --- | --- |
| 默认图谱 | `GET /api/rings/{ring_id}/graph` | ring_id | graph、nodes、edges |
| 创建节点 | `POST /api/rings/{ring_id}/graph` | label、node_type、content、tags | node |
| 更新节点 | `PUT /api/rings/{ring_id}/graph/nodes/{node_id}` | node fields | node |
| 删除节点 | `DELETE /api/rings/{ring_id}/graph/nodes/{node_id}` | node_id | success |
| 创建边 | `POST /api/rings/{ring_id}/graph/edges` | source_id、target_id、relation | edge |
| 多图谱 | `/graphs` | name | graphs |

## 用例

### KG-01 获取空图谱

- 前置条件：新建 Ring。
- 输入：ring_id。
- 步骤：访问 `GET /api/rings/{ring_id}/graph`。
- 期望输出：HTTP 200，返回 nodes/edges 数组。
- 问题记录：

### KG-02 创建图谱节点

- 前置条件：Ring 存在。
- 输入：

```json
{"label":"权限模型","node_type":"topic","tags":["协作","安全"],"content":"# 权限模型\n\n包含 creator/admin/member/guest。"}
```

- 步骤：提交 `POST /api/rings/{ring_id}/graph`。
- 期望输出：HTTP 201，节点字段正确，搜索索引可查到关键词。
- 问题记录：

### KG-03 创建节点关系边

- 前置条件：存在 `权限模型` 和 `邀请机制` 两个节点。
- 输入：

```json
{"source_id":"<node-a>","target_id":"<node-b>","relation":"depends_on","label":"权限影响邀请"}
```

- 步骤：提交 `POST /api/rings/{ring_id}/graph/edges`。
- 期望输出：HTTP 201，图谱中出现边。
- 问题记录：

### KG-04 删除节点级联删除边

- 前置条件：节点 A 与多个节点存在边。
- 输入：节点 A id。
- 步骤：删除节点 A，再获取图谱。
- 期望输出：节点 A 不存在；所有关联边也不存在。
- 问题记录：

### KG-05 多图谱隔离

- 前置条件：Ring 存在。
- 输入：

```json
{"name":"业务流程图谱"}
```

- 步骤：创建新图谱，在新图谱下创建节点，再查看默认图谱。
- 期望输出：不同 graph_id 下节点互不混淆。
- 问题记录：

### KG-06 Session 结论提取到图谱

- 前置条件：Session 已有 summary。
- 输入：无。
- 步骤：调用 `POST /api/rings/{ring_id}/sessions/{session_id}/extract-to-graph`。
- 期望输出：返回 suggestions，建议节点/边和 Session 结论相关。
- 问题记录：
