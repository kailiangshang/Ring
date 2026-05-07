import { memo, type ReactNode } from 'react'

interface TabItemProps {
  label: string
  count?: number
  active: boolean
  onClick: () => void
  icon?: ReactNode
}

export const TabItem = memo(function TabItem({ label, count, active, onClick, icon }: TabItemProps) {
  return (
    <button
      onClick={onClick}
      style={{
        background: 'none',
        border: 'none',
        color: active ? 'var(--accent-ice)' : 'var(--text-muted)',
        fontSize: 12,
        fontWeight: active ? 700 : 400,
        cursor: 'pointer',
        padding: '8px 12px',
        display: 'flex',
        alignItems: 'center',
        gap: 4,
        borderBottom: active ? '2px solid var(--accent-cyan)' : '2px solid transparent',
        letterSpacing: '0.03em',
      }}
    >
      {icon}
      {label}
      {count !== undefined && (
        <span style={{ fontSize: 10, color: 'var(--text-dim)', marginLeft: 2 }}>
          {count}
        </span>
      )}
    </button>
  )
})
