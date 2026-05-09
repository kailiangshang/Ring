import { create } from 'zustand'
import { api } from '../services/api'
import { useWsStore } from './ws-store'
import { useRingStore } from './ring-store'
import type { Session, SessionParticipant, SessionMessage, CreateSessionInput, SessionMaterial } from '../types/session'

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

interface GraphSuggestion {
  title: string
  content: string
}

interface SessionState {
  active_session: Session | null
  participants: SessionParticipant[]
  messages: SessionMessage[]
  materials: SessionMaterial[]
  loading: boolean
  error: string | null
  list: Session[]
  sessions_by_ring: Record<string, Session[]>
  graph_suggestions: GraphSuggestion[]
  createSession: (input: CreateSessionInput) => Promise<Session | null>
  fetchActiveSession: (ring_id: string) => Promise<void>
  fetchSessions: (ring_id: string) => Promise<void>
  fetchSessionsForSidebar: (ring_id: string) => Promise<void>
  closeSession: (ring_id: string, session_id: string) => Promise<void>
  reopenSession: (ring_id: string, session_id: string) => Promise<void>
  deleteSession: (ring_id: string, session_id: string) => Promise<void>
  inviteParticipants: (ring_id: string, session_id: string, token_ids: string[]) => Promise<void>
  removeParticipant: (ring_id: string, session_id: string, token_id: string) => Promise<void>
  toggleArchive: (ring_id: string, session_id: string, enabled: boolean) => Promise<void>
  startSession: (ring_id: string, session_id: string) => Promise<void>
  fetchMaterials: (ring_id: string, session_id: string) => Promise<void>
  highlightMaterial: (ring_id: string, session_id: string, material_id: string, note: string) => Promise<void>
  updateMaterial: (ring_id: string, session_id: string, material_id: string, title: string, content: string) => Promise<void>
  createMaterial: (ring_id: string, session_id: string, item_type: string, title: string, content: string) => Promise<void>
  updateSummary: (ring_id: string, session_id: string, summary: string) => Promise<void>
  extractToGraph: (ring_id: string, session_id: string) => Promise<void>
  sendMessage: (session_id: string, content: string) => void
  handleWsMessage: (data: unknown) => void
  fetchMessages: (ring_id: string, session_id: string) => Promise<void>
  clearActive: () => void
}

