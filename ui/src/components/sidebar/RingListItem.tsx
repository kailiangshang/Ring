import type { Ring } from '../../types/ring'
import { useRingStore } from '../../stores/ring-store'
import { useAppStore } from '../../stores/app-store'

interface RingListItemProps {
  ring: Ring
}

export function RingListItem({ ring }: RingListItemProps) {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const selectRing = useRingStore((s) => s.selectRing)
  const setActiveRing = useAppStore((s) => s.setActiveRing)
  const isActive = active_ring_id === ring.id

  return (
    <div
      onClick={() => {
        selectRing(ring.id)
        setActiveRing(ring.id)
      }}
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
