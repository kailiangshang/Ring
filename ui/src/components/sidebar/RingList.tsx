import { useEffect } from 'react'
import { useRingStore } from '../../stores/ring-store'
import { RingListItem } from './RingListItem'
import { SessionIndicator } from './SessionIndicator'
import { useAppStore } from '../../stores/app-store'

export function RingList() {
  const rings = useRingStore((s) => s.rings)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const fetchRings = useRingStore((s) => s.fetchRings)
  const setActiveRing = useAppStore((s) => s.setActiveRing)
  const selectRing = useRingStore((s) => s.selectRing)

  useEffect(() => {
    fetchRings()
  }, [fetchRings])

  if (rings.length === 0) {
    return (
      <div style={{ padding: '12px', color: 'var(--text-dim)', fontSize: 11 }}>
        No rings yet. Use !new to create one.
      </div>
    )
  }

  return (
    <div style={{ padding: '8px 0' }}>
      {rings.map((ring) => (
        <div key={ring.id}>
          <div onClick={() => { selectRing(ring.id); setActiveRing(ring.id) }}>
            <RingListItem ring={ring} isActive={active_ring_id === ring.id} />
          </div>
          {ring.id === active_ring_id && ring.has_active_session && (
            <SessionIndicator />
          )}
        </div>
      ))}
    </div>
  )
}
