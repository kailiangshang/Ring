import { create } from 'zustand'
import type { Ring } from '../types/ring'
import { MOCK_RINGS } from '../services/mock-data'

interface RingState {
  rings: Ring[]
  active_ring_id: string | null
  setRings: (rings: Ring[]) => void
  selectRing: (id: string | null) => void
}

export const useRingStore = create<RingState>((set) => ({
  rings: MOCK_RINGS,
  active_ring_id: null,
  setRings: (rings) => set({ rings }),
  selectRing: (id) => set({ active_ring_id: id }),
}))
