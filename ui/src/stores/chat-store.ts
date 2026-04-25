import { create } from 'zustand'
import type { ChatMessage } from '../types/chat'
import { parseCommand } from '../services/command-parser'
import { streamChat } from '../services/sse'
import { getPreferences, updatePreferences, listSkills, installSkill, removeSkill, getToken } from '../services/api'
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

const SCOPE_LABELS: Record<string, string> = {
  super: 'Super',
  ring: 'Ring',
  session: 'Session',
}

function buildHelpContent(): string {
  type CmdInfo = { prefix: string; cmd: string; desc: string; scopes: string[] }

  const slashCmds: CmdInfo[] = [
    { prefix: '/', cmd: 'graph', desc: 'Open graph panel', scopes: ['ring'] },
    { prefix: '/', cmd: 'archive', desc: 'Open archive panel', scopes: ['ring'] },
    { prefix: '/', cmd: 'config', desc: 'Open config panel', scopes: ['ring'] },
    { prefix: '/', cmd: 'session [create/close/start/summarize]', desc: 'Session operations', scopes: ['ring', 'session'] },
    { prefix: '/', cmd: 'new <name>', desc: 'Create new ring', scopes: ['ring'] },
    { prefix: '/', cmd: 'save', desc: 'Archive conversation', scopes: ['ring', 'session'] },
    { prefix: '/', cmd: 'node [add/link]', desc: 'Graph node operations', scopes: ['ring'] },
    { prefix: '/', cmd: 'mode [auto/normal]', desc: 'Set interaction mode', scopes: ['ring'] },
    { prefix: '/', cmd: 'prefs [set key value]', desc: 'Show/set preferences', scopes: ['super', 'ring'] },
    { prefix: '/', cmd: 'skill [list/install/remove]', desc: 'Manage skills', scopes: ['super', 'ring'] },
    { prefix: '/', cmd: 'members', desc: 'Show members', scopes: ['ring'] },
    { prefix: '/', cmd: 'invite [open/audit]', desc: 'Create invite', scopes: ['ring'] },
    { prefix: '/', cmd: 'help [command]', desc: 'Show help', scopes: ['super', 'ring', 'session'] },
  ]

  const atCmds: CmdInfo[] = [
    { prefix: '@', cmd: 'self [message]', desc: 'Talk to Self', scopes: ['super', 'ring', 'session'] },
    { prefix: '@', cmd: 'ring [message]', desc: 'Talk to Ring AI', scopes: ['ring'] },
    { prefix: '@', cmd: 'super [message]', desc: 'Talk to Super Ring', scopes: ['super', 'ring'] },
    { prefix: '@', cmd: 'node <name>', desc: 'Reference graph node', scopes: ['ring'] },
  ]

  const currentContext = useAppStore.getState().current_context

  const renderTable = (cmds: CmdInfo[]) => {
    const header = '| Command | Description | Scope |'
    const sep = '|---------|-------------|-------|'
    const rows = cmds.map(c => {
      const scopeStr = c.scopes.map(s => SCOPE_LABELS[s] ?? s).join(', ')
      const marker = c.scopes.includes(currentContext) ? '' : ' 🔒'
      return `| ${c.prefix}${c.cmd}${marker} | ${c.desc} | ${scopeStr} |`
    })
    return [header, sep, ...rows].join('\n')
  }

  return `## Commands\n\n> Scope: **Super** = Super Ring only · **Ring** = Group Ring · **Session** = Active session · 🔒 = not available in current view\n\n### Slash Commands (/ prefix)\n${renderTable(slashCmds)}\n\n### Addressing (@ prefix)\n${renderTable(atCmds)}`
}

