import { create } from 'zustand'
import type { ChatMessage } from '../types/chat'
import { streamChat } from '../services/sse'

interface SelfChatState {
  messages: ChatMessage[]
  input: string
  sending: boolean
  streaming_message_id: string | null
  setInput: (val: string) => void
  send: () => void
  loadHistory: () => Promise<void>
}

export const useSelfChatStore = create<SelfChatState>((set, get) => ({
  messages: [],
  input: '',
  sending: false,
  streaming_message_id: null,

  setInput: (val) => set({ input: val }),

  send: () => {
    const { input, sending } = get()
    if (!input.trim() || sending) return

    const user_content = input

    set({ input: '', sending: true })

    streamChat('/api/self/chat', { content: user_content, node_refs: [], tag_refs: [] }, {
      onStart: (data) => {
        const aiMsg: ChatMessage = {
          id: data.message_id,
          role: 'self' as ChatMessage['role'],
          sender_name: 'SELF',
          content: '',
          created_at: new Date().toISOString(),
        }
        set((s) => ({
          messages: [...s.messages, {
            id: `msg-${Date.now()}`,
            role: 'user',
            sender_name: 'You',
            content: user_content,
            created_at: new Date().toISOString(),
          }, aiMsg],
          streaming_message_id: data.message_id,
        }))
      },
      onDelta: (data) => {
        const { streaming_message_id, messages } = get()
        if (!streaming_message_id) return
        set({
          messages: messages.map((m) =>
            m.id === streaming_message_id ? { ...m, content: m.content + data.content } : m
          ),
        })
      },
      onEnd: () => {
        set({ sending: false, streaming_message_id: null })
      },
      onError: (data) => {
        const { streaming_message_id, messages } = get()
        if (streaming_message_id) {
          set({
            messages: messages.map((m) =>
              m.id === streaming_message_id
                ? { ...m, content: m.content + `\n\nError: ${data.error}` }
                : m
            ),
            sending: false,
            streaming_message_id: null,
          })
        } else {
          set({ sending: false })
        }
      },
    })
  },

  loadHistory: async () => {
    try {
      const res = await fetch('/api/self/chat/history?limit=50', {
        headers: { 'X-Ring-Token': localStorage.getItem('ring_token') ?? '' },
      })
      if (!res.ok) return
      const data = await res.json()
      set({ messages: data.messages ?? [] })
    } catch {}
  },
}))
