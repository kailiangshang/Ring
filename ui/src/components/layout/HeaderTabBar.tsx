import { usePanelStore, type PanelType } from '../../stores/panel-store'
import { useRingStore } from '../../stores/ring-store'
import { useSessionStore } from '../../stores/session-store'
import { TabItem } from '../header/TabItem'
import { HeaderActions } from '../header/HeaderActions'
import { NotificationBell } from '../NotificationBell'
import { ExportButton } from '../chat/ExportButton'

const TABS: { type: PanelType; label: string }[] = [
  { type: 'session', label: 'Session' },
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
  const active_session = useSessionStore((s) => s.active_session)

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
          icon={tab.type === 'session' && active_session ? (
            <span style={{
              display: 'inline-block',
              width: 6,
              height: 6,
              borderRadius: '50%',
              background: active_session.phase === 'closed' ? 'var(--accent-amber)' : 'var(--accent-green)',
            }} />
          ) : undefined}
        />
      ))}

      <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 8 }}>
        <ExportButton />
        <NotificationBell />
        <HeaderActions />
      </div>
    </div>
  )
}
