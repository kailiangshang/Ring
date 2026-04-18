import { create } from 'zustand'
import type { Ring } from '../types/ring'
import { api } from '../services/api'

interface RingState {
  rings: Ring[]
  loading: boolean
  active_ring_id: string | null
  fetchRings: () => Promise<void>
  createRing: (name: string, role_description: string) => Promise<string | null>
  setRings: (rings: Ring[]) => void
  selectRing: (id: string | null) => void
}

export const useRingStore = create<RingState>((set, get) => ({
  rings: [],
  loading: false,
  active_ring_id: null,

  fetchRings: async () => {
    set({ loading: true })
    try {
      const res = await api.get<{ rings: Ring[] }>('/rings')
      set({ rings: res.rings, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  createRing: async (name, role_description) => {
    try {
      const res = await api.post<{ id: string; name: string; role: string }>('/rings', {
        name,
        role_description,
      })
      await get().fetchRings()
      return res.id
    } catch {
      return null
    }
  },

  setRings: (rings) => set({ rings }),
  selectRing: (id) => set({ active_ring_id: id }),
}))
