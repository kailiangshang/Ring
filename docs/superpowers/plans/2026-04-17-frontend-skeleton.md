# Frontend Skeleton + UI Framework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the complete Ring frontend UI framework with navigation, panels, Self floating window, and Setup flow — all interactive with mock data.

**Architecture:** React 19 + TypeScript + Zustand for state, Vite for build. IceChat dark theme (CLI aesthetic). Three-column layout: sidebar | chat | stackable panels. All data mocked via Zustand stores; API layer abstracted behind service interfaces for future swap.

**Tech Stack:** React 19, TypeScript, Zustand 5, Vite 8, Vitest, React Router 7, Testing Library

**Specs:** `docs/product/UI-DESIGN.md`, `docs/superpowers/specs/2026-04-17-cli-command-system-design.md`, `docs/product/PRD.md`

---

## File Structure

```
ring-frontend/src/
├── main.tsx                          # Entry point
├── App.tsx                           # Root layout + router
├── index.css                         # Global styles + IceChat theme vars
│
├── types/
│   ├── ring.ts                       # Ring, Member, Role types
│   ├── graph.ts                      # Graph, Node, Edge types
│   ├── chat.ts                       # Message, ChatState types
│   ├── session.ts                    # Session, SessionPhase types
│   └── config.ts                     # LLMConfig, Mode types
│
├── stores/
│   ├── app-store.ts                  # Global app state (setup status, current ring)
│   ├── ring-store.ts                 # Ring list + active ring
│   ├── panel-store.ts                # Panel stack (open/close/order)
│   ├── chat-store.ts                 # Messages + input state
│   ├── session-store.ts              # Active sessions
│   ├── self-store.ts                 # Self floating window state
│   └── mode-store.ts                 # Interaction mode + skill permission mode
│
├── services/
│   ├── api.ts                        # Base fetch wrapper (proxy to /api)
│   ├── mock-data.ts                  # All mock data for development
│   └── sse.ts                        # SSE connection helper
│
├── components/
│   ├── layout/
│   │   ├── AppLayout.tsx             # Three-column shell
│   │   ├── Sidebar.tsx               # Left sidebar
│   │   ├── HeaderTabBar.tsx          # Top header tabs
│   │   └── PanelStack.tsx            # Right panel stack container
│   │
│   ├── sidebar/
│   │   ├── SuperRingEntry.tsx        # Super Ring pinned entry
│   │   ├── RingList.tsx              # Flat ring list
│   │   ├── RingListItem.tsx          # Single ring row
│   │   └── SessionIndicator.tsx      # Session dot under active ring
│   │
│   ├── header/
│   │   ├── TabItem.tsx               # Single tab (Chat/Graph/Archive/Config)
│   │   └── HeaderActions.tsx         # Auto/Export buttons
│   │
│   ├── chat/
│   │   ├── ChatArea.tsx              # Message list + input
│   │   ├── MessageList.tsx           # Scrollable message list
│   │   ├── MessageItem.tsx           # Single message bubble
│   │   ├── InputArea.tsx             # Input box + mode indicator + send
│   │   ├── ModeIndicator.tsx         # [ring] / [ring·auto] clickable
│   │   ├── ModeSelector.tsx          # Dropdown: interaction mode + skill mode
│   │   └── CommandHints.tsx          # Bottom hints bar
│   │
│   ├── panels/
│   │   ├── PanelWrapper.tsx          # Shared panel chrome (header + close)
│   │   ├── GraphPanel.tsx            # Graph tree view (placeholder)
│   │   ├── ArchivePanel.tsx          # PR queue + files (placeholder)
│   │   ├── ConfigPanel.tsx           # Members + Blueprint tabs
│   │   └── SessionPanel.tsx          # Session detail view
│   │
│   ├── self/
│   │   ├── SelfFloat.tsx             # Floating window container
│   │   ├── SelfTrigger.tsx           # 🐱 trigger button (draggable)
│   │   ├── SelfChat.tsx              # Chat tab content
│   │   ├── SelfMemory.tsx            # Memory tab content
│   │   └── SelfSettings.tsx          # Settings tab content
│   │
│   ├── setup/
│   │   ├── SetupWizard.tsx           # Multi-step wizard shell
│   │   ├── StepWelcome.tsx           # Welcome step
│   │   ├── StepIdentity.tsx          # Display name + avatar
│   │   ├── StepLLM.tsx              # LLM provider config
│   │   ├── StepGitLab.tsx           # GitLab config
│   │   └── StepDone.tsx             # Done + command cheat sheet
│   │
│   └── common/
│       ├── Avatar.tsx                # Letter/emoji avatar
│       ├── Badge.tsx                 # Count badge
│       └── ScrollContainer.tsx       # Custom thin scrollbar
│
├── hooks/
│   ├── use-drag.ts                   # Generic drag hook
│   └── use-click-or-drag.ts          # Distinguish click vs drag
│
└── test/
    ├── setup.ts                      # Vitest setup (jsdom, matchers)
    ├── stores/
    │   ├── panel-store.test.ts
    │   └── mode-store.test.ts
    └── components/
        ├── Sidebar.test.tsx
        └── InputArea.test.tsx
```

---

### Task 1: Project Foundation — Types + Theme + Global CSS

**Files:**
- Create: `src/types/ring.ts`
- Create: `src/types/graph.ts`
- Create: `src/types/chat.ts`
- Create: `src/types/session.ts`
- Create: `src/types/config.ts`
- Create: `src/index.css`

- [ ] **Step 1: Create type definitions**

`src/types/ring.ts`:
```typescript
export type Role = 'creator' | 'admin' | 'member' | 'readonly'

export interface Ring {
  id: string
  name: string
  role: Role
  member_count: number
  node_count: number
  last_activity_at: string
  has_active_session: boolean
}

export interface Member {
  token_id: string
  display_name: string
  avatar: string | null
  role: Role
  joined_at: string
  online: boolean
}
```

`src/types/graph.ts`:
```typescript
export type NodeType = 'topic' | 'category' | 'leaf'
export type EdgeRelation = 'depends_on' | 'related_to' | 'derives_from' | 'contradicts'

export interface GraphNode {
  id: string
  label: string
  parent_id: string | null
  markdown_path: string
  node_type: NodeType
  tags: string[]
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface GraphEdge {
  id: string
  source_id: string
  target_id: string
  relation: EdgeRelation
  label: string
  created_at: string
}

export interface Graph {
  id: string
  name: string
  ring_id: string
  nodes: GraphNode[]
  edges: GraphEdge[]
  created_at: string
  updated_at: string
}
```

`src/types/chat.ts`:
```typescript
export type MessageRole = 'user' | 'group_ring' | 'super_ring' | 'session_ring' | 'self' | 'system'

export interface ChatMessage {
  id: string
  role: MessageRole
  sender_name: string
  content: string
  node_refs?: string[]
  tag_refs?: string[]
  created_at: string
}
```

`src/types/session.ts`:
```typescript
export type SessionPhase = 'material_prep' | 'discussion' | 'summary' | 'closed'
export type SessionSkill = 'decision' | 'research' | 'review' | 'retrospective' | 'knowledge_sharing' | 'discussion'

export interface Session {
  id: string
  title: string
  description: string
  skill: SessionSkill
  phase: SessionPhase
  owner: string
  participants: string[]
  archivable: boolean
  archive_enabled: boolean
  created_at: string
}
```

`src/types/config.ts`:
```typescript
export type LLMProvider = 'openai' | 'anthropic' | 'ollama'
export type InteractionMode = 'normal' | 'auto'
export type SkillPermissionMode = 'auto' | 'plan' | 'edit'

export interface LLMConfig {
  provider: LLMProvider
  model: string
  api_key_set: boolean
  base_url: string | null
}

export interface RingMode {
  interaction_mode: InteractionMode
  skill_permission_mode: SkillPermissionMode
}
```

- [ ] **Step 2: Create IceChat theme CSS**

`src/index.css`:
```css
:root {
  --bg-base: #06080c;
  --bg-panel: #0a0e14;
  --bg-sidebar: #080c12;
  --bg-input: #0d1117;
  --bg-hover: #0d1420;
  --bg-active: #0d2a35;
  --border: #1a2030;
  --text-primary: #bfc7d5;
  --text-secondary: #8892a0;
  --text-muted: #6b7d8e;
  --text-dim: #3a4550;
  --accent-cyan: #0891B2;
  --accent-ice: #67E8F9;
  --accent-teal: #06B6D4;
  --accent-green: #22c55e;
  --accent-amber: #f59e0b;
  --placeholder: #2a3540;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #root {
  height: 100%;
  background: var(--bg-base);
  color: var(--text-primary);
  font-family: 'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace;
  font-size: 13px;
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
}

::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}

::selection {
  background: var(--accent-cyan);
  color: var(--bg-base);
}

@font-face {
  font-family: 'Cascadia Code';
  src: url('https://cdn.jsdelivr.net/gh/microsoft/cascadia-code@main/fonts/static/CascadiaCode-Regular.otf') format('opentype');
  font-weight: 400;
  font-display: swap;
}

@font-face {
  font-family: 'Cascadia Code';
  src: url('https://cdn.jsdelivr.net/gh/microsoft/cascadia-code@main/fonts/static/CascadiaCode-Bold.otf') format('opentype');
  font-weight: 700;
  font-display: swap;
}
```

