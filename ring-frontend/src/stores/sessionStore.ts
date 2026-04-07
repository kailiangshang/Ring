import { create } from 'zustand'
import * as api from '../api/client'
import type { SessionData, CreateSessionRequest } from '../types'

interface SessionState {
  sessions: SessionData[]
  current_session: SessionData | null
  loading: boolean
  error: string | null

  load_sessions: (ring_id: string, status?: string) => Promise<void>
  create_session: (ring_id: string, req: CreateSessionRequest) => Promise<SessionData | null>
  close_session: (ring_id: string, session_id: string) => Promise<void>
  leave_session: (ring_id: string, session_id: string) => Promise<void>
  toggle_archive: (ring_id: string, session_id: string, enabled: boolean) => Promise<void>
  delete_session: (ring_id: string, session_id: string) => Promise<void>
  clear_error: () => void
}

export const useSessionStore = create<SessionState>((set, get) => ({
  sessions: [],
  current_session: null,
  loading: false,
  error: null,

  load_sessions: async (ring_id, status) => {
    set({ loading: true, error: null })
    try {
      const sessions = await api.list_sessions(ring_id, status)
      set({ sessions, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  create_session: async (ring_id, req) => {
    set({ loading: true, error: null })
    try {
      const session = await api.create_session(ring_id, req)
      await get().load_sessions(ring_id)
      set({ loading: false })
      return session
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
      return null
    }
  },

  close_session: async (ring_id, session_id) => {
    set({ loading: true, error: null })
    try {
      await api.close_session(ring_id, session_id)
      await get().load_sessions(ring_id)
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  leave_session: async (ring_id, session_id) => {
    set({ loading: true, error: null })
    try {
      await api.leave_session(ring_id, session_id)
      set({ current_session: null, loading: false })
      await get().load_sessions(ring_id)
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  toggle_archive: async (ring_id, session_id, enabled) => {
    set({ loading: true, error: null })
    try {
      await api.toggle_session_archive(ring_id, session_id, enabled)
      await get().load_sessions(ring_id)
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  delete_session: async (ring_id, session_id) => {
    set({ loading: true, error: null })
    try {
      await api.delete_session(ring_id, session_id)
      await get().load_sessions(ring_id)
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  clear_error: () => set({ error: null }),
}))
