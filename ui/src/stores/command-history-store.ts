import { create } from 'zustand'

interface CommandHistoryState {
  history: string[]
  add: (cmd: string) => void
  getHistory: () => string[]
}

export const useCommandHistoryStore = create<CommandHistoryState>((set, get) => ({
  history: [],
  add: (cmd: string) => {
    if (!cmd.startsWith('/') && !cmd.startsWith('@')) return
    set((state) => ({
      history: [cmd, ...state.history].slice(0, 50)
    }))
  },
  getHistory: () => get().history,
}))
