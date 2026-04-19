import { create } from 'zustand'
import { useAppStore } from '../../stores/app-store'

interface CommandDef {
  trigger: string
  cmd: string
  desc: string
  context: ('super' | 'ring' | 'session')[]
}

const COMMANDS: CommandDef[] = [
  { trigger: '/', cmd: 'help', desc: 'Show all commands', context: ['super', 'ring', 'session'] },
  { trigger: '/', cmd: 'graph', desc: 'Open graph panel', context: ['ring'] },
  { trigger: '/', cmd: 'archive', desc: 'Open archive panel', context: ['ring'] },
  { trigger: '/', cmd: 'config', desc: 'Open config panel', context: ['ring'] },
  { trigger: '/', cmd: 'session', desc: 'Open session panel', context: ['ring'] },
  { trigger: '/', cmd: 'new', desc: 'Create new ring', context: ['ring'] },
  { trigger: '/', cmd: 'save', desc: 'Archive conversation', context: ['ring', 'session'] },
  { trigger: '/', cmd: 'node', desc: 'Add graph node', context: ['ring'] },
  { trigger: '/', cmd: 'auto', desc: 'Toggle auto mode', context: ['ring'] },
  { trigger: '/', cmd: 'prefs', desc: 'Show/set preferences', context: ['super', 'ring'] },
  { trigger: '/', cmd: 'skill', desc: 'Manage skills', context: ['super', 'ring'] },
  { trigger: '/', cmd: 'mode', desc: 'Set interaction mode', context: ['ring'] },
  { trigger: '/', cmd: 'self', desc: 'Talk to Self', context: ['super', 'ring', 'session'] },
  { trigger: '!', cmd: 'graph', desc: 'Open graph panel', context: ['ring'] },
  { trigger: '!', cmd: 'archive', desc: 'Open archive panel', context: ['ring'] },
  { trigger: '!', cmd: 'config', desc: 'Open config panel', context: ['ring'] },
  { trigger: '!', cmd: 'session', desc: 'Open session panel', context: ['ring'] },
  { trigger: '!', cmd: 'new', desc: 'Create new ring', context: ['ring'] },
  { trigger: '!', cmd: 'save', desc: 'Archive conversation', context: ['ring', 'session'] },
  { trigger: '!', cmd: 'node', desc: 'Add graph node', context: ['ring'] },
  { trigger: '!', cmd: 'auto', desc: 'Toggle auto mode', context: ['ring'] },
  { trigger: '%', cmd: 'prefs', desc: 'Show/set preferences', context: ['super', 'ring'] },
  { trigger: '%', cmd: 'skill', desc: 'Manage skills', context: ['super', 'ring'] },
  { trigger: '%', cmd: 'mode', desc: 'Set interaction mode', context: ['ring'] },
  { trigger: '@', cmd: 'self', desc: 'Talk to Self', context: ['super', 'ring', 'session'] },
]

interface AutocompleteState {
  visible: boolean
  matches: CommandDef[]
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
    const trigger = trimmed.startsWith('/') ? '/' : trimmed.startsWith('!') ? '!' : trimmed.startsWith('%') ? '%' : trimmed.startsWith('@') ? '@' : null
    if (!trigger) {
      set({ visible: false, matches: [], selectedIndex: 0 })
      return
    }
    const partial = trimmed.slice(1).toLowerCase()
    if (partial.includes(' ')) {
      set({ visible: false, matches: [], selectedIndex: 0 })
      return
    }
    const context = useAppStore.getState().current_context
    const matches = COMMANDS.filter(
      (c) =>
        c.trigger === trigger &&
        c.cmd.startsWith(partial) &&
        c.context.includes(context as 'super' | 'ring' | 'session')
    )
    set({ visible: matches.length > 0, matches, selectedIndex: 0 })
  },

  moveUp: () => set((s) => ({ selectedIndex: Math.max(s.selectedIndex - 1, 0) })),
  moveDown: () => set((s) => ({ selectedIndex: Math.min(s.selectedIndex + 1, s.matches.length - 1) })),

  getSelected: () => {
    const { matches, selectedIndex } = get()
    const m = matches[selectedIndex]
    return m ? `${m.trigger}${m.cmd} ` : null
  },

  hide: () => set({ visible: false, matches: [], selectedIndex: 0 }),
}))

export function CommandAutocomplete({ onSelect }: { onSelect: (val: string) => void }) {
  const visible = useAutocompleteStore((s) => s.visible)
  const matches = useAutocompleteStore((s) => s.matches)
  const selectedIndex = useAutocompleteStore((s) => s.selectedIndex)

  if (!visible || matches.length === 0) return null

  return (
    <div
      style={{
        position: 'absolute',
        bottom: '100%',
        left: 0,
        right: 0,
        background: 'var(--bg-panel)',
        border: '1px solid var(--border)',
        borderRadius: '4px 4px 0 0',
        maxHeight: 200,
        overflow: 'auto',
        zIndex: 100,
      }}
    >
      {matches.map((cmd, i) => (
        <div
          key={`${cmd.trigger}${cmd.cmd}`}
          onMouseDown={(e) => {
            e.preventDefault()
            onSelect(`${cmd.trigger}${cmd.cmd} `)
            useAutocompleteStore.getState().hide()
          }}
          onMouseEnter={() => set_selected(i)}
          style={{
            padding: '6px 12px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            background: i === selectedIndex ? 'var(--bg-hover)' : 'transparent',
            fontSize: 12,
          }}
        >
          <span style={{ color: 'var(--accent-cyan)', fontWeight: 700, minWidth: 70 }}>
            {cmd.trigger}{cmd.cmd}
          </span>
          <span style={{ color: 'var(--text-muted)' }}>{cmd.desc}</span>
        </div>
      ))}
    </div>
  )
}

function set_selected(i: number) {
  useAutocompleteStore.setState({ selectedIndex: i })
}
