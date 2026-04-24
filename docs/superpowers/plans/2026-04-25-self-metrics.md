# Self Metrics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement dwell_time and tool_usage metrics, wire existing metric stubs into routes, inject metrics into Self AI prompt, add frontend heartbeat and expanded metrics display.

**Architecture:** File-based metrics in `~/.ring/self/metrics/`, in-memory dwell buffer with periodic flush, `record_tool_usage()` calls instrumented across routes, `metrics_context()` summary injected into Self system prompt.

**Tech Stack:** Rust + Axum (backend), React + TypeScript + Zustand (frontend)

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `server/src/services/self_data.rs` | Add `record_dwell_heartbeat()`, `record_tool_usage()` |
| Modify | `server/src/state.rs` | Add `DwellBuffer` to `AppState` |
| Modify | `server/src/routes/self_data.rs` | Add `heartbeat` handler |
| Modify | `server/src/routes/mod.rs` | Register heartbeat route + add `metrics` module |
| Modify | `server/src/prompts.rs` | Add `metrics_context()` to `self_chat` module |
| Modify | `server/src/services/chat.rs` | Call `metrics_context()` in `build_system_prompt()` |
| Modify | `server/src/routes/chat.rs` | Wire `record_archive_operation()` on archive intent |
| Modify | `server/src/routes/archive.rs` | Wire `record_archive_operation()` on quick archive |
| Modify | `server/src/routes/session.rs` | Wire `record_session_created()` (already done), add `record_tool_usage` for summarize |
| Modify | `server/src/routes/graph.rs` | Add `record_tool_usage("graph_edit")` on CRUD |
| Modify | `server/src/routes/upload.rs` | Add `record_tool_usage("upload")` |
| Modify | `server/src/routes/export.rs` | Add `record_tool_usage("export")` |
| Modify | `server/src/routes/blueprint.rs` | Add `record_tool_usage("blueprint")` |
| Modify | `server/src/routes/super_chat.rs` | Add `record_tool_usage("search")` after FTS5 |
| Modify | `server/src/services/self_memory.rs` | Add `record_tool_usage("memory_extract")` on extraction |
| Create | `ui/src/services/metrics.ts` | Heartbeat sender |
| Modify | `ui/src/components/self/SelfMemory.tsx` | Expanded metrics display |
| Modify | `server/tests/integration.rs` | Integration tests for heartbeat + tool_usage |
| Modify | `server/src/main.rs` | Start dwell flush background task |

---

### Task 1: Add `record_tool_usage()` to `self_data.rs`

**Files:**
- Modify: `server/src/services/self_data.rs:177` (after `record_ring_joined`)

- [ ] **Step 1: Write the function**

Add after `record_ring_joined` (line 177) in `server/src/services/self_data.rs`:

```rust
pub fn record_tool_usage(self_dir: &Path, tool_name: &str) -> Result<()> {
    let mut usage = read_metric_file(self_dir, "tool_usage");
    let tools = usage
        .as_object_mut()
        .get_or_insert_with(|| serde_json::Map::new());
    let count = tools
        .get("tools")
        .and_then(|t| t.get(tool_name))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + 1;
    if tools.get("tools").is_none() {
        tools.insert("tools".into(), serde_json::json!({}));
    }
    tools["tools"][tool_name] = serde_json::json!(count);
    if tools.get("last_used").is_none() {
        tools.insert("last_used".into(), serde_json::json!({}));
    }
    let now = chrono::Local::now().to_rfc3339();
    tools["last_used"][tool_name] = serde_json::json!(now);
    write_metric_file(self_dir, "tool_usage", &serde_json::Value::Object(tools.clone()))
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`
Expected: no errors in `self_data.rs`

- [ ] **Step 3: Commit**

```bash
git add server/src/services/self_data.rs
git commit -m "feat(metrics): add record_tool_usage writer"
```

---

### Task 2: Add `DwellBuffer` to `AppState` + `record_dwell_heartbeat()`

**Files:**
- Modify: `server/src/state.rs`
- Modify: `server/src/services/self_data.rs` (add `record_dwell_heartbeat`)

