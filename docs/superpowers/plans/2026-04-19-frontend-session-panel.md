# Frontend SessionPanel Implementation Plan (5c)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the complete frontend Session experience — WebSocket client, session store, and full SessionPanel UI with create form + real-time chat + sidebar integration.

**Architecture:** Three-layer approach: `ws-client.ts` (raw WebSocket utility) → `ws-store.ts` (Zustand connection state + message routing) → `session-store.ts` (session CRUD + messages). SessionPanel renders create form or chat view based on active session state. Sidebar shows live session indicator.

**Tech Stack:** React 19, TypeScript, Zustand 5, Vite 8. No new dependencies.

---

## File Structure

```
ui/src/
├── types/
│   └── session.ts              # MODIFY — add SessionMessage, SessionParticipant, CreateSessionInput, WS message types
├── services/
│   └── ws-client.ts            # CREATE — WebSocket client wrapper with reconnect + heartbeat
├── stores/
│   ├── ws-store.ts             # CREATE — WS connection state + message routing
│   └── session-store.ts        # CREATE — Session CRUD + message state
├── components/
│   └── panels/
│       └── SessionPanel.tsx    # REWRITE — full UI (create form + chat view)
│   └── sidebar/
│       └── SessionIndicator.tsx # MODIFY — show title + participants, clickable
```

---

### Task 1: Update session types

**Files:**
- Modify: `ui/src/types/session.ts`

- [ ] **Step 1: Rewrite session.ts with full types matching backend response**

```typescript
export type SessionPhase = 'material_prep' | 'discussion' | 'summary' | 'closed'
export type SessionSkill = 'decision' | 'research' | 'review' | 'retrospective' | 'knowledge_sharing' | 'discussion'

export interface Session {
  id: string
  ring_id: string
  title: string
  description: string
  skill: SessionSkill
  phase: SessionPhase
  owner: string
  archivable: boolean
  archive_enabled: boolean
  summary: string | null
  created_at: string
  updated_at: string
}

export interface SessionParticipant {
  session_id: string
  token_id: string
  role: 'owner' | 'participant'
  joined_at: string
}

export interface SessionMessage {
  id: string
  session_id: string
  seq_num: number
  sender: string
  sender_name: string
  content: string
  message_type: 'user' | 'system' | 'ai_delta' | 'ai_end'
  created_at: string
}

export interface CreateSessionInput {
  title: string
  description?: string
  skill: SessionSkill
  archivable?: boolean
  invitees?: string[]
}

export interface SessionDetail {
  session: Session
  participants: SessionParticipant[]
}
```

- [ ] **Step 2: Verify no TypeScript errors from type changes**

Run: `cd ui && npx tsc --noEmit 2>&1 | head -20`

Fix any files that import from `types/session.ts` (they will break because `Session` gained `ring_id`, `summary`, `updated_at` and lost `participants`). Update mock-data.ts to match.

- [ ] **Step 3: Commit**

```bash
git add ui/src/types/session.ts ui/src/services/mock-data.ts
git commit -m "refactor(ui): expand session types to match backend API"
```

---

### Task 2: Create WebSocket client

**Files:**
- Create: `ui/src/services/ws-client.ts`

- [ ] **Step 1: Create ws-client.ts**

