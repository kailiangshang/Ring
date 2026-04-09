import { Outlet } from 'react-router-dom'
import { HubNavBar } from './HubNavBar'
import './AppShell.css'

export function AppShell() {
  return (
    <div className="app-shell">
      <HubNavBar />
      <div className="app-shell-body">
        <Outlet />
      </div>
    </div>
  )
}
