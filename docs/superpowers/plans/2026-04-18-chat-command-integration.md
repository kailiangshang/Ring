# Chat + Command System + API Integration Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace all mock data with real backend API calls — Setup wizard creates a real user, Ring list comes from the database, CLI command parsing (@/#/!/%) works, and mode changes sync to the server.

**Architecture:** Add a thin `services/api.ts` fetch wrapper that reads the auth token from localStorage. All stores gain async fetch methods. Setup wizard submits real data. CLI command parser runs locally in the chat input handler — no backend parsing needed yet (SSE/LLM streaming deferred to Plan 4).

**Tech Stack:** React 19, TypeScript, Zustand 5, Vite 8, Vitest

**Scope:** This plan covers the API integration layer, Setup flow, Ring CRUD, Member list, Config panel, Mode sync, and CLI command parsing. Chat/SSE streaming, Graph visualization, Session lifecycle, and WebSocket are deferred to later plans.

---

## File Structure

```
ui/src/
├── services/
│   ├── api.ts                      # NEW: fetch wrapper with auth token
│   ├── mock-data.ts                # MODIFY: keep for fallback, remove imports from stores
│   └── command-parser.ts           # NEW: @/#/!/% command parser
├── stores/
│   ├── app-store.ts                # MODIFY: add init() that checks setup status
│   ├── ring-store.ts               # MODIFY: fetch from API instead of mock
│   ├── chat-store.ts               # MODIFY: command dispatch on send
│   ├── mode-store.ts               # MODIFY: sync mode changes to API
│   └── auth-store.ts               # NEW: token + user identity persistence
├── components/
│   ├── setup/
│   │   ├── SetupWizard.tsx         # MODIFY: collect all data, submit to API on Done
│   │   ├── StepIdentity.tsx        # MODIFY: lift state to wizard
│   │   ├── StepLLM.tsx             # MODIFY: lift state to wizard
│   │   ├── StepGitLab.tsx          # MODIFY: lift state to wizard
│   │   └── StepDone.tsx            # MODIFY: show token confirmation
│   ├── chat/
│   │   ├── InputArea.tsx           # MODIFY: handle Enter key, dispatch commands
│   │   ├── CommandHints.tsx        # MODIFY: already functional, no changes needed
│   │   ├── Autocomplete.tsx        # NEW: autocomplete dropdown for commands
│   │   └── ModeIndicator.tsx       # MODIFY: sync on click
│   ├── sidebar/
│   │   └── RingList.tsx            # MODIFY: fetch rings on mount
│   ├── panels/
│   │   ├── ConfigPanel.tsx         # MODIFY: fetch members + config from API
│   │   └── GraphPanel.tsx          # No change (placeholder)
│   └── layout/
│       └── AppLayout.tsx           # MODIFY: init app on mount
└── test/
    └── services/
        └── command-parser.test.ts  # NEW: unit tests for command parser
```

---

### Task 1: API Service Layer + Auth Store

**Files:**
- Create: `ui/src/services/api.ts`
- Create: `ui/src/stores/auth-store.ts`

- [ ] **Step 1: Create services/api.ts**

```typescript
const API_BASE = '/api'

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
  ) {
    super(message)
  }
}

async function getToken(): Promise<string | null> {
  return localStorage.getItem('ring_token')
}

export async function setToken(token: string): Promise<void> {
  localStorage.setItem('ring_token', token)
}

export async function clearToken(): Promise<void> {
  localStorage.removeItem('ring_token')
}

async function request<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const token = await getToken()
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string>),
  }
  if (token) {
    headers['X-Ring-Token'] = token
  }

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
  })

  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new ApiError(
      res.status,
      body?.error?.code ?? 'unknown',
      body?.error?.message ?? res.statusText,
    )
  }

  if (res.status === 204) return undefined as T
  return res.json()
}

export const api = {
  get: <T>(path: string) => request<T>(path),

  post: <T>(path: string, body: unknown) =>
    request<T>(path, { method: 'POST', body: JSON.stringify(body) }),

  put: <T>(path: string, body: unknown) =>
    request<T>(path, { method: 'PUT', body: JSON.stringify(body) }),

  delete: <T>(path: string) =>
    request<T>(path, { method: 'DELETE' }),
}
```

- [ ] **Step 2: Create stores/auth-store.ts**