```typescript
type MessageHandler = (data: unknown) => void

export class WsClient {
  private ws: WebSocket | null = null
  private url: string
  private on_message: MessageHandler
  private on_open: () => void
  private on_close: () => void
  private reconnect_attempts = 0
  private max_reconnect_delay = 30_000
  private reconnect_timer: ReturnType<typeof setTimeout> | null = null
  private heartbeat_timer: ReturnType<typeof setInterval> | null = null
  private stopped = false

  constructor(
    url: string,
    on_message: MessageHandler,
    on_open: () => void,
    on_close: () => void,
  ) {
    this.url = url
    this.on_message = on_message
    this.on_open = on_open
    this.on_close = on_close
  }

  connect(): void {
    this.stopped = false
    this.ws = new WebSocket(this.url)

    this.ws.onopen = () => {
      this.reconnect_attempts = 0
      this.start_heartbeat()
      this.on_open()
    }

    this.ws.onmessage = (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data as string)
        if (data.type === 'ping') {
          this.send({ type: 'pong', data: data.data ?? '' })
          return
        }
        this.on_message(data)
      } catch {
        // skip malformed
      }
    }

    this.ws.onclose = () => {
      this.stop_heartbeat()
      this.on_close()
      if (!this.stopped) this.schedule_reconnect()
    }

    this.ws.onerror = () => {
      this.ws?.close()
    }
  }

  send(data: unknown): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data))
    }
  }

  disconnect(): void {
    this.stopped = true
    if (this.reconnect_timer) {
      clearTimeout(this.reconnect_timer)
      this.reconnect_timer = null
    }
    this.stop_heartbeat()
    this.ws?.close()
    this.ws = null
  }

  get connected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN
  }

  private schedule_reconnect(): void {
    const delay = Math.min(1000 * 2 ** this.reconnect_attempts, this.max_reconnect_delay)
    this.reconnect_attempts++
    this.reconnect_timer = setTimeout(() => this.connect(), delay)
  }

  private start_heartbeat(): void {
    this.stop_heartbeat()
    this.heartbeat_timer = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ type: 'ping' }))
      }
    }, 30_000)
  }

  private stop_heartbeat(): void {
    if (this.heartbeat_timer) {
      clearInterval(this.heartbeat_timer)
      this.heartbeat_timer = null
    }
  }
}
```

- [ ] **Step 2: Verify compiles**

Run: `cd ui && npx tsc --noEmit`

- [ ] **Step 3: Commit**

```bash
git add ui/src/services/ws-client.ts
git commit -m "feat(ui): WebSocket client with reconnect and heartbeat"
```

---

### Task 3: Create WebSocket store

**Files:**
- Create: `ui/src/stores/ws-store.ts`

- [ ] **Step 1: Create ws-store.ts**

This store manages WS connection lifecycle and routes incoming messages to session-store.

```typescript
import { create } from 'zustand'
import { WsClient } from '../services/ws-client'

type WsMessageHandler = (data: unknown) => void

interface WsState {
  connected: boolean
  connecting: boolean
  client: WsClient | null
  handlers: WsMessageHandler[]
  addHandler: (handler: WsMessageHandler) => void
  removeHandler: (handler: WsMessageHandler) => void
  connect: () => void
  disconnect: () => void
  send: (data: unknown) => void
}

export const useWsStore = create<WsState>((set, get) => ({
  connected: false,
  connecting: false,
  client: null,
  handlers: [],

  addHandler: (handler) => {
    set((s) => ({ handlers: [...s.handlers, handler] }))
  },

  removeHandler: (handler) => {
    set((s) => ({ handlers: s.handlers.filter((h) => h !== handler) }))
  },

  connect: () => {
    const token = localStorage.getItem('ring_token')
    if (!token) return

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const url = `${protocol}//${window.location.host}/api/ws?token=${encodeURIComponent(token)}`

    const client = new WsClient(
      url,
      (data) => {
        const { handlers } = get()
        for (const handler of handlers) {
          handler(data)
        }
      },
      () => set({ connected: true, connecting: false }),
      () => set({ connected: false, connecting: false }),
    )

    set({ client, connecting: true })
    client.connect()
  },

  disconnect: () => {
    const { client } = get()
    client?.disconnect()
    set({ client: null, connected: false, connecting: false })
  },

  send: (data) => {
    get().client?.send(data)
  },
}))
```

- [ ] **Step 2: Verify compiles**

Run: `cd ui && npx tsc --noEmit`

- [ ] **Step 3: Commit**

```bash
git add ui/src/stores/ws-store.ts
git commit -m "feat(ui): WebSocket store with message routing"
```

---

### Task 4: Create session store

**Files:**
- Create: `ui/src/stores/session-store.ts`

- [ ] **Step 1: Create session-store.ts**

```typescript
import { create } from 'zustand'
import { api } from '../services/api'
import { useWsStore } from './ws-store'
import { useRingStore } from './ring-store'
import type { Session, SessionParticipant, SessionMessage, CreateSessionInput, SessionDetail } from '../types/session'

