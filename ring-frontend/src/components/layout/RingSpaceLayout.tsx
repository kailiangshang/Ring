import { useState, useEffect, createContext, useContext } from 'react'
import { Outlet, Link, useParams } from 'react-router-dom'
import { AvatarGroup } from '../ui/AvatarGroup'
import { NotificationBell } from '../ui/NotificationBell'
import { RingSidebar } from './RingSidebar'
import { RightPanel } from './RightPanel'
import * as api from '../../api/client'
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
  const { ringId } = useParams<{ ringId: string }>()
  const [panel, set_panel] = useState<RightPanelState>({ open: false, content: null, data: null })
  const [sidebar_collapsed, set_sidebar_collapsed] = useState(false)
  const [ring_name, set_ring_name] = useState('Ring')
  const [member_names, set_member_names] = useState<string[]>([])

  useEffect(() => {
    if (!ringId) return
    api.get_ring(ringId).then((ring) => set_ring_name(ring.name)).catch(() => {})
    api.list_members(ringId).then((members) => {
      set_member_names(members.map((m) => m.display_name))
    }).catch(() => {})
  }, [ringId])

  return (
    <RightPanelContext.Provider value={{ panel, set_panel }}>
      <div className="ring-space">
        <div className="ring-space-header">
          <Link to="/" className="ring-space-back">&larr; Hub</Link>
          <div className="ring-space-name">{ring_name}</div>
          <div className="ring-space-header-right">
            <AvatarGroup names={member_names} size="sm" />
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
