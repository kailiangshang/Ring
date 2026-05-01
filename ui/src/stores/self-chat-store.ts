import { create } from 'zustand'
import type { ChatMessage } from '../types/chat'
import { streamChat } from '../services/sse'
import { getToken } from '../services/api'

interface SelfChatState {
  messages: ChatMessage[]
  input: string
  sending: boolean
  streaming_message_id: string | null
  setInput: (val: string) => void
  send: () => void
  stopStreaming: () => void
  loadHistory: () => Promise<void>
  _abortController: AbortController | null
}

export const useSelfChatStore = create<SelfChatState>((set, get) => ({
  messages: [],
  input: '',
  sending: false,
  streaming_message_id: null,
  _abortController: null,

  setInput: (val) => set({ input: val }),

  send: () => {
    const { input, sending } = get()
    if (!input.trim() || sending) return

    const user_content = input
    const userMsg: ChatMessage = {
      id: `msg-${Date.now()}`,
      role: 'user',
      sender_name: 'You',
      content: user_content,
      created_at: new Date().toISOString(),
    }

    set((s) => ({ input: '', sending: true, messages: [...s.messages, userMsg] }))

    const controller = streamChat('/api/self/chat', { content: user_content, node_refs: [], tag_refs: [] }, {
      onStart: (data) => {
        const aiMsg: ChatMessage = {
          id: data.message_id,
          role: 'self' as ChatMessage['role'],
          sender_name: 'SELF',
          content: '',
          created_at: new Date().toISOString(),
        }
        set((s) => ({
          messages: [...s.messages, aiMsg],
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
      onEnd: (data) => {
        const { streaming_message_id, messages } = get()
        if (streaming_message_id && data.usage) {
          set({
            messages: messages.map((m) =>
              m.id === streaming_message_id ? { ...m, token_usage: data.usage } : m
            ),
          })
        }
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
          const errMsg: ChatMessage = {
            id: `err-${Date.now()}`,
            role: 'self' as ChatMessage['role'],
            sender_name: 'SELF',
            content: 'Connection failed. Please try again.',
            created_at: new Date().toISOString(),
          }
          set((s) => ({ messages: [...s.messages, errMsg], sending: false }))
        }
      },
    })

    set({ _abortController: controller })
  },

  stopStreaming: () => {
    get()._abortController?.abort()
    set({ sending: false, streaming_message_id: null, _abortController: null })
  },

  loadHistory: async () => {
    try {
      const token = await getToken()
      const res = await fetch('/api/self/chat/history?limit=50', {
        headers: { 'X-Ring-Token': token ?? '' },
        signal: AbortSignal.timeout(15000),
      })
      if (!res.ok) return
      const data = await res.json()
      const messages = (data.messages ?? []).map((m: any) => ({
        ...m,
        content: typeof m.content === 'string' ? m.content : JSON.stringify(m.content),
      }))
      set({ messages })
    } catch (e) {
      console.error('loadMessages error:', e)
    }
  },
}))
