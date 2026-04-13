import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { GraphPreviewCard } from './GraphPreviewCard'
import type { GraphPreview } from '../../types'

const mock_graph: GraphPreview = {
  name: '知识图谱',
  nodes: [
    { id: 'n1', label: '根节点', node_type: 'category' },
    { id: 'n2', label: '子节点', node_type: 'document' },
    { id: 'n3', label: '叶节点', node_type: 'topic' },
  ],
  edges: [
    { source_id: 'n1', target_id: 'n2', relation: 'contains' },
    { source_id: 'n2', target_id: 'n3', relation: 'references' },
  ],
}

describe('GraphPreviewCard', () => {
  it('renders graph name and stats', () => {
    render(<GraphPreviewCard graph={mock_graph} />)
    expect(screen.getByText('知识图谱')).toBeTruthy()
    expect(screen.getByText('3 节点')).toBeTruthy()
    expect(screen.getByText('2 边')).toBeTruthy()
  })

  it('renders node labels', () => {
    render(<GraphPreviewCard graph={mock_graph} />)
    expect(screen.getByText('根节点')).toBeTruthy()
    expect(screen.getByText('子节点')).toBeTruthy()
    expect(screen.getByText('叶节点')).toBeTruthy()
  })

  it('renders type and relation tags', () => {
    render(<GraphPreviewCard graph={mock_graph} />)
    expect(screen.getByText('category')).toBeTruthy()
    expect(screen.getByText('document')).toBeTruthy()
    expect(screen.getByText('contains')).toBeTruthy()
    expect(screen.getByText('references')).toBeTruthy()
  })

  it('renders edit button when on_edit provided', () => {
    const on_edit = vi.fn()
    render(<GraphPreviewCard graph={mock_graph} on_edit={on_edit} />)
    const btn = screen.getByText('✏️ 编辑')
    fireEvent.click(btn)
    expect(on_edit).toHaveBeenCalled()
  })

  it('shows overflow count when many nodes', () => {
    const big_graph: GraphPreview = {
      name: 'Big',
      nodes: Array.from({ length: 12 }, (_, i) => ({ id: `n${i}`, label: `Node ${i}`, node_type: 'topic' })),
      edges: [],
    }
    render(<GraphPreviewCard graph={big_graph} />)
    expect(screen.getByText('+4')).toBeTruthy()
  })
})
