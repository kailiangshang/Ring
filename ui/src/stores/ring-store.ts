import { create } from 'zustand'
import type { Ring } from '../types/ring'
import { api, deleteRing as deleteRingApi } from '../services/api'

interface CreateRingInput {
  name: string
  role_description: string
  storage_mode: 'local' | 'gitlab'
  gitlab_repo_url?: string
}

interface RingState {
  rings: Ring[]
  loading: boolean
  active_ring_id: string | null
  fetchRings: () => Promise<void>
  createRing: (input: CreateRingInput) => Promise<string | null>
  deleteRing: (ringId: string) => Promise<boolean>
  setRings: (rings: Ring[]) => void
  selectRing: (id: string | null) => void
}

export const useRingStore = create<RingState>((set, get) => ({
  rings: [],
  loading: false,
  active_ring_id: (() => { try { return localStorage.getItem('ring_active_ring_id') || null } catch { return null } })(),

  fetchRings: async () => {
    set({ loading: true })
    try {
      const res = await api.get<{ rings: Ring[] }>('/rings')
      set({ rings: res.rings, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  createRing: async (input) => {
    try {
      const body: Record<string, unknown> = {
        name: input.name,
        role_description: input.role_description,
        storage_mode: input.storage_mode,
      }
      if (input.storage_mode === 'gitlab' && input.gitlab_repo_url) {
        body.gitlab_repo_url = input.gitlab_repo_url
      }
      const res = await api.post<{ id: string; name: string; role: string }>('/rings', body)
      await get().fetchRings()
      return res.id
    } catch {
      return null
    }
  },

  deleteRing: async (ringId) => {
    try {
      await deleteRingApi(ringId)
      set((s) => {
        const isActive = s.active_ring_id === ringId
        if (isActive) localStorage.removeItem('ring_active_ring_id')
        return {
          rings: s.rings.filter((r) => r.id !== ringId),
          active_ring_id: isActive ? null : s.active_ring_id,
        }
      })
      return true
    } catch {
      return false
    }
  },

  setRings: (rings) => set({ rings }),
  selectRing: (id) => {
    if (id) {
      localStorage.setItem('ring_active_ring_id', id)
    } else {
      localStorage.removeItem('ring_active_ring_id')
    }
    set({ active_ring_id: id })
  },
}))
