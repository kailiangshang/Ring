import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { RingSuperChat } from './RingSuperChat'

vi.mock('../../api/client', () => ({
  ring_super_chat: vi.fn(),
}))

describe('RingSuperChat', () => {
  it('renders chat input', () => {
    render(<RingSuperChat />)
    expect(screen.getByPlaceholderText('Type a message...')).toBeInTheDocument()
    expect(screen.getByText('Ring Super')).toBeInTheDocument()
  })

  it('renders send button', () => {
    render(<RingSuperChat />)
    expect(screen.getByText('Send')).toBeInTheDocument()
  })
})
