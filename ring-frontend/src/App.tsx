import { useEffect, useState } from 'react'
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { SetupWizard } from './pages/Setup/SetupWizard'
import { RingHub } from './pages/RingHub/RingHub'
import { get_setup_status } from './api/client'

function SetupGuard({ children }: { children: React.ReactNode }) {
  const [checking, set_checking] = useState(true)
  const [completed, set_completed] = useState(false)

  useEffect(() => {
    get_setup_status()
      .then((status) => {
        set_completed(status.setup_completed)
        set_checking(false)
      })
      .catch(() => set_checking(false))
  }, [])

  if (checking) return <p>Loading...</p>
  if (!completed) return <Navigate to="/setup" replace />
  return <>{children}</>
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/setup" element={<SetupWizard />} />
        <Route
          path="/"
          element={
            <SetupGuard>
              <RingHub />
            </SetupGuard>
          }
        />
        <Route path="/ring/:ringId" element={<SetupGuard><RingHub /></SetupGuard>} />
      </Routes>
    </BrowserRouter>
  )
}