interface SessionState {
  active_session: Session | null
  participants: SessionParticipant[]
  messages: SessionMessage[]
  loading: boolean
  list: Session[]
  createSession: (input: CreateSessionInput) => Promise<SessionDetail | null>
  fetchActiveSession: (ring_id: string) => Promise<void>
  fetchSessions: (ring_id: string) => Promise<void>
  closeSession: (ring_id: string, session_id: string) => Promise<void>
  reopenSession: (ring_id: string, session_id: string) => Promise<void>
  deleteSession: (ring_id: string, session_id: string) => Promise<void>
  inviteParticipants: (ring_id: string, session_id: string, token_ids: string[]) => Promise<void>
  removeParticipant: (ring_id: string, session_id: string, token_id: string) => Promise<void>
  toggleArchive: (ring_id: string, session_id: string) => Promise<void>
  sendMessage: (session_id: string, content: string) => void
  handleWsMessage: (data: unknown) => void
  fetchMessages: (ring_id: string, session_id: string) => Promise<void>
  clearActive: () => void
}

export const useSessionStore = create<SessionState>((set, get) => ({
  active_session: null,
  participants: [],
  messages: [],
  loading: false,
  list: [],

  createSession: async (input) => {
    const ring_id = useRingStore.getState().active_ring_id
    if (!ring_id) return null
    try {
      const res = await api.post<SessionDetail>(`/rings/${ring_id}/sessions`, input)
      set({
        active_session: res.session,
        participants: res.participants,
        messages: [],
      })
      return res
    } catch {
      return null
    }
  },

  fetchActiveSession: async (ring_id) => {
    set({ loading: true })
    try {
      const res = await api.get<{ sessions: SessionDetail[] }>(`/rings/${ring_id}/sessions?status=active`)
      if (res.sessions && res.sessions.length > 0) {
        const detail = res.sessions[0]
        set({
          active_session: detail.session,
          participants: detail.participants,
          loading: false,
        })
        get().fetchMessages(ring_id, detail.session.id)
      } else {
        set({ active_session: null, participants: [], messages: [], loading: false })
      }
    } catch {
      set({ loading: false })
    }
  },

  fetchSessions: async (ring_id) => {
    try {
      const res = await api.get<{ sessions: SessionDetail[] }>(`/rings/${ring_id}/sessions`)
      const sessions = res.sessions.map((s) => s.session)
      set({ list: sessions })
    } catch {
      // keep existing
    }
  },

  closeSession: async (ring_id, session_id) => {
    try {
      await api.post<SessionDetail>(`/rings/${ring_id}/sessions/${session_id}/close`, {})
      const detail = await api.get<SessionDetail>(`/rings/${ring_id}/sessions/${session_id}`)
      set({ active_session: detail.session })
    } catch {
      // keep state
    }
  },

  reopenSession: async (ring_id, session_id) => {
    try {
      await api.post<SessionDetail>(`/rings/${ring_id}/sessions/${session_id}/reopen`, {})
      const detail = await api.get<SessionDetail>(`/rings/${ring_id}/sessions/${session_id}`)
      set({ active_session: detail.session, messages: [] })
    } catch {
      // keep state
    }
  },

  deleteSession: async (ring_id, session_id) => {
    try {
      await api.delete(`/rings/${ring_id}/sessions/${session_id}`)
      set({ active_session: null, participants: [], messages: [] })
    } catch {
      // keep state
    }
  },

  inviteParticipants: async (ring_id, session_id, token_ids) => {
    try {
      await api.post(`/rings/${ring_id}/sessions/${session_id}/participants`, { token_ids })
      const detail = await api.get<SessionDetail>(`/rings/${ring_id}/sessions/${session_id}`)
      set({ participants: detail.participants })
    } catch {
      // keep state
    }
  },

  removeParticipant: async (ring_id, session_id, token_id) => {
    try {
      await api.delete(`/rings/${ring_id}/sessions/${session_id}/participants/${token_id}`)
      set((s) => ({
        participants: s.participants.filter((p) => p.token_id !== token_id),
      }))
    } catch {
      // keep state
    }
  },

  toggleArchive: async (ring_id, session_id) => {
    try {
      await api.put(`/rings/${ring_id}/sessions/${session_id}/archive-toggle`, {})
      if (get().active_session) {
        set((s) => ({
          active_session: s.active_session
            ? { ...s.active_session, archive_enabled: !s.active_session.archive_enabled }
            : null,
        }))
      }
    } catch {
      // keep state
    }
  },

  sendMessage: (session_id, content) => {
    useWsStore.getState().send({
      type: 'session_message',
      session_id,
      content,
    })
  },

  handleWsMessage: (data: unknown) => {
    const msg = data as Record<string, unknown>
    if (!msg || typeof msg.type !== 'string') return

    const { active_session } = get()
    const session_id = msg.session_id as string | undefined

    switch (msg.type) {
      case 'session_message': {
        if (!session_id || !active_session || session_id !== active_session.id) return
        const incoming: SessionMessage = {
          id: msg.id as string,
          session_id,
          seq_num: msg.seq_num as number,
          sender: msg.sender as string,
          sender_name: msg.sender_name as string,
          content: msg.content as string,
          message_type: 'user',
          created_at: msg.created_at as string,
        }
        set((s) => ({ messages: [...s.messages, incoming] }))
        break
      }
      case 'session_catchup': {
        if (!session_id || !active_session || session_id !== active_session.id) return
        const messages = (msg.messages as SessionMessage[]) ?? []
        set({ messages })
        break
      }
      case 'session_paused': {
        if (active_session && session_id === active_session.id) {
          set((s) => ({
            active_session: s.active_session
              ? { ...s.active_session, phase: 'closed' as const }
              : null,
          }))
        }
        break
      }
      case 'session_resumed': {
        if (active_session && session_id === active_session.id) {
          set((s) => ({
            active_session: s.active_session
              ? { ...s.active_session, phase: 'discussion' as const }
              : null,
          }))
        }
        break
      }
    }
  },

  fetchMessages: async (ring_id, session_id) => {
    try {
      const res = await api.get<{ messages: SessionMessage[] }>(
        `/rings/${ring_id}/sessions/${session_id}/messages?after_seq=0&limit=100`,
      )
      set({ messages: res.messages ?? [] })
    } catch {
      // keep existing
    }
  },

  clearActive: () => set({ active_session: null, participants: [], messages: [] }),
}))
```

- [ ] **Step 2: Verify compiles**

Run: `cd ui && npx tsc --noEmit`

- [ ] **Step 3: Commit**

```bash
git add ui/src/stores/session-store.ts
git commit -m "feat(ui): session store with CRUD and WS message handling"
```

---

### Task 5: Rewrite SessionPanel

**Files:**
- Rewrite: `ui/src/components/panels/SessionPanel.tsx`

- [ ] **Step 1: Rewrite SessionPanel.tsx with create form + chat view**

The panel has two views:
1. **No active session** → Create session form (title, description, skill selector, archive toggle, invitees)
2. **Active session** → Chat view (header with session info, message list, input, action buttons)

```tsx
import { useEffect, useState } from 'react'
import { useSessionStore } from '../../stores/session-store'
import { useRingStore } from '../../stores/ring-store'
import { useWsStore } from '../../stores/ws-store'
import { ScrollContainer } from '../common/ScrollContainer'
import type { SessionSkill } from '../../types/session'

