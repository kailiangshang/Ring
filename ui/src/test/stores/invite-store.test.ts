import { describe, it, expect, beforeEach } from 'vitest'
import { useInviteStore } from '../../stores/invite-store'

describe('inviteStore', () => {
  beforeEach(() => {
    useInviteStore.setState({
      tokens: [],
      join_requests: [],
      loading: false,
      modal_open: false,
    })
  })

  it('opens and closes modal', () => {
    useInviteStore.getState().open_modal()
    expect(useInviteStore.getState().modal_open).toBe(true)
    useInviteStore.getState().close_modal()
    expect(useInviteStore.getState().modal_open).toBe(false)
  })

  it('initializes with empty tokens and requests', () => {
    const state = useInviteStore.getState()
    expect(state.tokens).toEqual([])
    expect(state.join_requests).toEqual([])
    expect(state.loading).toBe(false)
    expect(state.modal_open).toBe(false)
  })
})
