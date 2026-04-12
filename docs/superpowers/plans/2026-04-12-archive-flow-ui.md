# Archive Flow UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect existing archive backend APIs to real frontend interactions — export button, enhanced ArchiveSuggestion, confirm dialog with node selector, and PR archive queue display.

**Architecture:** Extend chatStore with archive actions (trigger_archive, dismiss_suggestion). Add export button to ChatView (mode-aware). Enhance ArchiveSuggestion to show node placement and open ArchiveConfirmDialog. Add ArchiveQueueBar to PrList. All API calls already exist in `api/client.ts`.

**Tech Stack:** React 19 + TypeScript + Zustand + MSW for mocking + Vitest for testing

---

### Task 1: Update mock data for archive flow

**Files:**
- Modify: `ring-frontend/src/mocks/handlers.ts`

- [ ] **Step 1: Update POST /rings/:ringId/archive mock response**

In `ring-frontend/src/mocks/handlers.ts`, find the `http.post(\`${BASE}/rings/:ringId/archive\`)` handler and replace it with:

```typescript
  http.post(`${BASE}/rings/:ringId/archive`, async ({ request }) => {
    const body = await request.json() as { message_ids?: string[]; label?: string }
    return HttpResponse.json({
      archive_id: `arch-${Date.now()}`,
      markdown_path: `.ring/docs/${(body.label || 'untitled').slice(0, 30).replace(/\s+/g, '-')}.md`,
      git_status: 'pending',
      pr_url: null,
      queue_position: 2,
    }, { status: 201 })
  }),
```

- [ ] **Step 2: Update GET /rings/:ringId/archive/queue mock response**

Find the `http.get(\`${BASE}/rings/:ringId/archive/queue\`)` handler and replace it with:

```typescript
  http.get(`${BASE}/rings/:ringId/archive/queue`, () => {
    return HttpResponse.json({
      current_review: { pr_id: 3, author: 'Li', title: '添加学习笔记', position: 1 },
      queue: [
        { pr_id: 4, author: 'Ming', title: '更新技术对比', position: 2 },
        { pr_id: 5, author: 'Kai', title: '添加产品决策记录', position: 3 },
      ],
    })
  }),
```

- [ ] **Step 3: Add archive_suggestion to SSE mock**

Find the `http.post(\`${BASE}/rings/:ringId/conversations/:convId/messages\`)` handler. In the `chunks` array, after the `text` event chunk and before the `done` event chunk, add an `archive_suggestion` event:

```typescript
        const chunks = [
          `event: message\ndata: ${JSON.stringify({ type: 'text', content: '这是 mock 回复。在实际环境中，AI 会根据上下文生成回复。' })}\n\n`,
          `event: message\ndata: ${JSON.stringify({ type: 'archive_suggestion', data: { reason: '这段对话包含了有价值的产品决策信息', suggested_title: '产品决策记录', suggested_parent: { id: 'n4', label: '产品决策' }, action_preview: '将创建新节点「产品决策记录」在「产品决策」下', target_node_id: null } })}\n\n`,
          `event: message\ndata: ${JSON.stringify({ type: 'done', message_id: null, token_usage: null })}\n\n`,
        ]
```

- [ ] **Step 4: Run tests**

Run: `cd ring-frontend && npm test -- --run`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/mocks/handlers.ts
git commit -m "test: update mock data for archive flow"
```

---

### Task 2: Add chatStore archive actions

**Files:**
- Modify: `ring-frontend/src/stores/chatStore.ts`
- Create: `ring-frontend/src/stores/chatStore.test.ts`

- [ ] **Step 1: Write tests for new chatStore actions**

Create `ring-frontend/src/stores/chatStore.test.ts`:

```typescript
import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useChatStore } from './chatStore'

vi.mock('../api/client', () => ({
  archive_content: vi.fn().mockResolvedValue({
    archive_id: 'arch-test-1',
    markdown_path: '.ring/docs/test.md',
    git_status: 'pending',
    pr_url: null,
    queue_position: 1,
  }),
  confirm_archive: vi.fn().mockResolvedValue(undefined),
  create_conversation: vi.fn().mockResolvedValue({ id: 'conv-test', ring_id: 'ring-1', title: 'Test', mode: 'ring_group', context_mode: 'storage', token_count: 0, token_limit: 8000, auto_compact: false, summary: null, compacted_at: null, created_by: 'user-1', created_at: '', updated_at: '' }),
  get_messages: vi.fn().mockResolvedValue([]),
  send_message: vi.fn().mockResolvedValue(new Response(null, { status: 200 })),
  list_conversations: vi.fn().mockResolvedValue([]),
}))

