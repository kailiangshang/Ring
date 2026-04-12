import { create } from 'zustand'
import * as api from '../api/client'
import { parseSseStream } from '../components/chat/SseParser'
import type { Message, SseEvent, ToolEvent } from '../types'

export interface ArchivePending {
  archive_id: string
  suggested_title?: string
  suggested_parent?: { id: string; label: string }
  message_ids: string[]
  conversation_id: string
  graph_id: string
  label: string
}

interface ChatState {
  messages: Message[]
  tool_events: ToolEvent[]
  is_streaming: boolean
  current_conversation_id: string | null
  error: string | null
  archive_pending: ArchivePending | null
  token_count: number
  token_limit: number
  context_mode: string
  auto_compact: boolean
  compacting: boolean

  create_conversation: (ring_id: string, title: string, context_mode?: string) => Promise<string>
  load_history: (ring_id: string, conv_id: string) => Promise<void>
  send_message: (ring_id: string, content: string, active_tools?: string[]) => Promise<void>
  trigger_archive: (ring_id: string, graph_id: string) => Promise<void>
  dismiss_suggestion: (event_id: string) => void
  clear_archive_pending: () => void
  load_token_stats: (ring_id: string) => Promise<void>
  trigger_compact: (ring_id: string) => Promise<void>
  toggle_auto_compact: (ring_id: string) => Promise<void>
  reset: () => void
}

