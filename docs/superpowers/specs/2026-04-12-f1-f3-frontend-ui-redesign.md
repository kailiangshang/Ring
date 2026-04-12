# F1-F3 Frontend UI Redesign Spec

Date: 2026-04-12

## Scope

Three frontend modules developed independently with MSW mock data:

- F1: Ring Space Layout redesign
- F2: Minimal Permission UI (mode switch + role awareness)
- F3: Ring Hub card enhancement

No backend changes. All new data fields are mocked.

---

## F1: Ring Space Layout Redesign

### Current State

Three-column layout: Sidebar (240px, collapsible) + Main (Outlet) + RightPanel (280px, optional). Sidebar contains NodeTree at top + 5 navigation links at bottom (Chat/Graph/PRs/Members/Sessions). Top bar has back link + ring name + AvatarGroup + NotificationBell.

### New Layout

```
┌──────────────────────────────────────────────────┐
│  ← Hub │ Ring Name       [👥 avatars] [📎邀请] [🔔] │  TopBar 48px
├────┬───────────────────────────────────┬──────────┤
│    │  [💬Chat] [◉Graph] [📋PRs] [👥Members] [🔍Sessions] │  TabBar
│ N  ├───────────────────────────────────┤          │
│ o  │                                   │  Right   │
│ d  │    <Outlet /> main content        │  Panel   │
│ e  │    (ChatView / GraphView / ...)   │  280px   │
│ T  │                                   │  (optional)
│ r  │                                   │          │
│ e  │                                   │          │
│ e  ├───────────────────────────────────┤          │
│    │ [日常] [手动归档] [Auto] │ [tool1][tool2]... │  BottomBar
└────┴───────────────────────────────────┴──────────┘
 220px         flex: 1
```

### Components to Modify

**RingSpaceLayout.tsx** — Major rewrite:
- Remove navigation links from Sidebar, keep only NodeTree + collapse button
- Add TabBar between TopBar and main content area
- Add BottomBar below main content area
- TopBar: add invite button

**New components:**
- `TabBar.tsx` — 5 tabs rendered from route config, active tab highlighted, uses NavLink under the hood
- `BottomBar.tsx` — left section: 3 mode buttons; right section: tool toggles from Toolbar

**RingSidebar.tsx** — Simplify:
- Remove all NavLink items (Chat/Graph/PRs/Members/Sessions)
- Keep only: NodeTree section + collapse toggle

**RightPanel.tsx** — No changes to layout structure.

### TabBar Design

5 tabs, each corresponds to a child route:
| Tab | Icon | Route | Component |
|-----|------|-------|-----------|
| Chat | 💬 | `/ring/:id` (index) | ChatView |
| Graph | ◉ | `/ring/:id/graph` | GraphView |
| PRs | 📋 | `/ring/:id/prs` | PrList |
| Members | 👥 | `/ring/:id/members` | MemberList |
| Sessions | 🔍 | `/ring/:id/sessions` | SessionView |

Active tab is determined by current route. TabBar renders as a horizontal flex with subtle bottom border, integrated into the layout frame (not inside main content scroll area).

### BottomBar Design

Left section: 3 mode toggle buttons
- 日常 (daily) — default, highlighted when active
- 手动归档 (manual archive)
- Auto

Right section: Tool toggles (reuse existing Toolbar component's rendering logic)
- Only shown when in Chat view (hidden in Graph/PRs/Members/Sessions views)

### TopBar Changes

Add invite button (📎 icon) between AvatarGroup and NotificationBell. Clicking opens a placeholder modal (no real invite API yet). The modal shows a simple message: "邀请功能开发中" with a close button.

### Routing

No routing changes. Same routes as before, just the navigation moves from sidebar to TabBar.

---

## F2: Minimal Permission UI

### Mode Switch

**New store: `modeStore.ts`**
```typescript
type InteractionMode = 'daily' | 'manual_archive' | 'auto'

interface ModeState {
  mode: InteractionMode
  set_mode: (mode: InteractionMode) => void
}
```

Default: `'daily'`. The BottomBar reads and writes this store. Other components can read it to adjust behavior (future: Auto mode changes AI behavior, manual_archive shows export button prominently).

### Role Awareness

**Mock data approach:** Add a `current_role` field to the ring data in MSW handlers:
```typescript
// In mock handlers
const mockRole = 'creator' // or 'admin' | 'member' | 'readonly'
```

**Visibility rules (minimal):**
- `readonly` role: hide BottomBar entirely, hide ChatInput in ChatView
- All other roles: show everything

No fine-grained permission matrix yet. Just the readonly → hide actions pattern.

### Invite Button Placeholder

A button in TopBar that opens a Modal saying "邀请功能开发中". This reserves the UI location for future invite flow integration.

---

## F3: Ring Hub Card Enhancement

### Type Extension

In `types/index.ts`, extend Ring interface with optional mock fields:
```typescript
export interface Ring {
  // ... existing fields
  member_count?: number
  graph_node_count?: number
  last_active_at?: string
}
```

These are optional so they degrade gracefully when backend doesn't provide them.

### MSW Mock Data

Update mock handlers to include these fields:
```typescript
{
  member_count: Math.floor(Math.random() * 10) + 1,
  graph_node_count: Math.floor(Math.random() * 30),
  last_active_at: new Date(Date.now() - Math.random() * 7 * 24 * 3600 * 1000).toISOString(),
}
```

### Card Redesign

In `RingList.tsx`, add a stats row between description and date:
```
┌──────────────────────┐
│ ● Ring Name          │
│ Description text...  │
│ 👥 5 · ◉ 12 · 2h前  │  ← new stats row
│──────────────────────│
│ 2026-04-12 · active  │
└──────────────────────┘
```

The stats row shows: member count (👥), node count (◉), relative time since last_active_at.

### Empty State Fix

Fix the `on_action` in EmptyState — it should call the same handler as the CreateRing button (open the create modal).

### Privacy Notice

Already exists as footer text "对话记录仅保存在当前设备". Keep as-is.

---

## File Change Summary

### New Files
| File | Purpose |
|------|---------|
| `components/layout/TabBar.tsx` + `.css` | Tab navigation for 5 views |
| `components/layout/BottomBar.tsx` + `.css` | Mode switch + tool toggles |
| `stores/modeStore.ts` | Interaction mode state |

### Modified Files
| File | Changes |
|------|---------|
| `components/layout/RingSpaceLayout.tsx` | Major rewrite: add TabBar + BottomBar, restructure layout |
| `components/layout/RingSpaceLayout.css` | Layout grid/flex adjustments |
| `components/layout/RingSidebar.tsx` + `.css` | Remove nav items, keep only NodeTree |
| `components/layout/RightPanel.tsx` | No structural changes |
| `pages/RingHub/RingList.tsx` | Add stats row to cards |
| `pages/RingHub/RingHub.tsx` | Fix EmptyState on_action |
| `pages/RingHub/RingHub.css` | Card stat row styles |
| `types/index.ts` | Add member_count, graph_node_count, last_active_at |
| `mocks/handlers.ts` | Add mock fields to ring responses |

### Tests
- `TabBar.test.tsx` — renders 5 tabs, highlights active
- `BottomBar.test.tsx` — renders mode buttons, toggles mode on click
- `RingList.test.tsx` — update to verify stats display
- `modeStore.test.ts` — mode toggle behavior
