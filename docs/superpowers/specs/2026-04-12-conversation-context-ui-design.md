# Conversation Context UI Design (F7)

> **Date**: 2026-04-12
> **Scope**: F7 — Conversation context management UI (PRD 2.9)
> **Depends on**: Backend APIs (`GET token-stats`, `POST compact`, `POST conversations` with context_mode)

## Goal

Add token usage visibility and compact controls to ChatView. Users need to see how much token budget they've consumed and be able to trigger compact manually or automatically.

## Architecture

All UI is inline in the ChatView header bar. Token stats load on conversation init and update after each SSE `done` event. Compact is a single API call that returns before/after stats.

## Backend APIs

### GET /api/v1/rings/{ringId}/conversations/{convId}/token-stats

Returns:
```json
{
  "conversation_id": "conv-uuid",
  "context_mode": "storage",
  "token_count": 95000,
  "token_limit": 100000,
  "auto_compact": false,
  "usage_percent": 95,
  "warning": "对话上下文已使用 95%，建议 compact"
}
```

### POST /api/v1/rings/{ringId}/conversations/{convId}/compact

Returns:
```json
{
  "conversation_id": "conv-uuid",
  "token_count_before": 98000,
  "token_count_after": 5000,
  "messages_compacted": 45,
  "summary_length": 800
}
```

### SSE done event (existing)

```json
{ "type": "done", "token_usage": { "prompt_tokens": 100, "completion_tokens": 50, "total_tokens": 150 } }
```

## Module 1: Token Usage Bar

**File**: New component `ring-frontend/src/components/chat/TokenUsageBar.tsx` + CSS

A compact inline bar in the chat-header area:
- Shows a thin progress bar (token_count / token_limit percentage)
- Color coding: < 80% green, 80-95% amber, > 95% red
- Text label: "12.5k / 100k tokens" (abbreviated)
- Only visible in `storage` context_mode (hidden for ephemeral)

**Props**:
```typescript
interface TokenUsageBarProps {
  token_count: number
  token_limit: number
  context_mode: string
}
```

## Module 2: Compact Trigger

Integrated into the TokenUsageBar area:
- A "压缩" button appears when usage > 80%
- Clicking calls `POST /compact`
- While compacting: show "正在压缩对话上下文..." text in place of the button
- After compact: refresh token stats from `GET /token-stats`
- Compact button is hidden for ephemeral conversations

**New API functions** in `ring-frontend/src/api/client.ts`:
```typescript
export async function get_token_stats(ring_id: string, conv_id: string): Promise<TokenStatsResponse>
export async function compact_conversation(ring_id: string, conv_id: string): Promise<CompactResponse>
```

**New types** in `ring-frontend/src/types/index.ts`:
```typescript
interface TokenStatsResponse {
  conversation_id: string
  context_mode: string
  token_count: number
  token_limit: number
  auto_compact: boolean
  usage_percent: number
  warning: string | null
}

interface CompactResponse {
  conversation_id: string
  token_count_before: number
  token_count_after: number
  messages_compacted: number
  summary_length: number
}
```

## Module 3: auto_compact Toggle + Context Mode

In the chat-header, next to the TokenUsageBar:
- A small toggle switch for auto_compact (only in storage mode)
- Label: "自动压缩"
- Toggling updates the conversation via a PATCH/PUT API

For context_mode selection:
- When creating a new conversation, add a dropdown or toggle for storage/ephemeral
- Update `create_conversation` API call to pass `context_mode`

**New API function**:
```typescript
export async function update_conversation(ring_id: string, conv_id: string, updates: { auto_compact?: boolean }): Promise<Conversation>
```

## chatStore Additions

Extend `ChatState` with:
```typescript
token_count: number
token_limit: number
context_mode: string
auto_compact: boolean
compacting: boolean

load_token_stats: (ring_id: string) => Promise<void>
trigger_compact: (ring_id: string) => Promise<void>
toggle_auto_compact: (ring_id: string) => Promise<void>
```

Update `send_message` to accumulate `token_count` from the SSE `done` event's `token_usage.total_tokens`.

Update `create_conversation` to accept optional `context_mode` parameter.

## ChatView Changes

Replace the current `<div className="chat-header">Chat</div>` with an enhanced header row:
```tsx
<div className="chat-header">
  <span>Chat</span>
  <div className="chat-header-controls">
    {context_mode === 'storage' && (
      <>
        <TokenUsageBar token_count={token_count} token_limit={token_limit} context_mode={context_mode} />
        {usage_percent > 80 && !compacting && (
          <button onClick={handle_compact}>压缩</button>
        )}
        {compacting && <span>正在压缩对话上下文...</span>}
        <label>
          <input type="checkbox" checked={auto_compact} onChange={handle_toggle_auto} />
          自动压缩
        </label>
      </>
    )}
    {context_mode === 'ephemeral' && <span className="chat-ephemeral-badge">临时会话</span>}
  </div>
</div>
```

## Testing

- Unit tests for TokenUsageBar (renders percentage, color changes at thresholds)
- Unit tests for chatStore new actions (load_token_stats, trigger_compact, toggle_auto_compact)
- Unit tests for new API functions

## Mock Data Updates

In `ring-frontend/src/mocks/handlers.ts`:
- `GET /rings/:ringId/conversations/:convId/token-stats`: return mock stats with 35% usage
- `POST /rings/:ringId/conversations/:convId/compact`: return mock compact response
- `PUT /rings/:ringId/conversations/:convId`: accept auto_compact update
