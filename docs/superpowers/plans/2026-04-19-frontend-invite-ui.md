# Frontend Invite UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the complete frontend UI for the invite/join flow — creator-side invite management in ConfigPanel + Modal, and joiner-side join flow in SetupWizard.

**Architecture:** New types, store, and 3 new components. Modify ConfigPanel (add invite management), SetupWizard (add join branch), App.tsx (URL param detection). All inline styles, IceChat theme, no external CSS.

**Tech Stack:** React 19 + TypeScript + Zustand 5 + Vite 8, vitest for tests

---

### Task 1: Create invite types

**Files:**
- Create: `ui/src/types/invite.ts`

- [ ] **Step 1: Create types file**

Create `ui/src/types/invite.ts`:

```typescript
export interface InviteToken {
  token: string
  ring_id: string
  type: 'open' | 'audit'
  role: string
  max_uses: number
  use_count: number
  max_members: number | null
  expires_at: string
  revoked_at: string | null
  created_by: string
  created_at: string
}

export interface JoinRequest {
  id: string
  ring_id: string
  invite_token: string
  display_name: string
  message: string | null
  status: 'pending' | 'approved' | 'rejected'
  reviewer_id: string | null
  review_note: string | null
  reviewed_at: string | null
  created_at: string
}

export interface CreateInviteInput {
  type: 'open' | 'audit'
  role?: string
  max_uses?: number
  max_members?: number | null
  expires_in_hours?: number
}

export interface JoinInfo {
  valid: boolean
  reason?: string | null
  ring_id?: string | null
  ring_name?: string | null
  member_count?: number | null
  role?: string | null
  token_type?: string | null
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/types/invite.ts
git commit -m "Add invite types: InviteToken, JoinRequest, CreateInviteInput, JoinInfo"
```

---

### Task 2: Add invite API functions to api.ts

**Files:**
- Modify: `ui/src/services/api.ts`

- [ ] **Step 1: Add invite API functions**

Add at the END of `ui/src/services/api.ts`:

```typescript
export async function createInviteToken(ring_id: string, input: import('../types/invite').CreateInviteInput): Promise<import('../types/invite').InviteToken> {
  return api.post(`/rings/${ring_id}/invite-tokens`, input)
}

export async function listInviteTokens(ring_id: string): Promise<{ tokens: import('../types/invite').InviteToken[] }> {
  return api.get(`/rings/${ring_id}/invite-tokens`)
}

export async function revokeInviteToken(ring_id: string, token: string): Promise<void> {
  return api.delete(`/rings/${ring_id}/invite-tokens/${token}`)
}

export async function listJoinRequests(ring_id: string, status = 'pending'): Promise<{ requests: import('../types/invite').JoinRequest[] }> {
  return api.get(`/rings/${ring_id}/join-requests?status=${status}`)
}

export async function approveJoinRequest(ring_id: string, request_id: string): Promise<{ ok: boolean; token_id: string; ring_name: string; role: string }> {
  return api.post(`/rings/${ring_id}/join-requests/${request_id}/approve`, {})
}

export async function rejectJoinRequest(ring_id: string, request_id: string, note?: string): Promise<{ ok: boolean }> {
  return api.post(`/rings/${ring_id}/join-requests/${request_id}/reject`, note ? { note } : {})
}

export async function verifyJoinToken(token: string): Promise<import('../types/invite').JoinInfo> {
  return api.get(`/join/info?token=${encodeURIComponent(token)}`)
}

export async function joinRing(invite_token: string, display_name: string): Promise<{ token_id: string; ring_id: string; ring_name: string; role: string; gitlab_repo_url: string | null }> {
  return api.post('/join', { invite_token, display_name })
}

export async function localJoin(invite_token: string, creator_ip: string): Promise<{ ok: boolean; ring_id: string; ring_name: string; role: string }> {
  return api.post('/join/local', { invite_token, creator_ip })
}

export async function applyJoin(invite_token: string, display_name: string, message?: string): Promise<{ request_id: string; status: string; ring_name: string }> {
  return api.post('/join/apply', { invite_token, display_name, message })
}

export async function checkApplyStatus(request_id: string): Promise<{ request_id: string; status: string; ring_name: string | null; ring_id: string | null; role: string | null; review_note: string | null; token_id: string | null }> {
  return api.get(`/join/apply/status?id=${encodeURIComponent(request_id)}`)
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/services/api.ts
git commit -m "Add invite API functions: token CRUD, join, apply, approve/reject"
```

---

### Task 3: Create invite store

**Files:**
- Create: `ui/src/stores/invite-store.ts`
- Create: `ui/src/test/stores/invite-store.test.ts`

- [ ] **Step 1: Create invite store**

Create `ui/src/stores/invite-store.ts`:

```typescript
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
      // ignore
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
```

- [ ] **Step 2: Create invite store test**

Create `ui/src/test/stores/invite-store.test.ts`:

