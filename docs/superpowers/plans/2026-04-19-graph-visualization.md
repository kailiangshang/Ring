# Graph Visualization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add graph CRUD backend + D3.js force-directed graph visualization so users can create, view, and interact with knowledge graphs inside Ring.

**Architecture:** Backend adds a `graphs` table and extends `graph_nodes`/`graph_edges` with missing columns (`parent_id`, `node_type`, `tags`, `relation`, `graph_id`). CRUD routes follow existing handler→service→model pattern. Frontend adds a Zustand graph store, a D3 force-directed SVG renderer, and node/edge creation via `!graph` commands. Multi-graph support deferred — each Ring has one default graph for now.

**Tech Stack:** D3.js v7 (already installed), Zustand, SVG rendering. No new npm packages needed.

**Scope:** Single default graph per Ring. Node/edge CRUD. Force-directed visualization with zoom/pan/click. Search deferred. Export deferred. Multi-graph deferred.

---

## File Structure

```
server/
├── migrations/
│   └── 004_graphs_table.sql               # NEW: graphs table + schema migration
├── src/
│   ├── models/
│   │   └── graph.rs                        # NEW: GraphRow, GraphNodeRow, GraphEdgeRow, CRUD queries
│   ├── services/
│   │   └── graph.rs                        # NEW: get_graph, create_node, update_node, delete_node, create_edge, delete_edge
│   └── routes/
│       ├── mod.rs                          # MODIFY: add graph routes
│       └── graph.rs                        # NEW: 6 endpoints

ui/src/
├── stores/
│   └── graph-store.ts                      # NEW: fetch graph, create node/edge, selected node state
├── components/
│   └── panels/
│       ├── GraphPanel.tsx                  # MODIFY: full D3 visualization
│       └── GraphCanvas.tsx                 # NEW: D3 force-directed SVG renderer
├── services/
│   └── command-parser.ts                   # NO CHANGE (already handles !graph)
```

---

### Task 1: Database Migration — Graphs Table + Schema Fix

**Files:**
- Create: `server/migrations/004_graphs_table.sql`

- [ ] **Step 1: Create migration**

`server/migrations/004_graphs_table.sql`:

```sql
CREATE TABLE IF NOT EXISTS graphs (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT 'main',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

ALTER TABLE graph_nodes ADD COLUMN graph_id TEXT REFERENCES graphs(id) ON DELETE CASCADE;
ALTER TABLE graph_nodes ADD COLUMN parent_id TEXT REFERENCES graph_nodes(id) ON DELETE SET NULL;
ALTER TABLE graph_nodes RENAME COLUMN kind TO node_type;
ALTER TABLE graph_nodes ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
ALTER TABLE graph_nodes ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));

ALTER TABLE graph_edges ADD COLUMN graph_id TEXT REFERENCES graphs(id) ON DELETE CASCADE;
ALTER TABLE graph_edges ADD COLUMN relation TEXT NOT NULL DEFAULT 'related_to';

CREATE INDEX IF NOT EXISTS idx_graph_nodes_graph ON graph_nodes(graph_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_graph ON graph_edges(graph_id);

INSERT INTO graphs (id, ring_id, name)
SELECT 'graph-' || r.id, r.id, 'main'
FROM rings r
WHERE NOT EXISTS (SELECT 1 FROM graphs WHERE ring_id = r.id);

UPDATE graph_nodes SET graph_id = 'graph-' || ring_id WHERE graph_id IS NULL;
UPDATE graph_edges SET graph_id = 'graph-' || ring_id WHERE graph_id IS NULL;
```

- [ ] **Step 2: Verify build**

Run: `cd server && cargo build 2>&1`
Expected: Compiles (migration runs at startup, not build time).

- [ ] **Step 3: Commit**

```bash
git add server/migrations/004_graphs_table.sql
git commit -m "feat(server): add graphs table and extend graph_nodes/graph_edges schema"
```

---

### Task 2: Graph Model — CRUD Queries

**Files:**
- Create: `server/src/models/graph.rs`
- Modify: `server/src/models/mod.rs`

- [ ] **Step 1: Create graph model**