```typescript
import { create } from 'zustand'

interface AuthState {
  token: string | null
  display_name: string | null
  avatar: string | null
  isAuthenticated: boolean
  setAuth: (token: string, display_name: string, avatar: string | null) => void
  logout: () => void
  loadFromStorage: () => void
}

export const useAuthStore = create<AuthState>((set) => ({
  token: null,
  display_name: null,
  avatar: null,
  isAuthenticated: false,

  setAuth: (token, display_name, avatar) => {
    localStorage.setItem('ring_token', token)
    set({ token, display_name, avatar, isAuthenticated: true })
  },

  logout: () => {
    localStorage.removeItem('ring_token')
    set({ token: null, display_name: null, avatar: null, isAuthenticated: false })
  },

  loadFromStorage: () => {
    const token = localStorage.getItem('ring_token')
    if (token) {
      set({ token, isAuthenticated: true })
    }
  },
}))
```

- [ ] **Step 3: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add ui/src/services/api.ts ui/src/stores/auth-store.ts
git commit -m "feat(ui): add API service layer and auth store"
```

---

### Task 2: App Init — Check Setup Status on Load

**Files:**
- Modify: `ui/src/stores/app-store.ts`
- Modify: `ui/src/App.tsx`

- [ ] **Step 1: Update app-store.ts to add init()**

```typescript
import { create } from 'zustand'
import { api } from '../services/api'

interface AppState {
  is_setup: boolean
  loading: boolean
  current_context: 'super' | 'ring' | 'session' | 'self'
  active_ring_id: string | null
  active_session_id: string | null
  init: () => Promise<void>
  setSetup: (done: boolean) => void
  setContext: (ctx: AppState['current_context']) => void
  setActiveRing: (ring_id: string | null) => void
  setActiveSession: (session_id: string | null) => void
}

