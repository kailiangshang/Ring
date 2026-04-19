import { useEffect } from 'react'
import { useAppStore } from './stores/app-store'
import { useAuthStore } from './stores/auth-store'
import { AppLayout } from './components/layout/AppLayout'
import { SetupWizard } from './components/setup/SetupWizard'
import './index.css'

function getJoinParams(): { token?: string; creator_ip?: string } | undefined {
  const params = new URLSearchParams(window.location.search)
  const token = params.get('token')
  const creator_ip = params.get('creator_ip')
  if (token) return { token, creator_ip: creator_ip || undefined }
  return undefined
}

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

  const join_params = getJoinParams()

  if (!is_setup) {
    return <SetupWizard join_params={join_params} />
  }

  return <AppLayout />
}
