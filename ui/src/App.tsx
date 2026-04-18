import { useEffect } from 'react'
import { useAppStore } from './stores/app-store'
import { useAuthStore } from './stores/auth-store'
import { AppLayout } from './components/layout/AppLayout'
import { SetupWizard } from './components/setup/SetupWizard'
import './index.css'

export default function App() {
  const { is_setup, loading, init } = useAppStore()
  const loadFromStorage = useAuthStore((s) => s.loadFromStorage)

  useEffect(() => {
    loadFromStorage()
    init()
  }, [init, loadFromStorage])

  if (loading) {
    return (
      <div style={{ height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg-base)' }}>
        <span style={{ color: 'var(--text-dim)', fontSize: 12 }}>Loading...</span>
      </div>
    )
  }

  if (!is_setup) {
    return <SetupWizard />
  }

  return <AppLayout />
}
