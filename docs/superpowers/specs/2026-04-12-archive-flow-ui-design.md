# Archive Flow UI Design

> **Date**: 2026-04-12
> **Scope**: F5 — Archive workflow frontend UI (PRD 2.4, 6.2)
> **Depends on**: Existing backend APIs (`archive_content`, `confirm_archive`, `get_archive_queue`)

## Goal

Connect the existing archive backend APIs to real frontend interactions. Currently all archive UI is stubs: `ArchiveSuggestion` has no-op handlers, no export button exists, and the PR queue is not displayed.

## Architecture

The archive flow has 3 trigger points (per PRD 2.4):
1. User clicks export button in ChatView → manual archive
2. AI sends `archive_suggestion` SSE event → user accepts/rejects
3. Auto mode → AI archives without user confirmation (handled by backend)

All three converge on the same backend API: `POST /rings/:ringId/archive`.

## Module 1: Export Button in ChatView

**File**: `ring-frontend/src/pages/RingSpace/ChatView.tsx`

Add an export button (icon: 📥, label "归档") next to the ChatInput. When clicked:
1. Collect IDs of the last 5 un-archived messages from `chatStore.messages` (or all un-archived if fewer than 5)
2. Determine the `graph_id`: use `graphStore.current_graph_id` or the first graph from `graphStore.graphs`
3. Auto-generate `label` from the last user message (first 30 chars)
4. Call `archive_content(ring_id, { message_ids, conversation_id, graph_id, label })`
5. On success, show a brief toast ("归档请求已提交") and if the response contains `archive_id`, open `ArchiveConfirmDialog`

**New store actions in chatStore**:
- `trigger_archive(ring_id: string)` — gathers message IDs and calls `api.archive_content`

**UI**:
- The export button is only visible in `manual_archive` mode (from `modeStore`)
- In `daily` mode, the export button is hidden (use `archive_suggestion` SSE instead)
- In `auto` mode, no button needed (backend handles it)

## Module 2: Enhanced ArchiveSuggestion

**File**: `ring-frontend/src/components/chat/ArchiveSuggestion.tsx`

Enhance the current card to show:
- `reason` (existing): AI's explanation of why this should be archived
- `suggested_title` (existing): proposed node title
- `suggested_parent` (new field from SSE data): the parent node AI recommends
- `action_preview` (new field): brief preview of what will happen (e.g., "将创建新节点「会议纪要」在「会议记录」下")

Button behavior:
- **Accept** → open `ArchiveConfirmDialog` with the suggestion data
- **Dismiss** → remove the suggestion from `chatStore.tool_events`

**Data contract** (from backend SSE `archive_suggestion` event):
```typescript
interface ArchiveSuggestionData {
  reason: string
  suggested_title: string
  suggested_parent?: { id: string; label: string }
  target_node_id?: string
  action_preview?: string
}
```

## Module 3: ArchiveConfirmDialog

**Files**:
- Create: `ring-frontend/src/components/archive/ArchiveConfirmDialog.tsx`
- Create: `ring-frontend/src/components/archive/ArchiveConfirmDialog.css`

A modal dialog that shows:
1. **Title**: "确认归档"
2. **Content preview**: brief excerpt of what's being archived
3. **Node placement**:
   - Display AI's recommended node (read-only text)
   - "更改位置" button → opens a compact node selector (tree of graph nodes from sidebar)
4. **Action buttons**: "确认归档" / "取消"

Flow:
1. User clicks confirm → call `confirm_archive(ring_id, archive_id)` if `archive_id` exists
2. If no `archive_id` yet (e.g. direct from export button), call `archive_content` with the selected `target_node_id`
3. On success: show toast, close dialog, optionally refresh graph sidebar
4. On error: show error inline

**Props**:
```typescript
interface ArchiveConfirmDialogProps {
  open: boolean
  on_close: () => void
  ring_id: string
  archive_id?: string
  suggested_title?: string
  suggested_parent?: { id: string; label: string }
  message_ids: string[]
  conversation_id: string
  graph_id: string
  label: string
}
```

**Node selector**: Reuse the existing `NodeTree` component inside the dialog. User clicks a node to set it as the target parent. Show a "新节点" option at the top for creating under root.

## Module 4: PR Archive Queue in PrList

**File**: `ring-frontend/src/pages/RingSpace/PrList.tsx`

Add a queue status bar at the top of the PR list:
1. On mount, call `gitStore.load_archive_queue(ring_id)`
2. Display:
   - If `current_review` exists: "正在审核: {title} (by {author})"
   - Queue count: "排队中: {queue.length} 个"
3. Styling: subtle info bar with light background, not intrusive
4. Auto-refresh: poll every 30s while on the PRs tab

**New file**: `ring-frontend/src/components/archive/ArchiveQueueBar.tsx`

```typescript
interface ArchiveQueueBarProps {
  ring_id: string
}
```

## chatStore Additions

Add to `ring-frontend/src/stores/chatStore.ts`:

```typescript
interface ChatState {
  // ... existing fields
  archive_pending: ArchivePending | null
  trigger_archive: (ring_id: string) => Promise<void>
  dismiss_suggestion: (event_id: string) => void
  clear_archive_pending: () => void
}

interface ArchivePending {
  archive_id: string
  suggested_title: string
  suggested_parent?: { id: string; label: string }
  message_ids: string[]
  conversation_id: string
  graph_id: string
  label: string
}
```

- `trigger_archive`: collects message IDs, calls API, sets `archive_pending`
- `dismiss_suggestion`: removes an `archive_suggestion` ToolEvent by ID
- `clear_archive_pending`: resets `archive_pending` to null

When `archive_suggestion` SSE event arrives and user clicks Accept, set `archive_pending` from the event data and open `ArchiveConfirmDialog`.

## Mode-Aware Behavior

Using `modeStore.mode`:
- `daily`: export button hidden. Only `archive_suggestion` from AI triggers archive flow
- `manual_archive`: export button visible. Both manual and suggestion flows work
- `auto`: export button hidden. No confirmation dialog needed (backend auto-commits)

## Testing

- Unit tests for `ArchiveConfirmDialog` (renders suggestion data, confirm calls API)
- Unit tests for `ArchiveQueueBar` (renders queue status)
- Unit tests for `chatStore` new actions (`trigger_archive`, `dismiss_suggestion`)
- Unit tests for enhanced `ArchiveSuggestion` (renders new fields, buttons trigger callbacks)

## Mock Data Updates

In `ring-frontend/src/mocks/handlers.ts`:
- `POST /rings/:ringId/archive`: return realistic response with `archive_id`, `markdown_path`, `git_status: 'pending'`
- `GET /rings/:ringId/archive/queue`: return mock queue with 1 current review and 2 queued items
- SSE mock: add an `archive_suggestion` event after text response in message handler
