# Self Metrics Design

> Date: 2026-04-25
> Status: Approved
> Priority: Medium

## Goal

Implement the two missing Self metrics files (`dwell_time.json`, `tool_usage.json`), wire existing stub writers into actual routes, and feed metrics into Self AI's system prompt for proactive suggestions. Also display expanded metrics in the frontend.

## Storage

File-based, consistent with existing `~/.ring/self/metrics/` convention. No new database migrations.

## Metrics Data Shapes

### `dwell_time.json` (new writer)

```json
{
  "views": {
    "self_panel": 1800,
    "ring_chat": 3600,
    "graph": 480,
    "session": 1200,
    "archive": 300
  },
  "daily": {
    "2026-04-25": {
      "self_panel": 300,
      "ring_chat": 600,
      "graph": 120
    }
  },
  "last_heartbeat": "2026-04-25T14:30:00Z"
}
```

All durations in seconds. `views` = all-time totals. `daily` = per-day breakdown.

### `tool_usage.json` (new writer)

```json
{
  "tools": {
    "search": 5,
    "graph_edit": 12,
    "archive": 3,
    "upload": 2,
    "session_create": 8,
    "session_summarize": 4,
    "blueprint": 6,
    "export": 1,
    "memory_extract": 23
  },
  "last_used": {
    "search": "2026-04-25T14:30:00Z"
  }
}
```

### Existing metrics (wire stubs, no shape change)

- `chat_patterns.json` — already working
- `archive_patterns.json` — writer exists, wire into route
- `session_stats.json` — writer exists, wire into route
- `ring_activity.json` — writer exists, wire into route

## Backend Changes

### 1. New Writers in `self_data.rs`

**`record_dwell_heartbeat(self_dir, view, duration_s)`**

- Read `dwell_time.json` (or init empty)
- Add `duration_s` to `views[view]` and `daily[today][view]`
- Update `last_heartbeat`
- Write back to file

In-memory batching: a `tokio::sync::Mutex<HashMap<(view, ring_id), seconds>>` accumulates heartbeats. A background task flushes to file every 60 seconds.

**`record_tool_usage(self_dir, tool_name)`**

- Read `tool_usage.json` (or init empty)
- Increment `tools[tool_name]`
- Set `last_used[tool_name]` to now
- Write back to file

### 2. New Route

```
POST /api/self/metrics/heartbeat
Body: { view: string, ring_id?: string }
Response: 204 No Content
```

Views: `self_panel`, `ring_chat`, `graph`, `session`, `archive`.

The backend records 30 seconds (the heartbeat interval) against the given view.

### 3. Tool Usage Instrumentation Points

| Tool Name | Route/Service | Call Point |
|-----------|--------------|------------|
| `search` | `routes/chat.rs` (super_chat) | After FTS5 search returns results |
| `graph_edit` | `routes/graph.rs` | On add/update/delete node or edge |
| `archive` | `routes/chat.rs` or `routes/export.rs` | On archive trigger |
| `upload` | `routes/upload.rs` | On successful upload |
| `session_create` | `routes/session.rs` | On session creation |
| `session_summarize` | `routes/session.rs` | On summarize trigger |
| `blueprint` | `routes/graph.rs` | On blueprint confirm |
| `export` | `routes/export.rs` | On any export |
| `memory_extract` | `services/self_memory.rs` | On extraction completion |

### 4. Wire Existing Stubs

- `record_session_created()` → `routes/session.rs` create handler
- `record_ring_joined()` → `routes/rings.rs` join handler
- `record_archive_operation()` → archive trigger route

### 5. Metrics in Self AI Prompt

Add `metrics_context()` in `prompts.rs` `self_chat` module. Reads metrics files and formats a summary:

```
## 用户行为概览
- 总消息数: 142（其中 Self 对话: 23）
- 活跃 Ring: 3 个
- 归档次数: 7
- 常用功能: graph_edit(31), search(12), upload(5)
- 今日活跃: ring_chat 47分钟, self_panel 12分钟
```

Injected into `build_system_prompt()` for Self chat mode.

### 6. App State for Metrics

Add `MetricsState` to Axum app state:

```rust
pub struct MetricsState {
    pub dwell_buffer: tokio::sync::Mutex<HashMap<String, u64>>,
    pub self_dir: PathBuf,
}
```

Shared via `AppState`. Background `tokio::spawn` task flushes buffer every 60s.

## Frontend Changes

### 1. Heartbeat Service (`ui/src/services/metrics.ts`)

- `startHeartbeat()` — sends POST every 30s with current view + active ring_id
- Derives current view from: app-store context + panel states
- `stopHeartbeat()` on `beforeunload`
- Uses `setInterval` + cleanup on unmount

### 2. Metrics Display in `SelfMemory.tsx`

Expand the counters section to show three groups:

```
📊 活动统计
消息数: 142 | Self 对话: 23 | Ring: 3 | Session: 8 | 归档: 7

🔧 工具使用
图谱编辑: 31 | 搜索: 12 | 上传: 5 | 导出: 3

⏱ 今日活跃
聊天: 47分钟 | Self: 12分钟 | 图谱: 8分钟
```

Minimal text counters, no charts. Clean formatting.

### 3. Route Registration

Add `POST /api/self/metrics/heartbeat` to `routes/mod.rs`.

## Testing

- Unit test: `record_tool_usage()` increments correctly
- Unit test: `record_dwell_heartbeat()` accumulates and rolls to daily
- Integration test: `POST /api/self/metrics/heartbeat` returns 204
- Integration test: `GET /api/self/metrics` includes tool_usage and dwell_time
