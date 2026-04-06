# Phase 3 Implementation Plan — Knowledge Graph (TDD)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现知识图谱的节点/边 CRUD、层级管理、Markdown 文件关联、graph.json 同步、D3.js 力导向图可视化、节点树导航和搜索。

**Architecture:** 图数据层已有 PetgraphStore（Phase 1 已实现基础 CRUD）。Phase 3 补全 GraphService 业务逻辑层 + HTTP handlers + 搜索 API + 前端 D3.js 可视化 + 节点树导航。

**Tech Stack:** petgraph (已集成), SQLite FTS5 + jieba-rs (搜索), D3.js (前端可视化)

**Reference docs:**
- `docs/technical/knowledge-graph.md` — 图谱模型、GraphStore trait、搜索方案
- `docs/technical/api-design.md` section 5 — 图谱 API 端点
- `docs/technical/test-cases.md` TC-P3-001~003

---

## File Structure

```
ring-server/src/
├── services/
│   ├── graph_service.rs        # Graph business logic (Markdown gen, search)
│   └── search_service.rs       # FTS5 + jieba-rs search
├── handlers/
│   ├── graph.rs                # Graph CRUD + search endpoints
│   └── (routes.rs updated)
├── graph/
│   └── (existing, no changes expected)
└── models/
    └── graph_model.rs          # Request/response types for graph API

ring-frontend/src/
├── components/graph/
│   ├── ForceGraph.tsx          # D3.js force-directed graph
│   └── NodeTree.tsx            # Tree navigation sidebar
├── pages/RingSpace/
│   └── GraphView.tsx           # Graph visualization page
├── api/client.ts               # Add graph API functions
└── stores/
    └── graphStore.ts           # Graph state management
```

---

## Module 1: Graph Models + Service

**Files:**
- Create: `ring-server/src/models/graph_model.rs`
- Create: `ring-server/src/services/graph_service.rs`

- [ ] **Step 1: Write failing tests for GraphService**

Test against in-memory DB + PetgraphStore:

- `create_node_with_markdown` — create node, verify markdown_path generated
- `update_node_label` — update label, verify updated_at changed
- `delete_node_with_children_cascades` — delete parent, verify children gone
- `get_children_returns_correct_order` — create 3 children, verify order
- `get_neighbors_returns_edges` — create 2 nodes + edge, verify neighbors
- `get_root_nodes` — create nodes with/without parent, verify only roots returned
- `search_nodes_by_label` — create nodes, search, verify matches

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-server && cargo test --lib services::graph_service`
Expected: FAIL

- [ ] **Step 3: Implement**

`models/graph_model.rs` — request/response types:
- `CreateNodeRequest { label, node_type, parent_id?, description? }`
- `UpdateNodeRequest { label?, description?, node_type? }`
- `CreateEdgeRequest { source_id, target_id, relation, label? }`
- `NodeResponse` (from NodeData), `EdgeResponse` (from EdgeData)
- `GraphDetailResponse { graph_id, nodes: Vec<NodeResponse>, edges: Vec<EdgeResponse> }`
- `NodeContentResponse { node_id, label, markdown_path, content, last_modified }`

`services/graph_service.rs` — `GraphService`:
- Takes `Arc<dyn Repository>` + `Arc<RwLock<PetgraphStore>>`
- `create_node(ring_id, graph_id, req)` — create in petgraph + generate markdown_path
- `get_node(ring_id, graph_id, node_id)` — read from petgraph
- `update_node(ring_id, graph_id, node_id, req)` — update in petgraph
- `delete_node(ring_id, graph_id, node_id)` — cascade delete in petgraph
- `get_children(ring_id, graph_id, parent_id)` — query petgraph
- `get_neighbors(ring_id, graph_id, node_id)` — query petgraph edges
- `get_root_nodes(ring_id, graph_id)` — nodes with no parent_id
- `get_node_content(ring_id, graph_id, node_id)` — read markdown file
- `list_graphs(ring_id)` — list all graphs for a ring (from DB)

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/models/graph_model.rs ring-server/src/services/graph_service.rs
git commit -m "feat(phase3): add graph models and GraphService with tests"
```

---

## Module 2: Graph Handlers + Routes

**Files:**
- Create: `ring-server/src/handlers/graph.rs`
- Modify: `ring-server/src/routes.rs`
- Create: `ring-server/tests/graph_integration.rs`