export const useSessionStore = create<SessionState>((set, get) => ({
  active_session: null,
  participants: [],
  messages: [],
  materials: [],
  loading: false,
  error: null,
  list: [],
  sessions_by_ring: {},
  graph_suggestions: [],

  createSession: async (input) => {
    const ring_id = useRingStore.getState().active_ring_id
    if (!ring_id) return null
    try {
      set({ error: null })
      const res = await api.post<FlatSessionResponse>(`/rings/${ring_id}/sessions`, input)
      const session = toSession(res)
      set({
        active_session: session,
        participants: res.participants ?? [],
        messages: [],
      })
      return session
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Failed to create session' })
      return null
    }
  },

  fetchActiveSession: async (ring_id) => {
    set({ loading: true })
    try {
      // Query all sessions, not just active, to show closed sessions too
      const res = await api.get<{ sessions: FlatSessionResponse[] }>(`/rings/${ring_id}/sessions`)
      const items = res.sessions ?? []
      // Prefer active session, fallback to most recent closed
      const flat = items.find((s) => s.phase === 'active') ?? items[0]
      if (flat) {
        set({
          active_session: toSession(flat),
          participants: flat.participants ?? [],
          loading: false,
        })
        get().fetchMessages(ring_id, flat.id)
      } else {
        set({ active_session: null, participants: [], messages: [], loading: false })
      }
    } catch (e) {
      console.error('fetchActiveSession error:', e)
      set({ loading: false })
    }
  },

  fetchSessions: async (ring_id) => {
    try {
      const res = await api.get<{ sessions: FlatSessionResponse[] }>(`/rings/${ring_id}/sessions`)
      set({ list: (res.sessions ?? []).map(toSession) })
    } catch (e) {
      console.error('fetchSessions error:', e)
    }
  },

  fetchSessionsForSidebar: async (ring_id) => {
    try {
      const res = await api.get<{ sessions: FlatSessionResponse[] }>(`/rings/${ring_id}/sessions`)
      const sessions = (res.sessions ?? []).map(toSession)
      set((s) => ({ sessions_by_ring: { ...s.sessions_by_ring, [ring_id]: sessions } }))
    } catch (e) {
      console.error('fetchSessionsForSidebar error:', e)
    }
  },

  closeSession: async (ring_id, session_id) => {
    try {
      set({ error: null })
      const res = await api.post<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}/close`, {})
      set({ active_session: toSession(res) })
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Failed to close session' })
    }
  },

  reopenSession: async (ring_id, session_id) => {
    try {
      set({ error: null })
      const res = await api.post<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}/reopen`, {})
      set({ active_session: toSession(res), messages: [] })
      get().fetchMessages(ring_id, session_id)
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Failed to reopen session' })
    }
  },

  deleteSession: async (ring_id, session_id) => {
    try {
      set({ error: null })
      await api.delete(`/rings/${ring_id}/sessions/${session_id}`)
      set({ active_session: null, participants: [], messages: [] })
    } catch (e) {
      set({ error: e instanceof Error ? e.message : 'Failed to delete session' })
    }
  },

  inviteParticipants: async (ring_id, session_id, token_ids) => {
    try {
      await api.post(`/rings/${ring_id}/sessions/${session_id}/participants`, { token_ids })
      const res = await api.get<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}`)
      set({ participants: res.participants ?? [] })
    } catch (e) {
      console.error('inviteParticipants error:', e)
    }
  },

  removeParticipant: async (ring_id, session_id, token_id) => {
    try {
      await api.delete(`/rings/${ring_id}/sessions/${session_id}/participants/${token_id}`)
      set((s) => ({
        participants: s.participants.filter((p) => p.token_id !== token_id),
      }))
    } catch (e) {
      console.error('removeParticipant error:', e)
    }
  },

  toggleArchive: async (ring_id, session_id, enabled) => {
    try {
      const res = await api.put<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}/archive-toggle`, { enabled })
      set({ active_session: toSession(res) })
    } catch (e) {
      console.error('toggleArchive error:', e)
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
          content: typeof msg.content === 'string' ? msg.content : JSON.stringify(msg.content),
          message_type: 'user',
          created_at: msg.created_at as string,
        }
        set((s) => ({ messages: [...s.messages, incoming] }))
        break
      }
      case 'session_catchup': {
        if (!session_id || !active_session || session_id !== active_session.id) return
        const msgs = (msg.messages as SessionMessage[] ?? []).map((m) => ({
          ...m,
          content: typeof m.content === 'string' ? m.content : JSON.stringify(m.content),
        })) as SessionMessage[]
        set({ messages: msgs })
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
        break;
      }
      case 'session_material_added': {
        const matData = data as Record<string, unknown>
        if (matData.session_id === get().active_session?.id) {
          set((state) => ({
            materials: [...state.materials, matData.material as SessionMaterial],
          }))
        }
        break;
      }
      case 'session_ai_message': {
        if (!session_id || !active_session || session_id !== active_session.id) return
        const aiMsg: SessionMessage = {
          id: msg.id as string,
          session_id,
          seq_num: msg.seq_num as number,
          sender: msg.sender as string,
          sender_name: msg.sender_name as string,
          content: msg.content as string,
          message_type: 'ai',
          created_at: msg.created_at as string,
        }
        set((s) => ({ messages: [...s.messages, aiMsg] }))
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
    } catch (e) {
      console.error('fetchMessages error:', e)
    }
  },

  startSession: async (ring_id, session_id) => {
    try {
      const res = await api.post<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}/start`, {})
      set({ active_session: toSession(res) })
    } catch (e) {
      console.error('startSession error:', e)
    }
  },

  fetchMaterials: async (ring_id, session_id) => {
    try {
      const res = await api.get<{ materials: SessionMaterial[] }>(`/rings/${ring_id}/sessions/${session_id}/material-prep`)
      set({ materials: res.materials ?? [] })
    } catch (e) {
      console.error('fetchMaterials error:', e)
    }
  },

  highlightMaterial: async (ring_id, session_id, material_id, note) => {
    try {
      await api.post(`/rings/${ring_id}/sessions/${session_id}/material-prep/highlights`, { material_id, note })
      set((s) => ({
        materials: s.materials.map((m) =>
          m.id === material_id ? { ...m, highlight: note } : m
        ),
      }))
    } catch (e) {
      console.error('highlightMaterial error:', e)
    }
  },

  updateMaterial: async (ring_id, session_id, material_id, title, content) => {
    try {
      const res = await api.put<SessionMaterial>(`/rings/${ring_id}/sessions/${session_id}/material-prep/${material_id}`, { title, content })
      set((s) => ({
        materials: s.materials.map((m) =>
          m.id === material_id ? { ...m, title: res.title, content: res.content } : m
        ),
      }))
    } catch (e) {
      console.error('updateMaterial error:', e)
    }
  },

  createMaterial: async (ring_id, session_id, item_type, title, content) => {
    try {
      const res = await api.post<SessionMaterial>(`/rings/${ring_id}/sessions/${session_id}/material-prep`, { item_type, title, content })
      set((s) => ({ materials: [...s.materials, res] }))
    } catch (e) {
      console.error('createMaterial error:', e)
    }
  },

  updateSummary: async (ring_id, session_id, summary) => {
    try {
      const res = await api.put<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}/summary`, { summary })
      set({ active_session: toSession(res) })
    } catch (e) {
      console.error('updateSummary error:', e)
    }
  },

  extractToGraph: async (ring_id, session_id) => {
    try {
      const res = await api.post<{ suggestions: GraphSuggestion[] }>(`/rings/${ring_id}/sessions/${session_id}/extract-to-graph`, {})
      set({ graph_suggestions: res.suggestions ?? [] })
    } catch (e) {
      console.error('extractToGraph error:', e)
    }
  },

  clearActive: () => set({ active_session: null, participants: [], messages: [], materials: [], graph_suggestions: [] }),
}))