```typescript
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
```

- [ ] **Step 3: Run tests**

Run: `cd ui && npx vitest run src/test/stores/invite-store.test.ts`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add ui/src/stores/invite-store.ts ui/src/test/stores/invite-store.test.ts
git commit -m "Add invite store: token CRUD, join requests, modal state"
```

---

### Task 4: Create Modal component

**Files:**
- Create: `ui/src/components/common/Modal.tsx`

- [ ] **Step 1: Create Modal component**

Create `ui/src/components/common/Modal.tsx`:

```typescript
import type { ReactNode } from 'react'

interface ModalProps {
  open: boolean
  on_close: () => void
  children: ReactNode
}

export function Modal({ open, on_close, children }: ModalProps) {
  if (!open) return null

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 1000,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background: 'rgba(0, 0, 0, 0.5)',
        }}
        onClick={on_close}
      />
      <div
        style={{
          position: 'relative',
          zIndex: 1,
          width: '100%',
          maxWidth: 480,
          maxHeight: '90vh',
          overflowY: 'auto',
          background: 'var(--bg-panel)',
          border: '1px solid var(--border)',
          borderRadius: 8,
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {children}
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/common/Modal.tsx
git commit -m "Add generic Modal component with backdrop"
```

---

### Task 5: Create CreateInviteModal component

**Files:**
- Create: `ui/src/components/invite/CreateInviteModal.tsx`

- [ ] **Step 1: Create CreateInviteModal component**

Create `ui/src/components/invite/CreateInviteModal.tsx`:

```typescript
import { useState } from 'react'
import { Modal } from '../common/Modal'
import { useInviteStore } from '../../stores/invite-store'
import { useRingStore } from '../../stores/ring-store'
import type { InviteToken } from '../../types/invite'

export function CreateInviteModal() {
  const modal_open = useInviteStore((s) => s.modal_open)
  const close_modal = useInviteStore((s) => s.close_modal)
  const create_token = useInviteStore((s) => s.create_token)
  const active_ring_id = useRingStore((s) => s.active_ring_id)

  const [link_type, set_link_type] = useState<'open' | 'audit'>('open')
  const [role, set_role] = useState('member')
  const [max_uses, set_max_uses] = useState(1)
  const [max_members, set_max_members] = useState<string>('')
  const [expires_hours, set_expires_hours] = useState(24)
  const [created_token, set_created_token] = useState<InviteToken | null>(null)
  const [creating, set_creating] = useState(false)
  const [copied, set_copied] = useState(false)

  const handle_create = async () => {
    if (!active_ring_id) return
    set_creating(true)
    try {
      const token = await create_token(active_ring_id, {
        type: link_type,
        role,
        max_uses,
        max_members: max_members ? parseInt(max_members, 10) : null,
        expires_in_hours: expires_hours,
      })
      set_created_token(token)
    } catch {
      // error handled silently
    } finally {
      set_creating(false)
    }
  }

  const handle_copy = async () => {
    if (!created_token) return
    const base = window.location.host
    const link = `http://${base}/ring/join?token=${created_token.token}`
    await navigator.clipboard.writeText(link)
    set_copied(true)
    setTimeout(() => set_copied(false), 2000)
  }

  const handle_another = () => {
    set_created_token(null)
    set_copied(false)
  }

  const handle_done = () => {
    set_created_token(null)
    set_copied(false)
    set_link_type('open')
    set_role('member')
    set_max_uses(1)
    set_max_members('')
    set_expires_hours(24)
    close_modal()
  }

  const handleClose = () => {
    handle_done()
  }

  return (
    <Modal open={modal_open} on_close={handleClose}>
      {created_token ? (
        <div>
          <div style={{ padding: '14px 20px', borderBottom: '1px solid var(--border)', background: 'var(--bg-sidebar)', display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{ color: 'var(--accent-green)', fontSize: 12, fontWeight: 600, letterSpacing: 1 }}>✓ LINK CREATED</span>
            <span style={{ marginLeft: 'auto', color: 'var(--text-dim)', cursor: 'pointer', fontSize: 16, padding: '2px 6px', borderRadius: 3 }} onClick={handleClose}>×</span>
          </div>
          <div style={{ padding: 20 }}>
            <div style={{ background: 'var(--bg-active)', border: '1px solid var(--accent-cyan)', borderRadius: 4, padding: 12, marginBottom: 16 }}>
              <div style={{ fontSize: 9, color: 'var(--accent-cyan)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 6 }}>Invite Link</div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <code style={{ flex: 1, fontSize: 10, color: 'var(--accent-ice)', wordBreak: 'break-all', lineHeight: 1.5 }}>
                  {`${window.location.origin}/ring/join?token=${created_token.token}`}
                </code>
                <div
                  style={{ padding: '6px 12px', background: 'var(--accent-cyan)', color: 'var(--bg-base)', borderRadius: 3, fontSize: 9, fontWeight: 700, letterSpacing: 1, cursor: 'pointer', whiteSpace: 'nowrap' }}
                  onClick={handle_copy}
                >
                  {copied ? 'COPIED' : 'COPY'}
                </div>
              </div>
            </div>
            <div style={{ display: 'flex', gap: 16, marginBottom: 16, fontSize: 10, color: 'var(--text-secondary)' }}>
              <div><span style={{ color: 'var(--text-dim)' }}>Type:</span> {created_token.type}</div>
              <div><span style={{ color: 'var(--text-dim)' }}>Role:</span> {created_token.role}</div>
              <div><span style={{ color: 'var(--text-dim)' }}>Uses:</span> {created_token.use_count}/{created_token.max_uses}</div>
              <div><span style={{ color: 'var(--text-dim)' }}>Expires:</span> {expires_hours}h</div>
            </div>
            <div style={{ display: 'flex', gap: 8 }}>
              <div style={{ flex: 1, padding: 8, border: '1px solid var(--border)', borderRadius: 4, textAlign: 'center', fontSize: 10, color: 'var(--text-secondary)', cursor: 'pointer' }} onClick={handle_another}>CREATE ANOTHER</div>
              <div style={{ flex: 1, padding: 8, border: '1px solid var(--border)', borderRadius: 4, textAlign: 'center', fontSize: 10, color: 'var(--text-secondary)', cursor: 'pointer' }} onClick={handle_done}>DONE</div>
            </div>
          </div>
        </div>
      ) : (
        <div>
          <div style={{ padding: '14px 20px', borderBottom: '1px solid var(--border)', background: 'var(--bg-sidebar)', display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{ color: 'var(--accent-ice)', fontSize: 12, fontWeight: 600, letterSpacing: 1 }}>🔗 CREATE INVITE</span>
            <span style={{ marginLeft: 'auto', color: 'var(--text-dim)', cursor: 'pointer', fontSize: 16, padding: '2px 6px', borderRadius: 3 }} onClick={handleClose}>×</span>
          </div>
          <div style={{ padding: 20 }}>
            <div style={{ marginBottom: 16 }}>
              <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 8 }}>Link Type</div>
              <div style={{ display: 'flex', gap: 8 }}>
                <div
                  style={{ flex: 1, padding: '10px 14px', border: `1px solid ${link_type === 'open' ? 'var(--accent-cyan)' : 'var(--border)'}`, borderRadius: 4, background: link_type === 'open' ? 'var(--bg-active)' : 'transparent', cursor: 'pointer' }}
                  onClick={() => set_link_type('open')}
                >
                  <div style={{ fontSize: 11, color: link_type === 'open' ? 'var(--accent-cyan)' : 'var(--text-secondary)', fontWeight: 600 }}>Open Link</div>
                  <div style={{ fontSize: 9, color: 'var(--text-muted)', marginTop: 2 }}>Join directly, no approval needed</div>
                </div>
                <div
                  style={{ flex: 1, padding: '10px 14px', border: `1px solid ${link_type === 'audit' ? 'var(--accent-cyan)' : 'var(--border)'}`, borderRadius: 4, background: link_type === 'audit' ? 'var(--bg-active)' : 'transparent', cursor: 'pointer' }}
                  onClick={() => set_link_type('audit')}
                >
                  <div style={{ fontSize: 11, color: link_type === 'audit' ? 'var(--accent-cyan)' : 'var(--text-secondary)', fontWeight: 600 }}>Audit Link</div>
                  <div style={{ fontSize: 9, color: 'var(--text-muted)', marginTop: 2 }}>Requires creator approval</div>
                </div>
              </div>
            </div>

            <div style={{ marginBottom: 16 }}>
              <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 8 }}>Role</div>
              <div style={{ display: 'flex', gap: 8 }}>
                {['member', 'admin', 'readonly'].map((r) => (
                  <div
                    key={r}
                    style={{ flex: 1, padding: '8px 12px', border: `1px solid ${role === r ? 'var(--accent-cyan)' : 'var(--border)'}`, borderRadius: 4, textAlign: 'center', fontSize: 10, color: role === r ? 'var(--accent-cyan)' : 'var(--text-dim)', background: role === r ? 'var(--bg-active)' : 'transparent', cursor: 'pointer' }}
                    onClick={() => set_role(r)}
                  >
                    {r}
                  </div>
                ))}
              </div>
            </div>

            <div style={{ display: 'flex', gap: 12, marginBottom: 20 }}>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 6 }}>Max Uses</div>
                <input
                  type="number"
                  value={max_uses}
                  onChange={(e) => set_max_uses(parseInt(e.target.value, 10) || 0)}
                  style={{ width: '100%', background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontSize: 11, fontFamily: 'inherit', outline: 'none' }}
                />
              </div>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 6 }}>Max Members</div>
                <input
                  type="number"
                  value={max_members}
                  placeholder="no limit"
                  onChange={(e) => set_max_members(e.target.value)}
                  style={{ width: '100%', background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontSize: 11, fontFamily: 'inherit', outline: 'none' }}
                />
              </div>
              <div style={{ flex: 1 }}>
                <div style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, marginBottom: 6 }}>Expires (h)</div>
                <input
                  type="number"
                  value={expires_hours}
                  onChange={(e) => set_expires_hours(parseInt(e.target.value, 10) || 1)}
                  style={{ width: '100%', background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontSize: 11, fontFamily: 'inherit', outline: 'none' }}
                />
              </div>
            </div>

            <div
              style={{ padding: 10, background: 'var(--accent-cyan)', color: 'var(--bg-base)', borderRadius: 4, textAlign: 'center', fontSize: 11, fontWeight: 700, letterSpacing: 1, cursor: creating ? 'not-allowed' : 'pointer', opacity: creating ? 0.6 : 1 }}
              onClick={creating ? undefined : handle_create}
            >
              {creating ? 'GENERATING...' : 'GENERATE LINK'}
            </div>
          </div>
        </div>
      )}
    </Modal>
  )
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/invite/CreateInviteModal.tsx
git commit -m "Add CreateInviteModal: form + link display with copy"
```

---

### Task 6: Update ConfigPanel with invite management

**Files:**
- Modify: `ui/src/components/panels/ConfigPanel.tsx`

- [ ] **Step 1: Update ConfigPanel**

Replace the full content of `ui/src/components/panels/ConfigPanel.tsx` with:

```typescript
import { useEffect } from 'react'
import type { Member } from '../../types/ring'
import type { LLMConfig, LLMProvider } from '../../types/config'
import { api } from '../../services/api'
import { useRingStore } from '../../stores/ring-store'
import { useInviteStore } from '../../stores/invite-store'
import { useAuthStore } from '../../stores/auth-store'

