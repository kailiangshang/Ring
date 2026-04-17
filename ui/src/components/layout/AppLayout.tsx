import { Sidebar } from './Sidebar'
import { HeaderTabBar } from './HeaderTabBar'
import { PanelStack } from './PanelStack'
import { ChatArea } from '../chat/ChatArea'
import { SelfFloat } from '../self/SelfFloat'
import { SelfTrigger } from '../self/SelfTrigger'
import { useAppStore } from '../../stores/app-store'

export function AppLayout() {
  const current_context = useAppStore((s) => s.current_context)

  return (
    <div style={{ display: 'flex', height: '100%', width: '100%' }}>
      <Sidebar />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        {current_context !== 'super' && <HeaderTabBar />}
        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
          <ChatArea />
          <PanelStack />
        </div>
      </div>
      <SelfFloat />
      <SelfTrigger />
    </div>
  )
}
