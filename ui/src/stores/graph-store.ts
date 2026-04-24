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
  collapsed_nodes: Set<string>
  fetchGraph: (ringId: string, graphId?: string) => Promise<void>
  fetchGraphs: (ringId: string) => Promise<void>
  createGraph: (ringId: string, name: string) => Promise<void>
  deleteGraph: (ringId: string, graphId: string) => Promise<void>
  switchGraph: (ringId: string, graphId: string) => Promise<void>
  createNode: (ringId: string, label: string, nodeType?: string) => Promise<void>
  deleteNode: (ringId: string, nodeId: string) => Promise<void>
  createEdge: (ringId: string, sourceId: string, targetId: string, relation?: string) => Promise<void>
  deleteEdge: (ringId: string, edgeId: string) => Promise<void>
  selectNode: (nodeId: string | null) => void
  toggleCollapse: (nodeId: string) => void
  isCollapsed: (nodeId: string) => boolean
  createNodesFromExtraction: (
    ringId: string,
    concepts: { label: string; node_type: string; tags: string[] }[],
    relations: { from: string; to: string; relation: string }[],
  ) => Promise<void>
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

function toGraphNode(r: NodeResponse): GraphNode {
  return {
    id: r.id,
    label: r.label,
    parent_id: r.parent_id,
    markdown_path: r.markdown_path ?? '',
    node_type: r.node_type as GraphNode['node_type'],
    tags: typeof r.tags === 'string' ? JSON.parse(r.tags) : r.tags,
    metadata: typeof r.metadata === 'string' ? JSON.parse(r.metadata) : r.metadata,
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
  graphs: [],
  loading: false,
  selected_node_id: null,
  collapsed_nodes: new Set<string>(),

  fetchGraphs: async (ringId: string) => {
    try {
      const res = await api.get<{ graphs: GraphInfo[] }>(`/rings/${ringId}/graphs`)
      set({ graphs: res.graphs })
    } catch {}
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
      set({
        graph_id: res.id,
        nodes: (res.nodes as unknown as NodeResponse[]).map(toGraphNode),
        edges: (res.edges as unknown as EdgeResponse[]).map(toGraphEdge),
        loading: false,
        collapsed_nodes: new Set(),
        selected_node_id: null,
      })
    } catch {
      set({ loading: false })
    }
  },

  fetchGraph: async (ringId: string) => {
    set({ loading: true })
    try {
      await get().fetchGraphs(ringId)
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

  createNodesFromExtraction: async (ringId, concepts, relations) => {
    await get().fetchGraphs(ringId)
    const { graph_id } = get()
    if (!graph_id) return

    const labelToId = new Map<string, string>()

    for (const concept of concepts) {
      try {
        const res = await api.post<{ id: string }>(`/rings/${ringId}/graph`, {
          label: concept.label,
          node_type: concept.node_type,
          tags: concept.tags,
        })
        labelToId.set(concept.label, res.id)
      } catch {}
    }

    for (const rel of relations) {
      const sourceId = labelToId.get(rel.from)
      const targetId = labelToId.get(rel.to)
      if (sourceId && targetId) {
        try {
          await api.post(`/rings/${ringId}/graph/edges`, {
            source_id: sourceId,
            target_id: targetId,
            relation: rel.relation,
          })
        } catch {}
      }
    }

    get().fetchGraph(ringId)
  },
}))
