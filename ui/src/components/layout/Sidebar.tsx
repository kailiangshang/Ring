import { SuperRingEntry } from '../sidebar/SuperRingEntry'
import { RingList } from '../sidebar/RingList'
import { usePanelStore } from '../../stores/panel-store'

export function Sidebar() {
  const openPanel = usePanelStore((s) => s.open)

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
      <div
        onClick={() => openPanel('prompts')}
        style={{
          padding: '8px 12px',
          borderTop: '1px solid var(--border)',
          color: 'var(--text-dim)',
          fontSize: 10,
          cursor: 'pointer',
          display: 'flex',
          alignItems: 'center',
          gap: 6,
        }}
      >
        <span style={{ fontSize: 12 }}>&#x2756;</span>
        <span>Prompts</span>
      </div>
    </div>
  )
}
