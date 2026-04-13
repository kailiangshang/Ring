import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { NodePicker } from './NodePicker'
import type { GraphNode } from '../../types'

const mock_nodes: GraphNode[] = [
  { id: 'n1', label: 'Root', node_type: 'category', parent_id: null, description: null, graph_id: 'g1', markdown_path: null, created_at: '', updated_at: '' },
  { id: 'n2', label: 'Child', node_type: 'document', parent_id: 'n1', description: null, graph_id: 'g1', markdown_path: null, created_at: '', updated_at: '' },
]

describe('NodePicker', () => {
  it('renders all nodes', () => {
    render(<NodePicker nodes={mock_nodes} selected="" on_select={() => {}} multiple={false} />)
    expect(screen.getByText('Root')).toBeTruthy()
    expect(screen.getByText('Child')).toBeTruthy()
  })

  it('calls on_select with single id in single mode', () => {
    const on_select = vi.fn()
    render(<NodePicker nodes={mock_nodes} selected="" on_select={on_select} multiple={false} />)
    fireEvent.click(screen.getByText('Root'))
    expect(on_select).toHaveBeenCalledWith('n1')
  })

  it('toggles selection in multiple mode', () => {
    const on_select = vi.fn()
    render(<NodePicker nodes={mock_nodes} selected={[]} on_select={on_select} multiple={true} />)
    fireEvent.click(screen.getByText('Root'))
    expect(on_select).toHaveBeenCalledWith(['n1'])
  })

  it('removes from selection in multiple mode', () => {
    const on_select = vi.fn()
    render(<NodePicker nodes={mock_nodes} selected={['n1']} on_select={on_select} multiple={true} />)
    fireEvent.click(screen.getByText('Root'))
    expect(on_select).toHaveBeenCalledWith([])
  })
})
