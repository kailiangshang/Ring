import { create } from 'zustand'

interface AppState {
  is_setup: boolean
  current_context: 'super' | 'ring' | 'session' | 'self'
  active_ring_id: string | null
  active_session_id: string | null
  setSetup: (done: boolean) => void
  setContext: (ctx: AppState['current_context']) => void
  setActiveRing: (ring_id: string | null) => void
  setActiveSession: (session_id: string | null) => void
}

export const useAppStore = create<AppState>((set) => ({
  is_setup: false,
  current_context: 'super',
  active_ring_id: null,
  active_session_id: null,
  setSetup: (done) => set({ is_setup: done }),
  setContext: (ctx) => set({ current_context: ctx }),
  setActiveRing: (ring_id) => set({ active_ring_id: ring_id, current_context: ring_id ? 'ring' : 'super' }),
  setActiveSession: (session_id) => set({ active_session_id: session_id, current_context: session_id ? 'session' : 'ring' }),
}))
