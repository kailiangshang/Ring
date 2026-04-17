import { Sidebar } from './Sidebar'
import { HeaderTabBar } from './HeaderTabBar'
import { PanelStack } from './PanelStack'
import { ChatArea } from '../chat/ChatArea'
import { SelfFloat } from '../self/SelfFloat'
import { SelfTrigger } from '../self/SelfTrigger'
import { useAppStore } from '../../stores/app-store'


function SuperRingHeader() {
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
          letterSpacing: '0.05em',
        }}
      >
        Super Ring
      </span>
      <span
        style={{
          marginLeft: 12,
          fontSize: 11,
          color: 'var(--text-dim)',
        }}
      >
        Global Assistant
      </span>
    </div>
  )
}

export function AppLayout() {
  const current_context = useAppStore((s) => s.current_context)

  return (
    <div style={{ display: 'flex', height: '100%', width: '100%' }}>
      <Sidebar />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        {current_context === 'super' ? (
          <>
            <SuperRingHeader />
            <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
              <ChatArea />
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
    </div>
  )
}
