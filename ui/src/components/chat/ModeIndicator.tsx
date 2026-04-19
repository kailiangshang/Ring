import { useState, useEffect } from 'react'
import { useModeStore } from '../../stores/mode-store'
import { useRingStore } from '../../stores/ring-store'
import { useAppStore } from '../../stores/app-store'
import { ModeSelector } from './ModeSelector'

export function ModeIndicator() {
  const context = useAppStore((s) => s.current_context)
  const interaction_mode = useModeStore((s) => s.interaction_mode)
  const syncing = useModeStore((s) => s.syncing)
  const fetchFromServer = useModeStore((s) => s.fetchFromServer)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const [showSelector, setShowSelector] = useState(false)

  const label = context === 'super' ? 'super' : context === 'session' ? 'session' : 'ring'

  useEffect(() => {
    if (active_ring_id && context !== 'super') {
      fetchFromServer(active_ring_id)
    }
  }, [active_ring_id, fetchFromServer, context])

  return (
    <div style={{ position: 'relative' }}>
      <button
        onClick={() => setShowSelector(!showSelector)}
        style={{
          background: syncing ? 'var(--bg-active)' : 'var(--bg-hover)',
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
        [{label}
        {interaction_mode === 'auto' && context !== 'super' && (
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
