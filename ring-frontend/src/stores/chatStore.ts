import { create } from 'zustand'
import * as api from '../api/client'
import { parseSseStream } from '../components/chat/SseParser'
import type { Message, SseEvent } from '../types'

interface ChatState {
  messages: Message[]
  is_streaming: boolean
  current_conversation_id: string | null
  error: string | null

  create_conversation: (ring_id: string, title: string) => Promise<string>
  load_history: (ring_id: string, conv_id: string) => Promise<void>
  send_message: (ring_id: string, content: string) => Promise<void>
  reset: () => void
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  is_streaming: false,
  current_conversation_id: null,
  error: null,

  create_conversation: async (ring_id, title) => {
    const conv = await api.create_conversation(ring_id, title)
    set({ current_conversation_id: conv.id, messages: [] })
    return conv.id
  },

  load_history: async (ring_id, conv_id) => {
    set({ current_conversation_id: conv_id })
    const msgs = await api.get_messages(ring_id, conv_id)
    set({ messages: msgs })
  },

  send_message: async (ring_id, content) => {
    const conv_id = get().current_conversation_id
    if (!conv_id) return

    const user_msg: Message = {
      id: `temp-${Date.now()}`,
      conversation_id: conv_id,
      role: 'user',
      content,
      sender_id: '',
      created_at: new Date().toISOString(),
    }

    set((s) => ({
      messages: [...s.messages, user_msg],
      is_streaming: true,
      error: null,
    }))

    try {
      const res = await api.send_message(ring_id, conv_id, content)

      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `request failed: ${res.status}`)
      }

      const reader = res.body?.getReader()
      if (!reader) throw new Error('no response body')

      let assistant_content = ''

      for await (const event of parseSseStream(reader) as AsyncGenerator<SseEvent>) {
        if (event.type === 'text' && event.content) {
          assistant_content += event.content
          const assistant_msg: Message = {
            id: `stream-${conv_id}`,
            conversation_id: conv_id,
            role: 'assistant',
            content: assistant_content,
            sender_id: '',
            created_at: new Date().toISOString(),
          }
          set((s) => ({
            messages: [
              ...s.messages.filter((m) => m.id !== `stream-${conv_id}`),
              assistant_msg,
            ],
          }))
        } else if (event.type === 'error') {
          throw new Error(event.message || 'stream error')
        }
      }

      set({ is_streaming: false })
    } catch (e) {
      set({
        is_streaming: false,
        error: (e as Error).message,
      })
    }
  },

  reset: () =>
    set({
      messages: [],
      is_streaming: false,
      current_conversation_id: null,
      error: null,
    }),
}))
