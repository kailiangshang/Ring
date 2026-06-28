import { create } from 'zustand'
import type { GraphNode, GraphEdge } from '../types/graph'
import { api } from '../services/api'

interface GraphInfo {
  id: string
  name: string
  ring_id: string
  created_at: string
  updated_at: string
}

interface GraphState {
  nodes: GraphNode[]
  edges: GraphEdge[]
  graph_id: string | null
  graphs: GraphInfo[]
  loading: boolean
  selected_node_id: string | null
  selected_edge_id: string | null
  collapsed_nodes: Set<string>
  float_open: boolean
  float_position: { x: number; y: number }
  float_size: { w: number; h: number }
  setFloatOpen: (v: boolean) => void
  toggleFloat: () => void
  setFloatPosition: (pos: { x: number; y: number }) => void
  setFloatSize: (size: { w: number; h: number }) => void
  fetchGraph: (ringId: string, graphId?: string) => Promise<void>
  fetchGraphs: (ringId: string) => Promise<void>
  createGraph: (ringId: string, name: string) => Promise<void>
  deleteGraph: (ringId: string, graphId: string) => Promise<void>
  switchGraph: (ringId: string, graphId: string) => Promise<void>
  createNode: (ringId: string, label: string, nodeType?: string) => Promise<void>
  deleteNode: (ringId: string, nodeId: string) => Promise<void>
  createEdge: (ringId: string, sourceId: string, targetId: string, relation?: string, label?: string) => Promise<void>
  updateEdge: (ringId: string, edgeId: string, input: { relation?: string; label?: string }) => Promise<void>
  deleteEdge: (ringId: string, edgeId: string) => Promise<void>
  selectNode: (nodeId: string | null) => void
  selectEdge: (edgeId: string | null) => void
  toggleCollapse: (nodeId: string) => void
  isCollapsed: (nodeId: string) => boolean
  expandAll: (parentIds: string[]) => void
  collapseAll: (parentIds: string[]) => void
  createNodesFromExtraction: (
    ringId: string,
    concepts: { label: string; node_type: string; tags: string[] }[],
    relations: { from: string; to: string; relation: string }[],
    targetGraphId?: string | null,
  ) => Promise<{ createdNodes: number; createdEdges: number }>
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
  markdown_path: string | null
  metadata: string
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

function safeParseJSON(val: unknown): unknown {
  if (typeof val === 'string') {
    if (val.trim() === '') return undefined
    try { return JSON.parse(val) } catch { return undefined }
  }
  return val
}

function ensureArray(val: unknown): string[] {
  const parsed = safeParseJSON(val)
  if (Array.isArray(parsed)) return parsed
  return []
}

function ensureObject(val: unknown): Record<string, unknown> {
  const parsed = safeParseJSON(val)
  if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) return parsed as Record<string, unknown>
  return {}
}

