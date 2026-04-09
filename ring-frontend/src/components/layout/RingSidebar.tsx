import { useParams, NavLink, useLocation } from 'react-router-dom'
import './RingSidebar.css'

interface RingSidebarProps { collapsed: boolean; on_toggle: () => void }

const NAV_ITEMS = [
  { path: '', label: 'Chat', icon: '💬' },
  { path: '/graph', label: 'Graph', icon: '◉' },
  { path: '/prs', label: 'PRs', icon: '📋' },
  { path: '/members', label: 'Members', icon: '👥' },
  { path: '/sessions', label: 'Sessions', icon: '🔍' },
]

export function RingSidebar({ collapsed, on_toggle }: RingSidebarProps) {
  const { ringId } = useParams<{ ringId: string }>()
  const location = useLocation()
  if (!ringId) return null

  return (
    <div className={`ring-sidebar${collapsed ? ' ring-sidebar-collapsed' : ''}`}>
      <div className="ring-sidebar-tree">
        {!collapsed && <div className="ring-sidebar-placeholder">图谱节点树（待数据接入）</div>}
      </div>
      <div className="ring-sidebar-divider" />
      <div className="ring-sidebar-nav">
        {NAV_ITEMS.map((item) => {
          const to = `/ring/${ringId}${item.path}`
          const is_active = item.path === ''
            ? location.pathname === `/ring/${ringId}` || location.pathname === `/ring/${ringId}/`
            : location.pathname.startsWith(to)
          return (
            <NavLink key={item.path} to={to} end={item.path === ''} className={`ring-sidebar-nav-item${is_active ? ' sidebar-active' : ''}`} title={collapsed ? item.label : undefined}>
              <span>{collapsed ? item.icon : item.label}</span>
            </NavLink>
          )
        })}
      </div>
      <button className="ring-sidebar-collapse-btn" onClick={on_toggle}>
        {collapsed ? '→' : '← 收起'}
      </button>
    </div>
  )
}