const SKILLS: { value: SessionSkill; label: string }[] = [
  { value: 'discussion', label: 'Discussion' },
  { value: 'decision', label: 'Decision' },
  { value: 'research', label: 'Research' },
  { value: 'review', label: 'Review' },
  { value: 'retrospective', label: 'Retrospective' },
  { value: 'knowledge_sharing', label: 'Knowledge Sharing' },
]

function CreateSessionForm() {
  const createSession = useSessionStore((s) => s.createSession)
  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [skill, setSkill] = useState<SessionSkill>('discussion')
  const [archivable, setArchivable] = useState(false)

  const handleCreate = async () => {
    if (!title.trim()) return
    await createSession({
      title: title.trim(),
      description: description.trim() || undefined,
      skill,
      archivable: archivable || undefined,
    })
    setTitle('')
    setDescription('')
    setSkill('discussion')
    setArchivable(false)
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)', letterSpacing: '0.05em' }}>
        New Session
      </div>

      <input
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        placeholder="Session title..."
        style={{
          background: 'var(--bg-input)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '8px 10px',
          color: 'var(--text-primary)',
          fontSize: 12,
          fontFamily: 'inherit',
          outline: 'none',
          width: '100%',
          boxSizing: 'border-box',
        }}
      />

      <textarea
        value={description}
        onChange={(e) => setDescription(e.target.value)}
        placeholder="Description (optional)..."
        rows={2}
        style={{
          background: 'var(--bg-input)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '8px 10px',
          color: 'var(--text-primary)',
          fontSize: 11,
          fontFamily: 'inherit',
          outline: 'none',
          resize: 'vertical',
          width: '100%',
          boxSizing: 'border-box',
        }}
      />

      <div>
        <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4, letterSpacing: '0.05em' }}>
          Skill
        </div>
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
          {SKILLS.map((s) => (
            <button
              key={s.value}
              onClick={() => setSkill(s.value)}
              style={{
                background: skill === s.value ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                color: skill === s.value ? 'var(--bg-base)' : 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '3px 8px',
                fontSize: 10,
                cursor: 'pointer',
              }}
            >
              {s.label}
            </button>
          ))}
        </div>
      </div>

      <label style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 11, color: 'var(--text-secondary)', cursor: 'pointer' }}>
        <input
          type="checkbox"
          checked={archivable}
          onChange={(e) => setArchivable(e.target.checked)}
          style={{ accentColor: 'var(--accent-cyan)' }}
        />
        Archive enabled
      </label>

      <button
        onClick={handleCreate}
        disabled={!title.trim()}
        style={{
          background: title.trim() ? 'var(--accent-cyan)' : 'var(--bg-hover)',
          color: title.trim() ? 'var(--bg-base)' : 'var(--text-dim)',
          border: 'none',
          borderRadius: 4,
          padding: '8px 16px',
          fontSize: 12,
          fontWeight: 700,
          cursor: title.trim() ? 'pointer' : 'default',
          letterSpacing: '0.05em',
        }}
      >
        CREATE
      </button>
    </div>
  )
}

