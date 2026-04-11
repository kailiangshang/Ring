import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import * as api from '../api/client'
import { parseSseStream } from '../components/chat/SseParser'
import type { Message, SseEvent } from '../types'

interface RingSuperState {
  messages: Message[]
  is_streaming: boolean
  error: string | null
  send_message: (content: string) => Promise<void>
}

export const useRingSuperStore = create<RingSuperState>()(
  persist(
    (set, get) => ({
      messages: [],
      is_streaming: false,
      error: null,

  send_message: async (content) => {
    const user_msg: Message = {
      id: `temp-${Date.now()}`,
      conversation_id: '',
      role: 'user',
      content,
      sender_id: '',
      tool_calls: null,
      archived: false,
      created_at: new Date().toISOString(),
    }

    const prev_messages = get().messages
    set({ messages: [...prev_messages, user_msg], is_streaming: true, error: null })

    const history = prev_messages.map((m) => ({ role: m.role, content: m.content }))

    try {
      const res = await api.ring_super_chat(content, history)
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `request failed: ${res.status}`)
      }

      const reader = res.body?.getReader()
      if (!reader) throw new Error('no response body')

      let assistant_content = ''
      const final_id = `super-${Date.now()}`

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
                  conversation_id: '',
                  role: 'assistant',
                  content: assistant_content,
                  sender_id: null,
                  tool_calls: null,
                  archived: false,
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
}),
{
  name: 'ring-super-messages',
  partialize: (state) => ({ messages: state.messages }),
},
))
