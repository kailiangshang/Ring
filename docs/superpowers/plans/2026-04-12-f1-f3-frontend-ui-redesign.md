# F1-F3 Frontend UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign Ring Space layout with TabBar + BottomBar, add minimal permission UI, and enhance Ring Hub cards.

**Architecture:** Rebuild RingSpaceLayout with new structural components (TabBar, BottomBar). Simplify RingSidebar to only show NodeTree. Add modeStore for interaction mode. Extend Ring type and mock data for card stats.

**Tech Stack:** React 19 + TypeScript + Zustand + CSS custom properties + MSW for mocking + Vitest for testing

---

### Task 1: Create modeStore

**Files:**
- Create: `ring-frontend/src/stores/modeStore.ts`
- Create: `ring-frontend/src/stores/modeStore.test.ts`

- [ ] **Step 1: Write the modeStore**

Create `ring-frontend/src/stores/modeStore.ts`:

```typescript
import { create } from 'zustand'

type InteractionMode = 'daily' | 'manual_archive' | 'auto'

interface ModeState {
  mode: InteractionMode
  set_mode: (mode: InteractionMode) => void
}

export const useModeStore = create<ModeState>((set) => ({
  mode: 'daily',
  set_mode: (mode) => set({ mode }),
}))

export type { InteractionMode }
```

- [ ] **Step 2: Write the test**

Create `ring-frontend/src/stores/modeStore.test.ts`:

```typescript
import { describe, it, expect, beforeEach } from 'vitest'
import { useModeStore } from './modeStore'

describe('modeStore', () => {
  beforeEach(() => {
    useModeStore.setState({ mode: 'daily' })
  })

  it('defaults to daily mode', () => {
    expect(useModeStore.getState().mode).toBe('daily')
  })

  it('switches to manual_archive', () => {
    useModeStore.getState().set_mode('manual_archive')
    expect(useModeStore.getState().mode).toBe('manual_archive')
  })

  it('switches to auto', () => {
    useModeStore.getState().set_mode('auto')
    expect(useModeStore.getState().mode).toBe('auto')
  })

  it('switches back to daily', () => {
    useModeStore.getState().set_mode('auto')
    useModeStore.getState().set_mode('daily')
    expect(useModeStore.getState().mode).toBe('daily')
  })
})
```

- [ ] **Step 3: Run test**

Run: `cd ring-frontend && npm test`
Expected: All tests pass (including new modeStore tests)

- [ ] **Step 4: Commit**

```bash
git add ring-frontend/src/stores/modeStore.ts ring-frontend/src/stores/modeStore.test.ts
git commit -m "feat: add modeStore for interaction mode switching"
```

---

### Task 2: Create TabBar component

**Files:**
- Create: `ring-frontend/src/components/layout/TabBar.tsx`
- Create: `ring-frontend/src/components/layout/TabBar.css`
- Create: `ring-frontend/src/components/layout/TabBar.test.tsx`

- [ ] **Step 1: Write TabBar CSS**

Create `ring-frontend/src/components/layout/TabBar.css`:

```css
.tab-bar {
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-primary);
  padding: 0 var(--space-4);
  flex-shrink: 0;
  gap: var(--space-1);
}

.tab-bar-item {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-2) var(--space-3);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  text-decoration: none;
  border-bottom: 2px solid transparent;
  transition: color 150ms ease, border-color 150ms ease;
}

.tab-bar-item:hover {
  color: var(--color-text-primary);
}

.tab-bar-item-active {
  color: var(--color-accent);
  border-bottom-color: var(--color-accent);
  font-weight: 500;
}
```

- [ ] **Step 2: Write TabBar component**

Create `ring-frontend/src/components/layout/TabBar.tsx`:

```tsx
import { useParams, useLocation } from 'react-router-dom'
import './TabBar.css'

const TABS = [
  { path: '', label: 'Chat', icon: '💬' },
  { path: '/graph', label: 'Graph', icon: '◉' },
  { path: '/prs', label: 'PRs', icon: '📋' },
  { path: '/members', label: 'Members', icon: '👥' },
  { path: '/sessions', label: 'Sessions', icon: '🔍' },
]

export function TabBar() {
  const { ringId } = useParams<{ ringId: string }>()
  const location = useLocation()

  if (!ringId) return null

  return (
    <nav className="tab-bar">
      {TABS.map((tab) => {
        const to = `/ring/${ringId}${tab.path}`
        const is_active = tab.path === ''
          ? location.pathname === `/ring/${ringId}` || location.pathname === `/ring/${ringId}/`
          : location.pathname.startsWith(to)
        return (
          <a
            key={tab.path}
            href={to}
            className={`tab-bar-item${is_active ? ' tab-bar-item-active' : ''}`}
          >
            <span>{tab.icon}</span>
            <span>{tab.label}</span>
          </a>
        )
      })}
    </nav>
  )
}
```

