import { useState } from 'react'
import { useRingStore } from '../../stores/ring-store'
import { RingListItem } from './RingListItem'
import { SessionIndicator } from './SessionIndicator'
import { useAppStore } from '../../stores/app-store'
import { usePanelStore } from '../../stores/panel-store'

export function RingList() {
  const rings = useRingStore((s) => s.rings)
  const loading = useRingStore((s) => s.loading)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const createRing = useRingStore((s) => s.createRing)
  const setActiveRing = useAppStore((s) => s.setActiveRing)
  const selectRing = useRingStore((s) => s.selectRing)
  const openPanel = usePanelStore((s) => s.open)
  const [creating, setCreating] = useState(false)
  const [newName, setNewName] = useState('')

  const handleCreate = async () => {
    if (!newName.trim()) return
    const ring_id = await createRing(newName.trim(), `You are a ${newName.trim()} assistant`)
    setNewName('')
    setCreating(false)
    if (ring_id) {
      selectRing(ring_id)
      setActiveRing(ring_id)
      openPanel('graph')
    }
  }

  return (
    <div style={{ padding: '8px 0' }}>
      {loading && rings.length === 0 && (
        <div style={{ padding: '12px', color: 'var(--text-dim)', fontSize: 11 }}>Loading rings...</div>
      )}
      {!loading && rings.length === 0 && (
        <div style={{ padding: '12px', color: 'var(--text-dim)', fontSize: 11, textAlign: 'center' }}>
          No rings yet. Create one below.
        </div>
      )}
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

      {creating ? (
        <div style={{ padding: '8px 12px' }}>
          <input
            autoFocus
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleCreate()
              if (e.key === 'Escape') setCreating(false)
            }}
            placeholder="Ring name..."
            style={{
              width: '100%',
              background: 'var(--bg-input)',
              border: '1px solid var(--accent-cyan)',
              borderRadius: 3,
              padding: '5px 8px',
              color: 'var(--text-primary)',
              fontSize: 11,
              fontFamily: 'inherit',
              outline: 'none',
              marginBottom: 4,
            }}
          />
          <div style={{ display: 'flex', gap: 4 }}>
            <button
              onClick={handleCreate}
              style={{
                flex: 1,
                background: 'var(--accent-cyan)',
                color: 'var(--bg-base)',
                border: 'none',
                borderRadius: 3,
                padding: '4px 0',
                fontSize: 10,
                fontWeight: 700,
                cursor: 'pointer',
              }}
            >
              CREATE
            </button>
            <button
              onClick={() => setCreating(false)}
              style={{
                flex: 1,
                background: 'var(--bg-hover)',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '4px 0',
                fontSize: 10,
                cursor: 'pointer',
              }}
            >
              CANCEL
            </button>
          </div>
        </div>
      ) : (
        <div
          onClick={() => setCreating(true)}
          style={{
            margin: '4px 12px',
            padding: '6px 0',
            border: '1px solid var(--accent-cyan)',
            borderRadius: 3,
            textAlign: 'center',
            color: 'var(--accent-cyan)',
            fontSize: 10,
            cursor: 'pointer',
            fontWeight: 700,
          }}
        >
          + new ring
        </div>
      )}
    </div>
  )
}