- [ ] **Step 1: Write integration tests**

Test TC-P3-001 and TC-P3-002 from test-cases.md:

- `create_and_get_node` — POST create, GET verify fields
- `update_node_label` — PUT update, verify changed
- `delete_node_returns_204_then_404`
- `create_edge_and_list_in_graph`
- `delete_edge_removes_from_graph`
- `get_children_of_node`
- `get_root_nodes`
- `get_node_returns_markdown_content`
- `create_node_empty_label_400`
- `create_node_nonexistent_parent_404`
- `delete_node_with_children_409`
- `create_self_loop_edge_400`
- `create_duplicate_edge_409`

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`handlers/graph.rs`:
- `list_graphs` — GET /rings/{ringId}/graphs
- `get_graph` — GET /rings/{ringId}/graphs/{graphId} → all nodes + edges
- `create_node` — POST /rings/{ringId}/graphs/{graphId}/nodes
- `get_node` — GET /rings/{ringId}/graphs/{graphId}/nodes/{nodeId}
- `update_node` — PUT /rings/{ringId}/graphs/{graphId}/nodes/{nodeId}
- `delete_node` — DELETE /rings/{ringId}/graphs/{graphId}/nodes/{nodeId}
- `get_node_content` — GET /rings/{ringId}/graphs/{graphId}/nodes/{nodeId}/content
- `create_edge` — POST /rings/{ringId}/graphs/{graphId}/edges
- `delete_edge` — DELETE /rings/{ringId}/graphs/{graphId}/edges/{edgeId}

`routes.rs` — add graph routes:
```
/api/v1/rings/{ringId}/graphs → GET list_graphs
/api/v1/rings/{ringId}/graphs/{graphId} → GET get_graph
/api/v1/rings/{ringId}/graphs/{graphId}/nodes → POST create_node
/api/v1/rings/{ringId}/graphs/{graphId}/nodes/{nodeId} → GET, PUT, DELETE
/api/v1/rings/{ringId}/graphs/{graphId}/nodes/{nodeId}/content → GET
/api/v1/rings/{ringId}/graphs/{graphId}/edges → POST create_edge
/api/v1/rings/{ringId}/graphs/{graphId}/edges/{edgeId} → DELETE
```

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/handlers/graph.rs ring-server/src/routes.rs ring-server/tests/graph_integration.rs
git commit -m "feat(phase3): add graph CRUD handlers and routes with integration tests"
```

---

## Module 3: Search Service

**Files:**
- Create: `ring-server/src/services/search_service.rs`
- Create: `ring-server/src/handlers/search.rs` (or add to graph handler)
- Modify: `ring-server/src/db/traits.rs` — add search methods
- Modify: `ring-server/src/db/sqlite.rs` — implement FTS5 search
- Modify: `ring-server/src/routes.rs` — add search routes

- [ ] **Step 1: Write failing tests**

- `keyword_search_returns_matching_nodes` — insert FTS data, search "定价", verify results
- `keyword_search_no_results` — search "不存在", verify empty
- `global_search_across_graphs` — search across multiple graphs
- `search_snippet_contains_highlight` — verify snippet field in results

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`services/search_service.rs`:
- `SearchService` with `Arc<dyn Repository>`
- `search_nodes(ring_id, query, graph_ids, limit)` — FTS5 keyword search
- `global_search(ring_id, query, include_conversations, limit)` — cross-graph search
- Use jieba-rs for Chinese tokenization before inserting/searching FTS5

Add to Repository trait:
- `search_nodes_fts(ring_id, tokenized_query, graph_ids, limit) -> Vec<SearchResult>`
- `search_conversations_fts(ring_id, tokenized_query, limit) -> Vec<SearchResult>`

`handlers/search.rs`:
- `search` — POST /rings/{ringId}/search
- `global_search` — POST /rings/{ringId}/search/global

Routes:
```
/api/v1/rings/{ringId}/search → POST search
/api/v1/rings/{ringId}/search/global → POST global_search
```

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/services/search_service.rs ring-server/src/handlers/search.rs ring-server/src/db/ ring-server/src/routes.rs
git commit -m "feat(phase3): add FTS5 keyword search with jieba-rs Chinese tokenization"
```

---

## Module 4: Frontend — Graph Visualization (D3.js)