function toGraphNode(r: NodeResponse): GraphNode {
  return {
    id: r.id,
    label: r.label ?? '',
    parent_id: r.parent_id ?? null,
    markdown_path: r.markdown_path ?? '',
    node_type: r.node_type as GraphNode['node_type'],
    tags: ensureArray(r.tags),
    metadata: ensureObject(r.metadata),
    created_at: r.created_at ?? '',
    updated_at: r.updated_at ?? '',
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
  graphs: [],
  loading: false,
  selected_node_id: null,
  selected_edge_id: null,
  collapsed_nodes: new Set<string>(),
  float_open: false,
  float_position: typeof window !== 'undefined'
    ? { x: Math.round(window.innerWidth * 0.05), y: Math.round(window.innerHeight * 0.05) }
    : { x: 100, y: 50 },
  float_size: typeof window !== 'undefined'
    ? { w: Math.round(window.innerWidth * 0.6), h: Math.round(window.innerHeight * 0.75) }
    : { w: 800, h: 600 },
  setFloatOpen: (v) => set({ float_open: v }),
  toggleFloat: () => set((s) => ({ float_open: !s.float_open })),
  setFloatPosition: (pos) => set({ float_position: pos }),
  setFloatSize: (size) => set({ float_size: size }),

  fetchGraphs: async (ringId: string) => {
    try {
      const res = await api.get<{ graphs: GraphInfo[] }>(`/rings/${ringId}/graphs`)
      set({ graphs: res.graphs })
    } catch (e) {
      console.error('fetchGraphs error:', e)
    }
  },

  createGraph: async (ringId, name) => {
    const res = await api.post<GraphInfo>(`/rings/${ringId}/graphs`, { name })
    set((s) => ({ graphs: [...s.graphs, res] }))
  },

  deleteGraph: async (ringId, graphId) => {
    await api.delete(`/rings/${ringId}/graphs/${graphId}`)
    const { graphs, graph_id } = get()
    const remaining = graphs.filter((g) => g.id !== graphId)
    set({ graphs: remaining })
    if (graph_id === graphId && remaining.length > 0) {
      get().switchGraph(ringId, remaining[0].id)
    }
  },

  switchGraph: async (ringId, graphId) => {
    set({ loading: true })
    try {
      const res = await api.get<GraphResponse>(`/rings/${ringId}/graph?graph_id=${graphId}`)
      const rawNodes = (res.nodes ?? []) as unknown as NodeResponse[]
      const rawEdges = (res.edges ?? []) as unknown as EdgeResponse[]
      set({
        graph_id: res.id,
        nodes: rawNodes.map(toGraphNode),
        edges: rawEdges.map(toGraphEdge),
        loading: false,
        collapsed_nodes: new Set(),
        selected_node_id: null,
        selected_edge_id: null,
      })
    } catch (e) {
      console.error('switchGraph error:', e)
      set({ loading: false })
    }
  },

  fetchGraph: async (ringId: string, graphId?: string) => {
    set({ loading: true })
    try {
      await get().fetchGraphs(ringId)
      const path = graphId
        ? `/rings/${ringId}/graph?graph_id=${graphId}`
        : `/rings/${ringId}/graph`
      const res = await api.get<GraphResponse>(path)
      const rawNodes = (res.nodes ?? []) as unknown as NodeResponse[]
      const rawEdges = (res.edges ?? []) as unknown as EdgeResponse[]
      set({
        graph_id: res.id,
        nodes: rawNodes.map(toGraphNode),
        edges: rawEdges.map(toGraphEdge),
        loading: false,
        selected_edge_id: null,
      })
    } catch (e) {
      console.error('fetchGraph error:', e)
      set({ loading: false })
    }
  },

  createNode: async (ringId, label, nodeType) => {
    const { graph_id } = get()
    const res = await api.post<NodeResponse>(`/rings/${ringId}/graph`, {
      label,
      node_type: nodeType ?? 'topic',
      graph_id: graph_id ?? undefined,
    })
    set((s) => ({ nodes: [...s.nodes, toGraphNode(res)] }))
  },

  deleteNode: async (ringId, nodeId) => {
    await api.delete(`/rings/${ringId}/graph/nodes/${nodeId}`)
    set((s) => {
      const newCollapsed = new Set(s.collapsed_nodes)
      newCollapsed.delete(nodeId)
      return {
        nodes: s.nodes.filter((n) => n.id !== nodeId),
        edges: s.edges.filter((e) => e.source_id !== nodeId && e.target_id !== nodeId),
        selected_node_id: s.selected_node_id === nodeId ? null : s.selected_node_id,
        collapsed_nodes: newCollapsed,
      }
    })
  },

  createEdge: async (ringId, sourceId, targetId, relation, label) => {
    const res = await api.post<EdgeResponse>(`/rings/${ringId}/graph/edges`, {
      graph_id: get().graph_id ?? undefined,
      source_id: sourceId,
      target_id: targetId,
      relation: relation ?? 'related_to',
      label: label ?? '',
    })
    set((s) => ({ edges: [...s.edges, toGraphEdge(res)] }))
  },

  updateEdge: async (ringId, edgeId, input) => {
    const res = await api.put<EdgeResponse>(`/rings/${ringId}/graph/edges/${edgeId}`, input)
    set((s) => ({
      edges: s.edges.map((edge) => (edge.id === edgeId ? toGraphEdge(res) : edge)),
      selected_edge_id: edgeId,
    }))
  },

  deleteEdge: async (ringId, edgeId) => {
    await api.delete(`/rings/${ringId}/graph/edges/${edgeId}`)
    set((s) => ({
      edges: s.edges.filter((e) => e.id !== edgeId),
      selected_edge_id: s.selected_edge_id === edgeId ? null : s.selected_edge_id,
    }))
  },

  selectNode: (nodeId) => set({ selected_node_id: nodeId, selected_edge_id: null }),

  selectEdge: (edgeId) => set({ selected_edge_id: edgeId, selected_node_id: null }),

  toggleCollapse: (nodeId: string) => {
    const { collapsed_nodes } = get()
    const newCollapsed = new Set(collapsed_nodes)
    if (newCollapsed.has(nodeId)) {
      newCollapsed.delete(nodeId)
    } else {
      newCollapsed.add(nodeId)
    }
    set({ collapsed_nodes: newCollapsed })
  },

  isCollapsed: (nodeId: string) => {
    return get().collapsed_nodes.has(nodeId)
  },

  expandAll: (parentIds: string[]) => {
    const newCollapsed = new Set(get().collapsed_nodes)
    for (const id of parentIds) newCollapsed.delete(id)
    set({ collapsed_nodes: newCollapsed })
  },

  collapseAll: (parentIds: string[]) => {
    const newCollapsed = new Set(get().collapsed_nodes)
    for (const id of parentIds) newCollapsed.add(id)
    set({ collapsed_nodes: newCollapsed })
  },

  createNodesFromExtraction: async (ringId, concepts, relations, targetGraphId) => {
    await get().fetchGraphs(ringId)
    let { graph_id } = get()
    const { graphs } = get()
    let activeGraphId =
      targetGraphId && graphs.some((graph) => graph.id === targetGraphId)
        ? targetGraphId
        : graph_id

    if (!activeGraphId) {
      await get().fetchGraph(ringId)
      graph_id = get().graph_id
      activeGraphId = graph_id
    }

    if (activeGraphId && get().graph_id !== activeGraphId) {
      await get().switchGraph(ringId, activeGraphId)
    }

    const { nodes, edges } = get()
    const existingLabels = new Map<string, string>()
    for (const node of nodes) {
      existingLabels.set(node.label.trim().toLowerCase(), node.id)
    }

    if (!activeGraphId) {
      return { createdNodes: 0, createdEdges: 0 }
    }

    const labelToId = new Map<string, string>()
    let createdNodes = 0
    let createdEdges = 0

    for (const [label, id] of existingLabels.entries()) {
      labelToId.set(label, id)
    }

    for (const concept of concepts) {
      const normalized = concept.label.trim().toLowerCase()
      if (!normalized) continue
      if (labelToId.has(normalized)) continue
      try {
        const res = await api.post<{ id: string }>(`/rings/${ringId}/graph`, {
          label: concept.label,
          graph_id: activeGraphId,
          node_type: concept.node_type,
          tags: concept.tags,
        })
        labelToId.set(normalized, res.id)
        createdNodes += 1
      } catch (e) {
        console.error('createNode error:', e)
      }
    }

    const existingEdgeKeys = new Set(
      edges.map((edge) => `${edge.source_id}|${edge.target_id}|${edge.relation}`),
    )

    for (const rel of relations) {
      const sourceId = labelToId.get(rel.from.trim().toLowerCase())
      const targetId = labelToId.get(rel.to.trim().toLowerCase())
      if (sourceId && targetId) {
        const edgeKey = `${sourceId}|${targetId}|${rel.relation}`
        if (existingEdgeKeys.has(edgeKey)) continue
        try {
          await api.post(`/rings/${ringId}/graph/edges`, {
            graph_id: activeGraphId,
            source_id: sourceId,
            target_id: targetId,
            relation: rel.relation,
          })
          existingEdgeKeys.add(edgeKey)
          createdEdges += 1
        } catch (e) {
          console.error('createEdge error:', e)
        }
      }
    }

    await get().fetchGraph(ringId, activeGraphId)
    return { createdNodes, createdEdges }
  },
}))