beforeEach(() => {
  useChatStore.setState({
    messages: [],
    tool_events: [],
    is_streaming: false,
    current_conversation_id: 'conv-1',
    error: null,
    archive_pending: null,
  })
})

describe('chatStore archive actions', () => {
  it('trigger_archive collects un-archived message IDs and calls API', async () => {
    useChatStore.setState({
      messages: [
        { id: 'm1', conversation_id: 'conv-1', role: 'user', content: 'first', sender_id: '', tool_calls: null, archived: false, created_at: '' },
        { id: 'm2', conversation_id: 'conv-1', role: 'assistant', content: 'reply', sender_id: null, tool_calls: null, archived: false, created_at: '' },
        { id: 'm3', conversation_id: 'conv-1', role: 'user', content: 'second message that is longer', sender_id: '', tool_calls: null, archived: true, created_at: '' },
      ],
    })

    await useChatStore.getState().trigger_archive('ring-1', 'graph-1')

    const api = await import('../api/client')
    expect(api.archive_content).toHaveBeenCalledWith('ring-1', expect.objectContaining({
      message_ids: ['m1', 'm2'],
      conversation_id: 'conv-1',
      graph_id: 'graph-1',
      label: 'first',
    }))

    expect(useChatStore.getState().archive_pending).toEqual(expect.objectContaining({
      archive_id: 'arch-test-1',
      label: 'first',
    }))
  })

  it('dismiss_suggestion removes an archive_suggestion event', () => {
    useChatStore.setState({
      tool_events: [
        { id: 'evt-1', type: 'tool_call' as const, tool_name: 'search', input: null, timestamp: 1 },
        { id: 'evt-2', type: 'archive_suggestion' as const, data: { reason: 'test' }, timestamp: 2 },
        { id: 'evt-3', type: 'archive_suggestion' as const, data: { reason: 'test2' }, timestamp: 3 },
      ],
    })

    useChatStore.getState().dismiss_suggestion('evt-2')

    const events = useChatStore.getState().tool_events
    expect(events).toHaveLength(2)
    expect(events.find((e) => e.id === 'evt-2')).toBeUndefined()
  })

  it('clear_archive_pending resets archive_pending to null', async () => {
    await useChatStore.getState().trigger_archive('ring-1', 'graph-1')
    expect(useChatStore.getState().archive_pending).not.toBeNull()

    useChatStore.getState().clear_archive_pending()
    expect(useChatStore.getState().archive_pending).toBeNull()
  })

  it('trigger_archive limits to last 5 un-archived messages', async () => {
    const msgs = Array.from({ length: 8 }, (_, i) => ({
      id: `m${i}`,
      conversation_id: 'conv-1',
      role: 'user' as const,
      content: `message ${i}`,
      sender_id: '',
      tool_calls: null,
      archived: false,
      created_at: '',
    }))
    useChatStore.setState({ messages: msgs })

    await useChatStore.getState().trigger_archive('ring-1', 'graph-1')

    const api = await import('../api/client')
    expect(api.archive_content).toHaveBeenCalledWith('ring-1', expect.objectContaining({
      message_ids: ['m3', 'm4', 'm5', 'm6', 'm7'],
    }))
  })
})
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd ring-frontend && npm test -- --run`
Expected: Tests fail — `trigger_archive`, `dismiss_suggestion`, `clear_archive_pending` not defined

- [ ] **Step 3: Add archive actions to chatStore**

In `ring-frontend/src/stores/chatStore.ts`, add the `ArchivePending` type and extend the state interface. Add after the existing imports:

```typescript
export interface ArchivePending {
  archive_id: string
  suggested_title?: string
  suggested_parent?: { id: string; label: string }
  message_ids: string[]
  conversation_id: string
  graph_id: string
  label: string
}
```

Extend `ChatState` interface to add after `error: string | null`:

```typescript
  archive_pending: ArchivePending | null
  trigger_archive: (ring_id: string, graph_id: string) => Promise<void>
  dismiss_suggestion: (event_id: string) => void
  clear_archive_pending: () => void
```

In the store creation, add initial state after `error: null,`:

```typescript
  archive_pending: null,