`server/src/models/graph.rs`:

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct GraphRow {
    pub id: String,
    pub ring_id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct GraphNodeRow {
    pub id: String,
    pub graph_id: String,
    pub ring_id: String,
    pub label: String,
    pub parent_id: Option<String>,
    pub node_type: String,
    pub content: String,
    pub tags: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct GraphEdgeRow {
    pub id: String,
    pub graph_id: String,
    pub ring_id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNodeInput {
    pub label: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default = "default_node_type")]
    pub node_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content: String,
}

fn default_node_type() -> String {
    "topic".into()
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeInput {
    pub label: Option<String>,
    pub tags: Option<Vec<String>>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEdgeInput {
    pub source_id: String,
    pub target_id: String,
    #[serde(default = "default_relation")]
    pub relation: String,
    #[serde(default)]
    pub label: String,
}

fn default_relation() -> String {
    "related_to".into()
}

pub async fn ensure_default_graph(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
) -> Result<GraphRow> {
    if let Some(graph) = sqlx::query_as::<_, GraphRow>(
        "SELECT * FROM graphs WHERE ring_id = ?1 LIMIT 1",
    )
    .bind(ring_id)
    .fetch_optional(pool)
    .await?
    {
        return Ok(graph);
    }

    let id = format!("graph-{ring_id}");
    sqlx::query_as::<_, GraphRow>(
        "INSERT INTO graphs (id, ring_id, name) VALUES (?1, ?2, 'main') RETURNING *",
    )
    .bind(&id)
    .bind(ring_id)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn get_graph_by_ring(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
) -> Result<GraphRow> {
    ensure_default_graph(pool, ring_id).await
}

pub async fn list_nodes(
    pool: &sqlx::SqlitePool,
    graph_id: &str,
) -> Result<Vec<GraphNodeRow>> {
    sqlx::query_as::<_, GraphNodeRow>(
        "SELECT * FROM graph_nodes WHERE graph_id = ?1 ORDER BY created_at",
    )
    .bind(graph_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn list_edges(
    pool: &sqlx::SqlitePool,
    graph_id: &str,
) -> Result<Vec<GraphEdgeRow>> {
    sqlx::query_as::<_, GraphEdgeRow>(
        "SELECT * FROM graph_edges WHERE graph_id = ?1 ORDER BY created_at",
    )
    .bind(graph_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn create_node(
    pool: &sqlx::SqlitePool,
    id: &str,
    graph_id: &str,
    ring_id: &str,
    input: &CreateNodeInput,
) -> Result<GraphNodeRow> {
    sqlx::query_as::<_, GraphNodeRow>(
        "INSERT INTO graph_nodes (id, graph_id, ring_id, label, parent_id, node_type, content, tags)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         RETURNING *",
    )
    .bind(id)
    .bind(graph_id)
    .bind(ring_id)
    .bind(&input.label)
    .bind(&input.parent_id)
    .bind(&input.node_type)
    .bind(&input.content)
    .bind(serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".into()))
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn update_node(
    pool: &sqlx::SqlitePool,
    node_id: &str,
    input: &UpdateNodeInput,
) -> Result<GraphNodeRow> {
    let current = sqlx::query_as::<_, GraphNodeRow>(
        "SELECT * FROM graph_nodes WHERE id = ?1",
    )
    .bind(node_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound("node not found".into()))?;

    let label = input.label.as_deref().unwrap_or(&current.label);
    let tags = input
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_else(|_| "[]".into()))
        .unwrap_or(current.tags);
    let content = input.content.as_deref().unwrap_or(&current.content);

    sqlx::query_as::<_, GraphNodeRow>(
        "UPDATE graph_nodes SET label = ?1, tags = ?2, content = ?3, updated_at = datetime('now')
         WHERE id = ?4 RETURNING *",
    )
    .bind(label)
    .bind(tags)
    .bind(content)
    .bind(node_id)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn delete_node(pool: &sqlx::SqlitePool, node_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM graph_nodes WHERE id = ?1")
        .bind(node_id)
        .execute(pool)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("node not found".into()));
    }
    Ok(())
}

pub async fn create_edge(
    pool: &sqlx::SqlitePool,
    id: &str,
    graph_id: &str,
    ring_id: &str,
    input: &CreateEdgeInput,
) -> Result<GraphEdgeRow> {
    sqlx::query_as::<_, GraphEdgeRow>(
        "INSERT INTO graph_edges (id, graph_id, ring_id, source_id, target_id, relation, label)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         RETURNING *",
    )
    .bind(id)
    .bind(graph_id)
    .bind(ring_id)
    .bind(&input.source_id)
    .bind(&input.target_id)
    .bind(&input.relation)
    .bind(&input.label)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn delete_edge(pool: &sqlx::SqlitePool, edge_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM graph_edges WHERE id = ?1")
        .bind(edge_id)
        .execute(pool)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("edge not found".into()));
    }
    Ok(())
}
```

- [ ] **Step 2: Register module**

In `server/src/models/mod.rs`, add `pub mod graph;`.

- [ ] **Step 3: Verify build**

Run: `cd server && cargo check 2>&1`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add server/src/models/graph.rs server/src/models/mod.rs
git commit -m "feat(server): graph model with CRUD queries for nodes and edges"
```

---

### Task 3: Graph Service + Routes

**Files:**
- Create: `server/src/services/graph.rs`
- Create: `server/src/routes/graph.rs`
- Modify: `server/src/services/mod.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Create graph service**

`server/src/services/graph.rs`:

```rust
use crate::error::Result;
use crate::models::graph;
use crate::state::AppState;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub id: String,
    pub name: String,
    pub ring_id: String,
    pub nodes: Vec<graph::GraphNodeRow>,
    pub edges: Vec<graph::GraphEdgeRow>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_full_graph(
    state: &AppState,
    ring_id: &str,
) -> Result<GraphResponse> {
    let g = graph::get_graph_by_ring(&state.db, ring_id).await?;
    let nodes = graph::list_nodes(&state.db, &g.id).await?;
    let edges = graph::list_edges(&state.db, &g.id).await?;
    Ok(GraphResponse {
        id: g.id,
        name: g.name,
        ring_id: g.ring_id,
        nodes,
        edges,
        created_at: g.created_at,
        updated_at: g.updated_at,
    })
}

pub async fn create_node(
    state: &AppState,
    ring_id: &str,
    input: &graph::CreateNodeInput,
) -> Result<graph::GraphNodeRow> {
    let g = graph::get_graph_by_ring(&state.db, ring_id).await?;
    let id = ulid::Ulid::new().to_string();
    graph::create_node(&state.db, &id, &g.id, ring_id, input).await
}

pub async fn update_node(
    state: &AppState,
    node_id: &str,
    input: &graph::UpdateNodeInput,
) -> Result<graph::GraphNodeRow> {
    graph::update_node(&state.db, node_id, input).await
}

pub async fn delete_node(state: &AppState, node_id: &str) -> Result<()> {
    graph::delete_node(&state.db, node_id).await
}

pub async fn create_edge(
    state: &AppState,
    ring_id: &str,
    input: &graph::CreateEdgeInput,
) -> Result<graph::GraphEdgeRow> {
    let g = graph::get_graph_by_ring(&state.db, ring_id).await?;
    let id = ulid::Ulid::new().to_string();
    graph::create_edge(&state.db, &id, &g.id, ring_id, input).await
}

pub async fn delete_edge(state: &AppState, edge_id: &str) -> Result<()> {
    graph::delete_edge(&state.db, edge_id).await
}
```

- [ ] **Step 2: Create graph routes**

`server/src/routes/graph.rs`:

```rust
use axum::extract::{Path, State};
use axum::Json;

use crate::error::{Result, RingError};
use crate::extractors::auth::AuthUser;
use crate::models::graph::{CreateEdgeInput, CreateNodeInput, UpdateNodeInput};
use crate::models::ring;
use crate::services;
use crate::state::AppState;

pub async fn get_graph(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<services::graph::GraphResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let graph = services::graph::get_full_graph(&state, &ring_id).await?;
    Ok(Json(graph))
}

pub async fn create_node(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateNodeInput>,
) -> Result<Json<crate::models::graph::GraphNodeRow>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let node = services::graph::create_node(&state, &ring_id, &body).await?;
    Ok(Json(node))
}

pub async fn update_node(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, node_id)): Path<(String, String)>,
    Json(body): Json<UpdateNodeInput>,
) -> Result<Json<crate::models::graph::GraphNodeRow>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let node = services::graph::update_node(&state, &node_id, &body).await?;
    Ok(Json(node))
}

pub async fn delete_node(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, node_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden("only creator/admin can delete nodes".into()));
    }
    services::graph::delete_node(&state, &node_id).await?;
    Ok(Json(serde_json::json!({"status": "deleted"})))
}

pub async fn create_edge(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateEdgeInput>,
) -> Result<Json<crate::models::graph::GraphEdgeRow>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let edge = services::graph::create_edge(&state, &ring_id, &body).await?;
    Ok(Json(edge))
}