function getCommandHelp(command: string): string {
  const helpMap: Record<string, string> = {
    graph: '### /graph\n\nOpen the graph panel to view and edit the knowledge graph.\n\n**Usage:** `/graph`',
    archive: '### /archive\n\nOpen the archive panel to view archived conversations.\n\n**Usage:** `/archive`',
    config: '### /config\n\nOpen the configuration panel.\n\n**Usage:** `/config`',
    session: '### /session\n\nSession operations.\n\n**Usage:**\n- `/session` - Open session panel\n- `/session create <title>` - Create new session\n- `/session close` - Close current session\n- `/session start` - Start discussion\n- `/session summarize` - AI summary',
    new: '### /new\n\nCreate a new Ring.\n\n**Usage:** `/new <ring-name>`',
    save: '### /save\n\nArchive the current conversation.\n\n**Usage:** `/save [optional-title]`',
    node: '### /node\n\nGraph node operations.\n\n**Usage:**\n- `/node add <name>` - Add new node\n- `/node link <from> <to>` - Link two nodes',
    mode: '### /mode\n\nSet interaction mode.\n\n**Usage:** `/mode [auto/normal]`',
    prefs: '### /prefs\n\nManage preferences.\n\n**Usage:**\n- `/prefs` - Show preferences\n- `/prefs set <key> <value>` - Set preference',
    skill: '### /skill\n\nManage skills.\n\n**Usage:**\n- `/skill list` - List skills\n- `/skill install <name> <url>` - Install skill\n- `/skill remove <name>` - Remove skill',
    members: '### /members\n\nShow member list.\n\n**Usage:** `/members`',
    invite: '### /invite\n\nCreate invitation tokens.\n\n**Usage:**\n- `/invite open` - Open invitation\n- `/invite audit` - Audit invitation',
    help: '### /help\n\nShow help information.\n\n**Usage:**\n- `/help` - Show all commands\n- `/help <command>` - Show specific command help',
    cross_ring_query: '### /cross-ring-query\n\nQuery across all your Rings.\n\n**Usage:** `/cross-ring-query <your question>`',
    cross_ring_analysis: '### /cross-ring-analysis\n\nAnalyze multiple Rings.\n\n**Usage:** `/cross-ring-analysis <compare|merge|summary> <ring1,ring2,...> [question]`',
  }

  return helpMap[command] || `No help available for command: ${command}`
}

const PREFS_KEY_MAP: Record<string, { section: string; key: string }> = {
  language: { section: '语言', key: 'default' },
  provider: { section: 'LLM', key: 'default_provider' },
  style: { section: '输出格式', key: 'style' },
  mode: { section: '默认模式', key: 'mode' },
}

async function handlePrefsShow(addMessage: (msg: ChatMessage) => void) {
  try {
    const { content, is_custom } = await getPreferences()
    const label = is_custom ? '当前偏好设置（自定义）：' : '当前偏好设置（默认）：'
    addMessage({
      id: `sys-prefs-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `${label}\n\`\`\`\n${content}\n\`\`\``,
      created_at: new Date().toISOString(),
    })
  } catch {
    addMessage({
      id: `sys-prefs-err-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: 'Failed to load preferences.',
      created_at: new Date().toISOString(),
    })
  }
}

async function handlePrefsSet(key: string, value: string, addMessage: (msg: ChatMessage) => void) {
  const mapping = PREFS_KEY_MAP[key]
  if (!mapping) {
    addMessage({
      id: `sys-prefs-err-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `Unknown preference key "${key}". Supported keys: ${Object.keys(PREFS_KEY_MAP).join(', ')}. For other changes, ask Super Ring.`,
      created_at: new Date().toISOString(),
    })
    return
  }

  try {
    const { content } = await getPreferences()
    const lines = content.split('\n')
    let inSection = false
    let found = false
    const updated = lines.map(line => {
      if (line.trim() === `## ${mapping.section}`) {
        inSection = true
        return line
      }
      if (inSection && line.trim().startsWith(`- ${mapping.key}:`)) {
        found = true
        return `- ${mapping.key}: ${value}`
      }
      if (line.startsWith('## ') && inSection) {
        inSection = false
      }
      return line
    }).join('\n')

    if (!found) {
      addMessage({
        id: `sys-prefs-err-${crypto.randomUUID()}`,
        role: 'system',
        sender_name: 'SYSTEM',
        content: `Could not find preference "${key}" in current settings. Please use Super Ring to modify.`,
        created_at: new Date().toISOString(),
      })
      return
    }

    await updatePreferences(updated)
    addMessage({
      id: `sys-prefs-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `Preference updated: ${key} = ${value}`,
      created_at: new Date().toISOString(),
    })
  } catch {
    addMessage({
      id: `sys-prefs-err-${crypto.randomUUID()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `Failed to update preference "${key}".`,
      created_at: new Date().toISOString(),
    })
  }
}

async function handleSkillList(addMessage: (msg: ChatMessage) => void) {
  try {
    const { skills } = await listSkills()
    if (skills.length === 0) {
      addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: 'No skills installed.', created_at: new Date().toISOString() })
      return
    }
    const lines = skills.map(s => {
      const tag = s.source === 'builtin' ? '[built-in]' : '[user]'
      return `- **${s.name}** ${tag}: ${s.description}`
    })
    addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `## Skills\n\n${lines.join('\n')}`, created_at: new Date().toISOString() })
  } catch {
    addMessage({ id: `sys-skill-err-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: 'Failed to load skills.', created_at: new Date().toISOString() })
  }
}

async function handleSkillInstall(name: string, url: string, addMessage: (msg: ChatMessage) => void) {
  try {
    const result = await installSkill(name, url)
    addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: result.ok ? `Skill "${result.name}" installed: ${result.description}` : 'Install failed', created_at: new Date().toISOString() })
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : 'Unknown error'
    addMessage({ id: `sys-skill-err-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `Skill install failed: ${msg}`, created_at: new Date().toISOString() })
  }
}

