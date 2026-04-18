import { create } from 'zustand'
import type { InteractionMode, SkillPermissionMode } from '../types/config'
import { api } from '../services/api'
import { useRingStore } from './ring-store'

interface ModeState {
  interaction_mode: InteractionMode
  skill_permission_mode: SkillPermissionMode
  syncing: boolean
  setInteractionMode: (mode: InteractionMode) => void
  setSkillMode: (mode: SkillPermissionMode) => void
  toggleAuto: () => void
  syncToServer: () => Promise<void>
  fetchFromServer: (ringId: string) => Promise<void>
  reset: () => void
}

export const useModeStore = create<ModeState>((set, get) => ({
  interaction_mode: 'normal',
  skill_permission_mode: 'plan',
  syncing: false,

  setInteractionMode: (mode) => {
    set({ interaction_mode: mode })
    get().syncToServer()
  },

  setSkillMode: (mode) => {
    set({ skill_permission_mode: mode })
    get().syncToServer()
  },

  toggleAuto: () => {
    set({ interaction_mode: get().interaction_mode === 'auto' ? 'normal' : 'auto' })
    get().syncToServer()
  },

  syncToServer: async () => {
    const ringId = useRingStore.getState().active_ring_id
    if (!ringId) return
    set({ syncing: true })
    try {
      const { interaction_mode, skill_permission_mode } = get()
      await api.put(`/rings/${ringId}/mode`, {
        interaction_mode,
        skill_permission_mode,
      })
    } catch {
      // silent fail — mode is local-first
    }
    set({ syncing: false })
  },

  fetchFromServer: async (ringId) => {
    try {
      const res = await api.get<{ interaction_mode: string; skill_permission_mode: string }>(`/rings/${ringId}/mode`)
      set({
        interaction_mode: res.interaction_mode as InteractionMode,
        skill_permission_mode: res.skill_permission_mode as SkillPermissionMode,
      })
    } catch {
      // keep defaults
    }
  },

  reset: () =>
    set({ interaction_mode: 'normal', skill_permission_mode: 'plan' }),
}))
