import { create } from 'zustand'
import * as api from '../api/client'
import type { GraphNode, GraphEdge, NodeContent, SearchResult } from '../types'

interface GraphState {
  graphs: string[]
  current_graph_id: string | null
  nodes: GraphNode[]
  edges: GraphEdge[]
  selected_node_id: string | null
  selected_node_content: NodeContent | null
  search_results: SearchResult[]
  loading: boolean
  error: string | null

  load_graphs: (ring_id: string) => Promise<void>
  select_graph: (ring_id: string, graph_id: string) => Promise<void>
  select_node: (ring_id: string, graph_id: string, node_id: string) => Promise<void>
  create_node: (ring_id: string, graph_id: string, req: { label: string; node_type: string; parent_id?: string }) => Promise<void>
  delete_node: (ring_id: string, graph_id: string, node_id: string) => Promise<void>
  create_edge: (ring_id: string, graph_id: string, req: { source_id: string; target_id: string; relation: string }) => Promise<void>
  delete_edge: (ring_id: string, graph_id: string, edge_id: string) => Promise<void>
  search_nodes: (ring_id: string, query: string) => Promise<void>
  reset: () => void
}

export const useGraphStore = create<GraphState>((set, get) => ({
  graphs: [],
  current_graph_id: null,
  nodes: [],
  edges: [],
  selected_node_id: null,
  selected_node_content: null,
  search_results: [],
  loading: false,
  error: null,

  load_graphs: async (ring_id) => {
    try {
      const graphs = await api.list_graphs(ring_id)
      set({ graphs })
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  select_graph: async (ring_id, graph_id) => {
    set({ loading: true, error: null })
    try {
      const detail = await api.get_graph(ring_id, graph_id)
      set({
        current_graph_id: graph_id,
        nodes: detail.nodes,
        edges: detail.edges,
        selected_node_id: null,
        selected_node_content: null,
        loading: false,
      })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  select_node: async (ring_id, graph_id, node_id) => {
    set({ selected_node_id: node_id })
    try {
      const content = await api.get_node_content(ring_id, graph_id, node_id)
      set({ selected_node_content: content })
    } catch {
      set({ selected_node_content: null })
    }
  },

  create_node: async (ring_id, graph_id, req) => {
    try {
      await api.create_node(ring_id, graph_id, req)
      await get().select_graph(ring_id, graph_id)
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  delete_node: async (ring_id, graph_id, node_id) => {
    try {
      await api.delete_node(ring_id, graph_id, node_id)
      if (get().selected_node_id === node_id) {
        set({ selected_node_id: null, selected_node_content: null })
      }
      await get().select_graph(ring_id, graph_id)
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  create_edge: async (ring_id, graph_id, req) => {
    try {
      await api.create_edge(ring_id, graph_id, req)
      await get().select_graph(ring_id, graph_id)
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  delete_edge: async (ring_id, graph_id, edge_id) => {
    try {
      await api.delete_edge(ring_id, graph_id, edge_id)
      await get().select_graph(ring_id, graph_id)
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  search_nodes: async (ring_id, query) => {
    try {
      const result = await api.search_nodes(ring_id, query)
      set({ search_results: result.results })
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  reset: () =>
    set({
      graphs: [],
      current_graph_id: null,
      nodes: [],
      edges: [],
      selected_node_id: null,
      selected_node_content: null,
      search_results: [],
      loading: false,
      error: null,
    }),
}))
