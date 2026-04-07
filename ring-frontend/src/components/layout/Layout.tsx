import { Outlet, useLocation } from 'react-router-dom'
import { NavBar } from './NavBar'
import { RingNavBar } from './RingNavBar'

export function Layout() {
  const location = useLocation()
  const is_ring_page = location.pathname.startsWith('/ring/')

  return (
    <>
      <NavBar />
      {is_ring_page && <RingNavBar />}
      <main className="main-content">
        <Outlet />
      </main>
    </>
  )
}
