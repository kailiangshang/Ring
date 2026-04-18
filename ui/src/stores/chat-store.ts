import { create } from 'zustand'
import type { ChatMessage } from '../types/chat'
import { parseCommand } from '../services/command-parser'
import { streamChat } from '../services/sse'
import { usePanelStore } from './panel-store'
import { useSelfStore } from './self-store'
import { useModeStore } from './mode-store'
import { useRingStore } from './ring-store'
import { useAppStore } from './app-store'
import { useGraphStore } from './graph-store'

interface ChatState {
  messages: ChatMessage[]
  input: string
  session_mode: 'storage' | 'ephemeral'
  sending: boolean
  streaming_message_id: string | null
  abort_controller: AbortController | null
  history_loaded: boolean
  setInput: (val: string) => void
  addMessage: (msg: ChatMessage) => void
  updateMessageContent: (id: string, content: string) => void
  send: () => void
  loadHistory: () => Promise<void>
  setSessionMode: (mode: 'storage' | 'ephemeral') => void
  stopStreaming: () => void
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  input: '',
  session_mode: 'storage',
  sending: false,
  streaming_message_id: null,
  abort_controller: null,
  history_loaded: false,

  setInput: (val) => set({ input: val }),

  addMessage: (msg) => set((s) => ({ messages: [...s.messages, msg] })),

  updateMessageContent: (id, content) =>
    set((s) => ({
      messages: s.messages.map((m) => (m.id === id ? { ...m, content } : m)),
    })),

  send: () => {
    const { input, addMessage, sending } = get()
    if (!input.trim() || sending) return

    const parsed = parseCommand(input)

    if (parsed) {
      for (const cmd of parsed) {
        switch (cmd.type) {
          case 'action': {
            if (cmd.action === 'graph') usePanelStore.getState().toggle('graph')
            else if (cmd.action === 'archive') usePanelStore.getState().toggle('archive')
            else if (cmd.action === 'config') usePanelStore.getState().toggle('config')
            else if (cmd.action === 'session') usePanelStore.getState().toggle('session')
            else if (cmd.action === 'auto') useModeStore.getState().toggleAuto()
            else if (cmd.action === 'new') {
              const name = cmd.args
              if (name) {
                useRingStore.getState().createRing(name, `You are a ${name} assistant`)
              }
            }
            else if (cmd.action === 'save') {
              addMessage({
                id: `sys-${Date.now()}`,
                role: 'system',
                sender_name: 'SYSTEM',
                content: '归档功能将在后续版本实现',
                created_at: new Date().toISOString(),
              })
            }
            else if (cmd.action === 'node') {
              const name = cmd.args
              const rid = useRingStore.getState().active_ring_id
              if (name && rid) {
                useGraphStore.getState().createNode(rid, name)
              }
            }
            break
          }
          case 'address': {
            if (cmd.target === 'self') useSelfStore.getState().setOpen(true)
            break
          }
          case 'meta': {
            if (cmd.key === 'mode' && cmd.value) useModeStore.getState().setInteractionMode(cmd.value as 'normal' | 'auto')
            else if (cmd.key === 'skill' && cmd.value) useModeStore.getState().setSkillMode(cmd.value as 'auto' | 'plan' | 'edit')
            break
          }
          case 'reference':
            break
        }
      }
    }

    addMessage({
      id: `msg-${Date.now()}`,
      role: 'user',
      sender_name: 'You',
      content: input,
      node_refs: parsed?.filter((c) => c.type === 'reference').map((c) => c.name),
      created_at: new Date().toISOString(),
    })

    const user_content = input
    const node_refs = parsed?.filter((c) => c.type === 'reference').map((c) => c.name) ?? []

    set({ input: '', sending: true })

    const context = useAppStore.getState().current_context
    const ring_id = useRingStore.getState().active_ring_id

    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat`
    } else {
      set({ sending: false })
      return
    }

    const controller = streamChat(url, { content: user_content, node_refs, tag_refs: [] }, {
      onStart: (data) => {
        const aiMsg: ChatMessage = {
          id: data.message_id,
          role: data.role as ChatMessage['role'],
          sender_name: data.role === 'group_ring' ? 'GROUP RING' : data.role.toUpperCase(),
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
            m.id === streaming_message_id ? { ...m, content: m.content + data.content } : m,
          ),
        })
      },
      onEnd: () => {
        set({ sending: false, streaming_message_id: null, abort_controller: null })
      },
      onError: (data) => {
        const { streaming_message_id, messages } = get()
        if (streaming_message_id) {
          set({
            messages: messages.map((m) =>
              m.id === streaming_message_id
                ? { ...m, content: m.content + `\n\n⚠ Error: ${data.error}` }
                : m,
            ),
            sending: false,
            streaming_message_id: null,
            abort_controller: null,
          })
        } else {
          addMessage({
            id: `err-${Date.now()}`,
            role: 'system',
            sender_name: 'SYSTEM',
            content: `Error: ${data.error}`,
            created_at: new Date().toISOString(),
          })
          set({ sending: false, abort_controller: null })
        }
      },
    })
    set({ abort_controller: controller })
  },

  loadHistory: async () => {
    const context = useAppStore.getState().current_context
    const ring_id = useRingStore.getState().active_ring_id
    const token = localStorage.getItem('ring_token')

    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat/history?limit=50`
    } else {
      return
    }

    try {
      const res = await fetch(url, {
        headers: { 'X-Ring-Token': token ?? '' },
      })
      if (!res.ok) return
      const data = await res.json()
      set({ messages: data.messages ?? [], history_loaded: true })
    } catch {
      // keep existing messages
    }
  },

  setSessionMode: (mode) => set({ session_mode: mode }),

  stopStreaming: () => {
    const { abort_controller } = get()
    abort_controller?.abort()
    set({ sending: false, streaming_message_id: null, abort_controller: null })
  },
}))
