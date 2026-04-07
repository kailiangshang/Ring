import { useEffect, useState } from 'react'
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { SetupWizard } from './pages/Setup/SetupWizard'
import { RingHub } from './pages/RingHub/RingHub'
import { ChatView } from './pages/RingSpace/ChatView'
import { BlueprintWizard } from './pages/RingSpace/BlueprintWizard'
import { GraphView } from './pages/RingSpace/GraphView'
import { PrList } from './pages/RingSpace/PrList'
import { PrDetail } from './pages/RingSpace/PrDetail'
import { SuperRingChat } from './pages/RingHub/SuperRingChat'
import { MemberList } from './components/member/MemberList'
import { SessionView } from './components/session/SessionView'
import { SettingsPage } from './pages/Settings/SettingsPage'
import { Layout } from './components/layout/Layout'
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

export default function App() {
  return (
    <BrowserRouter>
      <Toast />
      <Routes>
        <Route path="/setup" element={<SetupWizardRedirect />} />
        <Route
          element={
            <SetupGuard>
              <Layout />
            </SetupGuard>
          }
        >
          <Route path="/" element={<RingHub />} />
          <Route path="/ring/:ringId" element={<ChatView />} />
          <Route path="/ring/:ringId/blueprint" element={<BlueprintWizard />} />
          <Route path="/ring/:ringId/graph" element={<GraphView />} />
          <Route path="/ring/:ringId/prs" element={<PrList />} />
          <Route path="/ring/:ringId/prs/:prId" element={<PrDetail />} />
          <Route path="/ring/:ringId/members" element={<MemberList />} />
          <Route path="/ring/:ringId/sessions" element={<SessionView />} />
          <Route path="/ring/:ringId/sessions/:sessionId" element={<SessionView />} />
          <Route path="/super-ring" element={<SuperRingChat />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Route>
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  )
}
