import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import type { ChatMessage } from '../types/chat'
import { parseCommand } from '../services/command-parser'
import { streamChat } from '../services/sse'
import { getToken } from '../services/api'
import { useSelfChatStore } from './self-chat-store'
import { usePanelStore } from './panel-store'
import { useSelfStore } from './self-store'
import { useModeStore } from './mode-store'
import { useArchiveStore } from './archive-store'
import { useSessionStore } from './session-store'
import { useRingStore } from './ring-store'
import { useAppStore } from './app-store'
import { useGraphStore } from './graph-store'
import { useInviteStore } from './invite-store'
import { buildHelpContent, getCommandHelp, handlePrefsShow, handlePrefsSet, handleSkillList, handleSkillInstall, handleSkillRemove, handleCompact } from './command-handlers'
import { useCommandResultStore } from './command-result-store'
import * as api from '../services/api'

function showResult(title: string, content: string) {
  useCommandResultStore.getState().showCommandResult(title, content)
}

type SetFn = (partial: Partial<ChatState> | ((s: ChatState) => void)) => void
type GetFn = () => ChatState

function sysMsg(content: string): ChatMessage {
  return { id: `sys-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content, created_at: new Date().toISOString() }
}

function createSseCallbacks(
  addMessage: (msg: ChatMessage) => void,
  set: SetFn,
  get: GetFn,
) {
  return {
    onStart: (data: { message_id: string; role: string }) => {
      const aiMsg: ChatMessage = {
        id: data.message_id,
        role: data.role as ChatMessage['role'],
        sender_name: data.role === 'group_ring' ? 'GROUP RING' : data.role === 'super_ring' ? 'SUPER RING' : data.role.toUpperCase(),
        content: '',
        created_at: new Date().toISOString(),
      }
      set((s) => {
        s.messages.push(aiMsg)
        s.streaming_message_id = data.message_id
      })
    },
    onDelta: (data: { content: string }) => {
      const { streaming_message_id } = get()
      if (!streaming_message_id) return
      set((s) => {
        const idx = s.messages.findIndex((m) => m.id === streaming_message_id)
        if (idx === -1) return
        s.messages[idx].content += data.content
      })
    },
    onEnd: (data: { usage?: { prompt_tokens: number; completion_tokens: number } }) => {
      const { streaming_message_id } = get()
      if (streaming_message_id && data.usage) {
        set((s) => {
          const idx = s.messages.findIndex((m) => m.id === streaming_message_id)
          if (idx === -1) return
          s.messages[idx].token_usage = data.usage
        })
      }
      set({ sending: false, streaming_message_id: null, abort_controller: null })
      const ctx = useAppStore.getState().current_context
      const ringId = useRingStore.getState().active_ring_id
      if (ctx === 'ring' && ringId) {
        useGraphStore.getState().fetchGraph(ringId)
      }
    },
    onError: (data: { error: string }) => {
      const { streaming_message_id } = get()
      if (streaming_message_id) {
        set((s) => {
          const idx = s.messages.findIndex((m) => m.id === streaming_message_id)
          if (idx === -1) {
            s.sending = false
            s.streaming_message_id = null
            s.abort_controller = null
            return
          }
          s.messages[idx].content += `\n\n⚠ Error: ${data.error}`
          s.sending = false
          s.streaming_message_id = null
          s.abort_controller = null
        })
      } else {
        addMessage(sysMsg(`Error: ${data.error}`))
        set({ sending: false, abort_controller: null })
      }
    },
  }
}

interface ChatState {
  messages: ChatMessage[]
  input: string
  session_mode: 'storage' | 'ephemeral'
  sending: boolean
  streaming_message_id: string | null
  abort_controller: AbortController | null
  history_loaded: boolean
  selection_mode: boolean
  selected_messages: string[]
  setInput: (val: string) => void
  addMessage: (msg: ChatMessage) => void
  updateMessageContent: (id: string, content: string) => void
  send: (overrideContent?: string) => void
  loadHistory: () => Promise<void>
  setSessionMode: (mode: 'storage' | 'ephemeral') => void
  stopStreaming: () => void
  toggleMessageSelection: (id: string) => void
  clearSelection: () => void
  enterSelectionMode: (id: string) => void
  deleteSelected: () => Promise<void>
  deleteMessage: (messageId: string) => Promise<void>
}

export const useChatStore = create<ChatState>()(immer((set, get) => ({
  messages: [],
  input: '',
  session_mode: 'storage',
  sending: false,
  streaming_message_id: null,
  abort_controller: null,
  history_loaded: false,
  selection_mode: false,
  selected_messages: [],

  setInput: (val) => set({ input: val }),

  addMessage: (msg) => set((s) => { s.messages.push(msg) }),

  updateMessageContent: (id, content) =>
    set((s) => {
      const msg = s.messages.find((m: ChatMessage) => m.id === id)
      if (msg) msg.content = content
    }),

  send: (overrideContent?: string) => {
    const { input, addMessage, sending } = get()
    const user_content = overrideContent ?? input
    if (!user_content.trim() || sending) return

    // Only parse commands from explicit user input, never from file content
    const commandSource = overrideContent ? input.trim() : user_content
    const parsed = parseCommand(commandSource)

    const isUICommand = parsed && parsed.every(
      (cmd) => cmd.type === 'action' || cmd.type === 'help' || (cmd.type === 'address' && cmd.target === 'self')
    )

    if (parsed) {
      for (const cmd of parsed) {
        switch (cmd.type) {
          case 'action': {
            if (cmd.action === 'graph') usePanelStore.getState().toggle('graph')
            else if (cmd.action === 'archive') usePanelStore.getState().toggle('archive')
            else if (cmd.action === 'config') usePanelStore.getState().toggle('config')
            else if (cmd.action === 'blueprint') usePanelStore.getState().toggle('blueprint')
            else if (cmd.action === 'session') {
              if (cmd.subcommand === 'create') {
                const title = cmd.args.trim()
                const rid = useRingStore.getState().active_ring_id
                if (title && rid) {
                  useSessionStore.getState().createSession({ title, skill: 'decision' })
                  addMessage(sysMsg(`Creating session: ${title}`))
                }
              } else if (cmd.subcommand === 'close') {
                const sid = useSessionStore.getState().active_session?.id
                const rid = useRingStore.getState().active_ring_id
                if (sid && rid) {
                  useSessionStore.getState().closeSession(rid, sid)
                  addMessage(sysMsg('Session closed'))
                }
              } else if (cmd.subcommand === 'start') {
                const sid = useSessionStore.getState().active_session?.id
                const rid = useRingStore.getState().active_ring_id
                if (sid && rid) {
                  useSessionStore.getState().startSession(rid, sid)
                  addMessage(sysMsg('Starting session discussion'))
                }
              } else {
                usePanelStore.getState().toggle('session')
              }
            }
            else if (cmd.action === 'new') {
              const name = cmd.args.trim()
              if (name) {
                useRingStore.getState().createRing({ name, role_description: `You are a ${name} assistant`, storage_mode: 'local' })
                addMessage(sysMsg(`Creating ring: ${name}`))
              }
            }
            else if (cmd.action === 'save') {
              const rid = useRingStore.getState().active_ring_id
              const sid = useSessionStore.getState().active_session?.id
              const msgs = get().messages.filter((m) => m.role === 'user' || m.role === 'group_ring')
              const lastUserMsg = [...msgs].reverse().find((m) => m.role === 'user')
              const title = cmd.args.trim() || lastUserMsg?.content.slice(0, 40) || 'untitled'
              const content = msgs.slice(-6).map((m) => `${m.sender_name}: ${m.content}`).join('\n')
              if (rid) {
                useArchiveStore.getState().triggerArchive(rid, content, title, sid)
                addMessage(sysMsg(`Archiving: ${title}`))
              }
            }
            else if (cmd.action === 'node') {
              if (cmd.subcommand === 'add') {
                const name = cmd.args.trim()
                const rid = useRingStore.getState().active_ring_id
                if (name && rid) {
                  useGraphStore.getState().createNode(rid, name)
                  addMessage(sysMsg(`Node added: ${name}`))
                }
              }
            }
            else if (cmd.action === 'mode') {
              const mode = cmd.args.trim()
              if (mode === 'auto' || mode === 'normal') {
                useModeStore.getState().setInteractionMode(mode)
                addMessage(sysMsg(`Mode set: ${mode}`))
              }
            }
            else if (cmd.action === 'prefs') {
              if (cmd.subcommand === 'set') {
                const parts = cmd.args.trim().split(/\s+/)
                const key = parts[0]
                const value = parts.slice(1).join(' ')
                if (key && value) {
                  handlePrefsSet(key, value, addMessage)
                }
              } else {
                handlePrefsShow(addMessage, showResult)
              }
            }
            else if (cmd.action === 'skill') {
              if (cmd.subcommand === 'install') {
                const parts = cmd.args.trim().split(/\s+/)
                const name = parts[0]
                const url = parts[1]
                if (name && url) {
                  handleSkillInstall(name, url, addMessage)
                }
              } else if (cmd.subcommand === 'remove') {
                const name = cmd.args.trim()
                if (name) {
                  handleSkillRemove(name, addMessage)
                }
              } else {
                handleSkillList(addMessage, showResult)
              }
            }
            else if (cmd.action === 'cross-ring-query') {
              const question = cmd.args.trim()
              if (question) {
                addMessage({ id: `msg-${crypto.randomUUID()}`, role: 'user', sender_name: 'You', content: `/cross-ring-query ${question}`, created_at: new Date().toISOString() })
                set({ sending: true })
                const controller = streamChat('/api/super/cross-ring-query', { query: question }, createSseCallbacks(addMessage, set, get))
                set({ abort_controller: controller })
              }
            }
            else if (cmd.action === 'cross-ring-analysis') {
              const parts = cmd.args.trim().split(/\s+/)
              const analysisType = parts[0] as 'compare' | 'merge' | 'summary'
              const ringNames = parts[1]?.split(',') || []
              const question = parts.slice(2).join(' ')
              if (analysisType && ringNames.length > 0) {
                addMessage({ id: `msg-${crypto.randomUUID()}`, role: 'user', sender_name: 'You', content: `/cross-ring-analysis ${analysisType} ${ringNames.join(',')}${question ? ' ' + question : ''}`, created_at: new Date().toISOString() })
                set({ sending: true })
                const controller = streamChat('/api/super/cross-ring-analysis', { ring_names: ringNames, analysis_type: analysisType, question: question || undefined }, createSseCallbacks(addMessage, set, get))
                set({ abort_controller: controller })
              }
            }
            else if (cmd.action === 'invite') {
              if (cmd.subcommand === 'open') {
                const rid = useRingStore.getState().active_ring_id
                if (rid) {
                  useInviteStore.getState().create_token(rid, { type: 'open' })
                  addMessage(sysMsg('Creating open invite...'))
                }
              } else if (cmd.subcommand === 'audit') {
                const rid = useRingStore.getState().active_ring_id
                if (rid) {
                  useInviteStore.getState().create_token(rid, { type: 'audit' })
                  addMessage(sysMsg('Creating audit invite...'))
                }
              }
            }
            else if (cmd.action === 'compact') {
              handleCompact(addMessage)
            }
            break
          }
          case 'address': {
            if (cmd.target === 'self') {
              useSelfStore.getState().setOpen(true)
              useSelfStore.getState().setTab('chat')
              if (cmd.rest.trim()) {
                useSelfChatStore.getState().setInput(cmd.rest)
                setTimeout(() => useSelfChatStore.getState().send(), 0)
              }
            }
            else if (cmd.target === 'ring') {
              useAppStore.getState().setContext('ring')
            }
            else if (cmd.target === 'super') {
              useAppStore.getState().setContext('super')
            }
            break
          }
          case 'help': {
            const targetCmd = cmd.command
            showResult(targetCmd ? `/help ${targetCmd}` : '/help', targetCmd ? getCommandHelp(targetCmd) : buildHelpContent())
            break
          }
        }
      }
    }

    if (isUICommand) {
      set({ input: '' })
      return
    }

    addMessage({
      id: `msg-${crypto.randomUUID()}`,
      role: 'user',
      sender_name: 'You',
      content: user_content,
      node_refs: parsed?.filter((c): c is { type: 'address'; target: string; rest: string } => c.type === 'address' && c.target === 'node').map((c) => c.rest),
      created_at: new Date().toISOString(),
    })

    const node_refs = parsed?.filter((c): c is { type: 'address'; target: string; rest: string } => c.type === 'address' && c.target === 'node').map((c) => c.rest) ?? []

    set({ input: '', sending: true })

    const context = useAppStore.getState().current_context
    const ring_id = useRingStore.getState().active_ring_id

    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat`
    } else if (context === 'super') {
      url = '/api/super/chat'
    } else {
      addMessage(sysMsg('Chat is not available in this context.'))
      set({ sending: false })
      return
    }

    const controller = streamChat(url, { content: user_content, node_refs, tag_refs: [] }, createSseCallbacks(addMessage, set, get))
    set({ abort_controller: controller })
  },

  loadHistory: async () => {
    const context = useAppStore.getState().current_context
    const ring_id = useRingStore.getState().active_ring_id
    const token = await getToken()

    // Abort any in-flight SSE before switching context
    const { abort_controller } = get()
    abort_controller?.abort()

    // 立即清空旧消息，避免上下文切换时显示残留内容
    set({ messages: [], history_loaded: false, streaming_message_id: null, sending: false, abort_controller: null, selection_mode: false, selected_messages: [] })

    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat/history?limit=50`
    } else if (context === 'super') {
      url = '/api/super/chat/history?limit=50'
    } else {
      set({ messages: [], history_loaded: true })
      return
    }

    try {
      const res = await fetch(url, {
        headers: { 'X-Ring-Token': token ?? '' },
        signal: AbortSignal.timeout(15000),
      })
      if (!res.ok) {
        set({ messages: [], history_loaded: true })
        return
      }
      const data = await res.json()
      set((s) => {
        s.messages = (data.messages ?? []).map((m: Record<string, unknown>) => ({
          ...m,
          content: typeof m.content === 'string' ? m.content : JSON.stringify(m.content),
        }))
        s.history_loaded = true
      })
    } catch (e) {
      console.error('loadHistory error:', e)
      set({ messages: [], history_loaded: true })
    }
  },

  setSessionMode: (mode) => set({ session_mode: mode }),

  stopStreaming: () => {
    const { abort_controller } = get()
    abort_controller?.abort()
    set({ sending: false, streaming_message_id: null, abort_controller: null })
  },

  toggleMessageSelection: (id) => {
    set((s) => {
      const idx = s.selected_messages.indexOf(id)
      if (idx === -1) {
        s.selected_messages.push(id)
      } else {
        s.selected_messages.splice(idx, 1)
        if (s.selected_messages.length === 0) {
          s.selection_mode = false
        }
      }
    })
  },

  clearSelection: () => set({ selection_mode: false, selected_messages: [] }),

  enterSelectionMode: (id) => set({ selection_mode: true, selected_messages: [id] }),

  deleteSelected: async () => {
    const { selected_messages } = get()
    if (selected_messages.length === 0) return
    const ids = [...selected_messages]
    set({ selection_mode: false, selected_messages: [], messages: get().messages.filter((m) => !ids.includes(m.id)) })
    const context = useAppStore.getState().current_context
    const ring_id = useRingStore.getState().active_ring_id
    for (const id of ids) {
      try {
        if (context === 'ring' && ring_id) {
          await api.deleteRingMessage(ring_id, id)
        } else if (context === 'super') {
          await api.deleteSuperMessage(id)
        } else {
          await api.deleteSelfMessage(id)
        }
      } catch {
        // already removed from local state
      }
    }
  },

  deleteMessage: async (messageId) => {
    set((s) => {
      s.messages = s.messages.filter((m) => m.id !== messageId)
      const idx = s.selected_messages.indexOf(messageId)
      if (idx !== -1) s.selected_messages.splice(idx, 1)
      if (s.selected_messages.length === 0) s.selection_mode = false
    })
    const context = useAppStore.getState().current_context
    const ring_id = useRingStore.getState().active_ring_id
    try {
      if (context === 'ring' && ring_id) {
        await api.deleteRingMessage(ring_id, messageId)
      } else if (context === 'super') {
        await api.deleteSuperMessage(messageId)
      } else {
        await api.deleteSelfMessage(messageId)
      }
    } catch {
      // already removed from local state
    }
  },
})))
