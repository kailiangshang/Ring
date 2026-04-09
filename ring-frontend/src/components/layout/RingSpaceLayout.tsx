import { useState, createContext, useContext } from 'react'
import { Outlet, Link } from 'react-router-dom'
import { AvatarGroup } from '../ui/AvatarGroup'
import { NotificationBell } from '../ui/NotificationBell'
import { RingSidebar } from './RingSidebar'
import { RightPanel } from './RightPanel'
import './RingSpaceLayout.css'

interface RightPanelState {
  open: boolean
  content: 'node_detail' | 'diff' | 'node_selector' | null
  data: unknown
}

const RightPanelContext = createContext<{
  panel: RightPanelState
  set_panel: (s: RightPanelState) => void
}>({ panel: { open: false, content: null, data: null }, set_panel: () => {} })

export function useRightPanel() { return useContext(RightPanelContext) }

export function RingSpaceLayout() {
  const [panel, set_panel] = useState<RightPanelState>({ open: false, content: null, data: null })
  const [sidebar_collapsed, set_sidebar_collapsed] = useState(false)

  return (
    <RightPanelContext.Provider value={{ panel, set_panel }}>
      <div className="ring-space">
        <div className="ring-space-header">
          <Link to="/" className="ring-space-back">&larr; Hub</Link>
          <div className="ring-space-name">Ring</div>
          <div className="ring-space-header-right">
            <AvatarGroup names={[]} size="sm" />
            <NotificationBell items={[]} on_click={() => {}} />
          </div>
        </div>
        <div className="ring-space-body">
          <RingSidebar collapsed={sidebar_collapsed} on_toggle={() => set_sidebar_collapsed(!sidebar_collapsed)} />
          <div className="ring-space-main">
            <Outlet />
          </div>
          {panel.open && <RightPanel state={panel} on_close={() => set_panel({ open: false, content: null, data: null })} />}
        </div>
      </div>
    </RightPanelContext.Provider>
  )
}
