import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ChatInput } from './ChatInput'

describe('ChatInput', () => {
  it('renders input and send button', () => {
    render(<ChatInput on_send={vi.fn()} />)
    expect(screen.getByPlaceholderText('Type a message...')).toBeInTheDocument()
    expect(screen.getByText('Send')).toBeInTheDocument()
  })

  it('calls on_send with text and clears input', async () => {
    const on_send = vi.fn()
    const user = userEvent.setup()
    render(<ChatInput on_send={on_send} />)

    const input = screen.getByPlaceholderText('Type a message...')
    await user.type(input, 'hello world')
    await user.click(screen.getByText('Send'))

    expect(on_send).toHaveBeenCalledWith('hello world')
    expect(input).toHaveValue('')
  })

  it('does not call on_send when input is empty', async () => {
    const on_send = vi.fn()
    const user = userEvent.setup()
    render(<ChatInput on_send={on_send} />)

    await user.click(screen.getByText('Send'))
    expect(on_send).not.toHaveBeenCalled()
  })

  it('disables input and button when disabled', () => {
    render(<ChatInput on_send={vi.fn()} disabled />)
    expect(screen.getByPlaceholderText('Type a message...')).toBeDisabled()
    expect(screen.getByText('Send')).toBeDisabled()
  })
})
