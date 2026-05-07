import { create } from 'zustand'

interface CommandResultState {
  result: { title: string; content: string } | null
  showCommandResult: (title: string, content: string) => void
  closeCommandResult: () => void
}

export const useCommandResultStore = create<CommandResultState>()((set) => ({
  result: null,
  showCommandResult: (title, content) => set({ result: { title, content } }),
  closeCommandResult: () => set({ result: null }),
}))
