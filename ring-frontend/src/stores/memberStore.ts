import { create } from 'zustand'
import * as api from '../api/client'
import type { Member, InviteRequest } from '../types'

interface MemberState {
  members: Member[]
  loading: boolean
  error: string | null

  load_members: (ring_id: string) => Promise<void>
  generate_invite: (ring_id: string, req: InviteRequest) => Promise<string | null>
  update_role: (ring_id: string, member_id: string, role: string) => Promise<void>
  remove_member: (ring_id: string, member_id: string) => Promise<void>
  clear_error: () => void
}

export const useMemberStore = create<MemberState>((set, get) => ({
  members: [],
  loading: false,
  error: null,

  load_members: async (ring_id) => {
    set({ loading: true, error: null })
    try {
      const members = await api.list_members(ring_id)
      set({ members, loading: false })
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  generate_invite: async (ring_id, req) => {
    set({ loading: true, error: null })
    try {
      const token = await api.generate_invite(ring_id, req)
      set({ loading: false })
      return token.token
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
      return null
    }
  },

  update_role: async (ring_id, member_id, role) => {
    set({ loading: true, error: null })
    try {
      await api.update_member_role(ring_id, member_id, role)
      await get().load_members(ring_id)
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  remove_member: async (ring_id, member_id) => {
    set({ loading: true, error: null })
    try {
      await api.remove_member(ring_id, member_id)
      await get().load_members(ring_id)
    } catch (e) {
      set({ error: (e as Error).message, loading: false })
    }
  },

  clear_error: () => set({ error: null }),
}))
