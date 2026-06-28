import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('../../services/api', () => ({
  api: {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
  },
}))

import { api } from '../../services/api'
import { useGraphStore } from '../../stores/graph-store'

describe('graphStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useGraphStore.setState({
      nodes: [],
      edges: [],
      graph_id: null,
      graphs: [],
      loading: false,
      selected_node_id: null,
      selected_edge_id: null,
      collapsed_nodes: new Set<string>(),
    })
  })

  it('creates extracted nodes and edges in the selected graph without duplicates', async () => {
    const graphInfo = {
      id: 'graph-2',
      name: 'Secondary',
      ring_id: 'ring-1',
      created_at: '2026-05-31T00:00:00Z',
      updated_at: '2026-05-31T00:00:00Z',
    }

    vi.mocked(api.get)
      .mockResolvedValueOnce({ graphs: [graphInfo] })
      .mockResolvedValueOnce({
        id: graphInfo.id,
        name: graphInfo.name,
        ring_id: graphInfo.ring_id,
        nodes: [
          {
            id: 'node-a',
            graph_id: graphInfo.id,
            ring_id: graphInfo.ring_id,
            label: 'API Gateway',
            parent_id: null,
            node_type: 'topic',
            content: '',
            tags: '[]',
            markdown_path: null,
            metadata: '{}',
            created_at: '2026-05-31T00:00:00Z',
            updated_at: '2026-05-31T00:00:00Z',
          },
        ],
        edges: [],
      })
      .mockResolvedValueOnce({ graphs: [graphInfo] })
      .mockResolvedValueOnce({
        id: graphInfo.id,
        name: graphInfo.name,
        ring_id: graphInfo.ring_id,
        nodes: [],
        edges: [],
      })

    vi.mocked(api.post)
      .mockResolvedValueOnce({ id: 'node-b' })
      .mockResolvedValueOnce({ id: 'edge-ab' })

    const result = await useGraphStore.getState().createNodesFromExtraction(
      'ring-1',
      [
        { label: 'API Gateway', node_type: 'topic', tags: ['existing'] },
        { label: 'Order Service', node_type: 'topic', tags: ['service'] },
        { label: 'Order Service', node_type: 'topic', tags: ['service'] },
      ],
      [
        { from: 'API Gateway', to: 'Order Service', relation: 'related_to' },
        { from: 'API Gateway', to: 'Order Service', relation: 'related_to' },
      ],
      'graph-2',
    )

    expect(result).toEqual({ createdNodes: 1, createdEdges: 1 })
    expect(vi.mocked(api.post)).toHaveBeenCalledTimes(2)
    expect(vi.mocked(api.post)).toHaveBeenNthCalledWith(
      1,
      '/rings/ring-1/graph',
      expect.objectContaining({
        graph_id: 'graph-2',
        label: 'Order Service',
      }),
    )
    expect(vi.mocked(api.post)).toHaveBeenNthCalledWith(
      2,
      '/rings/ring-1/graph/edges',
      expect.objectContaining({
        graph_id: 'graph-2',
        source_id: 'node-a',
        target_id: 'node-b',
        relation: 'related_to',
      }),
    )
    expect(useGraphStore.getState().graph_id).toBe('graph-2')
  })

  it('updates an existing edge in place', async () => {
    useGraphStore.setState({
      edges: [
        {
          id: 'edge-1',
          source_id: 'node-a',
          target_id: 'node-b',
          relation: 'related_to',
          label: '',
          created_at: '2026-05-31T00:00:00Z',
        },
      ],
      selected_edge_id: null,
    })

    vi.mocked(api.put).mockResolvedValueOnce({
      id: 'edge-1',
      graph_id: 'graph-1',
      ring_id: 'ring-1',
      source_id: 'node-a',
      target_id: 'node-b',
      relation: 'depends_on',
      label: 'depends on',
      created_at: '2026-05-31T00:00:00Z',
    })

    await useGraphStore.getState().updateEdge('ring-1', 'edge-1', {
      relation: 'depends_on',
      label: 'depends on',
    })

    expect(vi.mocked(api.put)).toHaveBeenCalledWith(
      '/rings/ring-1/graph/edges/edge-1',
      { relation: 'depends_on', label: 'depends on' },
    )
    expect(useGraphStore.getState().edges[0].relation).toBe('depends_on')
    expect(useGraphStore.getState().edges[0].label).toBe('depends on')
    expect(useGraphStore.getState().selected_edge_id).toBe('edge-1')
  })
})
