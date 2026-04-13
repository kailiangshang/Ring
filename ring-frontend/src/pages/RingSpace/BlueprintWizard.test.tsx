import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import type { BlueprintTemplate } from '../../types'

const mock_navigate = vi.fn()

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
    useNavigate: () => mock_navigate,
  }
})

import { BlueprintWizard } from './BlueprintWizard'

describe('BlueprintWizard', () => {
  beforeEach(() => { mock_navigate.mockClear() })

  it('renders template cards when available', async () => {
    render(<BlueprintWizard />)
    expect(await screen.findByText('Team Wiki')).toBeTruthy()
    expect(screen.getByText('A wiki blueprint for teams')).toBeTruthy()
  })

  it('renders template and custom tabs', () => {
    render(<BlueprintWizard />)
    expect(screen.getByText('模板')).toBeTruthy()
    expect(screen.getByText('自定义')).toBeTruthy()
  })

  it('switches to custom tab', async () => {
    const user = (await import('@testing-library/user-event')).default.setup()
    render(<BlueprintWizard />)
    await user.click(screen.getByText('自定义'))
    expect(screen.getByText('从零开始构建蓝图')).toBeTruthy()
  })

  it('renders use and customize buttons per template', async () => {
    render(<BlueprintWizard />)
    expect(await screen.findByText('使用')).toBeTruthy()
    const customize_buttons = screen.getAllByText('自定义')
    expect(customize_buttons.length).toBeGreaterThanOrEqual(2)
  })
})
