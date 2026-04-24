import { api } from './api'
import { useAppStore } from '../stores/app-store'

const HEARTBEAT_INTERVAL = 30_000

let intervalId: ReturnType<typeof setInterval> | null = null

function getCurrentView(): string {
  const ctx = useAppStore.getState().current_context
  if (ctx === 'self') return 'self_panel'
  if (ctx === 'session') return 'session'
  return 'ring_chat'
}

export function startHeartbeat() {
  stopHeartbeat()
  intervalId = setInterval(async () => {
    const view = getCurrentView()
    try {
      await api.post('/self/metrics/heartbeat', { view })
    } catch {}
  }, HEARTBEAT_INTERVAL)
}

export function stopHeartbeat() {
  if (intervalId !== null) {
    clearInterval(intervalId)
    intervalId = null
  }
}
