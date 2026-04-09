import { create } from 'zustand'
import * as api from '../api/client'
import { parseSseStream } from '../components/chat/SseParser'
import type { Message, SseEvent } from '../types'

interface SessionChatState {
  messages: Message[]
  is_streaming: boolean
  error: string | null
  load_history: (ring_id: string, session_id: string) => Promise<void>
  send_message: (ring_id: string, session_id: string, content: string) => Promise<void>
  reset: () => void
}

export const useSessionChatStore = create<SessionChatState>((set, _get) => ({
  messages: [],
  is_streaming: false,
  error: null,

  load_history: async (ring_id, session_id) => {
    try {
      const msgs = await api.get_session_messages(ring_id, session_id)
      set({
        messages: msgs.map((m: any) => ({
          id: m.id,
          conversation_id: m.session_id,
          role: m.role,
          content: m.content,
          sender_id: m.sender_id,
          created_at: m.created_at,
        })),
      })
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  send_message: async (ring_id, session_id, content) => {
    const user_msg: Message = {
      id: `temp-${Date.now()}`,
      conversation_id: session_id,
      role: 'user',
      content,
      sender_id: '',
      created_at: new Date().toISOString(),
    }

    set((s) => ({ messages: [...s.messages, user_msg], is_streaming: true, error: null }))

    try {
      const res = await api.send_session_message(ring_id, session_id, content)
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `request failed: ${res.status}`)
      }

      const reader = res.body?.getReader()
      if (!reader) throw new Error('no response body')

      let assistant_content = ''
      const final_id = `sess-${Date.now()}`

      for await (const event of parseSseStream(reader) as AsyncGenerator<SseEvent>) {
        if (event.type === 'text' && event.content) {
          assistant_content += event.content
          set((s) => {
            const filtered = s.messages.filter((m) => m.id !== final_id)
            return {
              messages: [
                ...filtered,
                {
                  id: final_id,
                  conversation_id: session_id,
                  role: 'assistant',
                  content: assistant_content,
                  sender_id: '',
                  created_at: new Date().toISOString(),
                },
              ],
            }
          })
        } else if (event.type === 'error') {
          throw new Error(event.message || 'stream error')
        }
      }
    } catch (e) {
      set({ error: (e as Error).message })
    } finally {
      set({ is_streaming: false })
    }
  },

  reset: () => set({ messages: [], is_streaming: false, error: null }),
}))
