import { useParams, NavLink } from 'react-router-dom'

const RING_TABS = [
  { path: '', label: 'Chat' },
  { path: '/blueprint', label: 'Blueprint' },
  { path: '/graph', label: 'Graph' },
  { path: '/prs', label: 'PRs' },
  { path: '/members', label: 'Members' },
  { path: '/sessions', label: 'Sessions' },
]

export function RingNavBar() {
  const { ringId } = useParams<{ ringId: string }>()
  if (!ringId) return null

  return (
    <nav className="ring-navbar">
      <NavLink to="/" className="ring-back">
        &larr; Ring Group
      </NavLink>
      <div className="ring-tabs">
        {RING_TABS.map((tab) => (
          <NavLink
            key={tab.path}
            to={`/ring/${ringId}${tab.path}`}
            end={tab.path === ''}
            className={({ isActive }) =>
              isActive ? 'ring-tab active' : 'ring-tab'
            }
          >
            {tab.label}
          </NavLink>
        ))}
      </div>
    </nav>
  )
}
