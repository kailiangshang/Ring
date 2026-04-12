import { create } from 'zustand'

type InteractionMode = 'daily' | 'manual_archive' | 'auto'

interface ModeState {
  mode: InteractionMode
  set_mode: (mode: InteractionMode) => void
}

export const useModeStore = create<ModeState>((set) => ({
  mode: 'daily',
  set_mode: (mode) => set({ mode }),
}))

export type { InteractionMode }
