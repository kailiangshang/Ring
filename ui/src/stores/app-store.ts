import { create } from 'zustand'
import { api } from '../services/api'

interface AppState {
  is_setup: boolean
  loading: boolean
  current_context: 'super' | 'ring' | 'session' | 'self'
  active_session_id: string | null
  init: () => Promise<void>
  setSetup: (done: boolean) => void
  setContext: (ctx: AppState['current_context']) => void
  setActiveSession: (session_id: string | null) => void
}

export const useAppStore = create<AppState>((set) => ({
  is_setup: false,
  loading: true,
  current_context: 'super',
  active_session_id: null,

  init: async () => {
    try {
      const res = await api.get<{ is_setup: boolean; step: string | null }>('/setup/status')
      set({ is_setup: res.is_setup, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  setSetup: (done) => set({ is_setup: done }),
  setContext: (ctx) => set({ current_context: ctx }),
  setActiveSession: (session_id) => set({ active_session_id: session_id, current_context: session_id ? 'session' : 'ring' }),
}))
