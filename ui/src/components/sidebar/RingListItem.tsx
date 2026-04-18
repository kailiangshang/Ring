import type { Ring } from '../../types/ring'

interface RingListItemProps {
  ring: Ring
  isActive: boolean
}

export function RingListItem({ ring, isActive }: RingListItemProps) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '8px 12px',
        cursor: 'pointer',
        background: isActive ? 'var(--bg-active)' : 'transparent',
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
            width: 6,
            height: 6,
            borderRadius: '50%',
            background: 'var(--accent-green)',
            flexShrink: 0,
          }}
        />
      )}
    </div>
  )
}