function SessionChat() {
  const session = useSessionStore((s) => s.active_session)
  const participants = useSessionStore((s) => s.participants)
  const messages = useSessionStore((s) => s.messages)
  const sendMessage = useSessionStore((s) => s.sendMessage)
  const closeSession = useSessionStore((s) => s.closeSession)
  const reopenSession = useSessionStore((s) => s.reopenSession)
  const deleteSession = useSessionStore((s) => s.deleteSession)
  const toggleArchive = useSessionStore((s) => s.toggleArchive)
  const clearActive = useSessionStore((s) => s.clearActive)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const connected = useWsStore((s) => s.connected)

  const [input, setInput] = useState('')

  if (!session) return null

  const is_owner = true
  const is_closed = session.phase === 'closed'
  const is_discussion = session.phase === 'discussion'
  const can_send = is_discussion && connected && !is_closed

  const handleSend = () => {
    if (!input.trim()) return
    sendMessage(session.id, input.trim())
    setInput('')
  }

  const handleClose = async () => {
    if (!active_ring_id) return
    await closeSession(active_ring_id, session.id)
  }

  const handleReopen = async () => {
    if (!active_ring_id) return
    await reopenSession(active_ring_id, session.id)
  }

  const handleDelete = async () => {
    if (!active_ring_id) return
    await deleteSession(active_ring_id, session.id)
  }

  const handleArchive = async () => {
    if (!active_ring_id) return
    await toggleArchive(active_ring_id, session.id)
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)', marginBottom: 4 }}>
        <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 4 }}>
          {session.title}
        </div>
        <div style={{ fontSize: 10, color: 'var(--text-dim)', display: 'flex', gap: 8, flexWrap: 'wrap' }}>
          <span>Skill: {session.skill}</span>
          <span style={{ color: is_closed ? 'var(--accent-amber)' : 'var(--accent-green)' }}>
            Phase: {session.phase}
          </span>
          <span>{participants.length} participants</span>
          {!connected && <span style={{ color: 'var(--accent-amber)' }}>disconnected</span>}
        </div>
      </div>

      <ScrollContainer>
        {messages.map((msg) => (
          <div
            key={msg.id}
            style={{
              padding: '6px 0',
              borderBottom: '1px solid var(--border)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 2 }}>
              <span
                style={{
                  fontSize: 10,
                  fontWeight: 700,
                  color: msg.sender === session.owner ? 'var(--accent-ice)' : 'var(--accent-cyan)',
                  letterSpacing: '0.05em',
                }}
              >
                {msg.sender_name.toUpperCase()}
              </span>
              <span style={{ fontSize: 9, color: 'var(--text-dim)' }}>
                {new Date(msg.created_at).toLocaleTimeString()}
              </span>
            </div>
            <div style={{ color: 'var(--text-primary)', fontSize: 11, whiteSpace: 'pre-wrap', lineHeight: 1.5 }}>
              {msg.content}
            </div>
          </div>
        ))}
        {messages.length === 0 && (
          <div style={{ padding: '16px 0', color: 'var(--text-dim)', fontSize: 11, textAlign: 'center' }}>
            No messages yet
          </div>
        )}
      </ScrollContainer>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 8 }}>
        {can_send && (
          <div style={{ display: 'flex', gap: 6, marginBottom: 8 }}>
            <input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  handleSend()
                }
              }}
              placeholder="message..."
              style={{
                flex: 1,
                background: 'var(--bg-input)',
                border: '1px solid var(--border)',
                borderRadius: 4,
                padding: '6px 10px',
                color: 'var(--text-primary)',
                fontSize: 11,
                fontFamily: 'inherit',
                outline: 'none',
              }}
            />
            <button
              onClick={handleSend}
              disabled={!input.trim()}
              style={{
                background: input.trim() ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                color: input.trim() ? 'var(--bg-base)' : 'var(--text-dim)',
                border: 'none',
                borderRadius: 4,
                padding: '6px 12px',
                fontSize: 11,
                fontWeight: 700,
                cursor: input.trim() ? 'pointer' : 'default',
              }}
            >
              SEND
            </button>
          </div>
        )}

        <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
          {is_discussion && is_owner && (
            <button
              onClick={handleClose}
              style={{
                background: 'var(--bg-hover)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '3px 8px',
                fontSize: 10,
                color: 'var(--accent-amber)',
                cursor: 'pointer',
              }}
            >
              Close
            </button>
          )}
          {is_closed && (
            <>
              <button
                onClick={handleReopen}
                style={{
                  background: 'var(--bg-hover)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  color: 'var(--accent-green)',
                  cursor: 'pointer',
                }}
              >
                Reopen
              </button>
              <button
                onClick={handleDelete}
                style={{
                  background: 'var(--bg-hover)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '3px 8px',
                  fontSize: 10,
                  color: 'var(--accent-amber)',
                  cursor: 'pointer',
                }}
              >
                Delete
              </button>
            </>
          )}
          {session.archivable && (
            <button
              onClick={handleArchive}
              style={{
                background: 'var(--bg-hover)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '3px 8px',
                fontSize: 10,
                color: session.archive_enabled ? 'var(--accent-cyan)' : 'var(--text-dim)',
                cursor: 'pointer',
              }}
            >
              {session.archive_enabled ? 'Archive: ON' : 'Archive: OFF'}
            </button>
          )}
          <button
            onClick={clearActive}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--text-dim)',
              cursor: 'pointer',
              fontSize: 10,
              marginLeft: 'auto',
            }}
          >
            Back to list
          </button>
        </div>
      </div>
    </div>
  )
}

