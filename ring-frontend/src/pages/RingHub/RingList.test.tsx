import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { RingList } from './RingList'
import type { Ring } from '../../types'

const mock_rings: Ring[] = [
  {
    id: '1',
    name: 'Ring A',
    description: 'Ring A description',
    creator_id: 'user-1',
    gitlab_repo: 'git@gitlab.corp:user/a.git',
    local_path: '/home/.ring/repos/a',
    next_token_id: 2,
    status: 'active',
    created_at: '2026-04-05T10:00:00Z',
    updated_at: '2026-04-05T10:00:00Z',
  },
  {
    id: '2',
    name: 'Ring B',
    description: 'Ring B description',
    creator_id: 'user-2',
    gitlab_repo: 'git@gitlab.corp:user/b.git',
    local_path: '/home/.ring/repos/b',
    next_token_id: 1,
    status: 'active',
    created_at: '2026-04-04T08:00:00Z',
    updated_at: '2026-04-04T08:00:00Z',
  },
]

describe('RingList', () => {
  it('renders ring cards', () => {
    render(<RingList rings={mock_rings} on_select={vi.fn()} />)
    expect(screen.getByText('Ring A')).toBeInTheDocument()
    expect(screen.getByText('Ring B')).toBeInTheDocument()
  })

  it('shows empty state when no rings', () => {
    render(<RingList rings={[]} on_select={vi.fn()} />)
    expect(screen.getByText(/No rings yet/)).toBeInTheDocument()
  })

  it('calls on_select when a ring is clicked', async () => {
    const on_select = vi.fn()
    const user = userEvent.setup()
    render(<RingList rings={mock_rings} on_select={on_select} />)
    await user.click(screen.getByText('Ring A'))
    expect(on_select).toHaveBeenCalledWith('1')
  })
})
