import { create } from 'zustand'
import type { InteractionMode, SkillPermissionMode } from '../types/config'

interface ModeState {
  interaction_mode: InteractionMode
  skill_permission_mode: SkillPermissionMode
  setInteractionMode: (mode: InteractionMode) => void
  setSkillMode: (mode: SkillPermissionMode) => void
  toggleAuto: () => void
  reset: () => void
}

export const useModeStore = create<ModeState>((set, get) => ({
  interaction_mode: 'normal',
  skill_permission_mode: 'plan',
  setInteractionMode: (mode) => set({ interaction_mode: mode }),
  setSkillMode: (mode) => set({ skill_permission_mode: mode }),
  toggleAuto: () =>
    set({ interaction_mode: get().interaction_mode === 'auto' ? 'normal' : 'auto' }),
  reset: () =>
    set({ interaction_mode: 'normal', skill_permission_mode: 'plan' }),
}))