export function SessionPanel() {
  const active_session = useSessionStore((s) => s.active_session)
  const loading = useSessionStore((s) => s.loading)
  const fetchActiveSession = useSessionStore((s) => s.fetchActiveSession)
  const handleWsMessage = useSessionStore((s) => s.handleWsMessage)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const connected = useWsStore((s) => s.connected)
  const wsConnect = useWsStore((s) => s.connect)
  const addHandler = useWsStore((s) => s.addHandler)
  const removeHandler = useWsStore((s) => s.removeHandler)

  useEffect(() => {
    wsConnect()
  }, [wsConnect])

  useEffect(() => {
    addHandler(handleWsMessage)
    return () => removeHandler(handleWsMessage)
  }, [addHandler, removeHandler, handleWsMessage])

  useEffect(() => {
    if (active_ring_id && !active_session) {
      fetchActiveSession(active_ring_id)
    }
  }, [active_ring_id, active_session, fetchActiveSession])

  if (loading) {
    return (
      <div style={{ padding: 16, color: 'var(--text-dim)', fontSize: 12 }}>
        Loading session...
      </div>
    )
  }

  if (active_session) {
    return <SessionChat />
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <CreateSessionForm />
      {!connected && (
        <div style={{ marginTop: 8, fontSize: 10, color: 'var(--accent-amber)' }}>
          WebSocket disconnected — messages may be delayed
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Verify compiles**

Run: `cd ui && npx tsc --noEmit`

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/panels/SessionPanel.tsx
git commit -m "feat(ui): full SessionPanel with create form and real-time chat"
```

---

### Task 6: Update sidebar SessionIndicator

**Files:**
- Modify: `ui/src/components/sidebar/SessionIndicator.tsx`
- Modify: `ui/src/components/sidebar/RingList.tsx`

- [ ] **Step 1: Update SessionIndicator to show title + participant count + be clickable**

```tsx
import { useSessionStore } from '../../stores/session-store'
import { usePanelStore } from '../../stores/panel-store'

export function SessionIndicator() {
  const session = useSessionStore((s) => s.active_session)
  const participants = useSessionStore((s) => s.participants)
  const toggle = usePanelStore((s) => s.toggle)

  return (
    <div
      onClick={() => toggle('session')}
      style={{
        marginLeft: 28,
        padding: '4px 8px',
        fontSize: 11,
        color: 'var(--text-muted)',
        display: 'flex',
        alignItems: 'center',
        gap: 6,
        cursor: 'pointer',
      }}
    >
      <span
        style={{
          width: 5,
          height: 5,
          borderRadius: '50%',
          background: 'var(--accent-green)',
          flexShrink: 0,
        }}
      />
      {session ? `${session.title} · ${participants.length}` : '1 active session'}
    </div>
  )
}
```

- [ ] **Step 2: Verify RingList still works with updated SessionIndicator (no changes needed to RingList)**

Run: `cd ui && npx tsc --noEmit`

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/sidebar/SessionIndicator.tsx
git commit -m "feat(ui): session indicator shows title and participant count"
```

---

### Task 7: Final verification

- [ ] **Step 1: Full TypeScript check**

Run: `cd ui && npx tsc --noEmit`

- [ ] **Step 2: Build check**

Run: `cd ui && npm run build`

- [ ] **Step 3: Backend tests still pass**

Run: `cd server && cargo test`

- [ ] **Step 4: Final commit if any fixes needed**

---

## Notes

- The WS store auto-connects when SessionPanel mounts. If the user never opens a session, no WS connection is made (could optimize later to defer connection).
- `is_owner` in SessionChat is hardcoded to `true` — in a real multi-user scenario, compare `session.owner` with the current user's token_id from auth store. For single-user local app, this is always true.
- Material prep view and summarize view are deferred to Plan 5d.
- The session store's `handleWsMessage` is registered as a WS handler on mount and unregistered on unmount, following React cleanup pattern.
