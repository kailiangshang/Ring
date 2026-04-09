import { NavLink } from 'react-router-dom'
import { NotificationBell } from '../ui/NotificationBell'
import type { NotificationItem } from '../ui/NotificationBell'
import './HubNavBar.css'

interface HubNavBarProps {
  notifications?: NotificationItem[]
  on_notification_click?: (item: NotificationItem) => void
}

export function HubNavBar({ notifications = [], on_notification_click }: HubNavBarProps) {
  return (
    <nav className="hub-navbar">
      <NavLink to="/" className="hub-navbar-brand">Ring</NavLink>
      <div className="hub-navbar-links">
        <NavLink to="/" end className={({ isActive }) => `hub-nav-link${isActive ? ' hub-nav-active' : ''}`}>Ring Hub</NavLink>
        <NavLink to="/super-ring" className={({ isActive }) => `hub-nav-link${isActive ? ' hub-nav-active' : ''}`}>Super Ring</NavLink>
      </div>
      <div className="hub-navbar-right">
        <NotificationBell items={notifications} on_click={on_notification_click || (() => {})} />
        <NavLink to="/settings" className={({ isActive }) => `hub-nav-link${isActive ? ' hub-nav-active' : ''}`}>Settings</NavLink>
      </div>
    </nav>
  )
}
