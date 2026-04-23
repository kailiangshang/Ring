import { useState } from 'react'
import type { Ring } from '../../types/ring'

interface RingListItemProps {
  ring: Ring
  isActive: boolean
}

export function RingListItem({ ring, isActive }: RingListItemProps) {
  const [hovered, setHovered] = useState(false)

  return (
    <div
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '8px 12px',
        cursor: 'pointer',
        background: isActive ? 'var(--bg-active)' : hovered ? 'var(--bg-hover)' : 'transparent',
        borderRadius: 4,
        margin: '2px 6px',
      }}
    >
      <span
        style={{
          color: isActive ? 'var(--accent-ice)' : 'var(--text-primary)',
          fontWeight: isActive ? 700 : 400,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
        }}
      >
        {ring.name}
      </span>
      <span style={{ marginLeft: 'auto', color: 'var(--text-dim)', fontSize: 11 }}>
        {ring.member_count}
      </span>
      {ring.has_active_session && (
        <span
          title="Active session"
          style={{
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: 'var(--accent-green)',
            flexShrink: 0,
          }}
        />
      )}
    </div>
  )
}