pub async fn delete_edge(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, edge_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden("only creator/admin can delete edges".into()));
    }
    services::graph::delete_edge(&state, &edge_id).await?;
    Ok(Json(serde_json::json!({"status": "deleted"})))
}
```

- [ ] **Step 3: Register modules and routes**

In `server/src/services/mod.rs` add: `pub mod graph;`

In `server/src/routes/mod.rs`:
- Add `mod graph;` at the top with the other mod declarations
- Add these routes inside the `let api = Router::new()` block:

```rust
        .route(
            "/rings/{ring_id}/graph",
            get(graph::get_graph).post(graph::create_node),
        )
        .route(
            "/rings/{ring_id}/graph/nodes/{node_id}",
            put(graph::update_node).delete(graph::delete_node),
        )
        .route(
            "/rings/{ring_id}/graph/edges",
            post(graph::create_edge),
        )
        .route(
            "/rings/{ring_id}/graph/edges/{edge_id}",
            delete(graph::delete_edge),
        )
```

- [ ] **Step 4: Verify build**

Run: `cd server && cargo check 2>&1`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add server/src/services/graph.rs server/src/services/mod.rs server/src/routes/graph.rs server/src/routes/mod.rs
git commit -m "feat(server): graph CRUD routes — get graph, create/update/delete nodes, create/delete edges"
```

