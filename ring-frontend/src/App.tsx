import { useEffect, useState } from 'react'
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { SetupWizard } from './pages/Setup/SetupWizard'
import { RingHub } from './pages/RingHub/RingHub'
import { BlueprintWizard } from './pages/RingSpace/BlueprintWizard'
import { SettingsPage } from './pages/Settings/SettingsPage'
import { AppShell } from './components/layout/AppShell'
import { RingSpaceLayout } from './components/layout/RingSpaceLayout'
import { RingSuperDrawer } from './components/layout/RingSuperDrawer'
import { get_setup_status } from './api/client'
import { Toast } from './components/Toast'

function SetupGuard({ children }: { children: React.ReactNode }) {
  const [checking, set_checking] = useState(true)
  const [completed, set_completed] = useState(false)

  useEffect(() => {
    get_setup_status()
      .then((status) => {
        if (status.user_id) localStorage.setItem('ring_user_id', status.user_id)
        set_completed(status.setup_completed)
        set_checking(false)
      })
      .catch(() => set_checking(false))
  }, [])

  if (checking) return <div className="spinner-container"><div className="spinner" /></div>
  if (!completed) return <Navigate to="/setup" replace />
  return <>{children}</>
}

function SetupWizardRedirect() {
  const [checking, set_checking] = useState(true)
  const [completed, set_completed] = useState(false)

  useEffect(() => {
    get_setup_status()
      .then((status) => {
        if (status.user_id) localStorage.setItem('ring_user_id', status.user_id)
        set_completed(status.setup_completed)
        set_checking(false)
      })
      .catch(() => set_checking(false))
  }, [])

  if (checking) return <div className="spinner-container"><div className="spinner" /></div>
  if (completed) return <Navigate to="/" replace />
  return <SetupWizard />
}

function GlobalRingSuper() {
  const [open, set_open] = useState(false)
  return (
    <>
      <button className="global-super-fab" onClick={() => set_open(!open)} title="Ring Super 全局助手">⚡</button>
      <RingSuperDrawer open={open} on_close={() => set_open(false)} />
    </>
  )
}

export default function App() {
  return (
    <BrowserRouter>
      <Toast />
      <GlobalRingSuper />
      <Routes>
        <Route path="/setup" element={<SetupWizardRedirect />} />
        <Route element={<SetupGuard><AppShell /></SetupGuard>}>
          <Route path="/" element={<RingHub />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
        <Route path="/ring/:ringId" element={<SetupGuard><RingSpaceLayout /></SetupGuard>} />
        <Route path="/ring/:ringId/blueprint" element={<SetupGuard><BlueprintWizard /></SetupGuard>} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  )
}