export const useAppStore = create<AppState>((set) => ({
  is_setup: false,
  loading: true,
  current_context: 'super',
  active_ring_id: null,
  active_session_id: null,

  init: async () => {
    try {
      const res = await api.get<{ is_setup: boolean; step: string | null }>('/setup/status')
      set({ is_setup: res.is_setup, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  setSetup: (done) => set({ is_setup: done }),
  setContext: (ctx) => set({ current_context: ctx }),
  setActiveRing: (ring_id) => set({ active_ring_id: ring_id, current_context: ring_id ? 'ring' : 'super' }),
  setActiveSession: (session_id) => set({ active_session_id: session_id, current_context: session_id ? 'session' : 'ring' }),
}))
```

- [ ] **Step 2: Update App.tsx to call init()**

```typescript
import { useEffect } from 'react'
import { useAppStore } from './stores/app-store'
import { useAuthStore } from './stores/auth-store'
import { AppLayout } from './components/layout/AppLayout'
import { SetupWizard } from './components/setup/SetupWizard'
import './index.css'

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

  if (!is_setup) {
    return <SetupWizard />
  }

  return <AppLayout />
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add ui/src/stores/app-store.ts ui/src/App.tsx
git commit -m "feat(ui): app init checks setup status from API on load"
```

---

### Task 3: Setup Wizard — Submit Real Data

**Files:**
- Modify: `ui/src/components/setup/SetupWizard.tsx`
- Modify: `ui/src/components/setup/StepIdentity.tsx`
- Modify: `ui/src/components/setup/StepLLM.tsx`
- Modify: `ui/src/components/setup/StepGitLab.tsx`
- Modify: `ui/src/components/setup/StepDone.tsx`

- [ ] **Step 1: Rewrite SetupWizard.tsx with lifted state and API call**

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

export function SetupWizard() {
  const [step, setStep] = useState(0)
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

  const steps = [
    <StepWelcome onNext={goNext} />,
    <StepIdentity data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={goNext} onBack={goBack} />,
    <StepLLM data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={goNext} onBack={goBack} />,
    <StepGitLab data={data} onChange={(d) => setData((prev) => ({ ...prev, ...d }))} onNext={handleSubmit} onBack={goBack} error={error} />,
    <StepDone token={token} onEnter={() => setSetup(true)} />,
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

- [ ] **Step 2: Update StepIdentity.tsx to use props**

```typescript
import type { SetupData } from './SetupWizard'

interface StepProps {
  data: SetupData
  onChange: (partial: Partial<SetupData>) => void
  onNext: () => void
  onBack: () => void
}

const EMOJIS = ['🦊', '🐱', '🌟', '🚀', '🎯', '💡', '🔥', '🌈', '⚡', '🍀', '🦋', '🎪']

const navButtonStyle: React.CSSProperties = {
  background: 'var(--bg-hover)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 20px',
  fontSize: 12,
  cursor: 'pointer',
  fontFamily: 'inherit',
}

export function StepIdentity({ data, onChange, onNext, onBack }: StepProps) {
  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 1: Identity
      </h2>

      <label style={{ fontSize: 11, color: 'var(--text-dim)', letterSpacing: '0.05em' }}>
        Display Name
      </label>
      <input
        value={data.display_name}
        onChange={(e) => onChange({ display_name: e.target.value })}
        placeholder="Enter your name"
        style={{
          width: '100%',
          background: 'var(--bg-input)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '8px 12px',
          color: 'var(--text-primary)',
          fontSize: 13,
          fontFamily: 'inherit',
          outline: 'none',
          marginBottom: 16,
          marginTop: 4,
        }}
      />

      <label style={{ fontSize: 11, color: 'var(--text-dim)', letterSpacing: '0.05em' }}>
        Avatar
      </label>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, marginTop: 4 }}>
        {EMOJIS.map((emoji) => (
          <button
            key={emoji}
            onClick={() => onChange({ avatar: emoji })}
            style={{
              width: 36,
              height: 36,
              background: data.avatar === emoji ? 'var(--accent-amber)' : 'var(--bg-hover)',
              border: data.avatar === emoji ? '2px solid var(--accent-amber)' : '1px solid var(--border)',
              borderRadius: 4,
              fontSize: 18,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            {emoji}
          </button>
        ))}
      </div>

      <div style={{ display: 'flex', gap: 8, marginTop: 24 }}>
        <button onClick={onBack} style={navButtonStyle}>
          Back
        </button>
        <button
          onClick={onNext}
          disabled={!data.display_name.trim()}
          style={{
            ...navButtonStyle,
            opacity: data.display_name.trim() ? 1 : 0.4,
            marginLeft: 'auto',
          }}
        >
          Next
        </button>
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Update StepLLM.tsx to use props**

```typescript
import type { LLMProvider } from '../../types/config'
import type { SetupData } from './SetupWizard'

interface StepProps {
  data: SetupData
  onChange: (partial: Partial<SetupData>) => void
  onNext: () => void
  onBack: () => void
}

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 12px',
  color: 'var(--text-primary)',
  fontSize: 13,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 12,
  marginTop: 4,
}

const navButtonStyle: React.CSSProperties = {
  background: 'var(--bg-hover)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 20px',
  fontSize: 12,
  cursor: 'pointer',
  fontFamily: 'inherit',
}

export function StepLLM({ data, onChange, onNext, onBack }: StepProps) {
  const provider = data.llm_provider as LLMProvider

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 2: LLM Config
      </h2>

      <div style={{ display: 'flex', gap: 4, marginBottom: 16 }}>
        {(['openai', 'anthropic', 'ollama'] as const).map((p) => (
          <button
            key={p}
            onClick={() => onChange({ llm_provider: p })}
            style={{
              background: provider === p ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: provider === p ? 'var(--bg-base)' : 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '6px 14px',
              fontSize: 12,
              cursor: 'pointer',
              fontWeight: provider === p ? 700 : 400,
            }}
          >
            {p}
          </button>
        ))}
      </div>

      {provider !== 'ollama' && (
        <>
          <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>API Key</label>
          <input
            type="password"
            value={data.llm_api_key}
            onChange={(e) => onChange({ llm_api_key: e.target.value })}
            placeholder={`sk-${provider === 'openai' ? 'xxx' : 'ant-xxx'}`}
            style={inputStyle}
          />
        </>
      )}

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>
        Base URL {provider === 'ollama' ? '(e.g. http://localhost:11434)' : '(optional)'}
      </label>
      <input
        value={data.llm_base_url}
        onChange={(e) => onChange({ llm_base_url: e.target.value })}
        placeholder={provider === 'ollama' ? 'http://localhost:11434' : ''}
        style={inputStyle}
      />

      <div style={{ display: 'flex', gap: 8, marginTop: 24 }}>
        <button onClick={onBack} style={navButtonStyle}>Back</button>
        <button
          onClick={onNext}
          disabled={provider !== 'ollama' && !data.llm_api_key.trim()}
          style={{
            ...navButtonStyle,
            opacity: provider !== 'ollama' && !data.llm_api_key.trim() ? 0.4 : 1,
            marginLeft: 'auto',
          }}
        >
          Next
        </button>
      </div>
    </div>
  )
}
```

- [ ] **Step 4: Update StepGitLab.tsx to use props + error display**

```typescript
import type { SetupData } from './SetupWizard'

interface StepProps {
  data: SetupData
  onChange: (partial: Partial<SetupData>) => void
  onNext: () => void
  onBack: () => void
  error: string | null
}

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 12px',
  color: 'var(--text-primary)',
  fontSize: 13,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 12,
  marginTop: 4,
}

const navButtonStyle: React.CSSProperties = {
  background: 'var(--bg-hover)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 20px',
  fontSize: 12,
  cursor: 'pointer',
  fontFamily: 'inherit',
}

export function StepGitLab({ data, onChange, onNext, onBack, error }: StepProps) {
  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 3: GitLab Config
      </h2>

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>GitLab URL</label>
      <input
        value={data.gitlab_url}
        onChange={(e) => onChange({ gitlab_url: e.target.value })}
        placeholder="https://gitlab.company.com"
        style={inputStyle}
      />

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>Personal Access Token</label>
      <input
        type="password"
        value={data.gitlab_token}
        onChange={(e) => onChange({ gitlab_token: e.target.value })}
        placeholder="glpat-xxx"
        style={inputStyle}
      />

      {error && (
        <div style={{ color: 'var(--accent-amber)', fontSize: 11, marginBottom: 8 }}>
          {error}
        </div>
      )}

      <div style={{ display: 'flex', gap: 8, marginTop: 24 }}>
        <button onClick={onBack} style={navButtonStyle}>Back</button>
        <button
          onClick={onNext}
          disabled={!data.gitlab_url.trim() || !data.gitlab_token.trim()}
          style={{
            ...navButtonStyle,
            opacity: !data.gitlab_url.trim() || !data.gitlab_token.trim() ? 0.4 : 1,
            marginLeft: 'auto',
          }}
        >
          Done
        </button>
      </div>
    </div>
  )
}
```

- [ ] **Step 5: Update StepDone.tsx to show token + enter button**

```typescript
interface StepDoneProps {
  token: string | null
  onEnter: () => void
}

export function StepDone({ token, onEnter }: StepDoneProps) {
  return (
    <div style={{ textAlign: 'center', padding: '40px 20px' }}>
      <div style={{ fontSize: 48, marginBottom: 16 }}>&#10003;</div>
      <h1 style={{ fontSize: 20, fontWeight: 700, color: 'var(--accent-green)', marginBottom: 16 }}>
        Setup Complete
      </h1>

      <div
        style={{
          textAlign: 'left',
          background: 'var(--bg-input)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: 16,
          maxWidth: 360,
          margin: '0 auto 20px',
          color: 'var(--text-secondary)',
          fontSize: 12,
          lineHeight: 2,
        }}
      >
        <div style={{ color: 'var(--text-dim)', marginBottom: 4, letterSpacing: '0.05em' }}>
          Quick Commands
        </div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>@self</span> — Open Self</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>#node</span> — Reference graph node</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>!graph</span> — Open Graph panel</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>!save</span> — Trigger archive</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>!auto</span> — Toggle Auto mode</div>
      </div>

      <button
        onClick={onEnter}
        style={{
          background: 'var(--accent-cyan)',
          color: 'var(--bg-base)',
          border: 'none',
          borderRadius: 4,
          padding: '10px 32px',
          fontSize: 13,
          fontWeight: 700,
          cursor: 'pointer',
          letterSpacing: '0.05em',
        }}
      >
        Enter Ring
      </button>
    </div>
  )
}
```

- [ ] **Step 6: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 7: Commit**

```bash
git add ui/src/components/setup/
git commit -m "feat(ui): setup wizard submits real data to API"
```

---

### Task 4: Ring Store — Fetch from API

**Files:**
- Modify: `ui/src/stores/ring-store.ts`

- [ ] **Step 1: Rewrite ring-store.ts with API calls**

```typescript
import { create } from 'zustand'
import type { Ring } from '../types/ring'
import { api } from '../services/api'

interface RingState {
  rings: Ring[]
  loading: boolean
  active_ring_id: string | null
  fetchRings: () => Promise<void>
  createRing: (name: string, role_description: string) => Promise<string | null>
  setRings: (rings: Ring[]) => void
  selectRing: (id: string | null) => void
}

export const useRingStore = create<RingState>((set, get) => ({
  rings: [],
  loading: false,
  active_ring_id: null,

  fetchRings: async () => {
    set({ loading: true })
    try {
      const res = await api.get<{ rings: Ring[] }>('/rings')
      set({ rings: res.rings, loading: false })
    } catch {
      set({ loading: false })
    }
  },

  createRing: async (name, role_description) => {
    try {
      const res = await api.post<{ id: string; name: string; role: string }>('/rings', {
        name,
        role_description,
      })
      await get().fetchRings()
      return res.id
    } catch {
      return null
    }
  },

  setRings: (rings) => set({ rings }),
  selectRing: (id) => set({ active_ring_id: id }),
}))
```

- [ ] **Step 2: Update Sidebar RingList to fetch on mount**

Modify `ui/src/components/sidebar/RingList.tsx`:

```typescript
import { useEffect } from 'react'
import { useRingStore } from '../../stores/ring-store'
import { RingListItem } from './RingListItem'
import { SessionIndicator } from './SessionIndicator'
import { useAppStore } from '../../stores/app-store'

export function RingList() {
  const rings = useRingStore((s) => s.rings)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const fetchRings = useRingStore((s) => s.fetchRings)
  const setActiveRing = useAppStore((s) => s.setActiveRing)
  const selectRing = useRingStore((s) => s.selectRing)

  useEffect(() => {
    fetchRings()
  }, [fetchRings])

  if (rings.length === 0) {
    return (
      <div style={{ padding: '12px', color: 'var(--text-dim)', fontSize: 11 }}>
        No rings yet. Use !new to create one.
      </div>
    )
  }

  return (
    <div style={{ padding: '8px 0' }}>
      {rings.map((ring) => (
        <div key={ring.id}>
          <div onClick={() => { selectRing(ring.id); setActiveRing(ring.id) }}>
            <RingListItem ring={ring} isActive={active_ring_id === ring.id} />
          </div>
          {ring.id === active_ring_id && ring.has_active_session && (
            <SessionIndicator />
          )}
        </div>
      ))}
    </div>
  )
}
```

Also update `RingListItem.tsx` to accept `isActive` prop instead of reading from store:

```typescript
import type { Ring } from '../../types/ring'

interface RingListItemProps {
  ring: Ring
  isActive: boolean
}

export function RingListItem({ ring, isActive }: RingListItemProps) {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        padding: '8px 12px',
        cursor: 'pointer',
        background: isActive ? 'var(--bg-active)' : 'transparent',
        borderRadius: 4,
        margin: '2px 6px',
      }}
    >
      <span
        style={{
          color: isActive ? 'var(--accent-ice)' : 'var(--text-primary)',
          fontWeight: isActive ? 700 : 400,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
        }}
      >
        {ring.name}
      </span>
      <span style={{ marginLeft: 'auto', color: 'var(--text-dim)', fontSize: 11 }}>
        {ring.member_count}
      </span>
      {ring.has_active_session && (
        <span
          title="Active session"
          style={{
            width: 6,
            height: 6,
            borderRadius: '50%',
            background: 'var(--accent-green)',
            flexShrink: 0,
          }}
        />
      )}
    </div>
  )
}
```

- [ ] **Step 3: Update AppLayout to fetch rings on mount**

Modify `ui/src/components/layout/AppLayout.tsx` to add ring fetching:

```typescript
import { useEffect } from 'react'
import { Sidebar } from './Sidebar'
import { HeaderTabBar } from './HeaderTabBar'
import { PanelStack } from './PanelStack'
import { ChatArea } from '../chat/ChatArea'
import { SelfFloat } from '../self/SelfFloat'
import { SelfTrigger } from '../self/SelfTrigger'
import { useAppStore } from '../../stores/app-store'
import { useRingStore } from '../../stores/ring-store'


function SuperRingHeader() {
  return (
    <div
      style={{
        height: 38,
        background: 'var(--bg-panel)',
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        alignItems: 'center',
        padding: '0 12px',
      }}
    >
      <span
        style={{
          fontSize: 13,
          fontWeight: 700,
          color: 'var(--accent-ice)',
          letterSpacing: '0.05em',
        }}
      >
        Super Ring
      </span>
      <span
        style={{
          marginLeft: 12,
          fontSize: 11,
          color: 'var(--text-dim)',
        }}
      >
        Global Assistant
      </span>
    </div>
  )
}

export function AppLayout() {
  const current_context = useAppStore((s) => s.current_context)
  const fetchRings = useRingStore((s) => s.fetchRings)

  useEffect(() => {
    fetchRings()
  }, [fetchRings])

  return (
    <div style={{ display: 'flex', height: '100%', width: '100%' }}>
      <Sidebar />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        {current_context === 'super' ? (
          <>
            <SuperRingHeader />
            <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
              <ChatArea />
            </div>
          </>
        ) : (
          <>
            <HeaderTabBar />
            <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
              <ChatArea />
              <PanelStack />
            </div>
          </>
        )}
      </div>
      <SelfFloat />
      <SelfTrigger />
    </div>
  )
}
```

- [ ] **Step 4: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 5: Commit**

```bash
git add ui/src/stores/ring-store.ts ui/src/components/sidebar/RingList.tsx ui/src/components/sidebar/RingListItem.tsx ui/src/components/layout/AppLayout.tsx
git commit -m "feat(ui): ring store fetches from API, sidebar shows real rings"
```

---

### Task 5: Command Parser

**Files:**
- Create: `ui/src/services/command-parser.ts`
- Create: `ui/src/test/services/command-parser.test.ts`

- [ ] **Step 1: Write command parser tests**

`ui/src/test/services/command-parser.test.ts`:

```typescript
import { describe, it, expect } from 'vitest'
import { parseCommand, type ParsedCommand } from '../../services/command-parser'

describe('parseCommand', () => {
  it('returns null for plain text', () => {
    expect(parseCommand('hello world')).toBeNull()
  })

  it('parses @self', () => {
    const result = parseCommand('@self hello') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'address', target: 'self', rest: 'hello' })
  })

  it('parses @ring', () => {
    const result = parseCommand('@ring 分析一下') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'address', target: 'ring', rest: '分析一下' })
  })

  it('parses @super', () => {
    const result = parseCommand('@super 总结') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'address', target: 'super', rest: '总结' })
  })

  it('parses !graph', () => {
    const result = parseCommand('!graph') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'action', action: 'graph', args: '' })
  })

  it('parses !save', () => {
    const result = parseCommand('!save some content') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'action', action: 'save', args: 'some content' })
  })

  it('parses !auto as toggle', () => {
    const result = parseCommand('!auto') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'action', action: 'auto', args: '' })
  })

  it('parses %skill plan', () => {
    const result = parseCommand('%skill plan') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'meta', key: 'skill', value: 'plan' })
  })

  it('parses %mode auto', () => {
    const result = parseCommand('%mode auto') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'meta', key: 'mode', value: 'auto' })
  })

  it('parses #nodename', () => {
    const result = parseCommand('#竞品分析') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'reference', name: '竞品分析' })
  })

  it('parses multiple commands in one input', () => {
    const result = parseCommand('@ring #竞品分析 帮我看看这个节点') as ParsedCommand[]
    expect(result).toHaveLength(2)
    expect(result[0]).toEqual({ type: 'address', target: 'ring', rest: '' })
    expect(result[1]).toEqual({ type: 'reference', name: '竞品分析' })
  })

  it('parses !new with args for ring creation', () => {
    const result = parseCommand('!new 竞品分析组') as ParsedCommand[]
    expect(result).toHaveLength(1)
    expect(result[0]).toEqual({ type: 'action', action: 'new', args: '竞品分析组' })
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ui && npx vitest run src/test/services/command-parser.test.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Implement command parser**

`ui/src/services/command-parser.ts`:

```typescript
export type ParsedCommand =
  | { type: 'address'; target: string; rest: string }
  | { type: 'reference'; name: string }
  | { type: 'action'; action: string; args: string }
  | { type: 'meta'; key: string; value: string }

export function parseCommand(input: string): ParsedCommand[] | null {
  const trimmed = input.trim()
  if (!trimmed) return null

  const commands: ParsedCommand[] = []
  const tokens = trimmed.split(/\s+/)
  let hasCommand = false
  let i = 0

  while (i < tokens.length) {
    const token = tokens[i]

    if (token.startsWith('@')) {
      hasCommand = true
      const target = token.slice(1).toLowerCase()
      const rest = tokens.slice(i + 1).join(' ')
      commands.push({ type: 'address', target, rest })
      break
    }

    if (token.startsWith('#')) {
      hasCommand = true
      const name = token.slice(1)
      commands.push({ type: 'reference', name })
      i++
      continue
    }

    if (token.startsWith('!')) {
      hasCommand = true
      const action = token.slice(1).toLowerCase()
      const args = tokens.slice(i + 1).join(' ')
      commands.push({ type: 'action', action, args })
      break
    }

    if (token.startsWith('%')) {
      hasCommand = true
      const body = token.slice(1).toLowerCase()
      const nextToken = tokens[i + 1]
      commands.push({ type: 'meta', key: body, value: nextToken ?? '' })
      break
    }

    break
  }

  return hasCommand && commands.length > 0 ? commands : null
}
```

- [ ] **Step 4: Run tests**

Run: `cd ui && npx vitest run src/test/services/command-parser.test.ts`
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/services/command-parser.ts ui/src/test/services/command-parser.test.ts
git commit -m "feat(ui): CLI command parser with @/#/!/% prefix support"
```

---

### Task 6: Chat Input — Command Dispatch + Send

**Files:**
- Modify: `ui/src/stores/chat-store.ts`
- Modify: `ui/src/components/chat/InputArea.tsx`

- [ ] **Step 1: Update chat-store.ts to add send + command dispatch**

```typescript
import { create } from 'zustand'
import type { ChatMessage } from '../types/chat'
import { parseCommand } from '../services/command-parser'
import { usePanelStore } from './panel-store'
import { useSelfStore } from './self-store'
import { useModeStore } from './mode-store'
import { useRingStore } from './ring-store'
import { useAppStore } from './app-store'

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
```

- [ ] **Step 2: Update InputArea.tsx with Enter key and send handler**

```typescript
import { useChatStore } from '../../stores/chat-store'
import { ModeIndicator } from './ModeIndicator'
import { CommandHints } from './CommandHints'

export function InputArea() {
  const { input, setInput, send } = useChatStore()

  return (
    <div>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          padding: '8px 12px',
          borderTop: '1px solid var(--border)',
        }}
      >
        <ModeIndicator />
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault()
              send()
            }
          }}
          placeholder="message / command..."
          style={{
            flex: 1,
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '8px 12px',
            color: 'var(--text-primary)',
            fontSize: 13,
            fontFamily: 'inherit',
            outline: 'none',
          }}
        />
        <button
          onClick={send}
          style={{
            background: 'var(--accent-cyan)',
            color: 'var(--bg-base)',
            border: 'none',
            borderRadius: 4,
            padding: '8px 16px',
            fontSize: 12,
            fontWeight: 700,
            cursor: 'pointer',
            letterSpacing: '0.05em',
          }}
        >
          SEND
        </button>
      </div>
      <CommandHints />
    </div>
  )
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add ui/src/stores/chat-store.ts ui/src/components/chat/InputArea.tsx
git commit -m "feat(ui): chat input dispatches commands on Enter, sends messages"
```

---

### Task 7: Config Panel — Real Members + LLM Config

**Files:**
- Modify: `ui/src/components/panels/ConfigPanel.tsx`

- [ ] **Step 1: Rewrite ConfigPanel.tsx with API data**

```typescript
import { useEffect, useState } from 'react'
import type { Member } from '../../types/ring'
import type { LLMConfig } from '../../types/config'
import { api } from '../../services/api'
import { useRingStore } from '../../stores/ring-store'

export function ConfigPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const [members, setMembers] = useState<Member[]>([])
  const [llmConfig, setLlmConfig] = useState<LLMConfig | null>(null)

  useEffect(() => {
    api.get<{ provider: string; model: string; api_key_set: boolean; base_url: string | null }>('/config/llm')
      .then(setLlmConfig)
      .catch(() => {})
  }, [])

  useEffect(() => {
    if (!active_ring_id) return
    api.get<{ members: Member[] }>(`/rings/${active_ring_id}/members`)
      .then((res) => setMembers(res.members))
      .catch(() => {})
  }, [active_ring_id])

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
    </div>
  )
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/panels/ConfigPanel.tsx
git commit -m "feat(ui): config panel fetches members and LLM config from API"
```

---

### Task 8: Mode Sync — Persist to Server

**Files:**
- Modify: `ui/src/stores/mode-store.ts`
- Modify: `ui/src/components/chat/ModeIndicator.tsx`

- [ ] **Step 1: Update mode-store.ts to sync with API**

```typescript
import { create } from 'zustand'
import type { InteractionMode, SkillPermissionMode } from '../types/config'
import { api } from '../services/api'
import { useRingStore } from './ring-store'

interface ModeState {
  interaction_mode: InteractionMode
  skill_permission_mode: SkillPermissionMode
  syncing: boolean
  setInteractionMode: (mode: InteractionMode) => void
  setSkillMode: (mode: SkillPermissionMode) => void
  toggleAuto: () => void
  syncToServer: () => Promise<void>
  fetchFromServer: (ringId: string) => Promise<void>
  reset: () => void
}

export const useModeStore = create<ModeState>((set, get) => ({
  interaction_mode: 'normal',
  skill_permission_mode: 'plan',
  syncing: false,

  setInteractionMode: (mode) => {
    set({ interaction_mode: mode })
    get().syncToServer()
  },

  setSkillMode: (mode) => {
    set({ skill_permission_mode: mode })
    get().syncToServer()
  },

  toggleAuto: () => {
    set({ interaction_mode: get().interaction_mode === 'auto' ? 'normal' : 'auto' })
    get().syncToServer()
  },

  syncToServer: async () => {
    const ringId = useRingStore.getState().active_ring_id
    if (!ringId) return
    set({ syncing: true })
    try {
      const { interaction_mode, skill_permission_mode } = get()
      await api.put(`/rings/${ringId}/mode`, {
        interaction_mode,
        skill_permission_mode,
      })
    } catch {
      // silent fail — mode is local-first
    }
    set({ syncing: false })
  },

  fetchFromServer: async (ringId) => {
    try {
      const res = await api.get<{ interaction_mode: string; skill_permission_mode: string }>(`/rings/${ringId}/mode`)
      set({
        interaction_mode: res.interaction_mode as InteractionMode,
        skill_permission_mode: res.skill_permission_mode as SkillPermissionMode,
      })
    } catch {
      // keep defaults
    }
  },

  reset: () =>
    set({ interaction_mode: 'normal', skill_permission_mode: 'plan' }),
}))
```

- [ ] **Step 2: Update ModeIndicator.tsx to fetch mode when ring changes**

```typescript
import { useState, useEffect } from 'react'
import { useModeStore } from '../../stores/mode-store'
import { useRingStore } from '../../stores/ring-store'
import { ModeSelector } from './ModeSelector'

export function ModeIndicator() {
  const interaction_mode = useModeStore((s) => s.interaction_mode)
  const syncing = useModeStore((s) => s.syncing)
  const fetchFromServer = useModeStore((s) => s.fetchFromServer)
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const [showSelector, setShowSelector] = useState(false)

  useEffect(() => {
    if (active_ring_id) {
      fetchFromServer(active_ring_id)
    }
  }, [active_ring_id, fetchFromServer])

  return (
    <div style={{ position: 'relative' }}>
      <button
        onClick={() => setShowSelector(!showSelector)}
        style={{
          background: syncing ? 'var(--bg-active)' : 'var(--bg-hover)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '6px 10px',
          color: 'var(--text-secondary)',
          fontSize: 11,
          cursor: 'pointer',
          fontWeight: 700,
          whiteSpace: 'nowrap',
          display: 'flex',
          alignItems: 'center',
          gap: 4,
        }}
      >
        [ring
        {interaction_mode === 'auto' && (
          <span style={{ color: 'var(--accent-amber)' }}>·auto</span>
        )}
        ]
      </button>
      {showSelector && (
        <ModeSelector onClose={() => setShowSelector(false)} />
      )}
    </div>
  )
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add ui/src/stores/mode-store.ts ui/src/components/chat/ModeIndicator.tsx
git commit -m "feat(ui): mode changes sync to server, fetch on ring switch"
```

---

### Task 9: End-to-End Smoke Test

**Files:**
- No new files — manual verification only

- [ ] **Step 1: Build frontend**

Run: `cd ui && npm run build`
Expected: Success.

- [ ] **Step 2: Start backend**

Run: `cd server && cargo run &`
Expected: `ring-server listening on http://localhost:7420`

- [ ] **Step 3: Test full flow via curl**

```bash
curl -s http://localhost:7420/api/setup/status | python3 -m json.tool
# {"is_setup":false,"step":"identity"}

curl -s -X POST http://localhost:7420/api/setup \
  -H 'Content-Type: application/json' \
  -d '{"display_name":"Kai","avatar":"🦊","llm_provider":"openai","llm_api_key":"sk-test","gitlab_url":"https://gitlab.test.com","gitlab_token":"glpat-test"}'
# {"token_id":"user-XXX","display_name":"Kai","avatar":"🦊"}

curl -s -X POST http://localhost:7420/api/rings \
  -H 'Content-Type: application/json' \
  -H 'X-Ring-Token: user-XXX' \
  -d '{"name":"竞品分析组","role_description":"你是一个产品分析专家"}'
# {"id":"XXX","name":"竞品分析组","role":"creator","blueprint_status":"pending"}
```

- [ ] **Step 4: Open browser**

Navigate to `http://localhost:7420`. Expected:
1. Setup wizard appears (fresh db)
2. Complete setup → token saved to localStorage
3. Main interface loads with empty ring list
4. Type `!new 测试组` → ring appears in sidebar
5. Click ring → header shows ring name, mode indicator shows `[ring]`
6. Click Config tab → members list shows creator
7. Toggle auto mode → `[ring·auto]` shown

- [ ] **Step 5: Kill backend**

```bash
kill %1
```

- [ ] **Step 6: Commit (if any changes)**

```bash
git add -A
git commit -m "test: verify end-to-end setup and ring creation flow"
```

---

## Self-Review

### 1. Spec Coverage

| Requirement | Covered | Task |
|-------------|---------|------|
| Setup wizard submits to API | Yes | Task 3 |
| App checks setup status on load | Yes | Task 2 |
| Ring list from API | Yes | Task 4 |
| Ring creation via !new | Yes | Task 6 |
| CLI command parsing (@/#/!/% ) | Yes | Task 5 |
| Command dispatch on Enter | Yes | Task 6 |
| Mode sync to server | Yes | Task 8 |
| Config panel shows real members + LLM | Yes | Task 7 |
| Auth token persisted in localStorage | Yes | Task 1 |
| Loading states | Yes | Task 2 |

### 2. Placeholder Scan

No TBD/TODO/placeholders found. All steps contain complete code.

### 3. Type Consistency

- `SetupData` interface defined in SetupWizard.tsx, used by all step components
- `ParsedCommand` union type defined in command-parser.ts, used in chat-store.ts
- `api.get/post/put` signatures match backend response shapes
- `Ring`, `Member`, `LLMConfig` types from existing type files reused correctly
- `InteractionMode`, `SkillPermissionMode` types from config.ts reused in mode-store.ts

---

**Plan complete and saved to `docs/superpowers/plans/2026-04-18-chat-command-integration.md`. Two execution options:**

1. **Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

2. **Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
