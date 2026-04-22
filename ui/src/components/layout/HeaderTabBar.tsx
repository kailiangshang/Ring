import { usePanelStore, type PanelType } from '../../stores/panel-store'
import { useRingStore } from '../../stores/ring-store'
import { TabItem } from '../header/TabItem'
import { HeaderActions } from '../header/HeaderActions'
import { NotificationBell } from '../NotificationBell'

const TABS: { type: PanelType; label: string }[] = [
  { type: 'graph', label: 'Graph' },
  { type: 'archive', label: 'Archive' },
  { type: 'config', label: 'Config' },
]

export function HeaderTabBar() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const rings = useRingStore((s) => s.rings)
  const panels = usePanelStore((s) => s.panels)
  const toggle = usePanelStore((s) => s.toggle)
  const closeAll = usePanelStore((s) => s.closeAll)

  const activeRing = rings.find((r) => r.id === active_ring_id)
  if (!activeRing) return null

  return (
    <div
      style={{
        height: 38,
        background: 'var(--bg-panel)',
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        alignItems: 'center',
        padding: '0 12px',
      }}
    >
      <span
        style={{
          fontSize: 13,
          fontWeight: 700,
          color: 'var(--accent-ice)',
          marginRight: 16,
          whiteSpace: 'nowrap',
        }}
      >
        {activeRing.name}
      </span>

      <TabItem
        label="Chat"
        active={panels.length === 0}
        onClick={() => closeAll()}
      />

      {TABS.map((tab) => (
        <TabItem
          key={tab.type}
          label={tab.label}
          count={tab.type === 'graph' ? activeRing.node_count : undefined}
          active={panels.some((p) => p.type === tab.type)}
          onClick={() => toggle(tab.type)}
        />
      ))}

      <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 8 }}>
        <NotificationBell />
        <HeaderActions />
      </div>
    </div>
  )
}
