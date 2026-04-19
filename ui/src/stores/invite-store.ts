import { create } from 'zustand'
import type { InviteToken, JoinRequest, CreateInviteInput } from '../types/invite'
import {
  createInviteToken,
  listInviteTokens,
  revokeInviteToken,
  listJoinRequests,
  approveJoinRequest,
  rejectJoinRequest,
} from '../services/api'

interface InviteState {
  tokens: InviteToken[]
  join_requests: JoinRequest[]
  loading: boolean
  modal_open: boolean
  fetch_tokens: (ring_id: string) => Promise<void>
  create_token: (ring_id: string, input: CreateInviteInput) => Promise<InviteToken>
  revoke_token: (ring_id: string, token: string) => Promise<void>
  fetch_requests: (ring_id: string) => Promise<void>
  approve_request: (ring_id: string, request_id: string) => Promise<void>
  reject_request: (ring_id: string, request_id: string, note?: string) => Promise<void>
  open_modal: () => void
  close_modal: () => void
}

export const useInviteStore = create<InviteState>((set, get) => ({
  tokens: [],
  join_requests: [],
  loading: false,
  modal_open: false,

  fetch_tokens: async (ring_id) => {
    set({ loading: true })
    try {
      const res = await listInviteTokens(ring_id)
      set({ tokens: res.tokens.filter((t) => t.revoked_at === null), loading: false })
    } catch {
      set({ loading: false })
    }
  },

  create_token: async (ring_id, input) => {
    const token = await createInviteToken(ring_id, input)
    await get().fetch_tokens(ring_id)
    return token
  },

  revoke_token: async (ring_id, token) => {
    await revokeInviteToken(ring_id, token)
    await get().fetch_tokens(ring_id)
  },

  fetch_requests: async (ring_id) => {
    try {
      const res = await listJoinRequests(ring_id, 'pending')
      set({ join_requests: res.requests })
    } catch {
    }
  },

  approve_request: async (ring_id, request_id) => {
    await approveJoinRequest(ring_id, request_id)
    await get().fetch_requests(ring_id)
    await get().fetch_tokens(ring_id)
  },

  reject_request: async (ring_id, request_id, note) => {
    await rejectJoinRequest(ring_id, request_id, note)
    await get().fetch_requests(ring_id)
  },

  open_modal: () => set({ modal_open: true }),
  close_modal: () => set({ modal_open: false }),
}))
