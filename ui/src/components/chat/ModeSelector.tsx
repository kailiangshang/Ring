import { useModeStore } from '../../stores/mode-store'

interface ModeSelectorProps {
  onClose: () => void
}

export function ModeSelector({ onClose }: ModeSelectorProps) {
  const { interaction_mode, setInteractionMode, skill_permission_mode, setSkillMode } =
    useModeStore()

  return (
    <div
      style={{
        position: 'absolute',
        bottom: '100%',
        left: 0,
        marginBottom: 4,
        background: 'var(--bg-panel)',
        border: '1px solid var(--border)',
        borderRadius: 4,
        padding: 8,
        minWidth: 200,
        zIndex: 100,
      }}
    >
      <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4, letterSpacing: '0.05em' }}>
        Interaction Mode
      </div>
      {(['normal', 'auto'] as const).map((mode) => (
        <label
          key={mode}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            padding: '4px',
            cursor: 'pointer',
            color: interaction_mode === mode ? 'var(--accent-ice)' : 'var(--text-primary)',
          }}
        >
          <input
            type="radio"
            name="interaction_mode"
            checked={interaction_mode === mode}
            onChange={() => {
              setInteractionMode(mode)
              onClose()
            }}
            style={{ accentColor: 'var(--accent-cyan)' }}
          />
          <span style={{ fontSize: 12 }}>{mode === 'normal' ? 'Normal' : 'Auto'}</span>
        </label>
      ))}

      <div style={{ fontSize: 10, color: 'var(--text-dim)', marginTop: 8, marginBottom: 4, letterSpacing: '0.05em' }}>
        Skill Permission
      </div>
      <div style={{ display: 'flex', gap: 4 }}>
        {(['auto', 'plan', 'edit'] as const).map((mode) => (
          <button
            key={mode}
            onClick={() => {
              setSkillMode(mode)
              onClose()
            }}
            style={{
              background: skill_permission_mode === mode ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: skill_permission_mode === mode ? 'var(--bg-base)' : 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 3,
              padding: '2px 8px',
              fontSize: 11,
              cursor: 'pointer',
            }}
          >
            {mode}
          </button>
        ))}
      </div>
    </div>
  )
}
