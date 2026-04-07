import { create } from 'zustand'
import * as api from '../api/client'
import type { LlmConfig, GitlabConfig } from '../types'

interface SetupState {
  step: number
  error: string | null
  loading: boolean
  user_id: string | null
  redirect_home: boolean
  set_step: (step: number) => void
  submit_username: (display_name: string) => Promise<void>
  submit_llm: (config: LlmConfig) => Promise<void>
  submit_gitlab: (config: GitlabConfig) => Promise<void>
  complete: () => Promise<void>
  reset: () => void
}

function is_already_completed(e: unknown): boolean {
  return (e as Error).message.includes('already completed')
}

export const useSetupStore = create<SetupState>((set) => ({
  step: 0,
  error: null,
  loading: false,
  user_id: localStorage.getItem('ring_user_id'),
  redirect_home: false,

  set_step: (step) => set({ step, error: null }),

  submit_username: async (display_name) => {
    set({ loading: true, error: null })
    try {
      const user = await api.set_username(display_name)
      localStorage.setItem('ring_user_id', user.user_id)
      set({ step: 1, loading: false, user_id: user.user_id })
    } catch (e) {
      if (is_already_completed(e)) {
        set({ loading: false, redirect_home: true })
        return
      }
      set({ error: (e as Error).message, loading: false })
    }
  },

  submit_llm: async (config) => {
    set({ loading: true, error: null })
    try {
      await api.set_llm(config)
      set({ step: 2, loading: false })
    } catch (e) {
      if (is_already_completed(e)) {
        set({ loading: false, redirect_home: true })
        return
      }
      set({ error: (e as Error).message, loading: false })
    }
  },

  submit_gitlab: async (config) => {
    set({ loading: true, error: null })
    try {
      await api.set_gitlab(config)
      set({ loading: false })
    } catch (e) {
      if (is_already_completed(e)) {
        set({ loading: false, redirect_home: true })
        return
      }
      set({ error: (e as Error).message, loading: false })
    }
  },

  complete: async () => {
    set({ loading: true, error: null })
    try {
      await api.complete_setup()
      set({ loading: false, redirect_home: true })
    } catch (e) {
      if (is_already_completed(e)) {
        set({ loading: false, redirect_home: true })
        return
      }
      set({ error: (e as Error).message, loading: false })
    }
  },

  reset: () => set({ step: 0, error: null, loading: false }),
}))