```

Add the action implementations after the `reset` action:

```typescript
  trigger_archive: async (ring_id, graph_id) => {
    const { messages, current_conversation_id } = get()
    if (!current_conversation_id) return

    const unarchived = messages.filter((m) => !m.archived)
    const last_five = unarchived.slice(-5)
    if (last_five.length === 0) return

    const last_user_msg = [...last_five].reverse().find((m) => m.role === 'user')
    const label = (last_user_msg?.content || 'Archive').slice(0, 30)

    try {
      const res = await api.archive_content(ring_id, {
        message_ids: last_five.map((m) => m.id),
        conversation_id: current_conversation_id,
        graph_id,
        label,
      })
      set({
        archive_pending: {
          archive_id: res.archive_id,
          message_ids: last_five.map((m) => m.id),
          conversation_id: current_conversation_id,
          graph_id,
          label,
        },
      })
    } catch (e) {
      set({ error: (e as Error).message })
    }
  },

  dismiss_suggestion: (event_id) => {
    set((s) => ({ tool_events: s.tool_events.filter((e) => e.id !== event_id) }))
  },

  clear_archive_pending: () => set({ archive_pending: null }),
```

- [ ] **Step 4: Run tests**

Run: `cd ring-frontend && npm test -- --run`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/stores/chatStore.ts ring-frontend/src/stores/chatStore.test.ts
git commit -m "feat: add archive actions to chatStore (trigger_archive, dismiss_suggestion)"
```

---

### Task 3: Create ArchiveConfirmDialog component

**Files:**
- Create: `ring-frontend/src/components/archive/ArchiveConfirmDialog.tsx`
- Create: `ring-frontend/src/components/archive/ArchiveConfirmDialog.css`
- Create: `ring-frontend/src/components/archive/ArchiveConfirmDialog.test.tsx`

- [ ] **Step 1: Write ArchiveConfirmDialog CSS**

Create `ring-frontend/src/components/archive/ArchiveConfirmDialog.css`:

```css
.archive-confirm-label {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: var(--space-1);
}

.archive-confirm-value {
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: var(--space-3);
}

.archive-confirm-placement {
  background: var(--color-bg-secondary);
  border-radius: var(--radius-md);
  padding: var(--space-3);
  margin-bottom: var(--space-4);
}

.archive-confirm-placement-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-2);
}

.archive-confirm-placement-title {
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-text-primary);
}

.archive-confirm-change-btn {
  background: none;
  border: none;
  color: var(--color-accent);
  cursor: pointer;
  font-size: var(--font-size-xs);
  font-family: var(--font-sans);
}

.archive-confirm-change-btn:hover {
  text-decoration: underline;
}

.archive-confirm-node-selector {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-2);
  max-height: 200px;
  overflow-y: auto;
  margin-top: var(--space-2);
}

.archive-confirm-selected-node {
  font-size: var(--font-size-sm);
  color: var(--color-text-primary);
}

.archive-confirm-error {
  color: var(--color-error);
  font-size: var(--font-size-sm);
  margin-bottom: var(--space-2);
}
```

- [ ] **Step 2: Write ArchiveConfirmDialog component**

Create `ring-frontend/src/components/archive/ArchiveConfirmDialog.tsx`:

```tsx
import { useState } from 'react'
import { Button } from '../ui/Button'
import { Modal } from '../ui/Modal'
import { NodeTree } from '../graph/NodeTree'
import type { GraphNode } from '../../types'
import './ArchiveConfirmDialog.css'

interface ArchiveConfirmDialogProps {
  open: boolean
  on_close: () => void
  suggested_title?: string
  suggested_parent?: { id: string; label: string }
  nodes: GraphNode[]
  on_confirm: (target_node_id: string | undefined) => Promise<void>
  loading?: boolean
}

export function ArchiveConfirmDialog({
  open,
  on_close,
  suggested_title,
  suggested_parent,
  nodes,
  on_confirm,
  loading,
}: ArchiveConfirmDialogProps) {
  const [show_selector, set_show_selector] = useState(false)
  const [selected_parent_id, set_selected_parent_id] = useState<string | undefined>(
    suggested_parent?.id,
  )
  const [error, set_error] = useState<string | null>(null)
  const [confirming, set_confirming] = useState(false)

  const handle_confirm = async () => {
    set_error(null)
    set_confirming(true)
    try {
      await on_confirm(selected_parent_id)
      on_close()
    } catch (e) {
      set_error((e as Error).message)
    } finally {
      set_confirming(false)
    }
  }

  const selected_node = nodes.find((n) => n.id === selected_parent_id)

  return (
    <Modal
      open={open}
      on_close={on_close}
      title="确认归档"
      footer={
        <>
          <Button variant="secondary" onClick={on_close}>取消</Button>
          <Button onClick={handle_confirm} disabled={loading || confirming}>确认归档</Button>
        </>
      }
    >
      {error && <p className="archive-confirm-error" role="alert">{error}</p>}

      {suggested_title && (
        <div>
          <div className="archive-confirm-label">标题</div>
          <div className="archive-confirm-value">{suggested_title}</div>
        </div>
      )}

      <div className="archive-confirm-placement">
        <div className="archive-confirm-placement-header">
          <span className="archive-confirm-placement-title">节点位置</span>
          <button
            className="archive-confirm-change-btn"
            onClick={() => set_show_selector(!show_selector)}
          >
            {show_selector ? '收起' : '更改位置'}
          </button>
        </div>
        <div className="archive-confirm-selected-node">
          {selected_node ? selected_node.label : '(根节点下新建)'}
        </div>
        {show_selector && (
          <div className="archive-confirm-node-selector">
            <div
              style={{ padding: '4px 8px', cursor: 'pointer', fontSize: 'var(--font-size-sm)', color: selected_parent_id === undefined ? 'var(--color-accent)' : 'inherit' }}
              onClick={() => set_selected_parent_id(undefined)}
            >
              (根节点下新建)
            </div>
            <NodeTree
              nodes={nodes}
              selected_node_id={selected_parent_id ?? null}
              on_select={(id) => {
                set_selected_parent_id(id)
                set_show_selector(false)
              }}
            />
          </div>
        )}
      </div>
    </Modal>
  )
}
```

