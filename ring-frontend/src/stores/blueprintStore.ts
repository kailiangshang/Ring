import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import * as api from '../api/client'
import { parseSseStream } from '../components/chat/SseParser'
import type { Message, SseEvent, GraphPreview } from '../types'

interface BlueprintState {
  messages: Message[]
  is_streaming: boolean
  error: string | null
  preview_graphs: GraphPreview[] | null
  send_message: (ringId: string, content: string) => Promise<void>
  confirm: (ringId: string) => Promise<void>
  dismiss_preview: () => void
}

export const useBlueprintStore = create<BlueprintState>()(
  persist(
    (set, get) => ({
  messages: [],
  is_streaming: false,
  error: null,
  preview_graphs: null,

  send_message: async (ringId, content) => {
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
      const res = await api.blueprint_chat(ringId, content, history)
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `request failed: ${res.status}`)
      }

      const reader = res.body?.getReader()
      if (!reader) throw new Error('no response body')

      let assistant_content = ''
      const final_id = `bp-${Date.now()}`

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
        } else if (event.type === 'blueprint_proposal' && event.graphs) {
          set({ preview_graphs: event.graphs })
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

  confirm: async (ringId) => {
    const graphs = get().preview_graphs
    if (!graphs) return
    try {
      await api.blueprint_confirm(ringId, graphs)
      set({ preview_graphs: null, messages: [], error: null })
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  dismiss_preview: () => set({ preview_graphs: null }),
}),
{
  name: 'ring-blueprint-state',
  partialize: (state) => ({ messages: state.messages, preview_graphs: state.preview_graphs }),
},
))
