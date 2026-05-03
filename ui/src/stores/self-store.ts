import { create } from 'zustand'

interface SelfState {
  open: boolean
  position: { x: number; y: number }
  active_tab: 'chat' | 'memory' | 'activity' | 'settings'
  trigger_position: { x: number; y: number }
  setOpen: (open: boolean) => void
  toggle: () => void
  setPosition: (pos: { x: number; y: number }) => void
  setTab: (tab: SelfState['active_tab']) => void
  setTriggerPosition: (pos: { x: number; y: number }) => void
}

const TRIGGER_DEFAULT = { x: typeof window !== 'undefined' ? window.innerWidth - 70 : 300, y: typeof window !== 'undefined' ? window.innerHeight - 70 : 500 }
const FLOAT_DEFAULT = { x: typeof window !== 'undefined' ? window.innerWidth - 380 : 200, y: typeof window !== 'undefined' ? window.innerHeight - 420 : 200 }

export const useSelfStore = create<SelfState>((set, get) => ({
  open: false,
  position: FLOAT_DEFAULT,
  active_tab: 'chat',
  trigger_position: TRIGGER_DEFAULT,
  setOpen: (open) => set({ open }),
  toggle: () => set({ open: !get().open }),
  setPosition: (pos) => set({ position: pos }),
  setTab: (tab) => set({ active_tab: tab }),
  setTriggerPosition: (pos) => set({ trigger_position: pos }),
}))
