import { create } from 'zustand'
import type { ChatMessage } from '../types/chat'
import { MOCK_MESSAGES } from '../services/mock-data'

interface ChatState {
  messages: ChatMessage[]
  input: string
  session_mode: 'storage' | 'ephemeral'
  setInput: (val: string) => void
  addMessage: (msg: ChatMessage) => void
  setSessionMode: (mode: 'storage' | 'ephemeral') => void
}

export const useChatStore = create<ChatState>((set) => ({
  messages: MOCK_MESSAGES,
  input: '',
  session_mode: 'storage',
  setInput: (val) => set({ input: val }),
  addMessage: (msg) => set((s) => ({ messages: [...s.messages, msg] })),
  setSessionMode: (mode) => set({ session_mode: mode }),
}))