- [ ] **Step 3: Write ArchiveConfirmDialog test**

Create `ring-frontend/src/components/archive/ArchiveConfirmDialog.test.tsx`:

```tsx
import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ArchiveConfirmDialog } from './ArchiveConfirmDialog'
import type { GraphNode } from '../../types'

const mock_nodes: GraphNode[] = [
  { id: 'n1', label: '根节点', node_type: 'category', parent_id: null, description: null, graph_id: 'g1', markdown_path: null, created_at: '', updated_at: '' },
  { id: 'n2', label: '子节点', node_type: 'document', parent_id: 'n1', description: null, graph_id: 'g1', markdown_path: null, created_at: '', updated_at: '' },
]

describe('ArchiveConfirmDialog', () => {
  it('renders suggested title and parent', () => {
    render(
      <ArchiveConfirmDialog
        open={true}
        on_close={() => {}}
        suggested_title="会议纪要"
        suggested_parent={{ id: 'n1', label: '根节点' }}
        nodes={mock_nodes}
        on_confirm={vi.fn()}
      />,
    )
    expect(screen.getByText('会议纪要')).toBeInTheDocument()
    expect(screen.getByText('根节点')).toBeInTheDocument()
    expect(screen.getByText('确认归档')).toBeInTheDocument()
  })

  it('calls on_confirm when confirm button clicked', async () => {
    const on_confirm = vi.fn().mockResolvedValue(undefined)
    render(
      <ArchiveConfirmDialog
        open={true}
        on_close={() => {}}
        nodes={mock_nodes}
        on_confirm={on_confirm}
      />,
    )
    await fireEvent.click(screen.getByText('确认归档'))
    expect(on_confirm).toHaveBeenCalledWith(undefined)
  })

  it('shows node selector when change button clicked', () => {
    render(
      <ArchiveConfirmDialog
        open={true}
        on_close={() => {}}
        nodes={mock_nodes}
        on_confirm={vi.fn()}
      />,
    )
    fireEvent.click(screen.getByText('更改位置'))
    expect(screen.getByText('(根节点下新建)')).toBeInTheDocument()
    expect(screen.getByText('根节点')).toBeInTheDocument()
  })
})
```

- [ ] **Step 4: Run tests**

Run: `cd ring-frontend && npm test -- --run`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/components/archive/ArchiveConfirmDialog.tsx ring-frontend/src/components/archive/ArchiveConfirmDialog.css ring-frontend/src/components/archive/ArchiveConfirmDialog.test.tsx
git commit -m "feat: add ArchiveConfirmDialog with node selector"
```

---

### Task 4: Create ArchiveQueueBar component

**Files:**
- Create: `ring-frontend/src/components/archive/ArchiveQueueBar.tsx`
- Create: `ring-frontend/src/components/archive/ArchiveQueueBar.css`
- Create: `ring-frontend/src/components/archive/ArchiveQueueBar.test.tsx`

- [ ] **Step 1: Write ArchiveQueueBar CSS**

Create `ring-frontend/src/components/archive/ArchiveQueueBar.css`:

```css
.archive-queue-bar {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-2) var(--space-4);
  background: var(--color-bg-secondary);
  border-radius: var(--radius-md);
  margin-bottom: var(--space-4);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.archive-queue-bar-item {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.archive-queue-bar-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--color-accent);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.archive-queue-bar-empty {
  color: var(--color-text-tertiary);
}
```

- [ ] **Step 2: Write ArchiveQueueBar component**

Create `ring-frontend/src/components/archive/ArchiveQueueBar.tsx`:

```tsx
import { useEffect } from 'react'
import { useGitStore } from '../../stores/gitStore'
import './ArchiveQueueBar.css'

