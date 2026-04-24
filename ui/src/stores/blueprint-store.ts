import { create } from 'zustand'
import { api } from '../services/api'
import { streamChat } from '../services/sse'
import type { SseCallbacks } from '../services/sse'

interface BlueprintMessage {
  id: string
  role: string
  content: string
  token_usage?: { prompt_tokens: number; completion_tokens: number }
}

interface BlueprintGraph {
  name: string
  nodes: { label: string; node_type: string; tags: string[] }[]
  edges: { from: string; to: string; relation: string }[]
}

interface BlueprintState {
  mode: 'quick' | 'deep'
  messages: BlueprintMessage[]
  streaming: boolean
  current_blueprint: { graphs: BlueprintGraph[] } | null
  confirmed: boolean
  streaming_content: string
  abort_controller: AbortController | null
  setMode: (mode: 'quick' | 'deep') => void
  sendMessage: (ringId: string, content: string) => void
  loadHistory: (ringId: string) => Promise<void>
  confirm: (ringId: string) => Promise<void>
  checkStatus: (ringId: string) => Promise<void>
  stopStreaming: () => void
}

function extractBlueprint(text: string): { graphs: BlueprintGraph[] } | null {
  const match = text.match(/<blueprint>\s*([\s\S]*?)\s*<\/blueprint>/)
  if (!match) return null
  try {
    return JSON.parse(match[1])
  } catch {
    return null
  }
}

function stripBlueprintTags(text: string): string {
  return text.replace(/<blueprint>[\s\S]*?<\/blueprint>/g, '').trim()
}

export const useBlueprintStore = create<BlueprintState>((set, get) => ({
  mode: 'quick',
  messages: [],
  streaming: false,
  current_blueprint: null,
  confirmed: false,
  streaming_content: '',
  abort_controller: null,

  setMode: (mode) => set({ mode }),

  sendMessage: (ringId, content) => {
    const state = get()
    if (state.streaming) return

    const userMsg: BlueprintMessage = {
      id: Date.now().toString(),
      role: 'user',
      content,
    }

    const aiMsgId = (Date.now() + 1).toString()
    const aiMsg: BlueprintMessage = {
      id: aiMsgId,
      role: 'blueprint',
      content: '',
    }

    set({
      messages: [...state.messages, userMsg, aiMsg],
      streaming: true,
      streaming_content: '',
    })

    const callbacks: SseCallbacks = {
      onStart: () => {},
      onDelta: (data) => {
        set((s) => {
          const newContent = s.streaming_content + data.content
          const bp = extractBlueprint(newContent)
          const msgs = [...s.messages]
          const last = msgs[msgs.length - 1]
          if (last && last.id === aiMsgId) {
            msgs[msgs.length - 1] = { ...last, content: newContent }
          }
          return {
            messages: msgs,
            streaming_content: newContent,
            current_blueprint: bp ?? s.current_blueprint,
          }
        })
      },
      onEnd: (data) => {
        set((s) => {
          const msgs = [...s.messages]
          const last = msgs[msgs.length - 1]
          if (last && last.id === aiMsgId) {
            msgs[msgs.length - 1] = {
              ...last,
              content: s.streaming_content,
              token_usage: data.usage,
            }
          }
          return { messages: msgs, streaming: false }
        })
      },
      onError: (data) => {
        set((s) => {
          const msgs = [...s.messages]
          const last = msgs[msgs.length - 1]
          if (last && last.id === aiMsgId) {
            msgs[msgs.length - 1] = {
              ...last,
              content: last.content + `\n\nError: ${data.error}`,
            }
          }
          return { messages: msgs, streaming: false }
        })
      },
    }

    const controller = streamChat(
      `/api/rings/${ringId}/blueprint/chat`,
      {
        content,
        current_blueprint: state.current_blueprint,
      },
      callbacks,
    )
    set({ abort_controller: controller })
  },

  loadHistory: async (ringId) => {
    try {
      const res = await api.get<{ messages: BlueprintMessage[]; has_more: boolean }>(
        `/rings/${ringId}/blueprint/chat/history`,
      )
      const bpMsg = [...res.messages].reverse().find((m) => {
        const bp = extractBlueprint(m.content)
        return bp !== null
      })
      const bp = bpMsg ? extractBlueprint(bpMsg.content) : null
      set({ messages: res.messages, current_blueprint: bp })
    } catch {}
  },

  confirm: async (ringId) => {
    const state = get()
    try {
      await api.post(`/rings/${ringId}/blueprint/confirm`, {
        blueprint: state.current_blueprint
          ? { graphs: state.current_blueprint.graphs }
          : null,
      })
      set({ confirmed: true })
    } catch {}
  },

  checkStatus: async (ringId) => {
    try {
      const res = await api.get<{ status: string }>(`/rings/${ringId}/blueprint`)
      if (res.status === 'confirmed') {
        set({ confirmed: true })
      }
    } catch {}
  },

  stopStreaming: () => {
    const state = get()
    state.abort_controller?.abort()
    set({ streaming: false })
  },
}))

export { stripBlueprintTags, extractBlueprint }
export type { BlueprintGraph, BlueprintMessage }
