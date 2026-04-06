import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

vi.mock('../../stores/chatStore', () => ({
  useChatStore: () => ({
    messages: [
      {
        id: '1',
        conversation_id: 'c1',
        role: 'user',
        content: 'hello',
        sender_id: 'u1',
        created_at: '2026-04-06T00:00:00Z',
      },
      {
        id: '2',
        conversation_id: 'c1',
        role: 'assistant',
        content: 'hi there',
        sender_id: '',
        created_at: '2026-04-06T00:00:01Z',
      },
    ],
    is_streaming: false,
    error: null,
    current_conversation_id: 'c1',
    create_conversation: vi.fn(),
    load_history: vi.fn(),
    send_message: vi.fn(),
    reset: vi.fn(),
  }),
}))

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useParams: () => ({ ringId: 'ring-1' }),
  }
})

import { ChatView } from './ChatView'

describe('ChatView', () => {
  it('renders message history and input', () => {
    render(<ChatView />)
    expect(screen.getByText('hello')).toBeInTheDocument()
    expect(screen.getByText('hi there')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Type a message...')).toBeInTheDocument()
  })

  it('shows typing indicator when streaming', () => {
    vi.doMock('../../stores/chatStore', () => ({
      useChatStore: () => ({
        messages: [],
        is_streaming: true,
        error: null,
        current_conversation_id: 'c1',
        create_conversation: vi.fn(),
        load_history: vi.fn(),
        send_message: vi.fn(),
        reset: vi.fn(),
      }),
    }))
  })

  it('can type in input', async () => {
    const user = userEvent.setup()
    render(<ChatView />)
    const input = screen.getByPlaceholderText('Type a message...')
    await user.type(input, 'test message')
    expect(input).toHaveValue('test message')
  })
})