interface ArchiveQueueBarProps {
  ring_id: string
}

export function ArchiveQueueBar({ ring_id }: ArchiveQueueBarProps) {
  const { archive_queue, load_archive_queue } = useGitStore()

  useEffect(() => {
    load_archive_queue(ring_id)
    const interval = setInterval(() => load_archive_queue(ring_id), 30000)
    return () => clearInterval(interval)
  }, [ring_id, load_archive_queue])

  if (!archive_queue) return null

  const { current_review, queue } = archive_queue
  const has_activity = current_review || queue.length > 0

  if (!has_activity) {
    return (
      <div className="archive-queue-bar">
        <span className="archive-queue-bar-empty">归档队列空闲</span>
      </div>
    )
  }

  return (
    <div className="archive-queue-bar">
      {current_review && (
        <div className="archive-queue-bar-item">
          <span className="archive-queue-bar-dot" />
          <span>正在审核: {current_review.title} (by {current_review.author})</span>
        </div>
      )}
      {queue.length > 0 && (
        <div className="archive-queue-bar-item">
          <span>排队中: {queue.length} 个</span>
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 3: Write ArchiveQueueBar test**

Create `ring-frontend/src/components/archive/ArchiveQueueBar.test.tsx`:

```tsx
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ArchiveQueueBar } from './ArchiveQueueBar'
import { useGitStore } from '../../stores/gitStore'

vi.useFakeTimers()

beforeEach(() => {
  useGitStore.setState({
    archive_queue: null,
    loading: false,
    error: null,
    prs: [],
    current_pr: null,
    commit_log: [],
  })
})

describe('ArchiveQueueBar', () => {
  it('shows empty state when queue is empty', () => {
    useGitStore.setState({
      archive_queue: { current_review: null, queue: [] },
    })
    render(<ArchiveQueueBar ring_id="ring-1" />)
    expect(screen.getByText('归档队列空闲')).toBeInTheDocument()
  })

  it('shows current review and queue count', () => {
    useGitStore.setState({
      archive_queue: {
        current_review: { pr_id: 1, author: 'Li', title: '添加笔记', position: 1 },
        queue: [
          { pr_id: 2, author: 'Ming', title: '更新文档', position: 2 },
        ],
      },
    })
    render(<ArchiveQueueBar ring_id="ring-1" />)
    expect(screen.getByText(/正在审核: 添加笔记/)).toBeInTheDocument()
    expect(screen.getByText(/排队中: 1 个/)).toBeInTheDocument()
  })

  it('calls load_archive_queue on mount', () => {
    const load_fn = vi.fn()
    useGitStore.setState({ archive_queue: null })
    const orig = useGitStore.getState().load_archive_queue
    useGitStore.setState({ load_archive_queue: async (...args) => { load_fn(...args); return orig(...args) } })

    render(<ArchiveQueueBar ring_id="ring-1" />)
    expect(load_fn).toHaveBeenCalledWith('ring-1')
  })
})
```

- [ ] **Step 4: Run tests**

Run: `cd ring-frontend && npm test -- --run`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/components/archive/ArchiveQueueBar.tsx ring-frontend/src/components/archive/ArchiveQueueBar.css ring-frontend/src/components/archive/ArchiveQueueBar.test.tsx
git commit -m "feat: add ArchiveQueueBar with polling and queue display"
```

---

### Task 5: Enhance ArchiveSuggestion component

**Files:**
- Modify: `ring-frontend/src/components/chat/ArchiveSuggestion.tsx`
- Modify: `ring-frontend/src/components/chat/ArchiveSuggestion.css`

- [ ] **Step 1: Update ArchiveSuggestion CSS**

In `ring-frontend/src/components/chat/ArchiveSuggestion.css`, append after the `.archive-suggestion-actions` rule:

```css
.archive-suggestion-title {
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: var(--space-1);
}

.archive-suggestion-parent {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: var(--space-1);
}

.archive-suggestion-preview {
  font-size: var(--font-size-xs);
  color: var(--color-text-tertiary);
  margin-bottom: var(--space-2);
}
```

- [ ] **Step 2: Update ArchiveSuggestion component**

Replace the full content of `ring-frontend/src/components/chat/ArchiveSuggestion.tsx` with:

```tsx
import { Button } from '../ui/Button'
import './ArchiveSuggestion.css'

export interface ArchiveSuggestionData {
  reason?: string
  suggested_title?: string
  suggested_parent?: { id: string; label: string }
  action_preview?: string
  target_node_id?: string
}

export function ArchiveSuggestion({ data, on_accept, on_dismiss }: {
  data: unknown
  on_accept: (suggestion: ArchiveSuggestionData) => void
  on_dismiss: () => void
}) {
  const suggestion = data as ArchiveSuggestionData
  return (
    <div className="archive-suggestion">
      <div className="archive-suggestion-text">{suggestion.reason || 'AI 建议归档此对话内容'}</div>
      {suggestion.suggested_title && (
        <div className="archive-suggestion-title">📄 {suggestion.suggested_title}</div>
      )}
      {suggestion.suggested_parent && (
        <div className="archive-suggestion-parent">📂 {suggestion.suggested_parent.label}</div>
      )}
      {suggestion.action_preview && (
        <div className="archive-suggestion-preview">{suggestion.action_preview}</div>
      )}
      <div className="archive-suggestion-actions">
        <Button size="sm" onClick={() => on_accept(suggestion)}>归档</Button>
        <Button size="sm" variant="secondary" onClick={on_dismiss}>跳过</Button>
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Write ArchiveSuggestion test**

Create `ring-frontend/src/components/chat/ArchiveSuggestion.test.tsx`:

```tsx
import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ArchiveSuggestion } from './ArchiveSuggestion'

describe('ArchiveSuggestion', () => {
  it('renders reason and suggested title', () => {
    render(
      <ArchiveSuggestion
        data={{ reason: '有价值的信息', suggested_title: '会议纪要' }}
        on_accept={vi.fn()}
        on_dismiss={vi.fn()}
      />,
    )
    expect(screen.getByText('有价值的信息')).toBeInTheDocument()
    expect(screen.getByText('📄 会议纪要')).toBeInTheDocument()
  })

  it('renders parent and action preview', () => {
    render(
      <ArchiveSuggestion
        data={{
          reason: 'test',
          suggested_parent: { id: 'n1', label: '产品' },
          action_preview: '将创建新节点',
        }}
        on_accept={vi.fn()}
        on_dismiss={vi.fn()}
      />,
    )
    expect(screen.getByText('📂 产品')).toBeInTheDocument()
    expect(screen.getByText('将创建新节点')).toBeInTheDocument()
  })

  it('calls on_accept with suggestion data when 归档 clicked', () => {
    const on_accept = vi.fn()
    const data = { reason: 'test', suggested_title: '记录', suggested_parent: { id: 'n1', label: '根' } }
    render(<ArchiveSuggestion data={data} on_accept={on_accept} on_dismiss={vi.fn()} />)
    fireEvent.click(screen.getByText('归档'))
    expect(on_accept).toHaveBeenCalledWith(data)
  })

  it('calls on_dismiss when 跳过 clicked', () => {
    const on_dismiss = vi.fn()
    render(<ArchiveSuggestion data={{}} on_accept={vi.fn()} on_dismiss={on_dismiss} />)
    fireEvent.click(screen.getByText('跳过'))
    expect(on_dismiss).toHaveBeenCalled()
  })
})
```

- [ ] **Step 4: Run tests**

Run: `cd ring-frontend && npm test -- --run`
Expected: All tests pass (including new ArchiveSuggestion tests)

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/components/chat/ArchiveSuggestion.tsx ring-frontend/src/components/chat/ArchiveSuggestion.css ring-frontend/src/components/chat/ArchiveSuggestion.test.tsx
git commit -m "feat: enhance ArchiveSuggestion with title, parent, action preview"
```

---

### Task 6: Wire archive flow into ChatView

**Files:**
- Modify: `ring-frontend/src/pages/RingSpace/ChatView.tsx`
- Modify: `ring-frontend/src/pages/RingSpace/ChatView.css`

- [ ] **Step 1: Add export button CSS**

In `ring-frontend/src/pages/RingSpace/ChatView.css`, replace the `.chat-bottom` rule (lines 27-30) with nothing (delete it), then add the following styles at the end of the file:

```css
.chat-input-area {
  display: flex;
  align-items: flex-end;
  gap: var(--space-2);
  padding: var(--space-3);
  border-top: 1px solid var(--color-border);
}

.chat-export-btn {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg-primary);
  cursor: pointer;
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  flex-shrink: 0;
  transition: border-color 150ms ease, color 150ms ease;
}

.chat-export-btn:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}
```

- [ ] **Step 2: Update ChatView with export button and ArchiveConfirmDialog**

Replace the full content of `ring-frontend/src/pages/RingSpace/ChatView.tsx` with:

```tsx
import { useEffect, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { useChatStore } from '../../stores/chatStore'
import { useGraphStore } from '../../stores/graphStore'
import { useModeStore } from '../../stores/modeStore'
import * as api from '../../api/client'
import { ChatBubble } from '../../components/chat/ChatBubble'
import { ChatInput } from '../../components/chat/ChatInput'
import { ToolCallBubble } from '../../components/chat/ToolCallBubble'
import { ToolResultBubble } from '../../components/chat/ToolResultBubble'
import { ArchiveSuggestion, type ArchiveSuggestionData } from '../../components/chat/ArchiveSuggestion'
import { ArchiveConfirmDialog } from '../../components/archive/ArchiveConfirmDialog'
import { useTools } from '../../components/layout/RingSpaceLayout'
import './ChatView.css'

export function ChatView() {
  const { ringId } = useParams<{ ringId: string }>()
  const {
    messages,
    tool_events,
    is_streaming,
    error,
    current_conversation_id,
    create_conversation,
    load_history,
    send_message,
    reset,
    archive_pending,
    trigger_archive,
    dismiss_suggestion,
    clear_archive_pending,
  } = useChatStore()
  const { graphs, current_graph_id, nodes, load_graphs } = useGraphStore()
  const mode = useModeStore((s) => s.mode)
  const bottom_ref = useRef<HTMLDivElement>(null)
  const { active_tool_names } = useTools()

  useEffect(() => {
    if (!ringId) return
    if (current_conversation_id) return

    const init = async () => {
      try {
        const convs = await api.list_conversations(ringId)
        if (convs.length > 0) {
          const last = convs[convs.length - 1]
          await load_history(ringId, last.id)
        } else {
          reset()
          const conv_id = await create_conversation(ringId, 'New Conversation')
          await load_history(ringId, conv_id)
        }
      } catch {
        reset()
        const conv_id = await create_conversation(ringId, 'New Conversation')
        await load_history(ringId, conv_id)
      }
    }

    init()
  }, [ringId])

  useEffect(() => {
    if (!ringId || graphs.length > 0) return
    load_graphs(ringId).then(() => {
      const g = useGraphStore.getState().graphs
      if (g.length > 0) useGraphStore.getState().select_graph(ringId, g[0])
    })
  }, [ringId])

  useEffect(() => {
    bottom_ref.current?.scrollIntoView?.({ behavior: 'smooth' })
  }, [messages, tool_events])

  const handle_send = (content: string) => {
    if (!ringId || !current_conversation_id) return
    send_message(ringId, content, active_tool_names.length > 0 ? active_tool_names : undefined)
  }

  const handle_export = () => {
    if (!ringId) return
    const graph_id = current_graph_id || graphs[0]
    if (!graph_id) return
    trigger_archive(ringId, graph_id)
  }

  const handle_suggestion_accept = async (suggestion: ArchiveSuggestionData) => {
    if (!ringId || !current_conversation_id) return
    const graph_id = current_graph_id || graphs[0]
    if (!graph_id) return

    const unarchived = messages.filter((m) => !m.archived).slice(-5)
    const msg_ids = unarchived.length > 0 ? unarchived.map((m) => m.id) : []
    const last_user_msg = [...messages].reverse().find((m) => m.role === 'user')
    const label = suggestion.suggested_title || (last_user_msg?.content || 'Archive').slice(0, 30)

    try {
      const res = await api.archive_content(ringId, {
        message_ids: msg_ids,
        conversation_id: current_conversation_id,
        graph_id,
        label,
        target_node_id: suggestion.target_node_id,
      })
      useChatStore.setState({
        archive_pending: {
          archive_id: res.archive_id,
          suggested_title: suggestion.suggested_title,
          suggested_parent: suggestion.suggested_parent,
          message_ids: msg_ids,
          conversation_id: current_conversation_id,
          graph_id,
          label,
        },
      })
    } catch (e) {
      useChatStore.setState({ error: (e as Error).message })
    }
  }

  const handle_archive_confirm = async (target_node_id: string | undefined) => {
    if (!ringId || !archive_pending) return
    if (archive_pending.archive_id) {
      await api.confirm_archive(ringId, archive_pending.archive_id)
    } else {
      await api.archive_content(ringId, {
        message_ids: archive_pending.message_ids,
        conversation_id: archive_pending.conversation_id,
        graph_id: archive_pending.graph_id,
        label: archive_pending.label,
        target_node_id,
      })
    }
    clear_archive_pending()
  }

  const show_export = mode === 'manual_archive'

  return (
    <div className="chat-view">
      <div className="chat-header">Chat</div>
      <div className="chat-messages">
        {messages.map((msg) => (
          <ChatBubble key={msg.id} role={msg.role} content={msg.content} />
        ))}
        {tool_events.map((evt) => {
          if (evt.type === 'tool_call') {
            const done = tool_events.some(
              (r) => r.type === 'tool_result' && r.tool_call_id === evt.tool_call_id,
            )
            return (
              <ToolCallBubble
                key={evt.id}
                tool_name={evt.tool_name ?? 'unknown'}
                input={evt.input}
                done={done}
              />
            )
          }
          if (evt.type === 'tool_result') {
            return (
              <ToolResultBubble
                key={evt.id}
                tool_name={evt.tool_name ?? 'unknown'}
                output={evt.output}
                success={evt.success}
              />
            )
          }
          if (evt.type === 'archive_suggestion') {
            return (
              <ArchiveSuggestion
                key={evt.id}
                data={evt.data}
                on_accept={handle_suggestion_accept}
                on_dismiss={() => dismiss_suggestion(evt.id)}
              />
            )
          }
          return null
        })}
        {is_streaming && <div className="chat-typing">AI is typing...</div>}
        <div ref={bottom_ref} />
      </div>
      {error && <p className="chat-error" role="alert">{error}</p>}
      <div className="chat-input-area">
        {show_export && (
          <button className="chat-export-btn" onClick={handle_export} disabled={is_streaming} title="归档">
            📥
          </button>
        )}
        <ChatInput on_send={handle_send} disabled={is_streaming} />
      </div>
      <ArchiveConfirmDialog
        open={archive_pending !== null}
        on_close={clear_archive_pending}
        suggested_title={archive_pending?.suggested_title}
        suggested_parent={archive_pending?.suggested_parent}
        nodes={nodes}
        on_confirm={handle_archive_confirm}
      />
    </div>
  )
}
```

- [ ] **Step 3: Update ChatView.css**

Replace the `.chat-bottom` rule in `ring-frontend/src/pages/RingSpace/ChatView.css` with the new `.chat-input-area` and `.chat-export-btn` styles. Remove the `.chat-bottom` rule (lines 27-30) and add the following at the end of the file:

```css
.chat-input-area {
  display: flex;
  align-items: flex-end;
  gap: var(--space-2);
  padding: var(--space-3);
  border-top: 1px solid var(--color-border);
}

.chat-export-btn {
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background: var(--color-bg-primary);
  cursor: pointer;
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  flex-shrink: 0;
  transition: border-color 150ms ease, color 150ms ease;
}

.chat-export-btn:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}
```

- [ ] **Step 4: Run tests**

Run: `cd ring-frontend && npm test -- --run`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/pages/RingSpace/ChatView.tsx ring-frontend/src/pages/RingSpace/ChatView.css
git commit -m "feat: add export button and archive confirm flow to ChatView"
```

---

### Task 7: Add ArchiveQueueBar to PrList

**Files:**
- Modify: `ring-frontend/src/pages/RingSpace/PrList.tsx`

- [ ] **Step 1: Add ArchiveQueueBar to PrList**

In `ring-frontend/src/pages/RingSpace/PrList.tsx`, add the import and component:

After the existing imports, add:

```typescript
import { ArchiveQueueBar } from '../../components/archive/ArchiveQueueBar'
```

In the JSX, add `<ArchiveQueueBar />` after the `<Tabs>` component and before the error/loading section:

```tsx
      <Tabs tabs={STATE_TABS} active_key={state_filter} on_change={set_state_filter} />

      <ArchiveQueueBar ring_id={ringId!} />

      {error && <p className="setup-error" role="alert">{error}</p>}
```

- [ ] **Step 2: Run tests**

Run: `cd ring-frontend && npm test -- --run`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add ring-frontend/src/pages/RingSpace/PrList.tsx
git commit -m "feat: add ArchiveQueueBar to PrList page"
```

---

### Task 8: Final verification

- [ ] **Step 1: Run full test suite**

Run: `cd ring-frontend && npm test -- --run`
Expected: All tests pass

- [ ] **Step 2: Run TypeScript check**

Run: `cd ring-frontend && npx tsc --noEmit`
Expected: No errors

- [ ] **Step 3: Run ESLint**

Run: `cd ring-frontend && npx eslint src/`
Expected: No new errors (pre-existing warnings are acceptable)