- [ ] **Step 3: Run build to verify**

Run: `cd ring-frontend && npx tsc --noEmit`
Expected: no type errors (files won't be imported yet but should compile)

- [ ] **Step 4: Commit**

```bash
git add src/types/ src/index.css
git commit -m "feat(fe): add type definitions and IceChat theme CSS"
```

---

### Task 2: Zustand Stores — Core State

**Files:**
- Create: `src/stores/app-store.ts`
- Create: `src/stores/panel-store.ts`
- Create: `src/stores/ring-store.ts`
- Create: `src/stores/mode-store.ts`
- Create: `src/stores/self-store.ts`
- Test: `src/test/stores/panel-store.test.ts`
- Test: `src/test/stores/mode-store.test.ts`

- [ ] **Step 1: Write panel store test**

`src/test/stores/panel-store.test.ts`:
```typescript
import { describe, it, expect, beforeEach } from 'vitest'
import { usePanelStore } from '../panel-store'

describe('panelStore', () => {
  beforeEach(() => {
    usePanelStore.getState().closeAll()
  })

  it('opens a panel', () => {
    usePanelStore.getState().open('graph')
    expect(usePanelStore.getState().panels).toEqual([{ type: 'graph', depth: 1 }])
  })

  it('stacks panels', () => {
    usePanelStore.getState().open('graph')
    usePanelStore.getState().open('archive')
    const state = usePanelStore.getState().panels
    expect(state).toHaveLength(2)
    expect(state[0].depth).toBe(1)
    expect(state[1].depth).toBe(2)
  })

  it('closes a single panel by index', () => {
    usePanelStore.getState().open('graph')
    usePanelStore.getState().open('archive')
    usePanelStore.getState().close(0)
    expect(usePanelStore.getState().panels).toHaveLength(1)
    expect(usePanelStore.getState().panels[0].type).toBe('archive')
    expect(usePanelStore.getState().panels[0].depth).toBe(1)
  })

  it('toggles panel (open if closed, close if already open)', () => {
    usePanelStore.getState().toggle('graph')
    expect(usePanelStore.getState().panels).toHaveLength(1)
    usePanelStore.getState().toggle('graph')
    expect(usePanelStore.getState().panels).toHaveLength(0)
  })

  it('closeAll removes all panels', () => {
    usePanelStore.getState().open('graph')
    usePanelStore.getState().open('archive')
    usePanelStore.getState().closeAll()
    expect(usePanelStore.getState().panels).toHaveLength(0)
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-frontend && npx vitest run src/test/stores/panel-store.test.ts`
Expected: FAIL — module not found

- [ ] **Step 3: Implement panel store**

`src/stores/panel-store.ts`:
```typescript
import { create } from 'zustand'

export type PanelType = 'graph' | 'archive' | 'config' | 'session'

export interface Panel {
  type: PanelType
  depth: number
}

interface PanelState {
  panels: Panel[]
  open: (type: PanelType) => void
  close: (index: number) => void
  closeAll: () => void
  toggle: (type: PanelType) => void
}

export const usePanelStore = create<PanelState>((set, get) => ({
  panels: [],
  open: (type) =>
    set((state) => {
      if (state.panels.some((p) => p.type === type)) return state
      return {
        panels: [...state.panels, { type, depth: state.panels.length + 1 }],
      }
    }),
  close: (index) =>
    set((state) => {
      const remaining = state.panels.filter((_, i) => i !== index)
      return {
        panels: remaining.map((p, i) => ({ ...p, depth: i + 1 })),
      }
    }),
  closeAll: () => set({ panels: [] }),
  toggle: (type) => {
    const { panels } = get()
    const existing = panels.findIndex((p) => p.type === type)
    if (existing >= 0) {
      get().close(existing)
    } else {
      get().open(type)
    }
  },
}))
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd ring-frontend && npx vitest run src/test/stores/panel-store.test.ts`
Expected: PASS

- [ ] **Step 5: Write mode store test**

`src/test/stores/mode-store.test.ts`:
```typescript
import { describe, it, expect, beforeEach } from 'vitest'
import { useModeStore } from '../mode-store'

describe('modeStore', () => {
  beforeEach(() => {
    useModeStore.getState().reset()
  })

  it('defaults to normal/plan', () => {
    const s = useModeStore.getState()
    expect(s.interaction_mode).toBe('normal')
    expect(s.skill_permission_mode).toBe('plan')
  })

  it('toggles auto mode', () => {
    useModeStore.getState().toggleAuto()
    expect(useModeStore.getState().interaction_mode).toBe('auto')
    useModeStore.getState().toggleAuto()
    expect(useModeStore.getState().interaction_mode).toBe('normal')
  })

  it('sets skill permission mode', () => {
    useModeStore.getState().setSkillMode('edit')
    expect(useModeStore.getState().skill_permission_mode).toBe('edit')
  })
})
```

- [ ] **Step 6: Implement mode store**

`src/stores/mode-store.ts`:
```typescript
import { create } from 'zustand'
import type { InteractionMode, SkillPermissionMode } from '../types/config'

interface ModeState {
  interaction_mode: InteractionMode
  skill_permission_mode: SkillPermissionMode
  setInteractionMode: (mode: InteractionMode) => void
  setSkillMode: (mode: SkillPermissionMode) => void
  toggleAuto: () => void
  reset: () => void
}

export const useModeStore = create<ModeState>((set, get) => ({
  interaction_mode: 'normal',
  skill_permission_mode: 'plan',
  setInteractionMode: (mode) => set({ interaction_mode: mode }),
  setSkillMode: (mode) => set({ skill_permission_mode: mode }),
  toggleAuto: () =>
    set({ interaction_mode: get().interaction_mode === 'auto' ? 'normal' : 'auto' }),
  reset: () =>
    set({ interaction_mode: 'normal', skill_permission_mode: 'plan' }),
}))
```

- [ ] **Step 7: Run mode store test**

Run: `cd ring-frontend && npx vitest run src/test/stores/mode-store.test.ts`
Expected: PASS

- [ ] **Step 8: Create app store**

`src/stores/app-store.ts`:
```typescript
import { create } from 'zustand'

interface AppState {
  is_setup: boolean
  current_context: 'super' | 'ring' | 'session' | 'self'
  active_ring_id: string | null
  active_session_id: string | null
  setSetup: (done: boolean) => void
  setContext: (ctx: AppState['current_context']) => void
  setActiveRing: (ring_id: string | null) => void
  setActiveSession: (session_id: string | null) => void
}

export const useAppStore = create<AppState>((set) => ({
  is_setup: false,
  current_context: 'super',
  active_ring_id: null,
  active_session_id: null,
  setSetup: (done) => set({ is_setup: done }),
  setContext: (ctx) => set({ current_context: ctx }),
  setActiveRing: (ring_id) => set({ active_ring_id: ring_id, current_context: ring_id ? 'ring' : 'super' }),
  setActiveSession: (session_id) => set({ active_session_id: session_id, current_context: session_id ? 'session' : 'ring' }),
}))
```

- [ ] **Step 9: Create ring store with mock data**

`src/stores/ring-store.ts`:
```typescript
import { create } from 'zustand'
import type { Ring } from '../types/ring'
import { MOCK_RINGS } from '../services/mock-data'

interface RingState {
  rings: Ring[]
  active_ring_id: string | null
  setRings: (rings: Ring[]) => void
  selectRing: (id: string | null) => void
}

export const useRingStore = create<RingState>((set) => ({
  rings: MOCK_RINGS,
  active_ring_id: null,
  setRings: (rings) => set({ rings }),
  selectRing: (id) => set({ active_ring_id: id }),
}))
```

- [ ] **Step 10: Create self store**

`src/stores/self-store.ts`:
```typescript
import { create } from 'zustand'

interface SelfState {
  open: boolean
  position: { x: number; y: number }
  active_tab: 'chat' | 'memory' | 'settings'
  trigger_position: { x: number; y: number }
  setOpen: (open: boolean) => void
  toggle: () => void
  setPosition: (pos: { x: number; y: number }) => void
  setTab: (tab: SelfState['active_tab']) => void
  setTriggerPosition: (pos: { x: number; y: number }) => void
}

const TRIGGER_DEFAULT = { x: window.innerWidth - 70, y: window.innerHeight - 70 }
const FLOAT_DEFAULT = { x: window.innerWidth - 380, y: window.innerHeight - 420 }

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
```

- [ ] **Step 11: Create mock data**

`src/services/mock-data.ts`:
```typescript
import type { Ring } from '../types/ring'
import type { ChatMessage } from '../types/chat'
import type { Member } from '../types/ring'
import type { Session } from '../types/session'

export const MOCK_RINGS: Ring[] = [
  {
    id: '01JTYRING1',
    name: '竞品分析组',
    role: 'creator',
    member_count: 5,
    node_count: 13,
    last_activity_at: '2026-04-17T08:00:00Z',
    has_active_session: true,
  },
  {
    id: '01JTYRING2',
    name: '技术架构组',
    role: 'member',
    member_count: 3,
    node_count: 8,
    last_activity_at: '2026-04-16T14:00:00Z',
    has_active_session: false,
  },
  {
    id: '01JTYRING3',
    name: '项目管理组',
    role: 'admin',
    member_count: 7,
    node_count: 21,
    last_activity_at: '2026-04-15T10:00:00Z',
    has_active_session: false,
  },
]

export const MOCK_MEMBERS: Member[] = [
  { token_id: 'user-001', display_name: 'Kai', avatar: '🦊', role: 'creator', joined_at: '2026-04-15T00:00:00Z', online: true },
  { token_id: 'user-002', display_name: 'Alice', avatar: '🐱', role: 'admin', joined_at: '2026-04-15T01:00:00Z', online: true },
  { token_id: 'user-003', display_name: 'Bob', avatar: null, role: 'member', joined_at: '2026-04-16T00:00:00Z', online: false },
  { token_id: 'user-004', display_name: 'Carol', avatar: '🌟', role: 'member', joined_at: '2026-04-16T02:00:00Z', online: true },
  { token_id: 'user-005', display_name: 'Dave', avatar: null, role: 'readonly', joined_at: '2026-04-17T00:00:00Z', online: false },
]

export const MOCK_MESSAGES: ChatMessage[] = [
  {
    id: 'msg-001',
    role: 'user',
    sender_name: 'Kai',
    content: '帮我看看 #竞品分析 里最近的内容',
    node_refs: ['01JTYN1'],
    tag_refs: ['竞品分析'],
    created_at: '2026-04-17T08:30:00Z',
  },
  {
    id: 'msg-002',
    role: 'group_ring',
    sender_name: 'GROUP RING',
    content: '根据 #竞品分析 节点的内容，最近有以下更新：\n\n1. **竞品 A** 发布了 v3.0 版本\n2. **竞品 B** 调整了定价策略\n3. **竞品 C** 新增了 AI 功能模块\n\n建议重点关注竞品 C 的 AI 功能，可能影响我们的产品路线图。',
    created_at: '2026-04-17T08:30:05Z',
  },
  {
    id: 'msg-003',
    role: 'user',
    sender_name: 'Kai',
    content: '归档这段到 #竞品动态 下面',
    node_refs: ['01JTYN2'],
    tag_refs: ['竞品动态'],
    created_at: '2026-04-17T08:31:00Z',
  },
  {
    id: 'msg-004',
    role: 'system',
    sender_name: 'SYSTEM',
    content: '已归档到「竞品动态」节点。commit: a1b2c3d',
    created_at: '2026-04-17T08:31:03Z',
  },
]

export const MOCK_SESSION: Session = {
  id: '01JTYSESS',
  title: '竞品 A 深度讨论',
  description: '讨论竞品 A 的最新功能更新',
  skill: 'decision',
  phase: 'discussion',
  owner: 'user-001',
  participants: ['user-001', 'user-002', 'user-003'],
  archivable: true,
  archive_enabled: true,
  created_at: '2026-04-17T08:00:00Z',
}
```

- [ ] **Step 12: Run all store tests**

Run: `cd ring-frontend && npx vitest run src/test/stores/`
Expected: all PASS

- [ ] **Step 13: Commit**

```bash
git add src/stores/ src/services/mock-data.ts src/test/
git commit -m "feat(fe): add Zustand stores (app, ring, panel, mode, self) with tests and mock data"
```

---

### Task 3: Common Components

**Files:**
- Create: `src/components/common/Avatar.tsx`
- Create: `src/components/common/Badge.tsx`
- Create: `src/components/common/ScrollContainer.tsx`

- [ ] **Step 1: Create Avatar component**

`src/components/common/Avatar.tsx`:
```tsx
interface AvatarProps {
  name: string
  avatar: string | null
  size?: number
}

export function Avatar({ name, avatar, size = 28 }: AvatarProps) {
  const isEmoji = avatar && /\p{Emoji}/u.test(avatar)
  const letter = name.charAt(0).toUpperCase()

  if (isEmoji) {
    return (
      <div
        style={{
          width: size,
          height: size,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: size * 0.6,
          borderRadius: 4,
          background: 'var(--bg-active)',
          flexShrink: 0,
        }}
      >
        {avatar}
      </div>
    )
  }

  return (
    <div
      style={{
        width: size,
        height: size,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: size * 0.45,
        fontWeight: 700,
        borderRadius: 4,
        background: 'var(--bg-active)',
        color: 'var(--accent-cyan)',
        flexShrink: 0,
      }}
    >
      {letter}
    </div>
  )
}
```

- [ ] **Step 2: Create Badge component**

`src/components/common/Badge.tsx`:
```tsx
interface BadgeProps {
  count: number
}

export function Badge({ count }: BadgeProps) {
  if (count <= 0) return null
  return (
    <span
      style={{
        background: 'var(--accent-cyan)',
        color: 'var(--bg-base)',
        fontSize: 10,
        fontWeight: 700,
        padding: '0 5px',
        borderRadius: 8,
        minWidth: 16,
        height: 16,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
      }}
    >
      {count > 99 ? '99+' : count}
    </span>
  )
}
```

- [ ] **Step 3: Create ScrollContainer component**

`src/components/common/ScrollContainer.tsx`:
```tsx
import type { ReactNode, CSSProperties } from 'react'

interface ScrollContainerProps {
  children: ReactNode
  style?: CSSProperties
  className?: string
}

export function ScrollContainer({ children, style, className }: ScrollContainerProps) {
  return (
    <div
      className={className}
      style={{
        overflowY: 'auto',
        flex: 1,
        ...style,
      }}
    >
      {children}
    </div>
  )
}
```

- [ ] **Step 4: Run build**

Run: `cd ring-frontend && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add src/components/common/
git commit -m "feat(fe): add Avatar, Badge, ScrollContainer components"
```

---

### Task 4: Drag Hooks

**Files:**
- Create: `src/hooks/use-drag.ts`
- Create: `src/hooks/use-click-or-drag.ts`

- [ ] **Step 1: Create useDrag hook**

`src/hooks/use-drag.ts`:
```typescript
import { useCallback, useRef } from 'react'

interface Position {
  x: number
  y: number
}

export function useDrag(
  onMove: (pos: Position) => void,
  bounds?: { width: number; height: number },
) {
  const startRef = useRef<{ mx: number; my: number; px: number; py: number } | null>(null)

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault()
      const target = e.currentTarget as HTMLElement
      const rect = target.getBoundingClientRect()
      startRef.current = {
        mx: e.clientX,
        my: e.clientY,
        px: rect.left,
        py: rect.top,
      }

      const handleMove = (ev: MouseEvent) => {
        if (!startRef.current) return
        const dx = ev.clientX - startRef.current.mx
        const dy = ev.clientY - startRef.current.my
        let x = startRef.current.px + dx
        let y = startRef.current.py + dy
        if (bounds) {
          x = Math.max(0, Math.min(x, window.innerWidth - bounds.width))
          y = Math.max(0, Math.min(y, window.innerHeight - bounds.height))
        }
        onMove({ x, y })
      }

      const handleUp = () => {
        startRef.current = null
        document.removeEventListener('mousemove', handleMove)
        document.removeEventListener('mouseup', handleUp)
      }

      document.addEventListener('mousemove', handleMove)
      document.addEventListener('mouseup', handleUp)
    },
    [onMove, bounds],
  )

  return { onMouseDown }
}
```

- [ ] **Step 2: Create useClickOrDrag hook**

`src/hooks/use-click-or-drag.ts`:
```typescript
import { useCallback, useRef } from 'react'

export function useClickOrDrag(
  onClick: () => void,
  onDragStart?: (e: React.MouseEvent) => void,
  threshold = 4,
) {
  const startRef = useRef<{ x: number; y: number; dragging: boolean } | null>(null)

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      startRef.current = { x: e.clientX, y: e.clientY, dragging: false }

      const handleMove = (ev: MouseEvent) => {
        if (!startRef.current) return
        const dx = ev.clientX - startRef.current.x
        const dy = ev.clientY - startRef.current.y
        if (Math.sqrt(dx * dx + dy * dy) > threshold) {
          if (!startRef.current.dragging) {
            startRef.current.dragging = true
            onDragStart?.(e)
          }
        }
      }

      const handleUp = () => {
        if (startRef.current && !startRef.current.dragging) {
          onClick()
        }
        startRef.current = null
        document.removeEventListener('mousemove', handleMove)
        document.removeEventListener('mouseup', handleUp)
      }

      document.addEventListener('mousemove', handleMove)
      document.addEventListener('mouseup', handleUp)
    },
    [onClick, onDragStart, threshold],
  )

  return { onMouseDown }
}
```

- [ ] **Step 3: Run build**

Run: `cd ring-frontend && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add src/hooks/
git commit -m "feat(fe): add useDrag and useClickOrDrag hooks"
```

---

### Task 5: Sidebar Components

**Files:**
- Create: `src/components/sidebar/SuperRingEntry.tsx`
- Create: `src/components/sidebar/RingListItem.tsx`
- Create: `src/components/sidebar/SessionIndicator.tsx`
- Create: `src/components/sidebar/RingList.tsx`
- Create: `src/components/layout/Sidebar.tsx`
- Test: `src/test/components/Sidebar.test.tsx`

- [ ] **Step 1: Write sidebar test**

`src/test/components/Sidebar.test.tsx`:
```tsx
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { Sidebar } from '../layout/Sidebar'

describe('Sidebar', () => {
  it('renders Super Ring entry', () => {
    render(<Sidebar />)
    expect(screen.getByText('Super Ring')).toBeDefined()
  })

  it('renders ring list from store', () => {
    render(<Sidebar />)
    expect(screen.getByText('竞品分析组')).toBeDefined()
    expect(screen.getByText('技术架构组')).toBeDefined()
    expect(screen.getByText('项目管理组')).toBeDefined()
  })

  it('shows session indicator on ring with active session', () => {
    render(<Sidebar />)
    const indicators = screen.getAllByTitle(/session/i)
    expect(indicators.length).toBeGreaterThanOrEqual(1)
  })
})
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd ring-frontend && npx vitest run src/test/components/Sidebar.test.tsx`
Expected: FAIL

- [ ] **Step 3: Implement SuperRingEntry**

`src/components/sidebar/SuperRingEntry.tsx`:
```tsx
import { useAppStore } from '../../stores/app-store'

export function SuperRingEntry() {
  const { current_context, setContext, setActiveRing } = useAppStore()
  const isActive = current_context === 'super'

  return (
    <div
      onClick={() => {
        setActiveRing(null)
        setContext('super')
      }}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 10,
        padding: '10px 12px',
        cursor: 'pointer',
        background: isActive ? 'var(--bg-active)' : 'transparent',
        borderBottom: '1px solid var(--border)',
      }}
    >
      <div
        style={{
          width: 28,
          height: 28,
          borderRadius: 6,
          background: 'linear-gradient(135deg, #0891B2, #67E8F9)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: 14,
          fontWeight: 700,
          color: '#06080c',
          flexShrink: 0,
        }}
      >
        R
      </div>
      <span
        style={{
          color: isActive ? 'var(--accent-ice)' : 'var(--text-primary)',
          fontWeight: isActive ? 700 : 400,
          fontSize: 12,
          letterSpacing: '0.05em',
        }}
      >
        Super Ring
      </span>
    </div>
  )
}
```

- [ ] **Step 4: Implement RingListItem**

`src/components/sidebar/RingListItem.tsx`:
```tsx
import type { Ring } from '../../types/ring'
import { useRingStore } from '../../stores/ring-store'
import { useAppStore } from '../../stores/app-store'

interface RingListItemProps {
  ring: Ring
}

export function RingListItem({ ring }: RingListItemProps) {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const selectRing = useRingStore((s) => s.selectRing)
  const setActiveRing = useAppStore((s) => s.setActiveRing)
  const isActive = active_ring_id === ring.id

  return (
    <div
      onClick={() => {
        selectRing(ring.id)
        setActiveRing(ring.id)
      }}
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

- [ ] **Step 5: Implement SessionIndicator**

`src/components/sidebar/SessionIndicator.tsx`:
```tsx
export function SessionIndicator() {
  return (
    <div
      style={{
        marginLeft: 28,
        padding: '4px 8px',
        fontSize: 11,
        color: 'var(--text-muted)',
        display: 'flex',
        alignItems: 'center',
        gap: 6,
      }}
    >
      <span
        style={{
          width: 5,
          height: 5,
          borderRadius: '50%',
          background: 'var(--accent-green)',
        }}
      />
      1 active session
    </div>
  )
}
```

- [ ] **Step 6: Implement RingList**

`src/components/sidebar/RingList.tsx`:
```tsx
import { useRingStore } from '../../stores/ring-store'
import { RingListItem } from './RingListItem'
import { SessionIndicator } from './SessionIndicator'

export function RingList() {
  const rings = useRingStore((s) => s.rings)
  const active_ring_id = useRingStore((s) => s.active_ring_id)

  return (
    <div style={{ padding: '8px 0' }}>
      {rings.map((ring) => (
        <div key={ring.id}>
          <RingListItem ring={ring} />
          {ring.id === active_ring_id && ring.has_active_session && (
            <SessionIndicator />
          )}
        </div>
      ))}
    </div>
  )
}
```

- [ ] **Step 7: Implement Sidebar layout**

`src/components/layout/Sidebar.tsx`:
```tsx
import { SuperRingEntry } from '../sidebar/SuperRingEntry'
import { RingList } from '../sidebar/RingList'

export function Sidebar() {
  return (
    <div
      style={{
        width: 220,
        minWidth: 220,
        height: '100%',
        background: 'var(--bg-sidebar)',
        borderRight: '1px solid var(--border)',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      <SuperRingEntry />
      <div style={{ flex: 1, overflow: 'auto' }}>
        <RingList />
      </div>
    </div>
  )
}
```

- [ ] **Step 8: Run sidebar test**

Run: `cd ring-frontend && npx vitest run src/test/components/Sidebar.test.tsx`
Expected: PASS

- [ ] **Step 9: Commit**

```bash
git add src/components/sidebar/ src/components/layout/Sidebar.tsx src/test/components/Sidebar.test.tsx
git commit -m "feat(fe): add Sidebar with Super Ring entry, ring list, session indicator"
```

---

### Task 6: Header Tab Bar + Panel Stack

**Files:**
- Create: `src/components/header/TabItem.tsx`
- Create: `src/components/header/HeaderActions.tsx`
- Create: `src/components/layout/HeaderTabBar.tsx`
- Create: `src/components/panels/PanelWrapper.tsx`
- Create: `src/components/panels/GraphPanel.tsx`
- Create: `src/components/panels/ArchivePanel.tsx`
- Create: `src/components/panels/ConfigPanel.tsx`
- Create: `src/components/panels/SessionPanel.tsx`
- Create: `src/components/layout/PanelStack.tsx`

- [ ] **Step 1: Create TabItem**

`src/components/header/TabItem.tsx`:
```tsx
import type { ReactNode } from 'react'

interface TabItemProps {
  label: string
  count?: number
  active: boolean
  onClick: () => void
  icon?: ReactNode
}

export function TabItem({ label, count, active, onClick, icon }: TabItemProps) {
  return (
    <button
      onClick={onClick}
      style={{
        background: 'none',
        border: 'none',
        color: active ? 'var(--accent-ice)' : 'var(--text-muted)',
        fontSize: 12,
        fontWeight: active ? 700 : 400,
        cursor: 'pointer',
        padding: '8px 12px',
        display: 'flex',
        alignItems: 'center',
        gap: 4,
        borderBottom: active ? '2px solid var(--accent-cyan)' : '2px solid transparent',
        letterSpacing: '0.03em',
      }}
    >
      {icon}
      {label}
      {count !== undefined && (
        <span
          style={{
            fontSize: 10,
            color: 'var(--text-dim)',
            marginLeft: 2,
          }}
        >
          {count}
        </span>
      )}
    </button>
  )
}
```

- [ ] **Step 2: Create HeaderActions**

`src/components/header/HeaderActions.tsx`:
```tsx
import { useModeStore } from '../../stores/mode-store'

export function HeaderActions() {
  const { interaction_mode, toggleAuto } = useModeStore()

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginLeft: 'auto' }}>
      <button
        onClick={toggleAuto}
        style={{
          background: interaction_mode === 'auto' ? 'var(--accent-amber)' : 'var(--bg-hover)',
          color: interaction_mode === 'auto' ? 'var(--bg-base)' : 'var(--text-secondary)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: '4px 10px',
          fontSize: 11,
          fontWeight: 700,
          cursor: 'pointer',
          letterSpacing: '0.05em',
        }}
      >
        AUTO
      </button>
    </div>
  )
}
```

- [ ] **Step 3: Create HeaderTabBar**

`src/components/layout/HeaderTabBar.tsx`:
```tsx
import { usePanelStore, type PanelType } from '../../stores/panel-store'
import { useRingStore } from '../../stores/ring-store'
import { TabItem } from '../header/TabItem'
import { HeaderActions } from '../header/HeaderActions'

