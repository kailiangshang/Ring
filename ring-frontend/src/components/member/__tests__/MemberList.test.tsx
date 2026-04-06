import { vi } from 'vitest'
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MemberList } from '../MemberList'

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useParams: () => ({ ringId: 'ring-1' }),
  }
})

vi.mock('../../../stores/memberStore', () => ({
  useMemberStore: () => ({
    members: [
      {
        id: 'm-1',
        ring_id: 'ring-1',
        user_id: 'u-1',
        token_id: 1,
        display_name: 'Alice',
        role: 'creator',
        joined_at: '2025-01-01T00:00:00Z',
      },
      {
        id: 'm-2',
        ring_id: 'ring-1',
        user_id: 'u-2',
        token_id: 2,
        display_name: 'Bob',
        role: 'member',
        joined_at: '2025-01-02T00:00:00Z',
      },
    ],
    loading: false,
    error: null,
    load_members: async () => {},
    generate_invite: async () => null,
    update_role: async () => {},
    remove_member: async () => {},
    clear_error: () => {},
  }),
}))

describe('MemberList', () => {
  it('renders member list with names and roles', () => {
    render(<MemberList />)
    expect(screen.getByText('Alice')).toBeInTheDocument()
    expect(screen.getByText('Bob')).toBeInTheDocument()
    expect(screen.getByText('creator')).toBeInTheDocument()
    expect(screen.getByText('member')).toBeInTheDocument()
  })

  it('renders generate invite button', () => {
    render(<MemberList />)
    expect(screen.getByText('Generate Invite')).toBeInTheDocument()
  })

  it('shows token IDs', () => {
    render(<MemberList />)
    expect(screen.getByText('#1')).toBeInTheDocument()
    expect(screen.getByText('#2')).toBeInTheDocument()
  })
})