export function ConfigPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const rings = useRingStore((s) => s.rings)
  const [members, setMembers] = useState<Member[]>([])
  const [llmConfig, setLlmConfig] = useState<LLMConfig | null>(null)

  const tokens = useInviteStore((s) => s.tokens)
  const join_requests = useInviteStore((s) => s.join_requests)
  const fetch_tokens = useInviteStore((s) => s.fetch_tokens)
  const revoke_token = useInviteStore((s) => s.revoke_token)
  const fetch_requests = useInviteStore((s) => s.fetch_requests)
  const approve_request = useInviteStore((s) => s.approve_request)
  const reject_request = useInviteStore((s) => s.reject_request)
  const open_modal = useInviteStore((s) => s.open_modal)
  const auth_token = useAuthStore((s) => s.token)

  const active_ring = rings.find((r) => r.id === active_ring_id)
  const is_admin = active_ring?.role === 'creator' || active_ring?.role === 'admin'

  useEffect(() => {
    api.get<{ provider: string; model: string; api_key_set: boolean; base_url: string | null }>('/config/llm')
      .then((res) => setLlmConfig({ ...res, provider: res.provider as LLMProvider }))
      .catch(() => {})
  }, [])

  useEffect(() => {
    if (!active_ring_id) return
    api.get<{ members: Member[] }>(`/rings/${active_ring_id}/members`)
      .then((res) => setMembers(res.members))
      .catch(() => {})
    if (is_admin) {
      fetch_tokens(active_ring_id)
      fetch_requests(active_ring_id)
    }
  }, [active_ring_id, is_admin])

  const handle_revoke = async (token: string) => {
    if (!active_ring_id) return
    await revoke_token(active_ring_id, token)
  }

  const handle_approve = async (request_id: string) => {
    if (!active_ring_id) return
    await approve_request(active_ring_id, request_id)
  }

  const handle_reject = async (request_id: string) => {
    if (!active_ring_id) return
    const note = window.prompt('Rejection reason (optional):')
    await reject_request(active_ring_id, request_id, note || undefined)
  }

  const time_remaining = (expires_at: string) => {
    const diff = new Date(expires_at).getTime() - Date.now()
    if (diff <= 0) return 'expired'
    const hours = Math.floor(diff / 3600000)
    if (hours > 24) return `${Math.floor(hours / 24)}d left`
    return `${hours}h left`
  }

  return (
    <div style={{ fontSize: 12 }}>
      <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
        LLM Config
      </p>
      {llmConfig && (
        <div style={{ marginBottom: 16, color: 'var(--text-primary)', lineHeight: 1.8 }}>
          <div>Provider: <span style={{ color: 'var(--accent-ice)' }}>{llmConfig.provider}</span></div>
          <div>Model: <span style={{ color: 'var(--accent-ice)' }}>{llmConfig.model}</span></div>
          <div>API Key: {llmConfig.api_key_set ? '✓' : '✗'}</div>
        </div>
      )}

      <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
        Members
      </p>
      {members.map((m) => (
        <div
          key={m.token_id}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            padding: '4px 0',
            color: 'var(--text-primary)',
          }}
        >
          <span>{m.display_name}</span>
          <span style={{ color: 'var(--text-dim)', fontSize: 11 }}>({m.role})</span>
          {m.online && (
            <span style={{ color: 'var(--accent-green)', fontSize: 10 }}>●</span>
          )}
        </div>
      ))}
      {members.length === 0 && (
        <p style={{ color: 'var(--text-dim)' }}>No members</p>
      )}

      {is_admin && (
        <div
          style={{ marginTop: 8, padding: '5px 8px', border: '1px solid var(--accent-cyan)', borderRadius: 3, textAlign: 'center', color: 'var(--accent-cyan)', cursor: 'pointer', fontSize: 10 }}
          onClick={open_modal}
        >
          + invite member
        </div>
      )}

      {is_admin && tokens.length > 0 && (
        <>
          <p style={{ marginTop: 16, marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
            Active Invites · {tokens.length}
          </p>
          {tokens.map((t) => (
            <div key={t.token} style={{ padding: '6px 8px', border: '1px solid var(--border)', borderRadius: 3, marginBottom: 3, display: 'flex', alignItems: 'center', gap: 6, fontSize: 10 }}>
              <span style={{ color: t.type === 'open' ? 'var(--accent-cyan)' : 'var(--accent-amber)', fontSize: 9 }}>
                {t.type}
              </span>
              <span style={{ flex: 1, color: 'var(--text-muted)', fontSize: 9 }}>
                {t.use_count}/{t.max_uses} uses · {time_remaining(t.expires_at)}
              </span>
              <span style={{ color: 'var(--text-dim)', fontSize: 9, cursor: 'pointer' }} onClick={() => handle_revoke(t.token)}>
                revoke
              </span>
            </div>
          ))}
        </>
      )}

      {is_admin && join_requests.length > 0 && (
        <>
          <p style={{ marginTop: 16, marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
            Pending Requests · {join_requests.length}
          </p>
          {join_requests.map((req) => (
            <div key={req.id} style={{ padding: 8, border: '1px solid var(--accent-amber)', borderRadius: 3, marginBottom: 3 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4, fontSize: 10 }}>
                <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>{req.display_name}</span>
                <span style={{ color: 'var(--accent-amber)', fontSize: 8 }}>{req.invite_token ? 'audit' : ''}</span>
              </div>
              {req.message && (
                <div style={{ color: 'var(--text-muted)', fontSize: 9, marginBottom: 6 }}>"{req.message}"</div>
              )}
              <div style={{ display: 'flex', gap: 6 }}>
                <div
                  style={{ flex: 1, padding: 4, background: 'var(--accent-green)', color: 'var(--bg-base)', borderRadius: 2, textAlign: 'center', fontSize: 9, fontWeight: 700, cursor: 'pointer' }}
                  onClick={() => handle_approve(req.id)}
                >
                  APPROVE
                </div>
                <div
                  style={{ flex: 1, padding: 4, border: '1px solid var(--border)', borderRadius: 2, textAlign: 'center', fontSize: 9, color: 'var(--text-secondary)', cursor: 'pointer' }}
                  onClick={() => handle_reject(req.id)}
                >
                  REJECT
                </div>
              </div>
            </div>
          ))}
        </>
      )}
    </div>
  )
}
```

**Important:** The file needs `import { useState } from 'react'` added at the top. The import block should be:

```typescript
import { useEffect, useState } from 'react'
```

- [ ] **Step 2: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/panels/ConfigPanel.tsx
git commit -m "Update ConfigPanel: add invite button, active invites, pending requests"
```

---

### Task 7: Mount CreateInviteModal in AppLayout

**Files:**
- Modify: `ui/src/components/layout/AppLayout.tsx`

- [ ] **Step 1: Add CreateInviteModal to AppLayout**

Add import at the top of `ui/src/components/layout/AppLayout.tsx`:

```typescript
import { CreateInviteModal } from '../invite/CreateInviteModal'
```

Then add `<CreateInviteModal />` as the last child inside the returned JSX, alongside existing `<SelfFloat />` and `<SelfTrigger />`.

- [ ] **Step 2: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/layout/AppLayout.tsx
git commit -m "Mount CreateInviteModal in AppLayout"
```

---

### Task 8: Create StepJoin component

**Files:**
- Create: `ui/src/components/setup/StepJoin.tsx`

- [ ] **Step 1: Create StepJoin component**

Create `ui/src/components/setup/StepJoin.tsx`:

```typescript
import { useState, useEffect } from 'react'
import { useAppStore } from '../../stores/app-store'
import { useAuthStore } from '../../stores/auth-store'
import { verifyJoinToken, localJoin, applyJoin, checkApplyStatus } from '../../services/api'
import type { JoinInfo } from '../../types/invite'

interface StepJoinProps {
  initial_token?: string
  initial_creator_ip?: string
}

export function StepJoin({ initial_token, initial_creator_ip }: StepJoinProps) {
  const [invite_link, set_invite_link] = useState(initial_token ? `token=${initial_token}` : '')
  const [creator_ip, set_creator_ip] = useState(initial_creator_ip || '')
  const [display_name, set_display_name] = useState('')
  const [join_info, set_join_info] = useState<JoinInfo | null>(null)
  const [message, set_message] = useState('')
  const [error, set_error] = useState<string | null>(null)
  const [loading, set_loading] = useState(false)
  const [status, set_status] = useState<'idle' | 'verified' | 'joining' | 'polling' | 'done'>('idle')

  const setSetup = useAppStore((s) => s.setSetup)
  const setAuth = useAuthStore((s) => s.setAuth)

  useEffect(() => {
    if (initial_token) {
      handle_verify(initial_token)
    }
  }, [])

  const parse_token = (input: string): { token: string; ip?: string } => {
    const trimmed = input.trim()
    try {
      const url = new URL(trimmed)
      const token = url.searchParams.get('token') || ''
      const ip = url.searchParams.get('creator_ip') || url.hostname
      return { token, ip: ip || undefined }
    } catch {
      if (trimmed.includes('=')) {
        const params = new URLSearchParams(trimmed)
        return { token: params.get('token') || '', ip: params.get('creator_ip') || undefined }
      }
      return { token: trimmed }
    }
  }

  const handle_verify = async (token_input?: string) => {
    const input = token_input || invite_link
    const { token, ip } = parse_token(input)
    if (!token) { set_error('No token found'); return }
    if (ip) set_creator_ip(ip)

    set_loading(true)
    set_error(null)
    try {
      const info = await verifyJoinToken(token)
      if (info.valid) {
        set_join_info(info)
        set_status('verified')
      } else {
        set_error(info.reason || 'Invalid invite link')
      }
    } catch {
      set_error('Failed to verify invite link')
    } finally {
      set_loading(false)
    }
  }

  const handle_join = async () => {
    if (!display_name.trim()) { set_error('Display name is required'); return }
    const { token } = parse_token(invite_link)
    if (!token) return

    set_loading(true)
    set_error(null)
    set_status('joining')

    try {
      if (join_info?.token_type === 'audit') {
        const res = await applyJoin(token, display_name.trim(), message || undefined)
        set_status('polling')
        poll_status(res.request_id)
      } else if (creator_ip) {
        const res = await localJoin(token, creator_ip)
        setAuth(res.ring_id, display_name.trim(), null)
        set_status('done')
        setSetup(true)
      } else {
        set_error('Creator IP is required for open join')
        set_loading(false)
      }
    } catch (e: unknown) {
      set_error(e instanceof Error ? e.message : 'Join failed')
      set_loading(false)
    }
  }

  const poll_status = (request_id: string) => {
    const interval = setInterval(async () => {
      try {
        const res = await checkApplyStatus(request_id)
        if (res.status === 'approved') {
          clearInterval(interval)
          set_status('done')
          setSetup(true)
        } else if (res.status === 'rejected') {
          clearInterval(interval)
          set_error(res.review_note ? `Rejected: ${res.review_note}` : 'Application rejected')
          set_loading(false)
        }
      } catch {
        // keep polling
      }
    }, 3000)
  }

  return (
    <div style={{ maxWidth: 480, width: '100%', padding: '40px 20px', textAlign: 'center' }}>
      <h1 style={{ fontSize: 20, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 6 }}>
        Join a Ring
      </h1>
      <p style={{ color: 'var(--text-secondary)', marginBottom: 24, fontSize: 12 }}>
        Paste the invite link shared by the Ring creator.
      </p>

      {status === 'idle' && (
        <>
          <div style={{ textAlign: 'left', marginBottom: 16 }}>
            <label style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, display: 'block', marginBottom: 6 }}>Invite Link / Code</label>
            <div style={{ display: 'flex', gap: 8 }}>
              <input
                value={invite_link}
                onChange={(e) => set_invite_link(e.target.value)}
                placeholder="http://192.168.x.x:7420/ring/join?token=xxx"
                style={{ flex: 1, background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontFamily: 'inherit', fontSize: 12, outline: 'none' }}
              />
              <button
                onClick={() => handle_verify()}
                disabled={loading}
                style={{ padding: '8px 16px', background: 'var(--accent-cyan)', color: 'var(--bg-base)', border: 'none', borderRadius: 4, fontSize: 10, fontWeight: 700, cursor: 'pointer', letterSpacing: 1 }}
              >
                VERIFY
              </button>
            </div>
          </div>
          {error && <p style={{ color: 'var(--accent-amber)', fontSize: 11, marginBottom: 12 }}>{error}</p>}
        </>
      )}

      {status === 'verified' && join_info && (
        <div style={{ textAlign: 'left' }}>
          <div style={{ background: 'var(--bg-active)', border: '1px solid var(--accent-cyan)', borderRadius: 4, padding: 12, marginBottom: 16 }}>
            <div style={{ fontSize: 14, fontWeight: 600, color: 'var(--accent-ice)', marginBottom: 4 }}>{join_info.ring_name}</div>
            <div style={{ fontSize: 10, color: 'var(--text-secondary)' }}>
              Members: {join_info.member_count} · Role: {join_info.role} · Type: {join_info.token_type}
            </div>
          </div>
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, display: 'block', marginBottom: 6 }}>Display Name</label>
            <input
              value={display_name}
              onChange={(e) => set_display_name(e.target.value)}
              placeholder="Your name"
              style={{ width: '100%', background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontFamily: 'inherit', fontSize: 12, outline: 'none' }}
            />
          </div>
          {join_info.token_type === 'audit' && (
            <div style={{ marginBottom: 12 }}>
              <label style={{ fontSize: 9, color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: 1.5, display: 'block', marginBottom: 6 }}>Message (optional)</label>
              <input
                value={message}
                onChange={(e) => set_message(e.target.value)}
                placeholder="Why do you want to join?"
                style={{ width: '100%', background: 'var(--bg-input)', border: '1px solid var(--border)', borderRadius: 4, padding: '8px 12px', color: 'var(--text-primary)', fontFamily: 'inherit', fontSize: 12, outline: 'none' }}
              />
            </div>
          )}
          {error && <p style={{ color: 'var(--accent-amber)', fontSize: 11, marginBottom: 12 }}>{error}</p>}
          <button
            onClick={handle_join}
            disabled={loading}
            style={{ width: '100%', padding: 10, background: 'var(--accent-cyan)', color: 'var(--bg-base)', border: 'none', borderRadius: 4, fontSize: 11, fontWeight: 700, cursor: loading ? 'not-allowed' : 'pointer', letterSpacing: 1, opacity: loading ? 0.6 : 1 }}
          >
            {loading ? 'JOINING...' : `JOIN "${join_info.ring_name}"`}
          </button>
        </div>
      )}

      {status === 'polling' && (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <p style={{ color: 'var(--text-secondary)', fontSize: 12 }}>Application submitted. Waiting for approval...</p>
          <p style={{ color: 'var(--text-dim)', fontSize: 10, marginTop: 8 }}>This page will auto-update when approved.</p>
        </div>
      )}

      {status === 'done' && (
        <div style={{ textAlign: 'center', padding: '40px 0' }}>
          <div style={{ fontSize: 32, marginBottom: 12 }}>🎉</div>
          <p style={{ color: 'var(--accent-green)', fontSize: 14, fontWeight: 600 }}>Successfully joined!</p>
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/setup/StepJoin.tsx
git commit -m "Add StepJoin: joiner invite verification and join flow"
```

---

### Task 9: Update StepWelcome + SetupWizard for join branch

**Files:**
- Modify: `ui/src/components/setup/StepWelcome.tsx`
- Modify: `ui/src/components/setup/SetupWizard.tsx`

- [ ] **Step 1: Update StepWelcome**

Replace `ui/src/components/setup/StepWelcome.tsx` with:

```typescript
interface StepProps {
  onNext: () => void
  onJoin: () => void
}

export function StepWelcome({ onNext, onJoin }: StepProps) {
  return (
    <div style={{ textAlign: 'center', padding: '40px 20px' }}>
      <div style={{ fontSize: 48, marginBottom: 16 }}>
        <img src="/logo-pixel.svg" alt="Ring" width="48" height="48" />
      </div>
      <h1 style={{ fontSize: 24, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 8 }}>
        Welcome to Ring
      </h1>
      <p style={{ color: 'var(--text-secondary)', marginBottom: 32, maxWidth: 400, margin: '0 auto 32px' }}>
        Group Knowledge Collaboration Space
      </p>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 12, maxWidth: 240, margin: '0 auto' }}>
        <button
          onClick={onNext}
          style={{
            background: 'var(--accent-cyan)',
            color: 'var(--bg-base)',
            border: 'none',
            borderRadius: 4,
            padding: '10px 32px',
            fontSize: 13,
            fontWeight: 700,
            cursor: 'pointer',
            letterSpacing: 1,
          }}
        >
          NEW USER
        </button>
        <button
          onClick={onJoin}
          style={{
            background: 'transparent',
            color: 'var(--text-secondary)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '10px 32px',
            fontSize: 13,
            fontWeight: 700,
            cursor: 'pointer',
            letterSpacing: 1,
          }}
        >
          JOIN EXISTING
        </button>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Update SetupWizard**

Replace `ui/src/components/setup/SetupWizard.tsx` with:

```typescript
import { useState } from 'react'
import { useAppStore } from '../../stores/app-store'
import { useAuthStore } from '../../stores/auth-store'
import { api } from '../../services/api'
import { StepWelcome } from './StepWelcome'
import { StepIdentity } from './StepIdentity'
import { StepLLM } from './StepLLM'
import { StepGitLab } from './StepGitLab'
import { StepDone } from './StepDone'
import { StepJoin } from './StepJoin'

export interface SetupData {
  display_name: string
  avatar: string | null
  llm_provider: string
  llm_api_key: string
  llm_model: string
  llm_base_url: string
  gitlab_url: string
  gitlab_token: string
}

interface JoinParams {
  token?: string
  creator_ip?: string
}

export function SetupWizard({ join_params }: { join_params?: JoinParams }) {
  const [step, setStep] = useState(0)
  const [mode, setMode] = useState<'setup' | 'join'>(join_params?.token ? 'join' : 'setup')
  const [data, setData] = useState<SetupData>({
    display_name: '',
    avatar: null,
    llm_provider: 'openai',
    llm_api_key: '',
    llm_model: 'gpt-4o',
    llm_base_url: '',
    gitlab_url: '',
    gitlab_token: '',
  })
  const [token, setToken] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const setSetup = useAppStore((s) => s.setSetup)
  const setAuth = useAuthStore((s) => s.setAuth)

  const goNext = () => setStep((s) => Math.min(s + 1, 4))
  const goBack = () => setStep((s) => Math.max(s - 1, 0))

  const handleSubmit = async () => {
    setError(null)
    try {
      const res = await api.post<{ token_id: string; display_name: string; avatar: string | null }>('/setup', {
        display_name: data.display_name,
        avatar: data.avatar,
        llm_provider: data.llm_provider,
        llm_api_key: data.llm_provider !== 'ollama' ? data.llm_api_key : null,
        llm_model: data.llm_model || undefined,
        llm_base_url: data.llm_base_url || undefined,
        gitlab_url: data.gitlab_url,
        gitlab_token: data.gitlab_token,
      })
      setToken(res.token_id)
      setAuth(res.token_id, res.display_name, res.avatar)
      goNext()
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Setup failed'
      setError(msg)
    }
  }

  if (mode === 'join') {
    return (
      <div style={{ height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg-base)' }}>
        <StepJoin initial_token={join_params?.token} initial_creator_ip={join_params?.creator_ip} />
      </div>
    )
  }

  const steps = [
    <StepWelcome key="welcome" onNext={goNext} onJoin={() => setMode('join')} />,
    <StepIdentity key="identity" data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={goNext} onBack={goBack} />,
    <StepLLM key="llm" data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={goNext} onBack={goBack} />,
    <StepGitLab key="gitlab" data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={handleSubmit} onBack={goBack} error={error} />,
    <StepDone key="done" token={token} onEnter={() => setSetup(true)} />,
  ]

  return (
    <div
      style={{
        height: '100%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'var(--bg-base)',
      }}
    >
      {steps[step]}
    </div>
  )
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add ui/src/components/setup/StepWelcome.tsx ui/src/components/setup/SetupWizard.tsx
git commit -m "Add join branch to SetupWizard with 'Join Existing' button"
```

---

### Task 10: Add URL parameter detection in App.tsx

**Files:**
- Modify: `ui/src/App.tsx`

- [ ] **Step 1: Update App.tsx**

Replace `ui/src/App.tsx` with:

```typescript
import { useEffect } from 'react'
import { useAppStore } from './stores/app-store'
import { useAuthStore } from './stores/auth-store'
import { AppLayout } from './components/layout/AppLayout'
import { SetupWizard } from './components/setup/SetupWizard'
import './index.css'

function getJoinParams(): { token?: string; creator_ip?: string } | undefined {
  const params = new URLSearchParams(window.location.search)
  const token = params.get('token')
  const creator_ip = params.get('creator_ip')
  if (token) return { token, creator_ip: creator_ip || undefined }
  return undefined
}

export default function App() {
  const { is_setup, loading, init } = useAppStore()
  const loadFromStorage = useAuthStore((s) => s.loadFromStorage)

  useEffect(() => {
    loadFromStorage()
    init()
  }, [init, loadFromStorage])

  if (loading) {
    return (
      <div style={{ height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', background: 'var(--bg-base)' }}>
        <span style={{ color: 'var(--text-dim)', fontSize: 12 }}>Loading...</span>
      </div>
    )
  }

  const join_params = getJoinParams()

  if (!is_setup) {
    return <SetupWizard join_params={join_params} />
  }

  return <AppLayout />
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/App.tsx
git commit -m "Add URL param detection for auto-join from install nav page"
```

---

### Task 11: Final verification

- [ ] **Step 1: Run all frontend tests**

Run: `cd ui && npx vitest run`
Expected: all tests pass

- [ ] **Step 2: Run TypeScript check**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Run ESLint**

Run: `cd ui && npx eslint src/`
Expected: no errors

- [ ] **Step 4: Run build**

Run: `cd ui && npm run build`
Expected: build succeeds
