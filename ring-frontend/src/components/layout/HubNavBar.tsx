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
      <NavLink to="/" className="hub-navbar-brand">
        <svg width="24" height="24" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
          <path d="M 2 14 C 16 8, 32 22, 44 38 C 52 50, 62 52, 72 48 C 76 46, 78 44, 78 44" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" opacity="0.75"/>
          <path d="M 2 66 C 14 58, 28 46, 42 40 C 52 36, 62 40, 70 44 C 75 46, 78 44, 78 44" stroke="currentColor" stroke-width="4" stroke-linecap="round"/>
          <path d="M 2 40 C 14 28, 30 26, 42 34 C 54 42, 60 46, 70 43 C 75 43, 78 44, 78 44" stroke="currentColor" stroke-width="2.8" stroke-linecap="round" opacity="0.9"/>
        </svg>
        <span>Ring</span>
      </NavLink>
      <div className="hub-navbar-links">
        <NavLink to="/" end className={({ isActive }) => `hub-nav-link${isActive ? ' hub-nav-active' : ''}`}>Ring Hub</NavLink>
        <NavLink to="/ring-super" className={({ isActive }) => `hub-nav-link${isActive ? ' hub-nav-active' : ''}`}>Ring Super</NavLink>
      </div>
      <div className="hub-navbar-right">
        <NotificationBell items={notifications} on_click={on_notification_click || (() => {})} />
        <NavLink to="/settings" className={({ isActive }) => `hub-nav-link${isActive ? ' hub-nav-active' : ''}`}>Settings</NavLink>
      </div>
    </nav>
  )
}
