import { create } from 'zustand'
import { api } from '../services/api'
import { useWsStore } from './ws-store'
import { useRingStore } from './ring-store'
import type { Session, SessionParticipant, SessionMessage, CreateSessionInput } from '../types/session'

interface FlatSessionResponse {
  id: string
  ring_id: string
  title: string
  description: string
  skill: string
  phase: string
  owner: string
  archivable: boolean
  archive_enabled: boolean
  summary: string | null
  created_at: string
  updated_at: string
  participants: SessionParticipant[]
}

function toSession(flat: FlatSessionResponse): Session {
  return {
    id: flat.id,
    ring_id: flat.ring_id,
    title: flat.title,
    description: flat.description,
    skill: flat.skill as Session['skill'],
    phase: flat.phase as Session['phase'],
    owner: flat.owner,
    archivable: flat.archivable,
    archive_enabled: flat.archive_enabled,
    summary: flat.summary,
    created_at: flat.created_at,
    updated_at: flat.updated_at,
  }
}

interface SessionState {
  active_session: Session | null
  participants: SessionParticipant[]
  messages: SessionMessage[]
  loading: boolean
  list: Session[]
  createSession: (input: CreateSessionInput) => Promise<Session | null>
  fetchActiveSession: (ring_id: string) => Promise<void>
  fetchSessions: (ring_id: string) => Promise<void>
  closeSession: (ring_id: string, session_id: string) => Promise<void>
  reopenSession: (ring_id: string, session_id: string) => Promise<void>
  deleteSession: (ring_id: string, session_id: string) => Promise<void>
  inviteParticipants: (ring_id: string, session_id: string, token_ids: string[]) => Promise<void>
  removeParticipant: (ring_id: string, session_id: string, token_id: string) => Promise<void>
  toggleArchive: (ring_id: string, session_id: string, enabled: boolean) => Promise<void>
  sendMessage: (session_id: string, content: string) => void
  handleWsMessage: (data: unknown) => void
  fetchMessages: (ring_id: string, session_id: string) => Promise<void>
  clearActive: () => void
}

export const useSessionStore = create<SessionState>((set, get) => ({
  active_session: null,
  participants: [],
  messages: [],
  loading: false,
  list: [],

  createSession: async (input) => {
    const ring_id = useRingStore.getState().active_ring_id
    if (!ring_id) return null
    try {
      const res = await api.post<FlatSessionResponse>(`/rings/${ring_id}/sessions`, input)
      const session = toSession(res)
      set({
        active_session: session,
        participants: res.participants ?? [],
        messages: [],
      })
      return session
    } catch {
      return null
    }
  },

  fetchActiveSession: async (ring_id) => {
    set({ loading: true })
    try {
      const res = await api.get<{ sessions: FlatSessionResponse[] }>(`/rings/${ring_id}/sessions?status=active`)
      const items = res.sessions ?? []
      if (items.length > 0) {
        const flat = items[0]
        set({
          active_session: toSession(flat),
          participants: flat.participants ?? [],
          loading: false,
        })
        get().fetchMessages(ring_id, flat.id)
      } else {
        set({ active_session: null, participants: [], messages: [], loading: false })
      }
    } catch {
      set({ loading: false })
    }
  },

  fetchSessions: async (ring_id) => {
    try {
      const res = await api.get<{ sessions: FlatSessionResponse[] }>(`/rings/${ring_id}/sessions`)
      set({ list: (res.sessions ?? []).map(toSession) })
    } catch {
      // keep existing
    }
  },

  closeSession: async (ring_id, session_id) => {
    try {
      const res = await api.post<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}/close`, {})
      set({ active_session: toSession(res) })
    } catch {
      // keep state
    }
  },

  reopenSession: async (ring_id, session_id) => {
    try {
      const res = await api.post<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}/reopen`, {})
      set({ active_session: toSession(res), messages: [] })
      get().fetchMessages(ring_id, session_id)
    } catch {
      // keep state
    }
  },

  deleteSession: async (ring_id, session_id) => {
    try {
      await api.delete(`/rings/${ring_id}/sessions/${session_id}`)
      set({ active_session: null, participants: [], messages: [] })
    } catch {
      // keep state
    }
  },

  inviteParticipants: async (ring_id, session_id, token_ids) => {
    try {
      await api.post(`/rings/${ring_id}/sessions/${session_id}/participants`, { token_ids })
      const res = await api.get<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}`)
      set({ participants: res.participants ?? [] })
    } catch {
      // keep state
    }
  },

  removeParticipant: async (ring_id, session_id, token_id) => {
    try {
      await api.delete(`/rings/${ring_id}/sessions/${session_id}/participants/${token_id}`)
      set((s) => ({
        participants: s.participants.filter((p) => p.token_id !== token_id),
      }))
    } catch {
      // keep state
    }
  },

  toggleArchive: async (ring_id, session_id, enabled) => {
    try {
      const res = await api.put<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}/archive-toggle`, { enabled })
      set({ active_session: toSession(res) })
    } catch {
      // keep state
    }
  },

  sendMessage: (session_id, content) => {
    useWsStore.getState().send({
      type: 'session_message',
      session_id,
      content,
    })
  },

  handleWsMessage: (data: unknown) => {
    const msg = data as Record<string, unknown>
    if (!msg || typeof msg.type !== 'string') return

    const { active_session } = get()
    const session_id = msg.session_id as string | undefined

    switch (msg.type) {
      case 'session_message': {
        if (!session_id || !active_session || session_id !== active_session.id) return
        const incoming: SessionMessage = {
          id: msg.id as string,
          session_id,
          seq_num: msg.seq_num as number,
          sender: msg.sender as string,
          sender_name: msg.sender_name as string,
          content: msg.content as string,
          message_type: 'user',
          created_at: msg.created_at as string,
        }
        set((s) => ({ messages: [...s.messages, incoming] }))
        break
      }
      case 'session_catchup': {
        if (!session_id || !active_session || session_id !== active_session.id) return
        set({ messages: (msg.messages as SessionMessage[]) ?? [] })
        break
      }
      case 'session_paused': {
        if (active_session && session_id === active_session.id) {
          set((s) => ({
            active_session: s.active_session
              ? { ...s.active_session, phase: 'closed' as const }
              : null,
          }))
        }
        break
      }
      case 'session_resumed': {
        if (active_session && session_id === active_session.id) {
          set((s) => ({
            active_session: s.active_session
              ? { ...s.active_session, phase: 'discussion' as const }
              : null,
          }))
        }
        break
      }
    }
  },

  fetchMessages: async (ring_id, session_id) => {
    try {
      const res = await api.get<{ messages: SessionMessage[] }>(
        `/rings/${ring_id}/sessions/${session_id}/messages?after_seq=0&limit=100`,
      )
      set({ messages: res.messages ?? [] })
    } catch {
      // keep existing
    }
  },

  clearActive: () => set({ active_session: null, participants: [], messages: [] }),
}))