---

### Task 4: Frontend Graph Store

**Files:**
- Create: `ui/src/stores/graph-store.ts`

- [ ] **Step 1: Create graph store**

`ui/src/stores/graph-store.ts`:

```typescript
import { create } from 'zustand'
import type { GraphNode, GraphEdge } from '../types/graph'
import { api } from './api'

interface GraphState {
  nodes: GraphNode[]
  edges: GraphEdge[]
  graph_id: string | null
  loading: boolean
  selected_node_id: string | null
  fetchGraph: (ringId: string) => Promise<void>
  createNode: (ringId: string, label: string, nodeType?: string) => Promise<void>
  deleteNode: (ringId: string, nodeId: string) => Promise<void>
  createEdge: (ringId: string, sourceId: string, targetId: string, relation?: string) => Promise<void>
  deleteEdge: (ringId: string, edgeId: string) => Promise<void>
  selectNode: (nodeId: string | null) => void
}

interface GraphResponse {
  id: string
  name: string
  ring_id: string
  nodes: GraphNode[]
  edges: GraphEdge[]
}

interface NodeResponse {
  id: string
  graph_id: string
  ring_id: string
  label: string
  parent_id: string | null
  node_type: string
  content: string
  tags: string
  created_at: string
  updated_at: string
}

interface EdgeResponse {
  id: string
  graph_id: string
  ring_id: string
  source_id: string
  target_id: string
  relation: string
  label: string
  created_at: string
}

function toGraphNode(r: NodeResponse): GraphNode {
  return {
    id: r.id,
    label: r.label,
    parent_id: r.parent_id,
    markdown_path: '',
    node_type: r.node_type as GraphNode['node_type'],
    tags: typeof r.tags === 'string' ? JSON.parse(r.tags) : r.tags,
    metadata: {},
    created_at: r.created_at,
    updated_at: r.updated_at,
  }
}

function toGraphEdge(r: EdgeResponse): GraphEdge {
  return {
    id: r.id,
    source_id: r.source_id,
    target_id: r.target_id,
    relation: r.relation as GraphEdge['relation'],
    label: r.label,
    created_at: r.created_at,
  }
}

export const useGraphStore = create<GraphState>((set, get) => ({
  nodes: [],
  edges: [],
  graph_id: null,
  loading: false,
  selected_node_id: null,

  fetchGraph: async (ringId: string) => {
    set({ loading: true })
    try {
      const res = await api.get<GraphResponse>(`/rings/${ringId}/graph`)
      set({
        graph_id: res.id,
        nodes: (res.nodes as unknown as NodeResponse[]).map(toGraphNode),
        edges: (res.edges as unknown as EdgeResponse[]).map(toGraphEdge),
        loading: false,
      })
    } catch {
      set({ loading: false })
    }
  },

  createNode: async (ringId, label, nodeType) => {
    const res = await api.post<NodeResponse>(`/rings/${ringId}/graph`, {
      label,
      node_type: nodeType ?? 'topic',
    })
    set((s) => ({ nodes: [...s.nodes, toGraphNode(res)] }))
  },

  deleteNode: async (ringId, nodeId) => {
    await api.delete(`/rings/${ringId}/graph/nodes/${nodeId}`)
    set((s) => ({
      nodes: s.nodes.filter((n) => n.id !== nodeId),
      edges: s.edges.filter((e) => e.source_id !== nodeId && e.target_id !== nodeId),
      selected_node_id: s.selected_node_id === nodeId ? null : s.selected_node_id,
    }))
  },

  createEdge: async (ringId, sourceId, targetId, relation) => {
    const res = await api.post<EdgeResponse>(`/rings/${ringId}/graph/edges`, {
      source_id: sourceId,
      target_id: targetId,
      relation: relation ?? 'related_to',
    })
    set((s) => ({ edges: [...s.edges, toGraphEdge(res)] }))
  },

  deleteEdge: async (ringId, edgeId) => {
    await api.delete(`/rings/${ringId}/graph/edges/${edgeId}`)
    set((s) => ({ edges: s.edges.filter((e) => e.id !== edgeId) }))
  },

  selectNode: (nodeId) => set({ selected_node_id: nodeId }),
}))
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add ui/src/stores/graph-store.ts
git commit -m "feat(ui): graph store with fetch, CRUD, and node selection"
```

