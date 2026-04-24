import { useEffect, useState } from 'react'
import { useAppStore } from './stores/app-store'
import { useAuthStore } from './stores/auth-store'
import { AppLayout } from './components/layout/AppLayout'
import { SetupWizard } from './components/setup/SetupWizard'
import { startHeartbeat, stopHeartbeat } from './services/metrics'
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
  const setAuth = useAuthStore((s) => s.setAuth)
  const [recovering, setRecovering] = useState(false)

  useEffect(() => {
    loadFromStorage()
    init()
  }, [init, loadFromStorage])

  useEffect(() => {
    startHeartbeat()
    const handleUnload = () => stopHeartbeat()
    window.addEventListener('beforeunload', handleUnload)
    return () => {
      stopHeartbeat()
      window.removeEventListener('beforeunload', handleUnload)
    }
  }, [])

  useEffect(() => {
    if (!loading && is_setup && !localStorage.getItem('ring_token') && !recovering) {
      setRecovering(true)
      fetch('/api/setup/recover')
        .then((res) => (res.ok ? res.json() : null))
        .then((data) => {
          if (data?.token_id) {
            setAuth(data.token_id, '', null)
          }
        })
        .catch(() => {})
    }
  }, [loading, is_setup, setAuth, recovering])

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
