import { usePanelStore, type PanelType } from '../../stores/panel-store'
import { useSelfStore } from '../../stores/self-store'

const HINTS: { label: string; action: PanelType | null }[] = [
  { label: '!graph', action: 'graph' },
  { label: '!archive', action: 'archive' },
  { label: '!config', action: 'config' },
  { label: '!session', action: 'session' },
  { label: '@self', action: null },
]

export function CommandHints() {
  const toggle = usePanelStore((s) => s.toggle)
  const toggleSelf = useSelfStore((s) => s.toggle)

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
      {HINTS.map((hint) => (
        <button
          key={hint.label}
          onClick={() => {
            if (hint.action) {
              toggle(hint.action)
            } else {
              toggleSelf()
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