---

### Task 5: D3 Force-Directed Graph Canvas

**Files:**
- Create: `ui/src/components/panels/GraphCanvas.tsx`

- [ ] **Step 1: Create D3 graph canvas**

`ui/src/components/panels/GraphCanvas.tsx`:

```tsx
import { useEffect, useRef, useCallback } from 'react'
import * as d3 from 'd3'
import type { GraphNode, GraphEdge } from '../../types/graph'

interface SimNode extends d3.SimulationNodeDatum {
  id: string
  label: string
  node_type: string
}

interface SimEdge extends d3.SimulationLinkDatum<SimNode> {
  id: string
  relation: string
  label: string
}

interface GraphCanvasProps {
  nodes: GraphNode[]
  edges: GraphEdge[]
  selectedNodeId: string | null
  onSelectNode: (id: string | null) => void
}

const NODE_COLORS: Record<string, string> = {
  topic: '#0891B2',
  category: '#22c55e',
  leaf: '#f59e0b',
}

export function GraphCanvas({ nodes, edges, selectedNodeId, onSelectNode }: GraphCanvasProps) {
  const svgRef = useRef<SVGSVGElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)

  const render = useCallback(() => {
    if (!svgRef.current || !containerRef.current) return

    const svg = d3.select(svgRef.current)
    svg.selectAll('*').remove()

    const width = containerRef.current.clientWidth || 400
    const height = containerRef.current.clientHeight || 300

    const simNodes: SimNode[] = nodes.map((n) => ({
      id: n.id,
      label: n.label,
      node_type: n.node_type,
    }))

    const simEdges: SimEdge[] = edges.map((e) => ({
      source: e.source_id,
      target: e.target_id,
      id: e.id,
      relation: e.relation,
      label: e.label,
    }))

    const g = svg
      .append('g')

    const zoom = d3
      .zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.3, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform)
      })

    svg.call(zoom)

    const simulation = d3
      .forceSimulation<SimNode>(simNodes)
      .force('link', d3.forceLink<SimNode, SimEdge>(simEdges).id((d) => d.id).distance(80))
      .force('charge', d3.forceManyBody().strength(-200))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(30))

    const link = g
      .append('g')
      .selectAll('line')
      .data(simEdges)
      .join('line')
      .attr('stroke', '#1a2030')
      .attr('stroke-width', 1.5)

    const linkLabel = g
      .append('g')
      .selectAll('text')
      .data(simEdges)
      .join('text')
      .attr('fill', '#3a4550')
      .attr('font-size', 8)
      .attr('text-anchor', 'middle')
      .text((d) => d.relation)

    const node = g
      .append('g')
      .selectAll<SVGCircleElement, SimNode>('circle')
      .data(simNodes)
      .join('circle')
      .attr('r', 12)
      .attr('fill', (d) => NODE_COLORS[d.node_type] ?? '#0891B2')
      .attr('stroke', (d) => (d.id === selectedNodeId ? '#67E8F9' : 'none'))
      .attr('stroke-width', (d) => (d.id === selectedNodeId ? 3 : 0))
      .attr('cursor', 'pointer')
      .on('click', (_event, d) => {
        onSelectNode(d.id === selectedNodeId ? null : d.id)
      })
      .call(
        d3
          .drag<SVGCircleElement, SimNode>()
          .on('start', (event, d) => {
            if (!event.active) simulation.alphaTarget(0.3).restart()
            d.fx = d.x
            d.fy = d.y
          })
          .on('drag', (event, d) => {
            d.fx = event.x
            d.fy = event.y
          })
          .on('end', (event, d) => {
            if (!event.active) simulation.alphaTarget(0)
            d.fx = null
            d.fy = null
          }),
      )

    const label = g
      .append('g')
      .selectAll('text')
      .data(simNodes)
      .join('text')
      .attr('dy', 24)
      .attr('text-anchor', 'middle')
      .attr('fill', '#bfc7d5')
      .attr('font-size', 10)
      .attr('font-family', 'inherit')
      .text((d) => d.label.length > 12 ? d.label.slice(0, 12) + '…' : d.label)

    simulation.on('tick', () => {
      link
        .attr('x1', (d) => (d.source as SimNode).x ?? 0)
        .attr('y1', (d) => (d.source as SimNode).y ?? 0)
        .attr('x2', (d) => (d.target as SimNode).x ?? 0)
        .attr('y2', (d) => (d.target as SimNode).y ?? 0)

      linkLabel
        .attr('x', (d) => (((d.source as SimNode).x ?? 0) + ((d.target as SimNode).x ?? 0)) / 2)
        .attr('y', (d) => (((d.source as SimNode).y ?? 0) + ((d.target as SimNode).y ?? 0)) / 2)

      node.attr('cx', (d) => d.x ?? 0).attr('cy', (d) => d.y ?? 0)

      label.attr('x', (d) => d.x ?? 0).attr('y', (d) => d.y ?? 0)
    })

    return () => {
      simulation.stop()
    }
  }, [nodes, edges, selectedNodeId, onSelectNode])

  useEffect(() => {
    const cleanup = render()
    return () => cleanup?.()
  }, [render])

  return (
    <div ref={containerRef} style={{ width: '100%', height: '100%', background: 'var(--bg-base)' }}>
      <svg ref={svgRef} width="100%" height="100%" />
    </div>
  )
}
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/panels/GraphCanvas.tsx
git commit -m "feat(ui): D3 force-directed graph canvas with zoom, pan, drag, select"
```

