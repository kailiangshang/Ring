import type { ReactNode } from 'react'

interface PanelWrapperProps {
  title: string
  depth: number
  onClose: () => void
  children: ReactNode
}

export function PanelWrapper({ title, depth, onClose, children }: PanelWrapperProps) {
  const bgColors = ['var(--bg-panel)', '#0b1018', '#0c1220']
  const bg = bgColors[Math.min(depth - 1, 2)]

  return (
    <div
      style={{
        width: 320,
        minWidth: 320,
        height: '100%',
        background: bg,
        borderLeft: '1px solid var(--border)',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '10px 12px',
          borderBottom: '1px solid var(--border)',
        }}
      >
        <span style={{ fontSize: 12, fontWeight: 700, letterSpacing: '0.05em' }}>
          {title}
        </span>
        <button
          onClick={onClose}
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--text-muted)',
            cursor: 'pointer',
            fontSize: 14,
            padding: '0 4px',
          }}
        >
          ×
        </button>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: 12 }}>{children}</div>
    </div>
  )
}
