import { SuperRingEntry } from '../sidebar/SuperRingEntry'
import { RingList } from '../sidebar/RingList'

export function Sidebar() {
  return (
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
    </div>
  )
}
