import { create } from 'zustand'
import type { ChatMessage } from '../types/chat'
import { parseCommand } from '../services/command-parser'
import { usePanelStore } from './panel-store'
import { useSelfStore } from './self-store'
import { useModeStore } from './mode-store'
import { useRingStore } from './ring-store'

interface ChatState {
  messages: ChatMessage[]
  input: string
  session_mode: 'storage' | 'ephemeral'
  setInput: (val: string) => void
  addMessage: (msg: ChatMessage) => void
  send: () => void
  setSessionMode: (mode: 'storage' | 'ephemeral') => void
}

export const useChatStore = create<ChatState>((set, get) => ({
  messages: [],
  input: '',
  session_mode: 'storage',

  setInput: (val) => set({ input: val }),

  addMessage: (msg) => set((s) => ({ messages: [...s.messages, msg] })),

  send: () => {
    const { input, addMessage } = get()
    if (!input.trim()) return

    const parsed = parseCommand(input)

    if (parsed) {
      for (const cmd of parsed) {
        switch (cmd.type) {
          case 'action': {
            if (cmd.action === 'graph') usePanelStore.getState().toggle('graph')
            else if (cmd.action === 'archive') usePanelStore.getState().toggle('archive')
            else if (cmd.action === 'config') usePanelStore.getState().toggle('config')
            else if (cmd.action === 'session') usePanelStore.getState().toggle('session')
            else if (cmd.action === 'auto') useModeStore.getState().toggleAuto()
            else if (cmd.action === 'new') {
              const name = cmd.args
              if (name) {
                useRingStore.getState().createRing(name, `You are a ${name} assistant`)
              }
            }
            else if (cmd.action === 'save') {
              addMessage({
                id: `sys-${Date.now()}`,
                role: 'system',
                sender_name: 'SYSTEM',
                content: '归档功能将在后续版本实现',
                created_at: new Date().toISOString(),
              })
            }
            break
          }
          case 'address': {
            if (cmd.target === 'self') useSelfStore.getState().setOpen(true)
            break
          }
          case 'meta': {
            if (cmd.key === 'mode' && cmd.value) useModeStore.getState().setInteractionMode(cmd.value as 'normal' | 'auto')
            else if (cmd.key === 'skill' && cmd.value) useModeStore.getState().setSkillMode(cmd.value as 'auto' | 'plan' | 'edit')
            break
          }
          case 'reference':
            break
        }
      }
    }

    addMessage({
      id: `msg-${Date.now()}`,
      role: 'user',
      sender_name: 'You',
      content: input,
      node_refs: parsed?.filter((c) => c.type === 'reference').map((c) => c.name),
      created_at: new Date().toISOString(),
    })

    set({ input: '' })
  },

  setSessionMode: (mode) => set({ session_mode: mode }),
}))