- [ ] **Step 3: Write TabBar test**

Create `ring-frontend/src/components/layout/TabBar.test.tsx`:

```tsx
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { TabBar } from './TabBar'

function renderWithRouter(ui: React.ReactElement, { path = '/ring/ring-1' } = {}) {
  window.history.pushState({}, '', path)
  return render(<BrowserRouter>{ui}</BrowserRouter>)
}

describe('TabBar', () => {
  it('renders all 5 tabs', () => {
    renderWithRouter(<TabBar />)
    expect(screen.getByText('Chat')).toBeInTheDocument()
    expect(screen.getByText('Graph')).toBeInTheDocument()
    expect(screen.getByText('PRs')).toBeInTheDocument()
    expect(screen.getByText('Members')).toBeInTheDocument()
    expect(screen.getByText('Sessions')).toBeInTheDocument()
  })
})
```

- [ ] **Step 4: Run test**

Run: `cd ring-frontend && npm test`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/components/layout/TabBar.tsx ring-frontend/src/components/layout/TabBar.css ring-frontend/src/components/layout/TabBar.test.tsx
git commit -m "feat: add TabBar component for view switching"
```

---

### Task 3: Create BottomBar component

**Files:**
- Create: `ring-frontend/src/components/layout/BottomBar.tsx`
- Create: `ring-frontend/src/components/layout/BottomBar.css`
- Create: `ring-frontend/src/components/layout/BottomBar.test.tsx`

- [ ] **Step 1: Write BottomBar CSS**

Create `ring-frontend/src/components/layout/BottomBar.css`:

```css
.bottom-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid var(--color-border);
  background: var(--color-bg-primary);
  padding: var(--space-2) var(--space-4);
  flex-shrink: 0;
}

.bottom-bar-left {
  display: flex;
  gap: var(--space-2);
}

.bottom-bar-right {
  display: flex;
  gap: var(--space-2);
}

.mode-btn {
  padding: 4px 12px;
  border-radius: var(--radius-full);
  border: 1px solid var(--color-border);
  background: var(--color-bg-primary);
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: var(--font-size-sm);
  font-family: var(--font-sans);
  transition: background 100ms ease, border-color 100ms ease, color 100ms ease;
}

.mode-btn:hover {
  border-color: var(--color-text-tertiary);
}

.mode-btn-active {
  border-color: var(--color-accent);
  background: var(--color-accent-light);
  color: var(--color-accent);
}
```

- [ ] **Step 2: Write BottomBar component**

Create `ring-frontend/src/components/layout/BottomBar.tsx`:

```tsx
import { useModeStore, type InteractionMode } from '../../stores/modeStore'
import { Toolbar, type ToolStatus } from '../toolbar/Toolbar'
import './BottomBar.css'

const MODES: { key: InteractionMode; label: string }[] = [
  { key: 'daily', label: '日常' },
  { key: 'manual_archive', label: '手动归档' },
  { key: 'auto', label: 'Auto' },
]

interface BottomBarProps {
  tools?: ToolStatus[]
  on_tool_toggle?: (tool_name: string) => void
  show_tools?: boolean
}

