import { NavLink } from 'react-router-dom'

export function NavBar() {
  return (
    <nav className="navbar">
      <NavLink to="/" className="navbar-brand">
        Ring
      </NavLink>
      <div className="navbar-links">
        <NavLink
          to="/"
          end
          className={({ isActive }) => isActive ? 'nav-link active' : 'nav-link'}
        >
          Ring Group
        </NavLink>
        <NavLink
          to="/super-ring"
          className={({ isActive }) => isActive ? 'nav-link active' : 'nav-link'}
        >
          Ring Super
        </NavLink>
        <NavLink
          to="/settings"
          className={({ isActive }) => isActive ? 'nav-link active' : 'nav-link'}
        >
          Settings
        </NavLink>
      </div>
    </nav>
  )
}
