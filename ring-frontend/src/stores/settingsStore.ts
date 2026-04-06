import { create } from 'zustand'
import * as api from '../api/client'

interface SettingsState {
  settings: Record<string, string>
  loading: boolean
  error: string | null
  load_settings: () => Promise<void>
  save_settings: (settings: Record<string, string>) => Promise<void>
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: {},
  loading: false,
  error: null,

  load_settings: async () => {
    set({ loading: true, error: null })
    try {
      const settings = await api.get_settings()
      set({ settings, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  save_settings: async (settings) => {
    set({ loading: true, error: null })
    try {
      await api.update_settings(settings)
      set({ settings, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },
}))
