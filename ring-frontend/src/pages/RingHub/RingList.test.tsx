import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { RingList } from './RingList'
import type { RingListItem } from '../../types'

const mock_rings: RingListItem[] = [
  {
    id: '1',
    name: 'Ring A',
    member_count: 5,
    graph_node_count: 10,
    last_activity_at: '2026-04-05T10:00:00Z',
    role: 'creator',
  },
  {
    id: '2',
    name: 'Ring B',
    member_count: 3,
    graph_node_count: 7,
    last_activity_at: '2026-04-04T08:00:00Z',
    role: 'member',
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
