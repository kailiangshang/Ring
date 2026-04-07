import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { SuperRingChat } from './SuperRingChat'

vi.mock('../../api/client', () => ({
  super_ring_chat: vi.fn(),
}))

describe('SuperRingChat', () => {
  it('renders chat input', () => {
    render(<SuperRingChat />)
    expect(screen.getByPlaceholderText('Type a message...')).toBeInTheDocument()
    expect(screen.getByText('Ring Super Chat')).toBeInTheDocument()
  })

  it('renders send button', () => {
    render(<SuperRingChat />)
    expect(screen.getByText('Send')).toBeInTheDocument()
  })
})
