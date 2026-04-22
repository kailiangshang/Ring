import { useEffect } from 'react'
import { Sidebar } from './Sidebar'
import { HeaderTabBar } from './HeaderTabBar'
import { PanelStack } from './PanelStack'
import { ChatArea } from '../chat/ChatArea'
import { SelfFloat } from '../self/SelfFloat'
import { SelfTrigger } from '../self/SelfTrigger'
import { CreateInviteModal } from '../invite/CreateInviteModal'
import { NotificationBell } from '../NotificationBell'
import { ExportButton } from '../chat/ExportButton'
import { useAppStore } from '../../stores/app-store'
import { useRingStore } from '../../stores/ring-store'
import { useChatStore } from '../../stores/chat-store'
import { usePanelStore } from '../../stores/panel-store'
import { TabItem } from '../header/TabItem'

function SuperRingHeader() {
  const panels = usePanelStore((s) => s.panels)
  const toggle = usePanelStore((s) => s.toggle)
  const closeAll = usePanelStore((s) => s.closeAll)

  return (
    <div style={{
      height: 38,
      background: 'var(--bg-panel)',
      borderBottom: '1px solid var(--border)',
      display: 'flex',
      alignItems: 'center',
      padding: '0 12px',
    }}>
      <span style={{
        fontSize: 13,
        fontWeight: 700,
        color: 'var(--accent-ice)',
        marginRight: 16,
        letterSpacing: '0.05em',
      }}>
        Super Ring
      </span>
      <TabItem
        label="Chat"
        active={panels.length === 0 || panels.every(p => p.type !== 'super_skills' && p.type !== 'super_settings')}
        onClick={() => {
          closeAll()
        }}
      />
      <TabItem
        label="Skills"
        active={panels.some((p) => p.type === 'super_skills')}
        onClick={() => toggle('super_skills')}
      />
      <TabItem
        label="Settings"
        active={panels.some((p) => p.type === 'super_settings')}
        onClick={() => toggle('super_settings')}
      />
      <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 8 }}>
        <ExportButton />
        <NotificationBell />
      </div>
    </div>
  )
}

export function AppLayout() {
  const current_context = useAppStore((s) => s.current_context)
  const fetchRings = useRingStore((s) => s.fetchRings)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const loadHistory = useChatStore((s) => s.loadHistory)

  useEffect(() => {
    fetchRings()
  }, [fetchRings])

  useEffect(() => {
    if (active_ring_id) {
      loadHistory()
    }
  }, [active_ring_id, loadHistory])

  return (
    <div style={{ display: 'flex', height: '100%', width: '100%' }}>
      <Sidebar />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        {current_context === 'super' ? (
          <>
            <SuperRingHeader />
            <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
              <ChatArea />
              <PanelStack />
            </div>
          </>
        ) : (
          <>
            <HeaderTabBar />
            <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
              <ChatArea />
              <PanelStack />
            </div>
          </>
        )}
      </div>
      <SelfFloat />
      <SelfTrigger />
      <CreateInviteModal />
    </div>
  )
}
