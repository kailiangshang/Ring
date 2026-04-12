import { useParams, useLocation } from 'react-router-dom'
import './TabBar.css'

const TABS = [
  { path: '', label: 'Chat', icon: '💬' },
  { path: '/graph', label: 'Graph', icon: '◉' },
  { path: '/prs', label: 'PRs', icon: '📋' },
  { path: '/members', label: 'Members', icon: '👥' },
  { path: '/sessions', label: 'Sessions', icon: '🔍' },
]

export function TabBar() {
  const { ringId } = useParams<{ ringId: string }>()
  const location = useLocation()

  if (!ringId) return null

  return (
    <nav className="tab-bar">
      {TABS.map((tab) => {
        const to = `/ring/${ringId}${tab.path}`
        const is_active = tab.path === ''
          ? location.pathname === `/ring/${ringId}` || location.pathname === `/ring/${ringId}/`
          : location.pathname.startsWith(to)
        return (
          <a
            key={tab.path}
            href={to}
            className={`tab-bar-item${is_active ? ' tab-bar-item-active' : ''}`}
          >
            <span>{tab.icon}</span>
            <span>{tab.label}</span>
          </a>
        )
      })}
    </nav>
  )
}