---

### Task 6: GraphPanel — Full Implementation

**Files:**
- Modify: `ui/src/components/panels/GraphPanel.tsx`

- [ ] **Step 1: Rewrite GraphPanel with real data**

`ui/src/components/panels/GraphPanel.tsx`:

```tsx
import { useEffect, useState } from 'react'
import { useGraphStore } from '../../stores/graph-store'
import { useRingStore } from '../../stores/ring-store'
import { GraphCanvas } from './GraphCanvas'

export function GraphPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const nodes = useGraphStore((s) => s.nodes)
  const edges = useGraphStore((s) => s.edges)
  const loading = useGraphStore((s) => s.loading)
  const selected_node_id = useGraphStore((s) => s.selected_node_id)
  const fetchGraph = useGraphStore((s) => s.fetchGraph)
  const createNode = useGraphStore((s) => s.createNode)
  const deleteNode = useGraphStore((s) => s.deleteNode)
  const selectNode = useGraphStore((s) => s.selectNode)

  const [newNodeLabel, setNewLabel] = useState('')

  useEffect(() => {
    if (active_ring_id) {
      fetchGraph(active_ring_id)
    }
  }, [active_ring_id, fetchGraph])

  const selectedNode = nodes.find((n) => n.id === selected_node_id)

  const handleCreateNode = () => {
    if (!newNodeLabel.trim() || !active_ring_id) return
    createNode(active_ring_id, newNodeLabel.trim())
    setNewLabel('')
  }

  if (loading) {
    return (
      <div style={{ padding: 16, color: 'var(--text-dim)', fontSize: 12 }}>
        Loading graph...
      </div>
    )
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '8px 12px', borderBottom: '1px solid var(--border)' }}>
        <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
          <input
            value={newNodeLabel}
            onChange={(e) => setNewLabel(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleCreateNode()
            }}
            placeholder="node label..."
            style={{
              flex: 1,
              background: 'var(--bg-input)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '4px 8px',
              color: 'var(--text-primary)',
              fontSize: 11,
              fontFamily: 'inherit',
              outline: 'none',
            }}
          />
          <button
            onClick={handleCreateNode}
            disabled={!newNodeLabel.trim()}
            style={{
              background: 'var(--accent-cyan)',
              color: 'var(--bg-base)',
              border: 'none',
              borderRadius: 4,
              padding: '4px 12px',
              fontSize: 11,
              fontWeight: 700,
              cursor: newNodeLabel.trim() ? 'pointer' : 'default',
              opacity: newNodeLabel.trim() ? 1 : 0.4,
            }}
          >
            +Node
          </button>
        </div>
        <div style={{ marginTop: 4, fontSize: 10, color: 'var(--text-dim)' }}>
          {nodes.length} nodes · {edges.length} edges
        </div>
      </div>

      <div style={{ flex: 1, minHeight: 0 }}>
        <GraphCanvas
          nodes={nodes}
          edges={edges}
          selectedNodeId={selected_node_id}
          onSelectNode={selectNode}
        />
      </div>

      {selectedNode && (
        <div
          style={{
            padding: '8px 12px',
            borderTop: '1px solid var(--border)',
            background: 'var(--bg-panel)',
            fontSize: 11,
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ color: 'var(--accent-ice)', fontWeight: 700 }}>
              {selectedNode.label}
            </span>
            <div style={{ display: 'flex', gap: 4 }}>
              <span
                style={{
                  fontSize: 9,
                  background: 'var(--bg-hover)',
                  padding: '2px 6px',
                  borderRadius: 2,
                  color: 'var(--text-dim)',
                }}
              >
                {selectedNode.node_type}
              </span>
              <button
                onClick={() => {
                  if (active_ring_id) deleteNode(active_ring_id, selectedNode.id)
                }}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--accent-amber)',
                  cursor: 'pointer',
                  fontSize: 10,
                  padding: '0 4px',
                }}
              >
                ×
              </button>
            </div>
          </div>
          {selectedNode.tags.length > 0 && (
            <div style={{ marginTop: 4, display: 'flex', gap: 4, flexWrap: 'wrap' }}>
              {selectedNode.tags.map((tag) => (
                <span
                  key={tag}
                  style={{
                    fontSize: 9,
                    background: 'var(--bg-hover)',
                    padding: '1px 6px',
                    borderRadius: 2,
                    color: 'var(--text-secondary)',
                  }}
                >
                  {tag}
                </span>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npx tsc --noEmit && npm run build`
