import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { BlueprintTemplate } from '../../types'

vi.mock('../../api/client', () => ({
  list_blueprint_templates: vi.fn().mockResolvedValue([
    {
      id: 't1',
      name: 'Team Wiki',
      description: 'A wiki blueprint for teams',
      graphs: JSON.stringify([{ name: 'docs', graph_type: 'tree', categories: ['documentation'] }]),
      is_system: true,
      created_by: null,
      created_at: '2026-04-01T00:00:00Z',
    },
  ] as BlueprintTemplate[]),
  blueprint_preview: vi.fn().mockResolvedValue({
    graphs: [{
      name: 'docs',
      nodes: [{ id: 'n1', label: 'docs', node_type: 'topic' }],
      edges: [],
    }],
  }),
  blueprint_confirm: vi.fn().mockResolvedValue({ blueprint_id: 'bp-1', graphs: [{ id: 'g1', name: 'docs', graph_type: 'tree' }], status: 'confirmed' }),
  blueprint_chat: vi.fn(),
}))

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useParams: () => ({ ringId: 'ring-1' }),
  }
})

import { BlueprintWizard } from './BlueprintWizard'

describe('BlueprintWizard', () => {
  it('renders template cards when available', async () => {
    render(<BlueprintWizard />)
    expect(await screen.findByText('Team Wiki')).toBeInTheDocument()
    expect(screen.getByText('A wiki blueprint for teams')).toBeInTheDocument()
  })

  it('renders template and custom tabs', () => {
    render(<BlueprintWizard />)
    expect(screen.getByText('模板')).toBeInTheDocument()
    expect(screen.getByText('自定义')).toBeInTheDocument()
  })

  it('switches to custom tab', async () => {
    const user = (await import('@testing-library/user-event')).default.setup()
    render(<BlueprintWizard />)
    await user.click(screen.getByText('自定义'))
    expect(screen.getByPlaceholderText('Type a message...')).toBeInTheDocument()
  })
})