const TABS: { type: PanelType; label: string; icon: string }[] = [
  { type: 'graph', label: 'Graph', icon: '' },
  { type: 'archive', label: 'Archive', icon: '' },
  { type: 'config', label: 'Config', icon: '' },
]

export function HeaderTabBar() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const rings = useRingStore((s) => s.rings)
  const panels = usePanelStore((s) => s.panels)
  const toggle = usePanelStore((s) => s.toggle)
  const closeAll = usePanelStore((s) => s.closeAll)

  const activeRing = rings.find((r) => r.id === active_ring_id)
  if (!activeRing) return null

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
          marginRight: 16,
          whiteSpace: 'nowrap',
        }}
      >
        {activeRing.name}
      </span>

      <TabItem
        label="💬 Chat"
        active={panels.length === 0}
        onClick={() => closeAll()}
      />

      {TABS.map((tab) => (
        <TabItem
          key={tab.type}
          label={tab.label}
          count={tab.type === 'graph' ? activeRing.node_count : undefined}
          active={panels.some((p) => p.type === tab.type)}
          onClick={() => toggle(tab.type)}
        />
      ))}

      <HeaderActions />
    </div>
  )
}
```

- [ ] **Step 4: Create PanelWrapper**

`src/components/panels/PanelWrapper.tsx`:
```tsx
import type { ReactNode } from 'react'

