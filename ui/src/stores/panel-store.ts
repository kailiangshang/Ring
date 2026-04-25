import { create } from 'zustand'

export type PanelType = 'graph' | 'archive' | 'config' | 'session' | 'super_skills' | 'super_settings' | 'blueprint' | 'prompts'

export interface Panel {
  type: PanelType
  depth: number
}

interface PanelState {
  panels: Panel[]
  open: (type: PanelType) => void
  close: (index: number) => void
  closeAll: () => void
  toggle: (type: PanelType) => void
}

export const usePanelStore = create<PanelState>((set, get) => ({
  panels: [],
  open: (type) =>
    set((state) => {
      if (state.panels.some((p) => p.type === type)) return state
      return {
        panels: [...state.panels, { type, depth: state.panels.length + 1 }],
      }
    }),
  close: (index) =>
    set((state) => {
      const remaining = state.panels.filter((_, i) => i !== index)
      return {
        panels: remaining.map((p, i) => ({ ...p, depth: i + 1 })),
      }
    }),
  closeAll: () => set({ panels: [] }),
  toggle: (type) => {
    const { panels } = get()
    const existing = panels.findIndex((p) => p.type === type)
    if (existing >= 0) {
      get().close(existing)
    } else {
      get().open(type)
    }
  },
}))