export function BottomBar({ tools = [], on_tool_toggle, show_tools = true }: BottomBarProps) {
  const mode = useModeStore((s) => s.mode)
  const set_mode = useModeStore((s) => s.set_mode)

  return (
    <div className="bottom-bar">
      <div className="bottom-bar-left">
        {MODES.map((m) => (
          <button
            key={m.key}
            className={`mode-btn${mode === m.key ? ' mode-btn-active' : ''}`}
            onClick={() => set_mode(m.key)}
          >
            {m.label}
          </button>
        ))}
      </div>
      {show_tools && tools.length > 0 && (
        <div className="bottom-bar-right">
          <Toolbar tools={tools} on_toggle={on_tool_toggle} />
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 3: Write BottomBar test**

Create `ring-frontend/src/components/layout/BottomBar.test.tsx`:

```tsx
import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { BottomBar } from './BottomBar'
import { useModeStore } from '../../stores/modeStore'

describe('BottomBar', () => {
  beforeEach(() => {
    useModeStore.setState({ mode: 'daily' })
  })

  it('renders three mode buttons', () => {
    render(<BottomBar />)
    expect(screen.getByText('日常')).toBeInTheDocument()
    expect(screen.getByText('手动归档')).toBeInTheDocument()
    expect(screen.getByText('Auto')).toBeInTheDocument()
  })

  it('switches mode on click', () => {
    render(<BottomBar />)
    fireEvent.click(screen.getByText('Auto'))
    expect(useModeStore.getState().mode).toBe('auto')
  })

  it('does not show tools when show_tools is false', () => {
    render(<BottomBar tools={[{ name: 'search', description: 'Search', active: true }]} show_tools={false} />)
    expect(screen.queryByText('search')).not.toBeInTheDocument()
  })
})
```

- [ ] **Step 4: Run test**

Run: `cd ring-frontend && npm test`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/components/layout/BottomBar.tsx ring-frontend/src/components/layout/BottomBar.css ring-frontend/src/components/layout/BottomBar.test.tsx
git commit -m "feat: add BottomBar with mode switch and tool toggles"
```

---

### Task 4: Rewrite RingSpaceLayout

**Files:**
- Modify: `ring-frontend/src/components/layout/RingSpaceLayout.tsx`
- Modify: `ring-frontend/src/components/layout/RingSpaceLayout.css`
- Modify: `ring-frontend/src/components/layout/RingSidebar.tsx`
- Modify: `ring-frontend/src/components/layout/RingSidebar.css`

- [ ] **Step 1: Rewrite RingSpaceLayout.tsx**

Replace the full content of `ring-frontend/src/components/layout/RingSpaceLayout.tsx` with:

```tsx
import { useState, useEffect, createContext, useContext } from 'react'
import { Outlet, Link, useParams, useLocation } from 'react-router-dom'
import { AvatarGroup } from '../ui/AvatarGroup'
import { NotificationBell } from '../ui/NotificationBell'
import { RingSidebar } from './RingSidebar'
import { RightPanel } from './RightPanel'
import { TabBar } from './TabBar'
import { BottomBar } from './BottomBar'
import * as api from '../../api/client'
import './RingSpaceLayout.css'

interface RightPanelState {
  open: boolean
  content: 'node_detail' | 'diff' | 'node_selector' | null
  data: unknown
}

const RightPanelContext = createContext<{
  panel: RightPanelState
  set_panel: (s: RightPanelState) => void
}>({ panel: { open: false, content: null, data: null }, set_panel: () => {} })

export function useRightPanel() { return useContext(RightPanelContext) }

export function RingSpaceLayout() {
  const { ringId } = useParams<{ ringId: string }>()
  const location = useLocation()
  const [panel, set_panel] = useState<RightPanelState>({ open: false, content: null, data: null })
  const [sidebar_collapsed, set_sidebar_collapsed] = useState(false)
  const [ring_name, set_ring_name] = useState('Ring')
  const [member_names, set_member_names] = useState<string[]>([])

  const is_chat_view = !location.pathname.includes('/graph') && !location.pathname.includes('/prs') && !location.pathname.includes('/members') && !location.pathname.includes('/sessions')

  useEffect(() => {
    if (!ringId) return
    api.get_ring(ringId).then((ring) => set_ring_name(ring.name)).catch(() => {})
    api.list_members(ringId).then((members) => {
      set_member_names(members.map((m) => m.display_name))
    }).catch(() => {})
  }, [ringId])

  return (
    <RightPanelContext.Provider value={{ panel, set_panel }}>
      <div className="ring-space">
        <div className="ring-space-header">
          <Link to="/" className="ring-space-back">&larr; Hub</Link>
          <div className="ring-space-name">{ring_name}</div>
          <div className="ring-space-header-right">
            <AvatarGroup names={member_names} size="sm" />
            <button className="ring-space-invite-btn" title="Invite">📎</button>
            <NotificationBell items={[]} on_click={() => {}} />
          </div>
        </div>
        <TabBar />
        <div className="ring-space-body">
          <RingSidebar collapsed={sidebar_collapsed} on_toggle={() => set_sidebar_collapsed(!sidebar_collapsed)} />
          <div className="ring-space-main">
            <Outlet />
          </div>
          {panel.open && <RightPanel state={panel} on_close={() => set_panel({ open: false, content: null, data: null })} />}
        </div>
        <BottomBar show_tools={is_chat_view} />
      </div>
    </RightPanelContext.Provider>
  )
}
```

- [ ] **Step 2: Update RingSpaceLayout.css**

Add the invite button style to `ring-frontend/src/components/layout/RingSpaceLayout.css`. The file currently has 8 rules. Add after the `.ring-space-header-right` rule:

```css
.ring-space-invite-btn {
  padding: var(--space-1) var(--space-2);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg-primary);
  cursor: pointer;
  font-size: var(--font-size-sm);
  transition: border-color 150ms ease;
}
.ring-space-invite-btn:hover {
  border-color: var(--color-accent);
}
```

- [ ] **Step 3: Simplify RingSidebar.tsx**

Replace the full content of `ring-frontend/src/components/layout/RingSidebar.tsx` with:

```tsx
import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import * as api from '../../api/client'
import { NodeTree } from '../graph/NodeTree'
import { useRightPanel } from './RingSpaceLayout'
import type { GraphNode } from '../../types'
import './RingSidebar.css'

interface RingSidebarProps { collapsed: boolean; on_toggle: () => void }

export function RingSidebar({ collapsed, on_toggle }: RingSidebarProps) {
  const { ringId } = useParams<{ ringId: string }>()
  const [nodes, set_nodes] = useState<GraphNode[]>([])
  const [selected_node_id, set_selected_node_id] = useState<string | null>(null)
  const { set_panel } = useRightPanel()

  useEffect(() => {
    if (!ringId) return
    api.list_graphs(ringId).then((graph_ids) => {
      if (graph_ids.length > 0) {
        api.get_graph(ringId, graph_ids[0]).then((detail) => set_nodes(detail.nodes))
      }
    }).catch(() => {})
  }, [ringId])

  const handle_node_select = (node_id: string) => {
    set_selected_node_id(node_id)
    const node = nodes.find((n) => n.id === node_id)
    if (node) {
      set_panel({ open: true, content: 'node_detail', data: node })
    }
  }

  if (!ringId) return null

  return (
    <div className={`ring-sidebar${collapsed ? ' ring-sidebar-collapsed' : ''}`}>
      <div className="ring-sidebar-tree">
        {!collapsed && nodes.length > 0 && (
          <NodeTree
            nodes={nodes}
            selected_node_id={selected_node_id}
            on_select={handle_node_select}
          />
        )}
        {!collapsed && nodes.length === 0 && (
          <div className="ring-sidebar-placeholder">暂无图谱节点</div>
        )}
      </div>
      <button className="ring-sidebar-collapse-btn" onClick={on_toggle}>
        {collapsed ? '→' : '← 收起'}
      </button>
    </div>
  )
}
```

- [ ] **Step 4: Simplify RingSidebar.css**

Remove the `.ring-sidebar-nav`, `.ring-sidebar-nav-item`, `.ring-sidebar-nav-badge`, and `.ring-sidebar-divider` rules from `ring-frontend/src/components/layout/RingSidebar.css` (they are no longer used). Keep: `.ring-sidebar`, `.ring-sidebar-collapsed`, `.ring-sidebar-tree`, `.ring-sidebar-placeholder`, `.ring-sidebar-collapse-btn`.

- [ ] **Step 5: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All tests pass (some existing tests may need updates if they reference sidebar nav items — fix as needed)

- [ ] **Step 6: Commit**

```bash
git add ring-frontend/src/components/layout/RingSpaceLayout.tsx ring-frontend/src/components/layout/RingSpaceLayout.css ring-frontend/src/components/layout/RingSidebar.tsx ring-frontend/src/components/layout/RingSidebar.css
git commit -m "feat: redesign Ring Space layout with TabBar, BottomBar, simplified sidebar"
```

---

### Task 5: Update ChatView to remove inline Toolbar

**Files:**
- Modify: `ring-frontend/src/pages/RingSpace/ChatView.tsx`

- [ ] **Step 1: Read ChatView.tsx and remove inline Toolbar**

Read `ring-frontend/src/pages/RingSpace/ChatView.tsx`. The Toolbar component is currently rendered inside ChatView with its own `tools` state. Since tools are now in the BottomBar, remove the Toolbar import and rendering from ChatView. Keep the `tools` state and `handle_toggle` function — they will be lifted up to the layout level in a future task. For now, simply remove the `<Toolbar>` JSX from the ChatView render output.

Also remove the import `import { Toolbar } from '../../components/toolbar/Toolbar'`.

- [ ] **Step 2: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add ring-frontend/src/pages/RingSpace/ChatView.tsx
git commit -m "refactor: remove inline Toolbar from ChatView (moved to BottomBar)"
```

---

### Task 6: Update Ring type and mock data for card stats

**Files:**
- Modify: `ring-frontend/src/types/index.ts`
- Modify: `ring-frontend/src/mocks/handlers.ts`

- [ ] **Step 1: Extend Ring type**

In `ring-frontend/src/types/index.ts`, add three optional fields to the `Ring` interface:

```typescript
export interface Ring {
  id: string
  name: string
  description: string | null
  creator_id: string
  gitlab_repo: string
  local_path: string
  next_token_id: number
  status: string
  created_at: string
  updated_at: string
  member_count?: number
  graph_node_count?: number
  last_active_at?: string
}
```

- [ ] **Step 2: Update mock ring data**

In `ring-frontend/src/mocks/handlers.ts`, add the three fields to each `mock_rings` entry:

For the first mock ring:
```typescript
    member_count: 5,
    graph_node_count: 12,
    last_active_at: new Date(Date.now() - 2 * 3600 * 1000).toISOString(),
```

For the second mock ring:
```typescript
    member_count: 3,
    graph_node_count: 8,
    last_active_at: new Date(Date.now() - 24 * 3600 * 1000).toISOString(),
```

Also add these fields to the `POST /rings` handler's response:
```typescript
      member_count: 1,
      graph_node_count: 0,
      last_active_at: new Date().toISOString(),
```

- [ ] **Step 3: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add ring-frontend/src/types/index.ts ring-frontend/src/mocks/handlers.ts
git commit -m "feat: extend Ring type with member_count, graph_node_count, last_active_at"
```

---

### Task 7: Enhance RingList cards with stats

**Files:**
- Modify: `ring-frontend/src/pages/RingHub/RingList.tsx`
- Modify: `ring-frontend/src/pages/RingHub/RingHub.css`
- Modify: `ring-frontend/src/pages/RingHub/RingHub.tsx`

- [ ] **Step 1: Add stats row to RingList cards**

In `ring-frontend/src/pages/RingHub/RingList.tsx`, replace the full content with:

```tsx
import { Badge } from '../../components/ui/Badge'
import { EmptyState } from '../../components/ui/EmptyState'
import type { Ring } from '../../types'
import './RingHub.css'

interface RingListProps {
  rings: Ring[]
  on_select: (id: string) => void
  on_create: () => void
}

function format_relative_time(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 60) return `${minutes}m ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

export function RingList({ rings, on_select, on_create }: RingListProps) {
  if (rings.length === 0) {
    return (
      <EmptyState
        icon="⊕"
        title="No rings yet"
        description="Create your first Ring to get started with collaborative knowledge management."
        action_label="Create Ring"
        on_action={on_create}
      />
    )
  }

  return (
    <div className="ring-hub-grid">
      {rings.map((ring) => (
        <div
          key={ring.id}
          className="ring-card"
          onClick={() => on_select(ring.id)}
        >
          <div className="ring-card-header">
            <span className="ring-card-dot" />
            <span className="ring-card-name">{ring.name}</span>
          </div>
          <div className="ring-card-desc">
            {ring.description || 'No description'}
          </div>
          {(ring.member_count != null || ring.graph_node_count != null || ring.last_active_at) && (
            <div className="ring-card-stats">
              {ring.member_count != null && <span>👥 {ring.member_count}</span>}
              {ring.graph_node_count != null && <span>◉ {ring.graph_node_count}</span>}
              {ring.last_active_at && <span>{format_relative_time(ring.last_active_at)}</span>}
            </div>
          )}
          <div className="ring-card-divider" />
          <div className="ring-card-footer">
            <span className="ring-card-meta">{ring.status}</span>
            <Badge status={ring.status}>{ring.status}</Badge>
          </div>
        </div>
      ))}
    </div>
  )
}
```

- [ ] **Step 2: Add stats CSS**

Add to `ring-frontend/src/pages/RingHub/RingHub.css`, after the `.ring-card-desc` rule:

```css
.ring-card-stats {
  display: flex;
  gap: var(--space-3);
  font-size: var(--font-size-xs);
  color: var(--color-text-tertiary);
  margin-bottom: var(--space-2);
}
```

- [ ] **Step 3: Update RingHub.tsx to pass on_create**

In `ring-frontend/src/pages/RingHub/RingHub.tsx`, add a `show_create` state and update `RingList` props:

Add state after existing state declarations:
```typescript
  const [show_create, set_show_create] = useState(false)
```

Change the `<RingList>` usage to pass `on_create`:
```tsx
        <RingList rings={rings} on_select={handle_select} on_create={() => set_show_create(true)} />
```

And wire `show_create` to the `CreateRing` modal (the `CreateRing` component already manages its own modal visibility, so just make the empty state button work by setting the same trigger). The simplest approach: pass `on_create` to `RingList` which calls `set_show_create(true)`, and the existing `CreateRing` button in the header already opens its own modal. For the empty state, we need to expose the same create flow.

Actually, looking at the existing code, `CreateRing` manages its own internal `open` state with a button. The empty state just needs to trigger the same open. The simplest fix: lift the create modal open state to RingHub and pass it down.

Replace the full content of `ring-frontend/src/pages/RingHub/RingHub.tsx` with:

```tsx
import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import * as api from '../../api/client'
import { Skeleton } from '../../components/ui/Skeleton'
import { RingList } from './RingList'
import { CreateRing } from './CreateRing'
import type { Ring, CreateRingRequest } from '../../types'
import './RingHub.css'

export function RingHub() {
  const [rings, set_rings] = useState<Ring[]>([])
  const [loading, set_loading] = useState(true)
  const [error, set_error] = useState<string | null>(null)
  const [show_create, set_show_create] = useState(false)
  const navigate = useNavigate()

  useEffect(() => {
    load_rings()
  }, [])

  const load_rings = async () => {
    set_loading(true)
    try {
      const data = await api.list_rings()
      set_rings(data)
    } catch (e) {
      set_error((e as Error).message)
    } finally {
      set_loading(false)
    }
  }

  const handle_create = async (req: CreateRingRequest) => {
    await api.create_ring(req)
    set_show_create(false)
    await load_rings()
  }

  const handle_select = (id: string) => {
    navigate(`/ring/${id}`)
  }

  return (
    <div className="ring-hub">
      <div className="ring-hub-header">
        <div>
          <h1 className="ring-hub-title">Ring Hub</h1>
          <p className="ring-hub-subtitle">你的群组知识协作空间</p>
        </div>
        <CreateRing on_create={handle_create} open={show_create} on_close={() => set_show_create(false)} />
      </div>

      {error && <p className="setup-error" role="alert">{error}</p>}

      {loading ? (
        <div className="ring-hub-grid">
          {[1, 2, 3].map((i) => (
            <Skeleton key={i} width="100%" height="120px" />
          ))}
        </div>
      ) : (
        <RingList rings={rings} on_select={handle_select} on_create={() => set_show_create(true)} />
      )}

      <div className="ring-hub-footer">对话记录仅保存在当前设备</div>
    </div>
  )
}
```

Note: This requires `CreateRing` to accept `open` and `on_close` props. Check if it already does — if not, update `CreateRing.tsx` to accept these props and use them to control the modal visibility.

- [ ] **Step 4: Update CreateRing.tsx if needed**

Read `ring-frontend/src/pages/RingHub/CreateRing.tsx`. If it uses internal `open` state with its own trigger button, update it to also accept external `open` and `on_close` props:

Add to the props interface:
```typescript
interface CreateRingProps {
  on_create: (req: CreateRingRequest) => Promise<void>
  open?: boolean
  on_close?: () => void
}
```

Merge internal and external open state:
```typescript
export function CreateRing({ on_create, open: external_open, on_close }: CreateRingProps) {
  const [internal_open, set_internal_open] = useState(false)
  const is_open = external_open ?? internal_open
  // ... rest stays the same, but close handler becomes:
  // on_close ? on_close() : set_internal_open(false)
```

- [ ] **Step 5: Run tests**

Run: `cd ring-frontend && npm test`
Expected: All tests pass (may need to update CreateRing test and RingList test for new props)

- [ ] **Step 6: Commit**

```bash
git add ring-frontend/src/pages/RingHub/RingList.tsx ring-frontend/src/pages/RingHub/RingHub.tsx ring-frontend/src/pages/RingHub/RingHub.css ring-frontend/src/pages/RingHub/CreateRing.tsx
git commit -m "feat: add card stats (members, nodes, activity) and fix empty state CTA"
```

---

### Task 8: Final verification

- [ ] **Step 1: Run full frontend test suite**

Run: `cd ring-frontend && npm test`
Expected: All tests pass

- [ ] **Step 2: Run ESLint**

Run: `cd ring-frontend && npx eslint src/`
Expected: No errors

- [ ] **Step 3: Verify no TypeScript errors**

Run: `cd ring-frontend && npx tsc --noEmit`
Expected: No errors