interface PanelWrapperProps {
  title: string
  depth: number
  onClose: () => void
  children: ReactNode
}

export function PanelWrapper({ title, depth, onClose, children }: PanelWrapperProps) {
  const bgColors = ['var(--bg-panel)', '#0b1018', '#0c1220']
  const bg = bgColors[Math.min(depth - 1, 2)]

  return (
    <div
      style={{
        width: 320,
        minWidth: 320,
        height: '100%',
        background: bg,
        borderLeft: '1px solid var(--border)',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '10px 12px',
          borderBottom: '1px solid var(--border)',
        }}
      >
        <span style={{ fontSize: 12, fontWeight: 700, letterSpacing: '0.05em' }}>
          {title}
        </span>
        <button
          onClick={onClose}
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--text-muted)',
            cursor: 'pointer',
            fontSize: 14,
            padding: '0 4px',
          }}
        >
          ×
        </button>
      </div>
      <div style={{ flex: 1, overflow: 'auto', padding: 12 }}>{children}</div>
    </div>
  )
}
```

- [ ] **Step 5: Create placeholder panels**

`src/components/panels/GraphPanel.tsx`:
```tsx
export function GraphPanel() {
  return (
    <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>
      <p style={{ marginBottom: 8, color: 'var(--text-secondary)' }}>图谱节点树</p>
      <p>（D3.js 图谱可视化将在 Plan 4 实现）</p>
    </div>
  )
}
```

`src/components/panels/ArchivePanel.tsx`:
```tsx
export function ArchivePanel() {
  return (
    <div style={{ color: 'var(--text-muted)', fontSize: 12 }}>
      <p style={{ marginBottom: 8, color: 'var(--text-secondary)' }}>归档 + PR 队列</p>
      <p>（归档视图将在 Plan 2 实现）</p>
    </div>
  )
}
```

`src/components/panels/ConfigPanel.tsx`:
```tsx
import { MOCK_MEMBERS } from '../../services/mock-data'

