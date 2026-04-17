import { useModeStore } from '../../stores/mode-store'

export function HeaderActions() {
  const { interaction_mode, toggleAuto } = useModeStore()

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginLeft: 'auto' }}>
      <button
        onClick={toggleAuto}
        style={{
          background: interaction_mode === 'auto' ? 'var(--accent-amber)' : 'var(--bg-hover)',
          color: interaction_mode === 'auto' ? 'var(--bg-base)' : 'var(--text-secondary)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '4px 10px',
          fontSize: 11,
          fontWeight: 700,
          cursor: 'pointer',
          letterSpacing: '0.05em',
        }}
      >
        AUTO
      </button>
    </div>
  )
}
