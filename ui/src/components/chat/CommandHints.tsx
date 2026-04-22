import { useAppStore } from '../../stores/app-store'
import { usePanelStore, type PanelType } from '../../stores/panel-store'
import { useSelfStore } from '../../stores/self-store'

interface HintItem {
  label: string
  action: PanelType | null
  selfAction?: boolean
  context: ('super' | 'ring' | 'session')[]
}

const HINTS: HintItem[] = [
  { label: '/graph', action: 'graph', context: ['ring'] },
  { label: '/archive', action: 'archive', context: ['ring'] },
  { label: '/config', action: 'config', context: ['ring'] },
  { label: '/session', action: 'session', context: ['ring'] },
  { label: '/skills', action: 'super_skills', context: ['super'] },
  { label: '/prefs', action: 'super_settings', context: ['super'] },
  { label: '/save', action: null, context: ['session'] },
  { label: '@self', action: null, selfAction: true, context: ['super', 'ring', 'session'] },
]

export function CommandHints() {
  const context = useAppStore((s) => s.current_context)
  const toggle = usePanelStore((s) => s.toggle)
  const toggleSelf = useSelfStore((s) => s.toggle)

  const visible = HINTS.filter((h) => h.context.includes(context as 'super' | 'ring' | 'session'))

  return (
    <div
      style={{
        display: 'flex',
        gap: 12,
        padding: '4px 16px 8px',
        color: 'var(--text-dim)',
        fontSize: 11,
      }}
    >
      {visible.map((hint) => (
        <button
          key={hint.label}
          onClick={() => {
            if (hint.selfAction) {
              toggleSelf()
            } else if (hint.action) {
              toggle(hint.action)
            }
          }}
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--text-dim)',
            cursor: 'pointer',
            fontSize: 11,
            padding: 0,
          }}
        >
          {hint.label}
        </button>
      ))}
    </div>
  )
}