export function ConfigPanel() {
  return (
    <div style={{ fontSize: 12 }}>
      <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
        Members
      </p>
      {MOCK_MEMBERS.map((m) => (
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
    </div>
  )
}
```

`src/components/panels/SessionPanel.tsx`:
```tsx
import { MOCK_SESSION } from '../../services/mock-data'

export function SessionPanel() {
  return (
    <div style={{ fontSize: 12 }}>
      <p style={{ color: 'var(--accent-ice)', fontWeight: 700, marginBottom: 8 }}>
        {MOCK_SESSION.title}
      </p>
      <p style={{ color: 'var(--text-muted)', marginBottom: 4 }}>
        Skill: {MOCK_SESSION.skill} · Phase: {MOCK_SESSION.phase}
      </p>
      <p style={{ color: 'var(--text-secondary)' }}>{MOCK_SESSION.description}</p>
    </div>
  )
}
```

- [ ] **Step 6: Create PanelStack**

`src/components/layout/PanelStack.tsx`:
```tsx
import { usePanelStore } from '../../stores/panel-store'
import { PanelWrapper } from '../panels/PanelWrapper'
import { GraphPanel } from '../panels/GraphPanel'
import { ArchivePanel } from '../panels/ArchivePanel'
import { ConfigPanel } from '../panels/ConfigPanel'
import { SessionPanel } from '../panels/SessionPanel'

const PANEL_CONTENT: Record<string, () => JSX.Element> = {
  graph: GraphPanel,
  archive: ArchivePanel,
  config: ConfigPanel,
  session: SessionPanel,
}

const PANEL_TITLES: Record<string, string> = {
  graph: 'Graph',
  archive: 'Archive',
  config: 'Config',
  session: 'Session',
}

export function PanelStack() {
  const panels = usePanelStore((s) => s.panels)
  const close = usePanelStore((s) => s.close)

  if (panels.length === 0) return null

  return (
    <div style={{ display: 'flex', height: '100%' }}>
      {panels.map((panel, index) => {
        const Content = PANEL_CONTENT[panel.type]
        return (
          <PanelWrapper
            key={panel.type}
            title={PANEL_TITLES[panel.type]}
            depth={panel.depth}
            onClose={() => close(index)}
          >
            {Content ? <Content /> : null}
          </PanelWrapper>
        )
      })}
    </div>
  )
}
```

- [ ] **Step 7: Run build**

Run: `cd ring-frontend && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 8: Commit**

```bash
git add src/components/header/ src/components/panels/ src/components/layout/HeaderTabBar.tsx src/components/layout/PanelStack.tsx
git commit -m "feat(fe): add HeaderTabBar, PanelStack with stackable panels and toggle"
```

---

### Task 7: Chat Area + Input + Mode Selector

**Files:**
- Create: `src/components/chat/MessageItem.tsx`
- Create: `src/components/chat/MessageList.tsx`
- Create: `src/components/chat/ModeIndicator.tsx`
- Create: `src/components/chat/ModeSelector.tsx`
- Create: `src/components/chat/CommandHints.tsx`
- Create: `src/components/chat/InputArea.tsx`
- Create: `src/components/chat/ChatArea.tsx`
- Create: `src/stores/chat-store.ts`
- Test: `src/test/components/InputArea.test.tsx`

- [ ] **Step 1: Create chat store**

`src/stores/chat-store.ts`:
```typescript
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
```

- [ ] **Step 2: Write InputArea test**

`src/test/components/InputArea.test.tsx`:
```tsx
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { InputArea } from '../chat/InputArea'

describe('InputArea', () => {
  it('renders mode indicator', () => {
    render(<InputArea />)
    expect(screen.getByText(/\[ring/)).toBeDefined()
  })

  it('renders command hints', () => {
    render(<InputArea />)
    expect(screen.getByText(/!graph/)).toBeDefined()
    expect(screen.getByText(/@self/)).toBeDefined()
  })
})
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd ring-frontend && npx vitest run src/test/components/InputArea.test.tsx`
Expected: FAIL

- [ ] **Step 4: Create MessageItem**

`src/components/chat/MessageItem.tsx`:
```tsx
import type { ChatMessage } from '../../types/chat'

const ROLE_COLORS: Record<string, string> = {
  user: 'var(--accent-ice)',
  group_ring: 'var(--accent-cyan)',
  super_ring: 'var(--accent-cyan)',
  session_ring: 'var(--accent-teal)',
  self: 'var(--accent-amber)',
  system: 'var(--accent-green)',
}

interface MessageItemProps {
  message: ChatMessage
}

export function MessageItem({ message }: MessageItemProps) {
  const labelColor = ROLE_COLORS[message.role] ?? 'var(--text-muted)'
  const label = message.role === 'user' ? 'YOU' : message.sender_name.toUpperCase()

  return (
    <div style={{ padding: '8px 16px', borderBottom: '1px solid var(--border)' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
        <span
          style={{
            fontSize: 10,
            fontWeight: 700,
            color: labelColor,
            letterSpacing: '0.1em',
          }}
        >
          {label}
        </span>
        <span style={{ fontSize: 10, color: 'var(--text-dim)' }}>
          {new Date(message.created_at).toLocaleTimeString()}
        </span>
      </div>
      <div
        style={{
          color: 'var(--text-primary)',
          whiteSpace: 'pre-wrap',
          lineHeight: 1.6,
        }}
      >
        {message.content}
      </div>
    </div>
  )
}
```

- [ ] **Step 5: Create MessageList**

`src/components/chat/MessageList.tsx`:
```tsx
import { useChatStore } from '../../stores/chat-store'
import { MessageItem } from './MessageItem'
import { ScrollContainer } from '../common/ScrollContainer'

export function MessageList() {
  const messages = useChatStore((s) => s.messages)

  return (
    <ScrollContainer>
      {messages.map((msg) => (
        <MessageItem key={msg.id} message={msg} />
      ))}
    </ScrollContainer>
  )
}
```

- [ ] **Step 6: Create ModeIndicator**

`src/components/chat/ModeIndicator.tsx`:
```tsx
import { useState } from 'react'
import { useModeStore } from '../../stores/mode-store'
import { ModeSelector } from './ModeSelector'

export function ModeIndicator() {
  const interaction_mode = useModeStore((s) => s.interaction_mode)
  const [showSelector, setShowSelector] = useState(false)

  return (
    <div style={{ position: 'relative' }}>
      <button
        onClick={() => setShowSelector(!showSelector)}
        style={{
          background: 'var(--bg-hover)',
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

- [ ] **Step 7: Create ModeSelector**

`src/components/chat/ModeSelector.tsx`:
```tsx
import { useModeStore } from '../../stores/mode-store'

interface ModeSelectorProps {
  onClose: () => void
}

export function ModeSelector({ onClose }: ModeSelectorProps) {
  const { interaction_mode, setInteractionMode, skill_permission_mode, setSkillMode } =
    useModeStore()

  return (
    <div
      style={{
        position: 'absolute',
        bottom: '100%',
        left: 0,
        marginBottom: 4,
        background: 'var(--bg-panel)',
        border: '1px solid var(--border)',
        borderRadius: 4,
        padding: 8,
        minWidth: 200,
        zIndex: 100,
      }}
    >
      <div style={{ fontSize: 10, color: 'var(--text-dim)', marginBottom: 4, letterSpacing: '0.05em' }}>
        交互模式
      </div>
      {(['normal', 'auto'] as const).map((mode) => (
        <label
          key={mode}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 6,
            padding: '4px 4px',
            cursor: 'pointer',
            color: interaction_mode === mode ? 'var(--accent-ice)' : 'var(--text-primary)',
          }}
        >
          <input
            type="radio"
            name="interaction_mode"
            checked={interaction_mode === mode}
            onChange={() => {
              setInteractionMode(mode)
              onClose()
            }}
            style={{ accentColor: 'var(--accent-cyan)' }}
          />
          <span style={{ fontSize: 12 }}>{mode === 'normal' ? '正常对话' : 'Auto'}</span>
        </label>
      ))}

      <div
        style={{
          fontSize: 10,
          color: 'var(--text-dim)',
          marginTop: 8,
          marginBottom: 4,
          letterSpacing: '0.05em',
        }}
      >
        工具确认级别
      </div>
      <div style={{ display: 'flex', gap: 4 }}>
        {(['auto', 'plan', 'edit'] as const).map((mode) => (
          <button
            key={mode}
            onClick={() => {
              setSkillMode(mode)
              onClose()
            }}
            style={{
              background: skill_permission_mode === mode ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: skill_permission_mode === mode ? 'var(--bg-base)' : 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 3,
              padding: '2px 8px',
              fontSize: 11,
              cursor: 'pointer',
            }}
          >
            {mode}
          </button>
        ))}
      </div>
    </div>
  )
}
```

- [ ] **Step 8: Create CommandHints**

`src/components/chat/CommandHints.tsx`:
```tsx
import { usePanelStore } from '../../stores/panel-store'
import { useSelfStore } from '../../stores/self-store'

const HINTS = [
  { label: '!graph', action: 'graph' as const },
  { label: '!archive', action: 'archive' as const },
  { label: '!config', action: 'config' as const },
  { label: '!session', action: 'session' as const },
  { label: '@self', action: null },
]

export function CommandHints() {
  const toggle = usePanelStore((s) => s.toggle)
  const toggleSelf = useSelfStore((s) => s.toggle)

  return (
    <div
      style={{
        display: 'flex',
        gap: 12,
        padding: '4px 16px 8px',
        color: 'var(--text-dim)',
        fontSize: 11,
      }}
    >
      {HINTS.map((hint) => (
        <button
          key={hint.label}
          onClick={() => {
            if (hint.action) {
              toggle(hint.action)
            } else {
              toggleSelf()
            }
          }}
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--text-dim)',
            cursor: 'pointer',
            fontSize: 11,
            padding: 0,
          }}
        >
          {hint.label}
        </button>
      ))}
    </div>
  )
}
```

- [ ] **Step 9: Create InputArea**

`src/components/chat/InputArea.tsx`:
```tsx
import { useChatStore } from '../../stores/chat-store'
import { ModeIndicator } from './ModeIndicator'
import { CommandHints } from './CommandHints'

