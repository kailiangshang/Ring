import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'

vi.mock('../../stores/graphStore', () => ({
  useGraphStore: () => ({
    graphs: ['graph-1'],
    current_graph_id: 'graph-1',
    nodes: [
      { id: 'n1', label: 'Root', node_type: 'category', parent_id: null, description: null, graph_id: 'graph-1', markdown_path: null, created_at: '', updated_at: '' },
      { id: 'n2', label: 'Child', node_type: 'concept', parent_id: 'n1', description: null, graph_id: 'graph-1', markdown_path: null, created_at: '', updated_at: '' },
    ],
    edges: [],
    selected_node_id: null,
    selected_node_content: null,
    search_results: [],
    loading: false,
    error: null,
    load_graphs: vi.fn(),
    select_graph: vi.fn(),
    select_node: vi.fn(),
    create_node: vi.fn(),
    delete_node: vi.fn(),
    create_edge: vi.fn(),
    delete_edge: vi.fn(),
    search_nodes: vi.fn(),
    reset: vi.fn(),
  }),
}))

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useParams: () => ({ ringId: 'ring-1' }),
  }
})

import { GraphView } from './GraphView'

describe('GraphView', () => {
  it('renders graph container', () => {
    render(<GraphView />)
    expect(screen.getByTestId('graph-container')).toBeInTheDocument()
  })

  it('renders node tree with nodes', () => {
    render(<GraphView />)
    expect(screen.getAllByText('Root').length).toBeGreaterThanOrEqual(1)
    expect(screen.getAllByText('Child').length).toBeGreaterThanOrEqual(1)
  })

  it('renders add node button', () => {
    render(<GraphView />)
    expect(screen.getByText('+ Node')).toBeInTheDocument()
  })
})