async function handleSkillRemove(name: string, addMessage: (msg: ChatMessage) => void) {
  try {
    await removeSkill(name)
    addMessage({ id: `sys-skill-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `Skill "${name}" removed.`, created_at: new Date().toISOString() })
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : 'Unknown error'
    addMessage({ id: `sys-skill-err-${crypto.randomUUID()}`, role: 'system', sender_name: 'SYSTEM', content: `Failed to remove skill: ${msg}`, created_at: new Date().toISOString() })
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
                }
              } else if (cmd.subcommand === 'close') {
                const sid = useSessionStore.getState().active_session?.id
                const rid = useRingStore.getState().active_ring_id
                if (sid && rid) {
                  useSessionStore.getState().closeSession(rid, sid)
                }
              } else if (cmd.subcommand === 'start') {
                const sid = useSessionStore.getState().active_session?.id
                const rid = useRingStore.getState().active_ring_id
                if (sid && rid) {
                  useSessionStore.getState().startSession(rid, sid)
                }
              } else {
                usePanelStore.getState().toggle('session')
              }
            }
            else if (cmd.action === 'new') {
              const name = cmd.args.trim()
              if (name) {
                useRingStore.getState().createRing(name, `You are a ${name} assistant`)
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
                addMessage({
                  id: `sys-${crypto.randomUUID()}`,
                  role: 'system',
                  sender_name: 'SYSTEM',
                  content: `Archiving: ${title}`,
                  created_at: new Date().toISOString(),
                })
              }
            }
            else if (cmd.action === 'node') {
              if (cmd.subcommand === 'add') {
                const name = cmd.args.trim()
                const rid = useRingStore.getState().active_ring_id
                if (name && rid) {
                  useGraphStore.getState().createNode(rid, name)
                }
              }
            }
            else if (cmd.action === 'mode') {
              const mode = cmd.args.trim()
              if (mode === 'auto' || mode === 'normal') {
                useModeStore.getState().setInteractionMode(mode)
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
                handlePrefsShow(addMessage)
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
                handleSkillList(addMessage)
              }
            }
            else if (cmd.action === 'cross-ring-query') {
              const question = cmd.args.trim()
              if (question) {
                handleCrossRingQuery(question, addMessage, set, get)
              }
            }
            else if (cmd.action === 'cross-ring-analysis') {
              const parts = cmd.args.trim().split(/\s+/)
              const analysisType = parts[0] as 'compare' | 'merge' | 'summary'
              const ringNames = parts[1]?.split(',') || []
              const question = parts.slice(2).join(' ')
              if (analysisType && ringNames.length > 0) {
                handleCrossRingAnalysis(analysisType, ringNames, question, addMessage, set, get)
              }
            }
            else if (cmd.action === 'invite') {
              if (cmd.subcommand === 'open') {
                const rid = useRingStore.getState().active_ring_id
                if (rid) {
                  useInviteStore.getState().create_token(rid, { type: 'open' })
                }
              } else if (cmd.subcommand === 'audit') {
                const rid = useRingStore.getState().active_ring_id
                if (rid) {
                  useInviteStore.getState().create_token(rid, { type: 'audit' })
                }
              }
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
              // Switch to ring context and send message
              useAppStore.getState().setContext('ring')
            }
            else if (cmd.target === 'super') {
              // Switch to super context and send message
              useAppStore.getState().setContext('super')
            }
            else if (cmd.target === 'node') {
              // Node reference - handled in message sending
            }
            break
          }
          case 'help': {
            const targetCmd = cmd.command
            if (targetCmd) {
              addMessage({
                id: `sys-help-${crypto.randomUUID()}`,
                role: 'system',
                sender_name: 'SYSTEM',
                content: getCommandHelp(targetCmd),
                created_at: new Date().toISOString(),
              })
            } else {
              addMessage({
                id: `sys-help-${crypto.randomUUID()}`,
                role: 'system',
                sender_name: 'SYSTEM',
                content: buildHelpContent(),
                created_at: new Date().toISOString(),
              })
            }
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
      content: input,
      node_refs: parsed?.filter((c): c is { type: 'address'; target: string; rest: string } => c.type === 'address' && c.target === 'node').map((c) => c.rest),
      created_at: new Date().toISOString(),
    })

    const user_content = input
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
      addMessage({
        id: `sys-${crypto.randomUUID()}`,
        role: 'system',
        sender_name: 'SYSTEM',
        content: 'Chat is not available in this context.',
        created_at: new Date().toISOString(),
      })
      set({ sending: false })
      return
    }

    const controller = streamChat(url, { content: user_content, node_refs, tag_refs: [] }, {
      onStart: (data) => {
        const aiMsg: ChatMessage = {
          id: data.message_id,
          role: data.role as ChatMessage['role'],
          sender_name: data.role === 'group_ring' ? 'GROUP RING' : data.role === 'super_ring' ? 'SUPER RING' : data.role.toUpperCase(),
          content: '',
          created_at: new Date().toISOString(),
        }
        set((s) => ({
          messages: [...s.messages, aiMsg],
          streaming_message_id: data.message_id,
        }))
      },
      onDelta: (data) => {
        const { streaming_message_id } = get()
        if (!streaming_message_id) return
        set((s) => {
          const idx = s.messages.findIndex((m) => m.id === streaming_message_id)
          if (idx === -1) return s
          const msgs = [...s.messages]
          msgs[idx] = { ...msgs[idx], content: msgs[idx].content + data.content }
          return { messages: msgs }
        })
      },
      onEnd: (data) => {
        const { streaming_message_id } = get()
        if (streaming_message_id && data.usage) {
          set((s) => {
            const idx = s.messages.findIndex((m) => m.id === streaming_message_id)
            if (idx === -1) return s
            const msgs = [...s.messages]
            msgs[idx] = { ...msgs[idx], token_usage: data.usage }
            return { messages: msgs }
          })
        }
        set({ sending: false, streaming_message_id: null, abort_controller: null })
      },
      onError: (data) => {
        const { streaming_message_id } = get()
        if (streaming_message_id) {
          set((s) => {
            const idx = s.messages.findIndex((m) => m.id === streaming_message_id)
            if (idx === -1) return { sending: false, streaming_message_id: null, abort_controller: null }
            const msgs = [...s.messages]
            msgs[idx] = { ...msgs[idx], content: msgs[idx].content + `\n\n⚠ Error: ${data.error}` }
            return { messages: msgs, sending: false, streaming_message_id: null, abort_controller: null }
          })
        } else {
          addMessage({
            id: `err-${crypto.randomUUID()}`,
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
    const token = await getToken()

    let url = ''
    if (context === 'ring' && ring_id) {
      url = `/api/rings/${ring_id}/chat/history?limit=50`
    } else if (context === 'super') {
      url = '/api/super/chat/history?limit=50'
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

async function handleCrossRingQuery(
  query: string,
  addMessage: (msg: ChatMessage) => void,
  set: any,
  get: any
) {
  addMessage({
    id: `msg-${crypto.randomUUID()}`,
    role: 'user',
    sender_name: 'You',
    content: `/cross-ring-query ${query}`,
    created_at: new Date().toISOString(),
  })

  set({ sending: true })

  const controller = streamChat('/api/super/cross-ring-query', { query }, {
    onStart: (data) => {
      const aiMsg: ChatMessage = {
        id: data.message_id,
        role: 'super_ring',
        sender_name: 'SUPER RING',
        content: '',
        created_at: new Date().toISOString(),
      }
      set((s: any) => ({
        messages: [...s.messages, aiMsg],
        streaming_message_id: data.message_id,
      }))
    },
    onDelta: (data) => {
      const { streaming_message_id } = get()
      if (!streaming_message_id) return
      set((s: any) => {
        const idx = s.messages.findIndex((m: ChatMessage) => m.id === streaming_message_id)
        if (idx === -1) return s
        const msgs = [...s.messages]
        msgs[idx] = { ...msgs[idx], content: msgs[idx].content + data.content }
        return { messages: msgs }
      })
    },
    onEnd: (data) => {
      const { streaming_message_id } = get()
      if (streaming_message_id && data.usage) {
        set((s: any) => {
          const idx = s.messages.findIndex((m: ChatMessage) => m.id === streaming_message_id)
          if (idx === -1) return s
          const msgs = [...s.messages]
          msgs[idx] = { ...msgs[idx], token_usage: data.usage }
          return { messages: msgs }
        })
      }
      set({ sending: false, streaming_message_id: null, abort_controller: null })
    },
    onError: (data) => {
      const { streaming_message_id } = get()
      if (streaming_message_id) {
        set((s: any) => {
          const idx = s.messages.findIndex((m: ChatMessage) => m.id === streaming_message_id)
          if (idx === -1) return { sending: false, streaming_message_id: null, abort_controller: null }
          const msgs = [...s.messages]
          msgs[idx] = { ...msgs[idx], content: msgs[idx].content + `\n\n⚠ Error: ${data.error}` }
          return { messages: msgs, sending: false, streaming_message_id: null, abort_controller: null }
        })
      } else {
        addMessage({
          id: `err-${crypto.randomUUID()}`,
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
}

async function handleCrossRingAnalysis(
  analysisType: 'compare' | 'merge' | 'summary',
  ringNames: string[],
  question: string,
  addMessage: (msg: ChatMessage) => void,
  set: any,
  get: any
) {
  addMessage({
    id: `msg-${crypto.randomUUID()}`,
    role: 'user',
    sender_name: 'You',
    content: `/cross-ring-analysis ${analysisType} ${ringNames.join(',')}${question ? ' ' + question : ''}`,
    created_at: new Date().toISOString(),
  })

  set({ sending: true })

  const controller = streamChat('/api/super/cross-ring-analysis', { 
    ring_names: ringNames, 
    analysis_type: analysisType,
    question: question || undefined 
  }, {
    onStart: (data) => {
      const aiMsg: ChatMessage = {
        id: data.message_id,
        role: 'super_ring',
        sender_name: 'SUPER RING',
        content: '',
        created_at: new Date().toISOString(),
      }
      set((s: any) => ({
        messages: [...s.messages, aiMsg],
        streaming_message_id: data.message_id,
      }))
    },
    onDelta: (data) => {
      const { streaming_message_id } = get()
      if (!streaming_message_id) return
      set((s: any) => {
        const idx = s.messages.findIndex((m: ChatMessage) => m.id === streaming_message_id)
        if (idx === -1) return s
        const msgs = [...s.messages]
        msgs[idx] = { ...msgs[idx], content: msgs[idx].content + data.content }
        return { messages: msgs }
      })
    },
    onEnd: (data) => {
      const { streaming_message_id } = get()
      if (streaming_message_id && data.usage) {
        set((s: any) => {
          const idx = s.messages.findIndex((m: ChatMessage) => m.id === streaming_message_id)
          if (idx === -1) return s
          const msgs = [...s.messages]
          msgs[idx] = { ...msgs[idx], token_usage: data.usage }
          return { messages: msgs }
        })
      }
      set({ sending: false, streaming_message_id: null, abort_controller: null })
    },
    onError: (data) => {
      const { streaming_message_id } = get()
      if (streaming_message_id) {
        set((s: any) => {
          const idx = s.messages.findIndex((m: ChatMessage) => m.id === streaming_message_id)
          if (idx === -1) return { sending: false, streaming_message_id: null, abort_controller: null }
          const msgs = [...s.messages]
          msgs[idx] = { ...msgs[idx], content: msgs[idx].content + `\n\n⚠ Error: ${data.error}` }
          return { messages: msgs, sending: false, streaming_message_id: null, abort_controller: null }
        })
      } else {
        addMessage({
          id: `err-${crypto.randomUUID()}`,
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
}
