import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useChatStore } from './chatStore'

vi.mock('../api/client', () => ({
  archive_content: vi.fn().mockResolvedValue({
    archive_id: 'arch-test-1',
    markdown_path: '.ring/docs/test.md',
    git_status: 'pending',
    pr_url: null,
    queue_position: 1,
  }),
  confirm_archive: vi.fn().mockResolvedValue(undefined),
  create_conversation: vi.fn().mockResolvedValue({ id: 'conv-test', ring_id: 'ring-1', title: 'Test', mode: 'ring_group', context_mode: 'storage', token_count: 0, token_limit: 8000, auto_compact: false, summary: null, compacted_at: null, created_by: 'user-1', created_at: '', updated_at: '' }),
  get_messages: vi.fn().mockResolvedValue([]),
  send_message: vi.fn().mockResolvedValue(new Response(null, { status: 200 })),
  list_conversations: vi.fn().mockResolvedValue([]),
}))

beforeEach(() => {
  useChatStore.setState({
    messages: [],
    tool_events: [],
    is_streaming: false,
    current_conversation_id: 'conv-1',
    error: null,
    archive_pending: null,
  })
})

describe('chatStore archive actions', () => {
  it('trigger_archive collects un-archived message IDs and calls API', async () => {
    useChatStore.setState({
      messages: [
        { id: 'm1', conversation_id: 'conv-1', role: 'user', content: 'first', sender_id: '', tool_calls: null, archived: false, created_at: '' },
        { id: 'm2', conversation_id: 'conv-1', role: 'assistant', content: 'reply', sender_id: null, tool_calls: null, archived: false, created_at: '' },
        { id: 'm3', conversation_id: 'conv-1', role: 'user', content: 'second message that is longer', sender_id: '', tool_calls: null, archived: true, created_at: '' },
      ],
    })

    await useChatStore.getState().trigger_archive('ring-1', 'graph-1')

    const api = await import('../api/client')
    expect(api.archive_content).toHaveBeenCalledWith('ring-1', expect.objectContaining({
      message_ids: ['m1', 'm2'],
      conversation_id: 'conv-1',
      graph_id: 'graph-1',
      label: 'first',
    }))

    expect(useChatStore.getState().archive_pending).toEqual(expect.objectContaining({
      archive_id: 'arch-test-1',
      label: 'first',
    }))
  })

  it('dismiss_suggestion removes an archive_suggestion event', () => {
    useChatStore.setState({
      tool_events: [
        { id: 'evt-1', type: 'tool_call' as const, tool_name: 'search', input: null, timestamp: 1 },
        { id: 'evt-2', type: 'archive_suggestion' as const, data: { reason: 'test' }, timestamp: 2 },
        { id: 'evt-3', type: 'archive_suggestion' as const, data: { reason: 'test2' }, timestamp: 3 },
      ],
    })

    useChatStore.getState().dismiss_suggestion('evt-2')

    const events = useChatStore.getState().tool_events
    expect(events).toHaveLength(2)
    expect(events.find((e) => e.id === 'evt-2')).toBeUndefined()
  })

  it('clear_archive_pending resets archive_pending to null', async () => {
    useChatStore.setState({
      messages: [
        { id: 'm1', conversation_id: 'conv-1', role: 'user', content: 'hello', sender_id: '', tool_calls: null, archived: false, created_at: '' },
      ],
    })
    await useChatStore.getState().trigger_archive('ring-1', 'graph-1')
    expect(useChatStore.getState().archive_pending).not.toBeNull()

    useChatStore.getState().clear_archive_pending()
    expect(useChatStore.getState().archive_pending).toBeNull()
  })

  it('trigger_archive limits to last 5 un-archived messages', async () => {
    const msgs = Array.from({ length: 8 }, (_, i) => ({
      id: `m${i}`,
      conversation_id: 'conv-1',
      role: 'user' as const,
      content: `message ${i}`,
      sender_id: '',
      tool_calls: null,
      archived: false,
      created_at: '',
    }))
    useChatStore.setState({ messages: msgs })

    await useChatStore.getState().trigger_archive('ring-1', 'graph-1')

    const api = await import('../api/client')
    expect(api.archive_content).toHaveBeenCalledWith('ring-1', expect.objectContaining({
      message_ids: ['m3', 'm4', 'm5', 'm6', 'm7'],
    }))
  })
})
