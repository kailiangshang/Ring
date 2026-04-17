import { useRingStore } from '../../stores/ring-store'
import { RingListItem } from './RingListItem'
import { SessionIndicator } from './SessionIndicator'

export function RingList() {
  const rings = useRingStore((s) => s.rings)
  const active_ring_id = useRingStore((s) => s.active_ring_id)

  return (
    <div style={{ padding: '8px 0' }}>
      {rings.map((ring) => (
        <div key={ring.id}>
          <RingListItem ring={ring} />
          {ring.id === active_ring_id && ring.has_active_session && (
            <SessionIndicator />
          )}
        </div>
      ))}
    </div>
  )
}
