import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ExportPanel, EXPORT_TYPES } from './ExportPanel'

vi.mock('../../api/client', () => ({
  list_graphs: vi.fn().mockResolvedValue(['graph-1']),
  get_graph: vi.fn().mockResolvedValue({
    graph_id: 'graph-1',
    nodes: [
      { id: 'n1', label: 'Node 1', node_type: 'category', parent_id: null, description: null, graph_id: 'graph-1', markdown_path: null, created_at: '', updated_at: '' },
      { id: 'n2', label: 'Node 2', node_type: 'document', parent_id: 'n1', description: null, graph_id: 'graph-1', markdown_path: null, created_at: '', updated_at: '' },
    ],
    edges: [],
  }),
  list_conversations: vi.fn().mockResolvedValue([
    { id: 'conv-1', ring_id: 'ring-1', title: 'Test Conv', mode: 'ring_group', context_mode: 'storage', token_count: 0, token_limit: 8000, auto_compact: false, summary: null, compacted_at: null, created_by: 'u1', created_at: '', updated_at: '' },
  ]),
  list_sessions: vi.fn().mockResolvedValue([
    { id: 'sess-1', title: 'Test Session', created_by: 'u1', member_count: 2, archive_enabled: true, status: 'active', created_at: '' },
  ]),
  export_graph_image: vi.fn().mockResolvedValue(undefined),
  export_graph_json: vi.fn().mockResolvedValue(undefined),
  export_markdown: vi.fn().mockResolvedValue(undefined),
  export_conversation: vi.fn().mockResolvedValue(undefined),
  export_report: vi.fn().mockResolvedValue(undefined),
  export_session: vi.fn().mockResolvedValue(undefined),
  export_backup: vi.fn().mockResolvedValue(undefined),
}))

describe('ExportPanel', () => {
  it('renders export type list with groups', () => {
    render(<ExportPanel ring_id="ring-1" on_close={() => {}} />)
    expect(screen.getByText('导出中心')).toBeTruthy()
    expect(screen.getByText('图谱图片')).toBeTruthy()
    expect(screen.getByText('整 Ring 备份')).toBeTruthy()
    expect(screen.getByText('选择左侧导出类型')).toBeTruthy()
  })

  it('shows config panel when type is selected', async () => {
    render(<ExportPanel ring_id="ring-1" on_close={() => {}} />)
    fireEvent.click(screen.getByText('图谱图片'))
    expect(screen.getByText('选择图谱')).toBeTruthy()
  })

  it('calls on_close when overlay is clicked', () => {
    const on_close = vi.fn()
    const { container } = render(<ExportPanel ring_id="ring-1" on_close={on_close} />)
    fireEvent.click(container.querySelector('.export-panel-overlay')!)
    expect(on_close).toHaveBeenCalled()
  })

  it('backup has export button immediately available', () => {
    render(<ExportPanel ring_id="ring-1" on_close={() => {}} />)
    fireEvent.click(screen.getByText('整 Ring 备份'))
    const btn = screen.getByText('导出')
    expect(btn).toBeTruthy()
  })

  it('exports all 7 types defined', () => {
    expect(EXPORT_TYPES).toHaveLength(7)
    const types = EXPORT_TYPES.map((t) => t.type)
    expect(types).toContain('graph_image')
    expect(types).toContain('graph_json')
    expect(types).toContain('markdown')
    expect(types).toContain('conversation')
    expect(types).toContain('session')
    expect(types).toContain('report')
    expect(types).toContain('backup')
  })
})