Expected: Clean build.

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/panels/GraphPanel.tsx
git commit -m "feat(ui): GraphPanel with D3 canvas, node creation, selection, deletion"
```

---

### Task 7: Chat Command Integration — `!graph` Creates Nodes

**Files:**
- Modify: `ui/src/stores/chat-store.ts`

- [ ] **Step 1: Add graph node creation via `!node <label>` command**

In `ui/src/stores/chat-store.ts`, find the command dispatch section in `send()`. Add a new case inside the `action` switch block, after the `!save` case:

```typescript
            else if (cmd.action === 'node') {
              const name = cmd.args
              if (name && ring_id) {
                useGraphStore.getState().createNode(ring_id, name)
              }
            }
```

Also add the import at the top:

```typescript
import { useGraphStore } from './graph-store'
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add ui/src/stores/chat-store.ts
git commit -m "feat(ui): !node <label> command creates graph nodes via chat"
```

---

### Task 8: E2E Smoke Test

**Files:** No new files

- [ ] **Step 1: Build + start**

```bash
cd ui && npm run build
cd ../server && rm -f ~/.ring/ring.db && cargo build && ./target/debug/ring-server &
sleep 3
```

- [ ] **Step 2: Test graph API**

```bash
# Setup
TOKEN=$(curl -s -X POST http://localhost:7420/api/setup -H 'Content-Type: application/json' -d '{"display_name":"Kai","avatar":"🦊","llm_provider":"openai","llm_api_key":"sk-test","llm_model":"gpt-4o","gitlab_url":"https://g.test","gitlab_token":"glpat-test"}' | python3 -c "import sys,json; print(json.load(sys.stdin)['token_id'])")
RING_ID=$(curl -s -X POST http://localhost:7420/api/rings -H 'Content-Type: application/json' -H "X-Ring-Token: $TOKEN" -d '{"name":"test","role_description":"test"}' | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")

# Get graph (should auto-create default)
curl -s -H "X-Ring-Token: $TOKEN" "http://localhost:7420/api/rings/$RING_ID/graph" | python3 -m json.tool

# Create node
curl -s -X POST -H 'Content-Type: application/json' -H "X-Ring-Token: $TOKEN" "http://localhost:7420/api/rings/$RING_ID/graph" -d '{"label":"竞品分析","node_type":"topic"}' | python3 -m json.tool

# Create second node
NODE2=$(curl -s -X POST -H 'Content-Type: application/json' -H "X-Ring-Token: $TOKEN" "http://localhost:7420/api/rings/$RING_ID/graph" -d '{"label":"市场趋势","node_type":"leaf"}' | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")

# Get graph again (should show nodes)
curl -s -H "X-Ring-Token: $TOKEN" "http://localhost:7420/api/rings/$RING_ID/graph" | python3 -m json.tool
```

Expected: Graph with 2 nodes, auto-created default graph.

- [ ] **Step 3: Kill backend**

```bash
kill %1
```

---

## Self-Review

### 1. Spec Coverage

| API Design Requirement | Covered | Task |
|------------------------|---------|------|
| GET graph (4.2 simplified) | Yes | Task 3 |
| POST create node (4.3) | Yes | Task 3 |
| PUT update node (4.4) | Yes | Task 3 |
| DELETE node (4.5) | Yes | Task 3 |
| POST create edge (4.6) | Yes | Task 3 |
| DELETE edge (4.7) | Yes | Task 3 |
| D3 force-directed visualization | Yes | Task 5-6 |
| Zoom, pan, node drag | Yes | Task 5 |
| Node selection + detail | Yes | Task 6 |
| Chat command integration | Yes | Task 7 |
| `!graph` panel toggle | Already works | — |

**Deferred:**
- Graph list (4.1) — single default graph for now
- Search (4.8) — deferred
- Multi-graph — deferred
- Export PNG/SVG/PDF — deferred
- Blueprint preview — deferred

### 2. Placeholder Scan

No TBD/TODO/placeholders found. All steps contain complete code.

### 3. Type Consistency

- `GraphNodeRow` (Rust) fields map to `NodeResponse` (TypeScript) → `toGraphNode()` converts to `GraphNode` (from `types/graph.ts`)
- `GraphEdgeRow` (Rust) maps to `EdgeResponse` → `toGraphEdge()` converts to `GraphEdge`
- `GraphCanvas` accepts `GraphNode[]` and `GraphEdge[]` from types — matches store state
- `CreateNodeInput` (Rust) matches `{ label, node_type }` from frontend `createNode()`
- `CreateEdgeInput` (Rust) matches `{ source_id, target_id, relation }` from frontend `createEdge()`

---

**Plan complete and saved to `docs/superpowers/plans/2026-04-19-graph-visualization.md`. Inline execution recommended (D3 rendering may need iteration). Ready to execute.**
