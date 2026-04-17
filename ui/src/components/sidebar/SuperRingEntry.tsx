import { useAppStore } from '../../stores/app-store'

export function SuperRingEntry() {
  const { current_context, setContext, setActiveRing } = useAppStore()
  const isActive = current_context === 'super'

  return (
    <div
      onClick={() => {
        setActiveRing(null)
        setContext('super')
      }}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 10,
        padding: '10px 12px',
        cursor: 'pointer',
        background: isActive ? 'var(--bg-active)' : 'transparent',
        borderBottom: '1px solid var(--border)',
      }}
    >
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: 6,
          background: 'linear-gradient(135deg, #0891B2, #67E8F9)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 14,
          fontWeight: 700,
          color: '#06080c',
          flexShrink: 0,
        }}
      >
        R
      </div>
      <span
        style={{
          color: isActive ? 'var(--accent-ice)' : 'var(--text-primary)',
          fontWeight: isActive ? 700 : 400,
          fontSize: 12,
          letterSpacing: '0.05em',
        }}
      >
        Super Ring
      </span>
    </div>
  )
}