- [ ] **Step 1: Add `DwellBuffer` type alias to `state.rs`**

Add at top of `server/src/state.rs`, after the use statements:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type DwellBuffer = Arc<Mutex<HashMap<String, u64>>>;
```

Add `dwell_buffer: DwellBuffer` field to `AppState` struct (after `encryption`):

```rust
pub dwell_buffer: DwellBuffer,
```

In `AppState::new()`, initialize it:

```rust
dwell_buffer: Arc::new(Mutex::new(HashMap::new())),
```

- [ ] **Step 2: Add `record_dwell_heartbeat()` to `self_data.rs`**

Add after `record_tool_usage` in `server/src/services/self_data.rs`:

```rust
pub fn record_dwell_heartbeat(self_dir: &Path, view: &str, duration_s: u64) -> Result<()> {
    let mut dwell = read_metric_file(self_dir, "dwell_time");
    let obj = dwell
        .as_object_mut()
        .get_or_insert_with(|| serde_json::Map::new());

    let views_total = obj
        .get("views")
        .and_then(|v| v.get(view))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + duration_s as i64;
    if obj.get("views").is_none() {
        obj.insert("views".into(), serde_json::json!({}));
    }
    obj["views"][view] = serde_json::json!(views_total);

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if obj.get("daily").is_none() {
        obj.insert("daily".into(), serde_json::json!({}));
    }
    let daily_total = obj["daily"]
        .get(&today)
        .and_then(|d| d.get(view))
        .and_then(|v| v.as_i64())
        .unwrap_or(0)
        + duration_s as i64;
    obj["daily"][&today][view] = serde_json::json!(daily_total);

    obj.insert(
        "last_heartbeat".into(),
        serde_json::json!(chrono::Local::now().to_rfc3339()),
    );

    write_metric_file(self_dir, "dwell_time", &serde_json::Value::Object(obj.clone()))
}
```

Add `flush_dwell_buffer()` function:

```rust
pub fn flush_dwell_buffer(
    self_dir: &Path,
    buffer: &std::collections::HashMap<String, u64>,
) -> Result<()> {
    for (view, seconds) in buffer {
        record_dwell_heartbeat(self_dir, view, *seconds)?;
    }
    Ok(())
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 4: Commit**

```bash
git add server/src/state.rs server/src/services/self_data.rs
git commit -m "feat(metrics): add DwellBuffer to AppState and record_dwell_heartbeat"
```

---

### Task 3: Add heartbeat route + background flush task

**Files:**
- Modify: `server/src/routes/self_data.rs`
- Modify: `server/src/routes/mod.rs`
- Modify: `server/src/main.rs`

- [ ] **Step 1: Add heartbeat handler to `self_data.rs` routes**

Add at end of `server/src/routes/self_data.rs`, before the closing (after `delete_memory`):

```rust
#[derive(Deserialize)]
pub struct HeartbeatRequest {
    pub view: String,
    pub ring_id: Option<String>,
}

pub async fn heartbeat(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<HeartbeatRequest>,
) -> Result<StatusCode> {
    let valid_views = ["self_panel", "ring_chat", "graph", "session", "archive"];
    if !valid_views.contains(&body.view.as_str()) {
        return Ok(StatusCode::BAD_REQUEST);
    }
    let mut buf = state.dwell_buffer.lock().await;
    let entry = buf.entry(body.view).or_insert(0);
    *entry += 30;
    Ok(StatusCode::NO_CONTENT)
}
```

- [ ] **Step 2: Register heartbeat route in `mod.rs`**

In `server/src/routes/mod.rs`, add after line 95 (`.route("/self/metrics", get(self_data::get_metrics))`):

```rust
        .route("/self/metrics/heartbeat", post(self_data::heartbeat))
```

- [ ] **Step 3: Add background flush task in `main.rs`**

Read `server/src/main.rs` to find where the server is started. Add a spawned task before the server start that flushes the dwell buffer every 60 seconds:

```rust
{
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let buf = {
                let mut guard = state_clone.dwell_buffer.lock().await;
                std::mem::take(&mut *guard)
            };
            if !buf.is_empty() {
                let self_dir = crate::services::self_data::get_self_dir("");
                let _ = crate::services::self_data::flush_dwell_buffer(&self_dir, &buf);
            }
        }
    });
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 5: Commit**

```bash
git add server/src/routes/self_data.rs server/src/routes/mod.rs server/src/main.rs
git commit -m "feat(metrics): add heartbeat route and background dwell flush"
```

---

### Task 4: Add `metrics_context()` to prompts + inject into Self prompt

**Files:**
- Modify: `server/src/prompts.rs` (self_chat module)
- Modify: `server/src/services/chat.rs` (build_system_prompt)

- [ ] **Step 1: Add `metrics_context()` to `self_chat` module in `prompts.rs`**

In `server/src/prompts.rs`, add inside the `pub mod self_chat` block (after the `system` function, before the closing `}`):

```rust
    pub fn metrics_context(metrics: &serde_json::Value) -> String {
        let cp = metrics.get("chat_patterns");
        let total_msgs = cp
            .and_then(|m| m.get("total_messages"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let self_msgs = cp
            .and_then(|m| m.get("self_messages"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let total_rings = metrics
            .get("ring_activity")
            .and_then(|m| m.get("total_rings"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let total_archives = metrics
            .get("archive_patterns")
            .and_then(|m| m.get("total_archives"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let total_sessions = metrics
            .get("session_stats")
            .and_then(|m| m.get("total_sessions"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let tools = metrics
            .get("tool_usage")
            .and_then(|m| m.get("tools"))
            .and_then(|t| t.as_object());
        let tools_summary = if let Some(tools) = tools {
            let mut entries: Vec<(String, i64)> = tools
                .iter()
                .filter_map(|(k, v)| v.as_i64().map(|i| (k.clone(), i)))
                .collect();
            entries.sort_by(|a, b| b.1.cmp(&a.1));
            entries
                .iter()
                .take(5)
                .map(|(k, v)| format!("{k}({v})"))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        };

        if total_msgs == 0 && total_rings == 0 {
            return String::new();
        }

        let mut ctx = format!(
            "## 用户行为概览\n- 总消息数: {total_msgs}（其中 Self 对话: {self_msgs}）\n- 活跃 Ring: {total_rings} 个\n- Session: {total_sessions} 次\n- 归档: {total_archives} 次"
        );
        if !tools_summary.is_empty() {
            ctx.push_str(&format!("\n- 常用功能: {tools_summary}"));
        }
        ctx
    }
```

- [ ] **Step 2: Call `metrics_context()` in `build_system_prompt()` in `chat.rs`**

In `server/src/services/chat.rs`, in the `build_system_prompt` function, after the memory context injection block (after line 217 `}`), add metrics injection:

Change lines 212-219 from:

```rust
    if ring_name.is_none() {
        let self_dir = crate::services::self_data::get_self_dir("");
        let memory_ctx = crate::services::self_memory::build_memory_context(&self_dir);
        if !memory_ctx.is_empty() {
            return format!("{prompt}\n\n{memory_ctx}");
        }
    }
    prompt
```

to:

```rust
    if ring_name.is_none() {
        let self_dir = crate::services::self_data::get_self_dir("");
        let mut extra = String::new();
        let memory_ctx = crate::services::self_memory::build_memory_context(&self_dir);
        if !memory_ctx.is_empty() {
            extra.push_str(&memory_ctx);
        }
        let metrics = crate::services::self_data::read_metrics(&self_dir);
        let metrics_ctx = crate::prompts::self_chat::metrics_context(&metrics);
        if !metrics_ctx.is_empty() {
            if !extra.is_empty() {
                extra.push_str("\n\n");
            }
            extra.push_str(&metrics_ctx);
        }
        if !extra.is_empty() {
            return format!("{prompt}\n\n{extra}");
        }
    }
    prompt
```

- [ ] **Step 3: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 4: Commit**

```bash
git add server/src/prompts.rs server/src/services/chat.rs
git commit -m "feat(metrics): inject metrics context into Self system prompt"
```

---

### Task 5: Wire `record_tool_usage()` across routes

**Files:**
- Modify: `server/src/routes/chat.rs` (archive intent detection)
- Modify: `server/src/routes/graph.rs` (node/edge CRUD)
- Modify: `server/src/routes/upload.rs` (file upload)
- Modify: `server/src/routes/export.rs` (export)
- Modify: `server/src/routes/blueprint.rs` (blueprint confirm)
- Modify: `server/src/routes/super_chat.rs` (FTS5 search)
- Modify: `server/src/routes/session.rs` (summarize)
- Modify: `server/src/services/self_memory.rs` (memory extraction)

- [ ] **Step 1: Add `record_tool_usage("search")` in `super_chat.rs`**

Read `server/src/routes/super_chat.rs` to find where FTS5 search results are retrieved (look for `search::search` or `cross_ring_context`). After the search call succeeds, add:

```rust
let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
let _ = crate::services::self_data::record_tool_usage(&self_dir, "search");
```

- [ ] **Step 2: Add `record_tool_usage("graph_edit")` in `graph.rs`**

Read `server/src/routes/graph.rs`. In each of these handlers, add the recording after the successful operation:
- `create_node_handler`
- `update_node`
- `delete_node`
- `create_edge_handler`
- `delete_edge`

In each, after the service call succeeds, add:

```rust
let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
let _ = crate::services::self_data::record_tool_usage(&self_dir, "graph_edit");
```

- [ ] **Step 3: Add `record_tool_usage("upload")` in `upload.rs`**

Read `server/src/routes/upload.rs`. In each upload handler (`upload_ring_file`, `upload_super_file`, `upload_session_file`), after successful upload processing, add:

```rust
let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
let _ = crate::services::self_data::record_tool_usage(&self_dir, "upload");
```

- [ ] **Step 4: Add `record_tool_usage("export")` in `export.rs`**

Read `server/src/routes/export.rs`. In each export handler, after successful data generation, add:

```rust
let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
let _ = crate::services::self_data::record_tool_usage(&self_dir, "export");
```

- [ ] **Step 5: Add `record_tool_usage("blueprint")` in `blueprint.rs`**

Read `server/src/routes/blueprint.rs`. In `confirm_blueprint_handler`, after successful confirmation, add:

```rust
let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
let _ = crate::services::self_data::record_tool_usage(&self_dir, "blueprint");
```

- [ ] **Step 6: Add `record_tool_usage("archive")` in `archive.rs`**

In `server/src/routes/archive.rs`, in `quick_archive_handler`, after the successful archive record insert (after line 162 `Ok(Json(...))`), add before the return:

```rust
let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
let _ = crate::services::self_data::record_tool_usage(&self_dir, "archive");
let _ = crate::services::self_data::record_archive_operation(&self_dir, &ring_id, &file_name);
```

Also add `record_tool_usage("archive")` in the `trigger_archive` handler where the archive intent fires from chat command. Read the handler to find the right spot.

- [ ] **Step 7: Add `record_tool_usage("session_summarize")` in `session.rs`**

Read `server/src/routes/session.rs`. In `summarize_session`, after successful summarize call, add:

```rust
let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
let _ = crate::services::self_data::record_tool_usage(&self_dir, "session_summarize");
```

- [ ] **Step 8: Add `record_tool_usage("memory_extract")` in `self_memory.rs`**

Read `server/src/services/self_memory.rs`. In `extract_memories()`, at the end of the function after successful extraction, add:

```rust
let self_dir = crate::services::self_data::get_self_dir(user_id);
let _ = crate::services::self_data::record_tool_usage(&self_dir, "memory_extract");
```

Note: `extract_memories` may take `user_id` as a parameter. Check the actual function signature.

- [ ] **Step 9: Verify everything compiles**

Run: `cd server && cargo check 2>&1 | tail -10`

- [ ] **Step 10: Commit**

```bash
git add server/src/routes/ server/src/services/self_memory.rs
git commit -m "feat(metrics): wire record_tool_usage across routes"
```

---

### Task 6: Wire `record_archive_operation()` into archive intent in `chat.rs`

**Files:**
- Modify: `server/src/routes/chat.rs`

- [ ] **Step 1: Add archive metric recording in `ring_chat` handler**

In `server/src/routes/chat.rs`, in the `ring_chat` handler (around line 90-105), in the archive intent detection block (`if chat::detect_archive_intent(&body.content)`), after the `quick_archive` call inside the stream, add:

After line 101 (`.await;`), add:

```rust
            let sd = crate::services::self_data::get_self_dir(&user_id_c);
            let _ = crate::services::self_data::record_tool_usage(&sd, "archive");
```

Note: `user_id_c` may not be in scope inside the stream. Check what variables are captured. If `user_id_c` isn't captured, add a clone before the stream.

- [ ] **Step 2: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 3: Commit**

```bash
git add server/src/routes/chat.rs
git commit -m "feat(metrics): record archive tool usage in chat archive intent"
```

---

### Task 7: Frontend heartbeat service

**Files:**
- Create: `ui/src/services/metrics.ts`

- [ ] **Step 1: Create the heartbeat service**

Create `ui/src/services/metrics.ts`:

```typescript
import { api } from './api'

const HEARTBEAT_INTERVAL = 30_000

let intervalId: ReturnType<typeof setInterval> | null = null

function getCurrentView(): string {
  const ctx = 'super'
  return ctx
}

export function startHeartbeat() {
  stopHeartbeat()
  intervalId = setInterval(async () => {
    const view = getCurrentView()
    try {
      await api.post('/self/metrics/heartbeat', { view })
    } catch {}
  }, HEARTBEAT_INTERVAL)
}

export function stopHeartbeat() {
  if (intervalId !== null) {
    clearInterval(intervalId)
    intervalId = null
  }
}
```

- [ ] **Step 2: Hook into `App.tsx` or main entry to start/stop heartbeat**

Read `ui/src/App.tsx` to find the main component. Add heartbeat start on mount and stop on unmount + beforeunload:

In the main App component, add:

```typescript
import { startHeartbeat, stopHeartbeat } from './services/metrics'

useEffect(() => {
  startHeartbeat()
  const handleUnload = () => stopHeartbeat()
  window.addEventListener('beforeunload', handleUnload)
  return () => {
    stopHeartbeat()
    window.removeEventListener('beforeunload', handleUnload)
  }
}, [])
```

- [ ] **Step 3: Implement `getCurrentView()` properly using stores**

The heartbeat needs to know which view is active. Update `metrics.ts` to read from Zustand stores:

```typescript
import { api } from './api'

const HEARTBEAT_INTERVAL = 30_000

let intervalId: ReturnType<typeof setInterval> | null = null

function getCurrentView(): string {
  try {
    const appStore = require('../stores/app-store').useAppStore.getState()
    const ringStore = require('../stores/ring-store').useRingStore.getState()
    const ctx = appStore.current_context
    if (ctx === 'self') return 'self_panel'
    if (ctx === 'session') return 'session'
    const activeRingId = ringStore.active_ring_id
    if (activeRingId) return 'ring_chat'
    return 'ring_chat'
  } catch {
    return 'ring_chat'
  }
}

export function startHeartbeat() {
  stopHeartbeat()
  intervalId = setInterval(async () => {
    const view = getCurrentView()
    try {
      await api.post('/self/metrics/heartbeat', { view })
    } catch {}
  }, HEARTBEAT_INTERVAL)
}

export function stopHeartbeat() {
  if (intervalId !== null) {
    clearInterval(intervalId)
    intervalId = null
  }
}
```

Note: Zustand stores expose `.getState()` for non-React usage. Use direct imports instead of `require`.

- [ ] **Step 4: Verify frontend builds**

Run: `cd ui && npm run build 2>&1 | tail -5`
Expected: build succeeds

- [ ] **Step 5: Commit**

```bash
git add ui/src/services/metrics.ts ui/src/App.tsx
git commit -m "feat(metrics): add frontend heartbeat service"
```

---

### Task 8: Expand frontend metrics display

**Files:**
- Modify: `ui/src/components/self/SelfMemory.tsx`

- [ ] **Step 1: Update Metrics interface and display**

In `ui/src/components/self/SelfMemory.tsx`, update the `Metrics` interface (lines 17-22) to include tool_usage and dwell_time:

```typescript
interface ToolUsage {
  tools?: Record<string, number>
}

interface DwellTime {
  views?: Record<string, number>
  daily?: Record<string, Record<string, number>>
}

interface Metrics {
  chat_patterns?: {
    total_messages?: number
    self_messages?: number
    total_chars?: number
  }
  session_stats?: { total_sessions?: number }
  archive_patterns?: { total_archives?: number }
  ring_activity?: { total_rings?: number }
  tool_usage?: ToolUsage
  dwell_time?: DwellTime
}
```

- [ ] **Step 2: Replace the metrics display section**

Replace lines 153-161 (the METRICS section) with:

```tsx
      <div style={{ marginTop: 12, fontSize: 11, fontWeight: 700, color: 'var(--text-dim)', letterSpacing: '0.1em', marginBottom: 8 }}>
        ACTIVITY
      </div>
      <div style={{ fontSize: 11, color: 'var(--text-secondary)', lineHeight: 1.8 }}>
        <div>消息: {metrics.chat_patterns?.total_messages ?? 0} (Self: {metrics.chat_patterns?.self_messages ?? 0})</div>
        <div>Ring: {metrics.ring_activity?.total_rings ?? 0} | Session: {metrics.session_stats?.total_sessions ?? 0} | 归档: {metrics.archive_patterns?.total_archives ?? 0}</div>
      </div>

      {metrics.tool_usage?.tools && Object.keys(metrics.tool_usage.tools).length > 0 && (
        <>
          <div style={{ marginTop: 10, fontSize: 11, fontWeight: 700, color: 'var(--text-dim)', letterSpacing: '0.1em', marginBottom: 8 }}>
            TOOLS
          </div>
          <div style={{ fontSize: 11, color: 'var(--text-secondary)', lineHeight: 1.8 }}>
            {Object.entries(metrics.tool_usage.tools)
              .sort(([, a], [, b]) => b - a)
              .slice(0, 6)
              .map(([name, count]) => (
                <div key={name}>{name}: {count}</div>
              ))}
          </div>
        </>
      )}

      {metrics.dwell_time?.daily && (() => {
        const today = new Date().toISOString().slice(0, 10)
        const todayData = metrics.dwell_time.daily[today]
        if (!todayData || Object.keys(todayData).length === 0) return null
        const fmtMin = (s: number) => s > 0 ? `${Math.round(s / 60)}min` : '0min'
        return (
          <>
            <div style={{ marginTop: 10, fontSize: 11, fontWeight: 700, color: 'var(--text-dim)', letterSpacing: '0.1em', marginBottom: 8 }}>
              TODAY
            </div>
            <div style={{ fontSize: 11, color: 'var(--text-secondary)', lineHeight: 1.8 }}>
              {todayData.self_panel && <div>Self: {fmtMin(todayData.self_panel)}</div>}
              {todayData.ring_chat && <div>聊天: {fmtMin(todayData.ring_chat)}</div>}
              {todayData.graph && <div>图谱: {fmtMin(todayData.graph)}</div>}
              {todayData.session && <div>Session: {fmtMin(todayData.session)}</div>}
              {todayData.archive && <div>归档: {fmtMin(todayData.archive)}</div>}
            </div>
          </>
        )
      })()}
```

- [ ] **Step 3: Verify frontend builds**

Run: `cd ui && npm run build 2>&1 | tail -5`

- [ ] **Step 4: Commit**

```bash
git add ui/src/components/self/SelfMemory.tsx
git commit -m "feat(metrics): expand SelfMemory metrics display with tools and dwell time"
```

---

### Task 9: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add test for heartbeat endpoint**

In `server/tests/integration.rs`, add:

```rust
#[sqlx::test(migrations = "./migrations")]
async fn test_metrics_heartbeat(pool: SqlitePool) {
    let (state, _) = setup_state(pool).await;
    let user = create_test_user(&state).await;

    let body = serde_json::json!({"view": "self_panel"});
    let req = TestRequest::post()
        .uri("/api/self/metrics/heartbeat")
        .json(&body)
        .auth(&user.token_id);
    let resp = req.send_with_state(state.clone()).await;
    assert_eq!(resp.status(), 204);

    let buf = state.dwell_buffer.lock().await;
    assert_eq!(buf.get("self_panel"), Some(&30u64));
}
```

Note: The exact test setup helpers may vary. Read the existing tests in `integration.rs` to match the pattern used (e.g., `TestRequest`, `setup_state`, `create_test_user`).

- [ ] **Step 2: Add test for tool_usage writer**

```rust
#[test]
fn test_record_tool_usage() {
    let dir = tempfile::tempdir().unwrap();
    let self_dir = dir.path();
    let _ = crate::services::self_data::record_tool_usage(self_dir, "search");
    let _ = crate::services::self_data::record_tool_usage(self_dir, "search");
    let _ = crate::services::self_data::record_tool_usage(self_dir, "upload");

    let metrics = crate::services::self_data::read_metrics(self_dir);
    let tools = metrics.get("tool_usage").unwrap().get("tools").unwrap();
    assert_eq!(tools.get("search").unwrap().as_i64(), Some(2));
    assert_eq!(tools.get("upload").unwrap().as_i64(), Some(1));
}
```

Note: This is a unit test, not integration. Add it as appropriate for the test structure. If `self_data` functions are not `pub`, adjust visibility.

- [ ] **Step 3: Add test for dwell_heartbeat writer**

```rust
#[test]
fn test_record_dwell_heartbeat() {
    let dir = tempfile::tempdir().unwrap();
    let self_dir = dir.path();
    crate::services::self_data::record_dwell_heartbeat(self_dir, "ring_chat", 30).unwrap();
    crate::services::self_data::record_dwell_heartbeat(self_dir, "ring_chat", 30).unwrap();

    let metrics = crate::services::self_data::read_metrics(self_dir);
    let dwell = metrics.get("dwell_time").unwrap();
    let views = dwell.get("views").unwrap();
    assert_eq!(views.get("ring_chat").unwrap().as_i64(), Some(60));
}
```

- [ ] **Step 4: Run all tests**

Run: `cd server && cargo test 2>&1 | tail -20`
Expected: all tests pass

- [ ] **Step 5: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test(metrics): add heartbeat, tool_usage, dwell_heartbeat tests"
```

---

### Task 10: Final verification + STATUS.md update

**Files:**
- Modify: `docs/STATUS.md`

- [ ] **Step 1: Run full test suite**

Run: `cd server && cargo test --test integration 2>&1 | tail -10`
Expected: all tests pass

Run: `cd ui && npm run build 2>&1 | tail -5`
Expected: build succeeds

- [ ] **Step 2: Update STATUS.md**

In `docs/STATUS.md`, update the PRD missing items table. Change the Self metrics row from:

```
| Self metrics（dwell_time/tool_usage） | 部分 | 中 |
```

to:

```
| Self metrics（dwell_time/tool_usage） | done（心跳式 dwell_time + 全路由 tool_usage + Self AI 注入） | 中 |
```

Add to the "本轮完成" section under "功能改进":

```
- **Self Metrics** — dwell_time 心跳式追踪（30s 前端心跳 + 后端批量刷盘），tool_usage 覆盖 9 类操作（search/graph_edit/archive/upload/export/blueprint/session_create/session_summarize/memory_extract），已有指标桩接入实际路由，metrics 摘要注入 Self 系统提示词，前端扩展指标面板
```

Update the test count line (should be +3 new tests).

- [ ] **Step 3: Commit**

```bash
git add docs/STATUS.md
git commit -m "docs: update STATUS.md for Self Metrics completion"
```
