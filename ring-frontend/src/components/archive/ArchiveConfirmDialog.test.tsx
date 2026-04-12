import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ArchiveConfirmDialog } from './ArchiveConfirmDialog'
import type { GraphNode } from '../../types'

const mock_nodes: GraphNode[] = [
  { id: 'n1', label: '根节点', node_type: 'category', parent_id: null, description: null, graph_id: 'g1', markdown_path: null, created_at: '', updated_at: '' },
  { id: 'n2', label: '子节点', node_type: 'document', parent_id: 'n1', description: null, graph_id: 'g1', markdown_path: null, created_at: '', updated_at: '' },
]

describe('ArchiveConfirmDialog', () => {
  it('renders suggested title and parent', () => {
    render(
      <ArchiveConfirmDialog
        open={true}
        on_close={() => {}}
        suggested_title="会议纪要"
        suggested_parent={{ id: 'n1', label: '根节点' }}
        nodes={mock_nodes}
        on_confirm={vi.fn()}
      />,
    )
    expect(screen.getByText('会议纪要')).toBeInTheDocument()
    expect(screen.getByText('根节点')).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: '确认归档' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '确认归档' })).toBeInTheDocument()
  })

  it('calls on_confirm when confirm button clicked', async () => {
    const on_confirm = vi.fn().mockResolvedValue(undefined)
    render(
      <ArchiveConfirmDialog
        open={true}
        on_close={() => {}}
        nodes={mock_nodes}
        on_confirm={on_confirm}
      />,
    )
    await fireEvent.click(screen.getByRole('button', { name: '确认归档' }))
    expect(on_confirm).toHaveBeenCalledWith(undefined)
  })

  it('shows node selector when change button clicked', () => {
    render(
      <ArchiveConfirmDialog
        open={true}
        on_close={() => {}}
        nodes={mock_nodes}
        on_confirm={vi.fn()}
      />,
    )
    fireEvent.click(screen.getByText('更改位置'))
    expect(screen.getByTestId('node-tree')).toBeInTheDocument()
    expect(screen.getByText('根节点')).toBeInTheDocument()
  })
})