**Files:**
- Create: `ring-frontend/src/components/graph/ForceGraph.tsx`
- Create: `ring-frontend/src/components/graph/NodeTree.tsx`
- Create: `ring-frontend/src/stores/graphStore.ts`
- Create: `ring-frontend/src/pages/RingSpace/GraphView.tsx`
- Modify: `ring-frontend/src/api/client.ts`
- Modify: `ring-frontend/src/App.tsx`

- [ ] **Step 1: Write tests**

- `NodeTree renders tree structure` — given nodes with parent_id, renders expandable tree
- `NodeTree click selects node` — click fires onSelect callback
- `GraphView renders graph container` — verify D3 container element exists

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`ForceGraph.tsx`:
- D3.js force-directed graph rendering
- Props: `nodes`, `edges`, `onNodeClick`, `onNodeHover`
- Features: zoom, pan, drag, node click → select, hover → tooltip
- Color coding by node_type
- Performance: virtual rendering for 100+ nodes

`NodeTree.tsx`:
- Recursive tree component showing parent→children hierarchy
- Expand/collapse nodes
- Click to select → highlight in ForceGraph
- Show node label + type icon

`graphStore.ts`:
- Zustand store: `graphs[]`, `currentGraphId`, `selectedNodeId`, `nodes[]`, `edges[]`
- Actions: `loadGraphs(ringId)`, `selectGraph(graphId)`, `selectNode(nodeId)`, `createNode(req)`, `deleteNode(nodeId)`, `createEdge(req)`, `deleteEdge(edgeId)`, `searchNodes(query)`

`GraphView.tsx`:
- Split layout: left sidebar (NodeTree) + main area (ForceGraph)
- Top bar: graph selector dropdown + search input
- Node click → show Markdown content in a slide-over panel
- Toolbar: zoom controls, layout reset

`client.ts` — add functions:
- `listGraphs(ringId)`, `getGraph(ringId, graphId)`, `createNode(ringId, graphId, req)`, `updateNode(ringId, graphId, nodeId, req)`, `deleteNode(ringId, graphId, nodeId)`, `getNodeContent(ringId, graphId, nodeId)`, `createEdge(ringId, graphId, req)`, `deleteEdge(ringId, graphId, edgeId)`, `searchNodes(ringId, query)`

`App.tsx` — add route: `/ring/:ringId/graph` → GraphView

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/components/graph/ ring-frontend/src/stores/graphStore.ts ring-frontend/src/pages/RingSpace/GraphView.tsx ring-frontend/src/api/client.ts ring-frontend/src/App.tsx
git commit -m "feat(phase3): add D3.js graph visualization, node tree, and graph store"
```

---

## Module 5: graph.json Sync Verification

**Files:**
- Create: `ring-server/tests/graph_sync_integration.rs`

- [ ] **Step 1: Write tests**

- `create_node_updates_graph_json` — create node, export graph.json, verify node present
- `delete_node_removes_from_graph_json` — delete node, export, verify absent
- `import_export_round_trip` — create nodes+edges, export, import to new store, verify identical
- `multiple_graphs_independent` — create nodes in 2 graphs, export each, verify no cross-contamination

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement fixes if needed**

Verify that PetgraphStore's import/export handles all edge cases. Fix any issues found.

- [ ] **Step 4: Run all tests**

Run: `cd ring-server && cargo test`

- [ ] **Step 5: Commit**

```bash
git add ring-server/tests/graph_sync_integration.rs
git commit -m "feat(phase3): add graph.json sync integration tests"
```

---

## Module 6: Integration Verification

- [ ] **Step 1: Run all backend tests**

Run: `cd ring-server && cargo test`
Expected: ALL PASS

- [ ] **Step 2: Run all frontend tests**

Run: `cd ring-frontend && npm test`
Expected: ALL PASS

- [ ] **Step 3: Run clippy + fmt**

```bash
cd ring-server && cargo fmt --check && cargo clippy -- -D warnings
```

- [ ] **Step 4: Manual smoke test**

1. Create Ring → Confirm blueprint (creates graphs)
2. Enter Ring → Switch to graph view
3. Create nodes → verify in tree + force graph
4. Create edges → verify connection visible
5. Click node → verify Markdown content shown
6. Search nodes → verify results

- [ ] **Step 5: Final commit**

```bash
git commit --allow-empty -m "milestone: Phase 3 complete — knowledge graph"
```
