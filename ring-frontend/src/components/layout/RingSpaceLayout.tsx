import { useState, useEffect, useCallback, createContext, useContext } from 'react'
import { Link, useParams, useNavigate } from 'react-router-dom'
import { RingLogo } from '../ui/RingLogo'
import { AvatarGroup } from '../ui/AvatarGroup'
import { NotificationBell } from '../ui/NotificationBell'
import type { NotificationItem } from '../ui/NotificationBell'
import { notification_to_item } from '../ui/NotificationBell'
import { FeaturePanelStrip } from './FeaturePanelStrip'
import { CenterView } from './CenterView'
import { FooterBar } from './FooterBar'
import { ExportPanel } from '../export/ExportPanel'
import { GraphView } from '../../pages/RingSpace/GraphView'
import { PrList } from '../../pages/RingSpace/PrList'
import { MemberList } from '../../components/member/MemberList'
import { SessionView } from '../../components/session/SessionView'
import type { ToolStatus } from '../toolbar/Toolbar'
import { useNotificationStore } from '../../stores/notificationStore'
import * as api from '../../api/client'
import './RingSpaceLayout.css'

const DEFAULT_TOOLS: ToolStatus[] = [
  { name: 'search', description: 'Search the knowledge graph', active: false },
  { name: 'text_clean', description: 'Clean and normalize text', active: false },
  { name: 'web_scrape', description: 'Extract text from web pages', active: false },
  { name: 'markdown_gen', description: 'Generate markdown documents', active: false },
  { name: 'privacy_filter', description: 'Filter sensitive information', active: false },
]

export type FeatureKey = 'graph' | 'prs' | 'members' | 'sessions'

const ToolsContext = createContext<{
  tools: ToolStatus[]
  active_tool_names: string[]
  toggle_tool: (name: string) => void
}>({ tools: [], active_tool_names: [], toggle_tool: () => {} })

export function useTools() { return useContext(ToolsContext) }

export function RingSpaceLayout() {
  const { ringId } = useParams<{ ringId: string }>()
  const navigate = useNavigate()
  const [ring_name, set_ring_name] = useState('Ring')
  const [member_names, set_member_names] = useState<string[]>([])
  const [tools, set_tools] = useState<ToolStatus[]>(DEFAULT_TOOLS)
  const [show_export, set_show_export] = useState(false)
  const [open_features, set_open_features] = useState<Set<FeatureKey>>(
    new Set(['graph', 'prs', 'members', 'sessions'])
  )

  const notifications = useNotificationStore((s) => s.notifications)
  const load_notifications = useNotificationStore((s) => s.load_notifications)
  const mark_read = useNotificationStore((s) => s.mark_read)

  const active_tool_names = tools.filter((t) => t.active).map((t) => t.name)
  const toggle_tool = (name: string) => {
    set_tools((prev) => prev.map((t) => (t.name === name ? { ...t, active: !t.active } : t)))
  }

  const toggle_feature = useCallback((key: FeatureKey) => {
    set_open_features((prev) => {
      const next = new Set(prev)
      if (next.has(key)) next.delete(key)
      else next.add(key)
      return next
    })
  }, [])

  const close_feature = useCallback((key: FeatureKey) => {
    set_open_features((prev) => {
      const next = new Set(prev)
      next.delete(key)
      return next
    })
  }, [])

  useEffect(() => {
    if (!ringId) return
    api.get_ring(ringId).then((ring) => set_ring_name(ring.name)).catch(() => {})
    api.list_members(ringId).then((members) => {
      set_member_names(members.map((m) => m.display_name))
    }).catch(() => {})
  }, [ringId])

  useEffect(() => { load_notifications() }, [load_notifications])

  const notif_items: NotificationItem[] = notifications.map(notification_to_item)
  const on_notif_click = useCallback((item: NotificationItem) => {
    mark_read(item.id)
    if (item.target_path) navigate(item.target_path)
  }, [mark_read, navigate])

  return (
    <ToolsContext.Provider value={{ tools, active_tool_names, toggle_tool }}>
      <div className="ring-space">
        <header className="ring-space-header">
          <Link to="/" className="ring-space-logo" title="Ring Hub">
            <RingLogo size={20} />
            <span>Ring</span>
          </Link>
          <span className="ring-space-divider" />
          <div className="ring-space-name">{ring_name}</div>
          <div className="ring-space-header-right">
            <AvatarGroup names={member_names} size="sm" />
            <button className="ring-space-icon-btn" title="导出" onClick={() => set_show_export(true)}>📤</button>
            <NotificationBell items={notif_items} on_click={on_notif_click} />
          </div>
        </header>

        <div className="ring-space-body">
          <FeaturePanelStrip open={open_features.has('graph')} title="图谱" on_close={() => close_feature('graph')}>
            <GraphView />
          </FeaturePanelStrip>

          <div className="ring-space-center">
            <CenterView />
          </div>

          <div className="ring-space-right-stack">
            <FeaturePanelStrip open={open_features.has('prs')} title="PRs" on_close={() => close_feature('prs')}>
              <PrList />
            </FeaturePanelStrip>
            <FeaturePanelStrip open={open_features.has('members')} title="成员" on_close={() => close_feature('members')}>
              <MemberList />
            </FeaturePanelStrip>
          </div>
        </div>

        {open_features.has('sessions') && (
          <div className="ring-space-bottom-strip">
            <div className="ring-space-bottom-header">
              <span className="ring-space-bottom-title">Sessions</span>
              <button className="ring-space-bottom-close" onClick={() => close_feature('sessions')}>✕</button>
            </div>
            <div className="ring-space-bottom-body">
              <SessionView />
            </div>
          </div>
        )}

        <FooterBar
          tools={tools}
          on_tool_toggle={toggle_tool}
          show_tools={true}
          open_features={open_features}
          on_feature_toggle={toggle_feature}
        />
        {show_export && ringId && <ExportPanel ring_id={ringId} on_close={() => set_show_export(false)} />}
      </div>
    </ToolsContext.Provider>
  )
}
