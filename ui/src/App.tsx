import { useEffect, useState } from 'react'
import { useAppStore } from './stores/app-store'
import { useAuthStore } from './stores/auth-store'
import { AppLayout } from './components/layout/AppLayout'
import { SetupWizard } from './components/setup/SetupWizard'
import { ErrorBoundary } from './components/common/ErrorBoundary'
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
  const token_expired = useAuthStore((s) => s.token_expired)
  const setTokenExpired = useAuthStore((s) => s.setTokenExpired)
  const [recovering, setRecovering] = useState(false)

  useEffect(() => {
    loadFromStorage()
    init()
  }, [init, loadFromStorage])

  useEffect(() => {
    if (is_setup) {
      startHeartbeat()
    }
    const handleUnload = () => stopHeartbeat()
    window.addEventListener('beforeunload', handleUnload)
    return () => {
      stopHeartbeat()
      window.removeEventListener('beforeunload', handleUnload)
    }
  }, [is_setup])

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

  if (token_expired) {
    return (
      <ErrorBoundary>
        <div style={{ height: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', background: 'var(--bg-base)', gap: 12 }}>
          <span style={{ color: '#f59e0b', fontSize: 14, fontWeight: 600 }}>Auth Token Expired</span>
          <span style={{ color: 'var(--text-dim)', fontSize: 12 }}>Your auth token has expired (90 days). Please re-setup.</span>
          <button
            onClick={() => setTokenExpired(false)}
            style={{ background: 'var(--accent-cyan)', color: 'var(--bg-base)', border: 'none', borderRadius: 4, padding: '6px 16px', fontSize: 12, cursor: 'pointer' }}
          >
            Go to Setup
          </button>
        </div>
      </ErrorBoundary>
    )
  }

  const join_params = getJoinParams()

  return (
    <ErrorBoundary>
      {!is_setup ? (
        <SetupWizard join_params={join_params} />
      ) : (
        <AppLayout />
      )}
    </ErrorBoundary>
  )
}