let abort_controller: AbortController | null = null

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  tool_events: [],
  is_streaming: false,
  current_conversation_id: null,
  error: null,
  archive_pending: null,
  token_count: 0,
  token_limit: 100000,
  context_mode: 'storage',
  auto_compact: false,
  compacting: false,

  create_conversation: async (ring_id, title) => {
    if (abort_controller) { abort_controller.abort(); abort_controller = null }
    const conv = await api.create_conversation(ring_id, title)
    set({ current_conversation_id: conv.id, messages: [] })
    return conv.id
  },

  load_history: async (ring_id, conv_id) => {
    if (abort_controller) { abort_controller.abort(); abort_controller = null }
    set({ current_conversation_id: conv_id })
    const msgs = await api.get_messages(ring_id, conv_id)
    set({ messages: msgs })
    try {
      const stats = await api.get_token_stats(ring_id, conv_id)
      set({
        token_count: stats.token_count,
        token_limit: stats.token_limit,
        context_mode: stats.context_mode,
        auto_compact: stats.auto_compact,
      })
    } catch (_e) {}
  },

  send_message: async (ring_id, content, active_tools) => {
    const conv_id = get().current_conversation_id
    if (!conv_id) return

    const user_msg: Message = {
      id: `temp-${Date.now()}`,
      conversation_id: conv_id,
      role: 'user',
      content,
      sender_id: '',
      tool_calls: null,
      archived: false,
      created_at: new Date().toISOString(),
    }

    set((s) => ({
      messages: [...s.messages, user_msg],
      is_streaming: true,
      error: null,
    }))

    if (abort_controller) abort_controller.abort()
    abort_controller = new AbortController()

    try {
      const res = await api.send_message(ring_id, conv_id, content, active_tools, abort_controller!.signal)

      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `request failed: ${res.status}`)
      }

      const reader = res.body?.getReader()
      if (!reader) throw new Error('no response body')

      let assistant_content = ''
      const stream_id = `stream-${Date.now()}`

      for await (const event of parseSseStream(reader) as AsyncGenerator<SseEvent>) {
        if (event.type === 'text' && event.content) {
          assistant_content += event.content
          const assistant_msg: Message = {
            id: stream_id,
            conversation_id: conv_id,
            role: 'assistant',
            content: assistant_content,
            sender_id: null,
            tool_calls: null,
            archived: false,
            created_at: new Date().toISOString(),
          }
          set((s) => ({
            messages: [
              ...s.messages.filter((m) => m.id !== stream_id),
              assistant_msg,
            ],
          }))
        } else if (event.type === 'error') {
          throw new Error(event.message || 'stream error')
        } else if (event.type === 'tool_call') {
          const tool_event: ToolEvent = {
            id: `tool-${Date.now()}-${Math.random()}`,
            type: 'tool_call',
            tool_call_id: event.tool_call_id,
            tool_name: event.tool,
            input: event.input,
            timestamp: Date.now(),
          }
          set((s) => ({ tool_events: [...s.tool_events, tool_event] }))
        } else if (event.type === 'tool_result') {
          const tool_event: ToolEvent = {
            id: `tool-${Date.now()}-${Math.random()}`,
            type: 'tool_result',
            tool_call_id: event.tool_call_id,
            tool_name: event.tool,
            output: event.output,
            success: event.success,
            timestamp: Date.now(),
          }
          set((s) => ({ tool_events: [...s.tool_events, tool_event] }))
        } else if (event.type === 'archive_suggestion') {
          const tool_event: ToolEvent = {
            id: `tool-${Date.now()}-${Math.random()}`,
            type: 'archive_suggestion',
            data: event.data,
            timestamp: Date.now(),
          }
          set((s) => ({ tool_events: [...s.tool_events, tool_event] }))
        } else if (event.type === 'done') {
          const usage = event.token_usage
          if (usage) {
            set((s) => ({ token_count: s.token_count + usage.total_tokens }))
          }
        }
      }

      set({ is_streaming: false })
    } catch (e) {
      if ((e as Error).name === 'AbortError') return
      set({ is_streaming: false, error: (e as Error).message })
    }
  },

  reset: () => {
    if (abort_controller) { abort_controller.abort(); abort_controller = null }
    set({
      messages: [],
      tool_events: [],
      is_streaming: false,
      current_conversation_id: null,
      error: null,
      archive_pending: null,
      token_count: 0,
      token_limit: 100000,
      context_mode: 'storage',
      auto_compact: false,
      compacting: false,
    })
  },

  trigger_archive: async (ring_id, graph_id) => {
    const { messages, current_conversation_id } = get()
    if (!current_conversation_id) return

    const unarchived = messages.filter((m) => !m.archived)
    const last_five = unarchived.slice(-5)
    if (last_five.length === 0) return

    const last_user_msg = [...last_five].reverse().find((m) => m.role === 'user')
    const label = (last_user_msg?.content || 'Archive').slice(0, 30)

    try {
      const res = await api.archive_content(ring_id, {
        message_ids: last_five.map((m) => m.id),
        conversation_id: current_conversation_id,
        graph_id,
        label,
      })
      set({
        archive_pending: {
          archive_id: res.archive_id,
          message_ids: last_five.map((m) => m.id),
          conversation_id: current_conversation_id,
          graph_id,
          label,
        },
      })
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  dismiss_suggestion: (event_id) => {
    set((s) => ({ tool_events: s.tool_events.filter((e) => e.id !== event_id) }))
  },

  clear_archive_pending: () => set({ archive_pending: null }),

  load_token_stats: async (ring_id) => {
    const conv_id = get().current_conversation_id
    if (!conv_id) return
    try {
      const stats = await api.get_token_stats(ring_id, conv_id)
      set({
        token_count: stats.token_count,
        token_limit: stats.token_limit,
        context_mode: stats.context_mode,
        auto_compact: stats.auto_compact,
      })
    } catch (_e) {}
  },

  trigger_compact: async (ring_id) => {
    const conv_id = get().current_conversation_id
    if (!conv_id) return
    set({ compacting: true })
    try {
      const res = await api.compact_conversation(ring_id, conv_id)
      set({ token_count: res.token_count_after, compacting: false })
    } catch (e) {
      set({ compacting: false, error: (e as Error).message })
    }
  },

  toggle_auto_compact: async (ring_id) => {
    const conv_id = get().current_conversation_id
    if (!conv_id) return
    const new_val = !get().auto_compact
    set({ auto_compact: new_val })
    try {
      await api.update_conversation(ring_id, conv_id, { auto_compact: new_val })
    } catch (e) {
      set({ auto_compact: !new_val, error: (e as Error).message })
    }
  },
}))
