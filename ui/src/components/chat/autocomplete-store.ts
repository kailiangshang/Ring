import { create } from 'zustand'
import { useAppStore } from '../../stores/app-store'

interface CommandDef {
  trigger: string
  cmd: string
  subcommands?: string[]
  desc: string
  context: ('super' | 'ring' | 'session')[]
}

const COMMANDS: CommandDef[] = [
  { trigger: '/', cmd: 'help', desc: 'Show all commands', context: ['super', 'ring', 'session'] },
  { trigger: '/', cmd: 'graph', desc: 'Open graph panel', context: ['ring'] },
  { trigger: '/', cmd: 'archive', desc: 'Open archive panel', context: ['ring'] },
  { trigger: '/', cmd: 'config', desc: 'Open config panel', context: ['ring'] },
  { trigger: '/', cmd: 'session', subcommands: ['create', 'close', 'start', 'summarize'], desc: 'Session operations', context: ['ring', 'session'] },
  { trigger: '/', cmd: 'new', desc: 'Create new ring', context: ['ring'] },
  { trigger: '/', cmd: 'save', desc: 'Archive conversation', context: ['ring', 'session'] },
  { trigger: '/', cmd: 'node', subcommands: ['add', 'link'], desc: 'Graph node operations', context: ['ring'] },
  { trigger: '/', cmd: 'mode', desc: 'Set interaction mode', context: ['ring'] },
  { trigger: '/', cmd: 'prefs', subcommands: ['set'], desc: 'Show/set preferences', context: ['super', 'ring'] },
  { trigger: '/', cmd: 'skill', subcommands: ['list', 'install', 'remove'], desc: 'Manage skills', context: ['super', 'ring'] },
  { trigger: '/', cmd: 'members', desc: 'Show members', context: ['ring'] },
  { trigger: '/', cmd: 'invite', subcommands: ['open', 'audit'], desc: 'Create invite', context: ['ring'] },
  { trigger: '/', cmd: 'cross-ring-query', desc: '跨 Ring 搜索知识', context: ['super', 'ring'] },
  { trigger: '/', cmd: 'cross-ring-analysis', desc: '跨 Ring 分析对比', context: ['super'] },
  { trigger: '@', cmd: 'self', desc: 'Talk to Self', context: ['super', 'ring', 'session'] },
  { trigger: '@', cmd: 'ring', desc: 'Talk to Ring AI', context: ['ring'] },
  { trigger: '@', cmd: 'super', desc: 'Talk to Super Ring', context: ['super', 'ring'] },
  { trigger: '@', cmd: 'node', desc: 'Reference node', context: ['ring'] },
]

export interface CommandMatch {
  trigger: string
  cmd: string
  subcommand?: string
  desc: string
}

interface AutocompleteState {
  visible: boolean
  matches: CommandMatch[]
  selectedIndex: number
  update: (input: string) => void
  moveUp: () => void
  moveDown: () => void
  getSelected: () => string | null
  hide: () => void
}

export const useAutocompleteStore = create<AutocompleteState>((set, get) => ({
  visible: false,
  matches: [],
  selectedIndex: 0,

  update: (input: string) => {
    const trimmed = input.trimStart()
    const trigger = trimmed.startsWith('/') ? '/' : trimmed.startsWith('@') ? '@' : null
    if (!trigger) {
      set({ visible: false, matches: [], selectedIndex: 0 })
      return
    }

    const afterTrigger = trimmed.slice(1)
    const parts = afterTrigger.split(/\s+/)
    const partial = parts[0]?.toLowerCase() || ''

    const hasSpace = afterTrigger.includes(' ')
    const parentCmd = hasSpace ? partial : null
    const subPartial = hasSpace ? (parts[1]?.toLowerCase() || '') : ''

    const context = useAppStore.getState().current_context

    if (hasSpace && parentCmd) {
      const parent = COMMANDS.find(c => c.trigger === trigger && c.cmd === parentCmd && c.subcommands)
      if (parent?.subcommands) {
        const matches: CommandMatch[] = parent.subcommands
          .filter(sc => sc.startsWith(subPartial))
          .map(sc => ({
            trigger: parent.trigger,
            cmd: parent.cmd,
            subcommand: sc,
            desc: parent.desc,
          }))
        set({ visible: matches.length > 0, matches, selectedIndex: 0 })
        return
      }
    }

    if (afterTrigger.includes(' ')) {
      set({ visible: false, matches: [], selectedIndex: 0 })
      return
    }

    const matches: CommandMatch[] = COMMANDS.filter(
      (c) =>
        c.trigger === trigger &&
        c.cmd.startsWith(partial) &&
        c.context.includes(context as 'super' | 'ring' | 'session')
    ).map(c => ({
      trigger: c.trigger,
      cmd: c.cmd,
      desc: c.desc,
    }))
    set({ visible: matches.length > 0, matches, selectedIndex: 0 })
  },

  moveUp: () => set((s) => ({ selectedIndex: Math.max(s.selectedIndex - 1, 0) })),
  moveDown: () => set((s) => ({ selectedIndex: Math.min(s.selectedIndex + 1, s.matches.length - 1) })),

  getSelected: () => {
    const { matches, selectedIndex } = get()
    const m = matches[selectedIndex]
    if (!m) return null
    if (m.subcommand) {
      return `${m.trigger}${m.cmd} ${m.subcommand} `
    }
    return `${m.trigger}${m.cmd} `
  },

  hide: () => set({ visible: false, matches: [], selectedIndex: 0 }),
}))
