import { useState } from 'react'
import { useModeStore } from '../../stores/mode-store'
import { ModeSelector } from './ModeSelector'

export function ModeIndicator() {
  const interaction_mode = useModeStore((s) => s.interaction_mode)
  const [showSelector, setShowSelector] = useState(false)

  return (
    <div style={{ position: 'relative' }}>
      <button
        onClick={() => setShowSelector(!showSelector)}
        style={{
          background: 'var(--bg-hover)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '6px 10px',
          color: 'var(--text-secondary)',
          fontSize: 11,
          cursor: 'pointer',
          fontWeight: 700,
          whiteSpace: 'nowrap',
          display: 'flex',
          alignItems: 'center',
          gap: 4,
        }}
      >
        [ring
        {interaction_mode === 'auto' && (
          <span style={{ color: 'var(--accent-amber)' }}>·auto</span>
        )}
        ]
      </button>
      {showSelector && (
        <ModeSelector onClose={() => setShowSelector(false)} />
      )}
    </div>
  )
}
