import { useState } from 'react'
import { SuperRingEntry } from '../sidebar/SuperRingEntry'
import { RingList } from '../sidebar/RingList'

export function Sidebar() {
  const [showPrompts, setShowPrompts] = useState(false)

  return (
    <>
      <div
        style={{
          width: 200,
          minWidth: 200,
          height: '100%',
          background: 'var(--bg-sidebar)',
          borderRight: '1px solid var(--border)',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
        }}
      >
        <SuperRingEntry />
        <div style={{ flex: 1, overflow: 'auto' }}>
          <RingList />
        </div>
        <div
          onClick={() => setShowPrompts(true)}
          style={{
            padding: '8px 12px',
            borderTop: '1px solid var(--border)',
            color: 'var(--text-dim)',
            fontSize: 10,
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            transition: 'color 0.15s',
          }}
          onMouseEnter={(e) => (e.currentTarget.style.color = 'var(--text-secondary)')}
          onMouseLeave={(e) => (e.currentTarget.style.color = 'var(--text-dim)')}
        >
          <span style={{ fontSize: 12 }}>⚙</span>
          <span>Prompts</span>
        </div>
      </div>
      {showPrompts && (
        <PromptsModal onClose={() => setShowPrompts(false)} />
      )}
    </>
  )
}

import { PromptsModal } from '../panels/PromptsPanel'