export function InputArea() {
  const { input, setInput } = useChatStore()

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

- [ ] **Step 10: Create ChatArea**

`src/components/chat/ChatArea.tsx`:
```tsx
import { MessageList } from './MessageList'
import { InputArea } from './InputArea'

export function ChatArea() {
  return (
    <div
      style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        minWidth: 0,
      }}
    >
      <MessageList />
      <InputArea />
    </div>
  )
}
```

- [ ] **Step 11: Run InputArea test**

Run: `cd ring-frontend && npx vitest run src/test/components/InputArea.test.tsx`
Expected: PASS

- [ ] **Step 12: Commit**

```bash
git add src/stores/chat-store.ts src/components/chat/ src/test/components/InputArea.test.tsx
git commit -m "feat(fe): add ChatArea with messages, input, mode selector, command hints"
```

---

### Task 8: Self Floating Window

**Files:**
- Create: `src/components/self/SelfTrigger.tsx`
- Create: `src/components/self/SelfChat.tsx`
- Create: `src/components/self/SelfMemory.tsx`
- Create: `src/components/self/SelfSettings.tsx`
- Create: `src/components/self/SelfFloat.tsx`

- [ ] **Step 1: Create SelfTrigger**

`src/components/self/SelfTrigger.tsx`:
```tsx
import { useSelfStore } from '../../stores/self-store'
import { useClickOrDrag } from '../../hooks/use-click-or-drag'

export function SelfTrigger() {
  const { open, toggle, trigger_position, setTriggerPosition } = useSelfStore()

  const { onMouseDown } = useClickOrDrag(
    () => toggle(),
    undefined,
    4,
  )

  const handleMouseDown = (e: React.MouseEvent) => {
    const startX = e.clientX
    const startY = e.clientY
    let moved = false

    const handleMove = (ev: MouseEvent) => {
      if (Math.abs(ev.clientX - startX) > 4 || Math.abs(ev.clientY - startY) > 4) {
        moved = true
        setTriggerPosition({
          x: Math.max(0, Math.min(ev.clientX - 14, window.innerWidth - 28)),
          y: Math.max(0, Math.min(ev.clientY - 14, window.innerHeight - 28)),
        })
      }
    }

    const handleUp = () => {
      if (!moved) toggle()
      document.removeEventListener('mousemove', handleMove)
      document.removeEventListener('mouseup', handleUp)
    }

    e.preventDefault()
    document.addEventListener('mousemove', handleMove)
    document.addEventListener('mouseup', handleUp)
  }

  return (
    <div
      onMouseDown={handleMouseDown}
      style={{
        position: 'fixed',
        left: trigger_position.x,
        top: trigger_position.y,
        width: 28,
        height: 28,
        borderRadius: '50%',
        background: open ? 'var(--accent-amber)' : 'var(--bg-panel)',
        border: '2px solid var(--accent-amber)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: 14,
        cursor: 'pointer',
        zIndex: 1000,
        userSelect: 'none',
        touchAction: 'none',
      }}
    >
      🐱
    </div>
  )
}
```

- [ ] **Step 2: Create SelfChat**

`src/components/self/SelfChat.tsx`:
```tsx
export function SelfChat() {
  return (
    <div style={{ padding: 8, flex: 1, display: 'flex', flexDirection: 'column' }}>
      <div style={{ flex: 1, color: 'var(--text-muted)', fontSize: 12, textAlign: 'center', paddingTop: 40 }}>
        和 Self 聊聊...
      </div>
      <div style={{ display: 'flex', gap: 8 }}>
        <input
          style={{
            flex: 1,
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: '6px 10px',
            color: 'var(--text-primary)',
            fontSize: 12,
            fontFamily: 'inherit',
            outline: 'none',
          }}
          placeholder="和 Self 聊聊..."
        />
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Create SelfMemory**

`src/components/self/SelfMemory.tsx`:
```tsx
export function SelfMemory() {
  return (
    <div style={{ padding: 8, color: 'var(--text-muted)', fontSize: 12 }}>
      <p style={{ marginBottom: 8 }}>Memory 100% 私有，存储在 ~/.ring/self/</p>
      <p>（Memory 内容将在 Plan 2 实现）</p>
    </div>
  )
}
```

- [ ] **Step 4: Create SelfSettings**

`src/components/self/SelfSettings.tsx`:
```tsx
export function SelfSettings() {
  return (
    <div style={{ padding: 8, fontSize: 12 }}>
      <div style={{ marginBottom: 12 }}>
        <label style={{ color: 'var(--text-dim)', fontSize: 10, letterSpacing: '0.05em' }}>
          身份定义
        </label>
        <textarea
          style={{
            width: '100%',
            background: 'var(--bg-input)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            padding: 8,
            color: 'var(--text-primary)',
            fontSize: 12,
            fontFamily: 'inherit',
            resize: 'vertical',
            minHeight: 60,
            outline: 'none',
          }}
          defaultValue="我是你的个人 AI 助手"
        />
      </div>
      <div style={{ marginBottom: 12 }}>
        <label style={{ color: 'var(--text-dim)', fontSize: 10, letterSpacing: '0.05em' }}>
          自主级别
        </label>
        <div style={{ display: 'flex', gap: 4, marginTop: 4 }}>
          {['suggest', 'assist', 'auto'].map((level) => (
            <button
              key={level}
              style={{
                background: 'var(--bg-hover)',
                color: 'var(--text-secondary)',
                border: '1px solid var(--border)',
                borderRadius: 3,
                padding: '2px 8px',
                fontSize: 11,
                cursor: 'pointer',
              }}
            >
              {level}
            </button>
          ))}
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 5: Create SelfFloat**

`src/components/self/SelfFloat.tsx`:
```tsx
import { useSelfStore } from '../../stores/self-store'
import { useDrag } from '../../hooks/use-drag'
import { SelfChat } from './SelfChat'
import { SelfMemory } from './SelfMemory'
import { SelfSettings } from './SelfSettings'

const TABS = [
  { key: 'chat' as const, label: 'Chat' },
  { key: 'memory' as const, label: 'Memory' },
  { key: 'settings' as const, label: 'Settings' },
]

const TAB_CONTENT = {
  chat: SelfChat,
  memory: SelfMemory,
  settings: SelfSettings,
}

export function SelfFloat() {
  const { open, position, setPosition, active_tab, setTab, setOpen } = useSelfStore()
  const { onMouseDown } = useDrag(setPosition, { width: 340, height: 380 })

  if (!open) return null

  const Content = TAB_CONTENT[active_tab]

  return (
    <div
      style={{
        position: 'fixed',
        left: position.x,
        top: position.y,
        width: 340,
        height: 380,
        background: 'var(--bg-panel)',
        border: '1px solid var(--accent-amber)',
        borderRadius: 8,
        display: 'flex',
        flexDirection: 'column',
        zIndex: 999,
        boxShadow: '0 8px 32px rgba(0,0,0,0.5)',
      }}
    >
      <div
        onMouseDown={onMouseDown}
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '8px 12px',
          borderBottom: '1px solid var(--border)',
          cursor: 'move',
          userSelect: 'none',
        }}
      >
        <span style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-amber)' }}>
          🐱 Self
        </span>
        <button
          onClick={() => setOpen(false)}
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--text-muted)',
            cursor: 'pointer',
            fontSize: 14,
          }}
        >
          ×
        </button>
      </div>

      <div style={{ display: 'flex', borderBottom: '1px solid var(--border)' }}>
        {TABS.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setTab(tab.key)}
            style={{
              flex: 1,
              background: 'none',
              border: 'none',
              borderBottom: active_tab === tab.key ? '2px solid var(--accent-amber)' : '2px solid transparent',
              color: active_tab === tab.key ? 'var(--accent-amber)' : 'var(--text-muted)',
              fontSize: 11,
              padding: '6px 0',
              cursor: 'pointer',
              fontWeight: active_tab === tab.key ? 700 : 400,
            }}
          >
            {tab.label}
          </button>
        ))}
      </div>

      <Content />
    </div>
  )
}
```

- [ ] **Step 6: Run build**

Run: `cd ring-frontend && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 7: Commit**

```bash
git add src/components/self/
git commit -m "feat(fe): add Self floating window with drag, Chat/Memory/Settings tabs"
```

---

### Task 9: Setup Wizard

**Files:**
- Create: `src/components/setup/StepWelcome.tsx`
- Create: `src/components/setup/StepIdentity.tsx`
- Create: `src/components/setup/StepLLM.tsx`
- Create: `src/components/setup/StepGitLab.tsx`
- Create: `src/components/setup/StepDone.tsx`
- Create: `src/components/setup/SetupWizard.tsx`

- [ ] **Step 1: Create StepWelcome**

`src/components/setup/StepWelcome.tsx`:
```tsx
interface StepProps {
  onNext: () => void
}

export function StepWelcome({ onNext }: StepProps) {
  return (
    <div style={{ textAlign: 'center', padding: '40px 20px' }}>
      <div style={{ fontSize: 48, marginBottom: 16 }}>💎</div>
      <h1 style={{ fontSize: 24, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 8 }}>
        Welcome to Ring
      </h1>
      <p style={{ color: 'var(--text-secondary)', marginBottom: 32, maxWidth: 400, margin: '0 auto 32px' }}>
        群组知识协作空间
      </p>
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
        }}
      >
        开始设置
      </button>
    </div>
  )
}
```

- [ ] **Step 2: Create StepIdentity**

`src/components/setup/StepIdentity.tsx`:
```tsx
import { useState } from 'react'

interface StepProps {
  onNext: () => void
  onBack: () => void
}

const EMOJIS = ['🦊', '🐱', '🌟', '🚀', '🎯', '💡', '🔥', '🌈', '⚡', '🍀', '🦋', '🎪']

export function StepIdentity({ onNext, onBack }: StepProps) {
  const [name, setName] = useState('')
  const [avatar, setAvatar] = useState<string | null>(null)

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 1: 你的身份
      </h2>

      <label style={{ fontSize: 11, color: 'var(--text-dim)', letterSpacing: '0.05em' }}>
        显示名称
      </label>
      <input
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder="输入你的名字"
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
        选择头像
      </label>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8, marginTop: 4 }}>
        {EMOJIS.map((emoji) => (
          <button
            key={emoji}
            onClick={() => setAvatar(emoji)}
            style={{
              width: 36,
              height: 36,
              background: avatar === emoji ? 'var(--accent-amber)' : 'var(--bg-hover)',
              border: avatar === emoji ? '2px solid var(--accent-amber)' : '1px solid var(--border)',
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
          ← Back
        </button>
        <button
          onClick={onNext}
          disabled={!name.trim()}
          style={{
            ...navButtonStyle,
            opacity: name.trim() ? 1 : 0.4,
            marginLeft: 'auto',
          }}
        >
          Next →
        </button>
      </div>
    </div>
  )
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
```

- [ ] **Step 3: Create StepLLM**

`src/components/setup/StepLLM.tsx`:
```tsx
import { useState } from 'react'
import type { LLMProvider } from '../../types/config'

interface StepProps {
  onNext: () => void
  onBack: () => void
}

export function StepLLM({ onNext, onBack }: StepProps) {
  const [provider, setProvider] = useState<LLMProvider>('openai')
  const [apiKey, setApiKey] = useState('')
  const [baseUrl, setBaseUrl] = useState('')

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 2: LLM 配置
      </h2>

      <div style={{ display: 'flex', gap: 4, marginBottom: 16 }}>
        {(['openai', 'anthropic', 'ollama'] as const).map((p) => (
          <button
            key={p}
            onClick={() => setProvider(p)}
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
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder={`sk-${provider === 'openai' ? 'xxx' : 'ant-xxx'}`}
            style={inputStyle}
          />
        </>
      )}

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>
        Base URL {provider === 'ollama' ? '(如 http://localhost:11434)' : '(可选)'}
      </label>
      <input
        value={baseUrl}
        onChange={(e) => setBaseUrl(e.target.value)}
        placeholder={provider === 'ollama' ? 'http://localhost:11434' : ''}
        style={inputStyle}
      />

      <div style={{ display: 'flex', gap: 8, marginTop: 24 }}>
        <button onClick={onBack} style={navButtonStyle}>← Back</button>
        <button
          onClick={onNext}
          disabled={provider !== 'ollama' && !apiKey.trim()}
          style={{
            ...navButtonStyle,
            opacity: provider !== 'ollama' && !apiKey.trim() ? 0.4 : 1,
            marginLeft: 'auto',
          }}
        >
          Next →
        </button>
      </div>
    </div>
  )
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
```

- [ ] **Step 4: Create StepGitLab**

`src/components/setup/StepGitLab.tsx`:
```tsx
import { useState } from 'react'

interface StepProps {
  onNext: () => void
  onBack: () => void
}

export function StepGitLab({ onNext, onBack }: StepProps) {
  const [url, setUrl] = useState('')
  const [token, setToken] = useState('')

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 3: GitLab 配置
      </h2>

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>GitLab URL</label>
      <input
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        placeholder="https://gitlab.company.com"
        style={inputStyle}
      />

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>Personal Access Token</label>
      <input
        type="password"
        value={token}
        onChange={(e) => setToken(e.target.value)}
        placeholder="glpat-xxx"
        style={inputStyle}
      />

      <div style={{ display: 'flex', gap: 8, marginTop: 24 }}>
        <button onClick={onBack} style={navButtonStyle}>← Back</button>
        <button
          onClick={onNext}
          disabled={!url.trim() || !token.trim()}
          style={{
            ...navButtonStyle,
            opacity: !url.trim() || !token.trim() ? 0.4 : 1,
            marginLeft: 'auto',
          }}
        >
          完成 →
        </button>
      </div>
    </div>
  )
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
```

- [ ] **Step 5: Create StepDone**

`src/components/setup/StepDone.tsx`:
```tsx
export function StepDone() {
  return (
    <div style={{ textAlign: 'center', padding: '40px 20px' }}>
      <div style={{ fontSize: 48, marginBottom: 16 }}>✓</div>
      <h1 style={{ fontSize: 20, fontWeight: 700, color: 'var(--accent-green)', marginBottom: 16 }}>
        设置完成
      </h1>

      <div
        style={{
          textAlign: 'left',
          background: 'var(--bg-input)',
          border: '1px solid var(--border)',
          borderRadius: 4,
          padding: 16,
          maxWidth: 360,
          margin: '0 auto',
          color: 'var(--text-secondary)',
          fontSize: 12,
          lineHeight: 2,
        }}
      >
        <div style={{ color: 'var(--text-dim)', marginBottom: 4, letterSpacing: '0.05em' }}>
          常用命令
        </div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>@self</span> — 打开 Self 浮窗</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>#节点名</span> — 引用图谱节点</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>!graph</span> — 打开图谱面板</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>!save</span> — 触发归档</div>
        <div><span style={{ color: 'var(--accent-cyan)' }}>!auto</span> — 切换 Auto 模式</div>
      </div>
    </div>
  )
}
```

- [ ] **Step 6: Create SetupWizard**

`src/components/setup/SetupWizard.tsx`:
```tsx
import { useState } from 'react'
import { useAppStore } from '../../stores/app-store'
import { StepWelcome } from './StepWelcome'
import { StepIdentity } from './StepIdentity'
import { StepLLM } from './StepLLM'
import { StepGitLab } from './StepGitLab'
import { StepDone } from './StepDone'

export function SetupWizard() {
  const [step, setStep] = useState(0)
  const setSetup = useAppStore((s) => s.setSetup)

  const goNext = () => setStep((s) => Math.min(s + 1, 4))
  const goBack = () => setStep((s) => Math.max(s - 1, 0))

  const handleFinish = () => {
    setSetup(true)
  }

  const steps = [
    <StepWelcome onNext={goNext} />,
    <StepIdentity onNext={goNext} onBack={goBack} />,
    <StepLLM onNext={goNext} onBack={goBack} />,
    <StepGitLab onNext={() => { goNext(); handleFinish() }} onBack={goBack} />,
    <StepDone />,
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

- [ ] **Step 7: Run build**

Run: `cd ring-frontend && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 8: Commit**

```bash
git add src/components/setup/
git commit -m "feat(fe): add Setup wizard (Welcome/Identity/LLM/GitLab/Done)"
```

---

### Task 10: App Shell — Router + Layout Integration

**Files:**
- Create: `src/App.tsx`
- Create: `src/main.tsx`
- Create: `src/components/layout/AppLayout.tsx`

- [ ] **Step 1: Create AppLayout**

`src/components/layout/AppLayout.tsx`:
```tsx
import { Sidebar } from './Sidebar'
import { HeaderTabBar } from './HeaderTabBar'
import { PanelStack } from './PanelStack'
import { ChatArea } from '../chat/ChatArea'
import { SelfFloat } from '../self/SelfFloat'
import { SelfTrigger } from '../self/SelfTrigger'
import { useAppStore } from '../../stores/app-store'

export function AppLayout() {
  const current_context = useAppStore((s) => s.current_context)

  return (
    <div style={{ display: 'flex', height: '100%', width: '100%' }}>
      <Sidebar />
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minWidth: 0 }}>
        {current_context !== 'super' && <HeaderTabBar />}
        <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
          <ChatArea />
          <PanelStack />
        </div>
      </div>
      <SelfFloat />
      <SelfTrigger />
    </div>
  )
}
```

- [ ] **Step 2: Create App**

`src/App.tsx`:
```tsx
import { useAppStore } from './stores/app-store'
import { AppLayout } from './components/layout/AppLayout'
import { SetupWizard } from './components/setup/SetupWizard'
import './index.css'

export default function App() {
  const is_setup = useAppStore((s) => s.is_setup)

  if (!is_setup) {
    return <SetupWizard />
  }

  return <AppLayout />
}
```

- [ ] **Step 3: Create main.tsx**

`src/main.tsx`:
```tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
```

- [ ] **Step 4: Run build**

Run: `cd ring-frontend && npx tsc --noEmit && npx vite build`
Expected: build succeeds

- [ ] **Step 5: Run all tests**

Run: `cd ring-frontend && npx vitest run`
Expected: all PASS

- [ ] **Step 6: Run lint**

Run: `cd ring-frontend && npx eslint src/`
Expected: no errors

- [ ] **Step 7: Commit**

```bash
git add src/App.tsx src/main.tsx src/components/layout/AppLayout.tsx
git commit -m "feat(fe): add App shell with router, layout integration, and setup gate"
```

---

### Task 11: Create test setup file

**Files:**
- Create: `src/test/setup.ts`

- [ ] **Step 1: Create test setup**

`src/test/setup.ts`:
```typescript
import '@testing-library/jest-dom/vitest'
```

- [ ] **Step 2: Commit**

```bash
git add src/test/setup.ts
git commit -m "feat(fe): add vitest test setup file"
```

---

### Task 12: Update index.html title + verify full dev server

**Files:**
- Modify: `index.html`

- [ ] **Step 1: Update page title**

Change `<title>ring-frontend</title>` to `<title>Ring</title>` in `index.html`.

- [ ] **Step 2: Start dev server and verify**

Run: `cd ring-frontend && npx vite --host`
Expected: dev server starts on localhost:5173, page loads Setup wizard

- [ ] **Step 3: Commit**

```bash
git add index.html
git commit -m "chore(fe): update page title to Ring"
```

---

## Self-Review

**Spec coverage check:**
- Sidebar (Super Ring + Ring list + Session indicator) → Task 5 ✅
- Header Tab Bar (Chat/Graph/Archive/Config) → Task 6 ✅
- Panel Stack (stackable, close individual) → Task 6 ✅
- Chat Area (messages + input) → Task 7 ✅
- Mode Indicator + Selector (interaction mode + skill mode) → Task 7 ✅
- Command Hints → Task 7 ✅
- Self Floating Window (draggable + 3 tabs) → Task 8 ✅
- Self Trigger (click/drag) → Task 8 ✅
- Setup Wizard (5 steps) → Task 9 ✅
- IceChat theme CSS → Task 1 ✅
- Types → Task 1 ✅
- All stores → Task 2 ✅

**Placeholder scan:** No TBD/TODO found. All steps have complete code.

**Type consistency:** All type imports reference files created in Task 1. Store interfaces match component props. Mock data uses correct types.
