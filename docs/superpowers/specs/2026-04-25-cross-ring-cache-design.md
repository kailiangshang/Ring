# Cross Ring Cache Design

**Date:** 2026-04-25
**Goal:** Speed up Super Ring by caching `ring_summary` and `ring_detail` results in memory, eliminating repeated SQL queries and file reads on every Super Chat call.

## Problem

Every Super Ring chat triggers:
- `build_ring_summary()` — 1 + 2N SQL queries (rings, member count per ring, archive titles per ring)
- `execute_query_ring_detail()` — reads `graph.json` from disk + scans archive directory
- `stream_cross_ring_query_inner()` — calls `execute_query_ring_detail()` for every ring sequentially

For a user with 5 rings, a single Super Chat can trigger 15+ SQL queries and 5+ file reads. Cross-ring analysis doubles this.

## Approach

In-memory TTL cache in `AppState` with active invalidation on data changes.

**Why in-memory:** localhost single-user tool; no persistence needed across restarts. The DwellBuffer pattern (`Arc<Mutex<HashMap>>`) is already established in the codebase.

## Data Structure

```rust
type CrossRingCacheInner = HashMap<String, (String, std::time::Instant)>;
pub type CrossRingCache = Arc<tokio::sync::Mutex<CrossRingCacheInner>>;
```

**Keys:**

| Key format | Value source | Source cost |
|---|---|---|
| `summary:{user_id}` | `build_ring_summary()` result | 1 + 2N SQL queries |
| `detail:{ring_id}` | `execute_query_ring_detail()` result | graph.json read + archive dir scan |
| `graph:{ring_id}` | raw `graph.json` file content | file read |

**TTL:** 5 minutes. Entries older than 5 min are treated as miss.

## New File

`server/src/services/cross_ring_cache.rs`

```rust
pub struct CrossRingCacheService;

impl CrossRingCacheService {
    pub fn new() -> CrossRingCache { ... }

    pub async fn get_summary(cache: &CrossRingCache, pool: &SqlitePool, user_id: &str) -> String
    // Check cache -> return if fresh, else compute via build_ring_summary + store

    pub async fn get_detail(cache: &CrossRingCache, pool: &SqlitePool, rings_dir: &Path, user_id: &str, ring_name: &str) -> String
    // Check cache -> return if fresh, else compute via execute_query_ring_detail + store

    pub async fn invalidate(cache: &CrossRingCache, key_prefix: &str)
    // Remove all entries starting with key_prefix

    pub async fn invalidate_ring(cache: &CrossRingCache, ring_id: &str)
    // Remove detail:{ring_id} + graph:{ring_id}

    pub async fn invalidate_summary(cache: &CrossRingCache, user_id: &str)
    // Remove summary:{user_id}
}
```

## State Changes

`server/src/state.rs` — add field:
```rust
pub cross_ring_cache: CrossRingCache,
```

`AppState::new()` — initialize with empty HashMap.

## Cache Consumers

### super_chat.rs changes

1. `stream_super_chat_inner()` — replace `build_ring_summary()` with `CrossRingCacheService::get_summary()`
2. `execute_query_ring_detail()` calls in tool execution — use `CrossRingCacheService::get_detail()`
3. `stream_cross_ring_query_inner()` — use cached detail for each ring instead of direct call

## Invalidation Points

Active invalidation ensures stale data never reaches the 5-min TTL:

| Trigger | Location | Invalidation |
|---|---|---|
| Archive created (`quick_archive`, `auto_archive_chat`) | `archive_service.rs` | `invalidate_ring(ring_id)` + `invalidate_summary(user_id)` |
| Graph node created/deleted | `graph.rs` routes | `invalidate_ring(ring_id)` |
| Graph edge created/deleted | `graph.rs` routes | `invalidate_ring(ring_id)` |
| Ring created/deleted | `ring.rs` routes | `invalidate_summary(user_id)` |
| Member added/removed | `member.rs` routes | `invalidate_summary(user_id)` |

Invalidation is fire-and-forget (spawned task or `.await` with error ignored) to not block the write path.

## File Map

| Action | File |
|---|---|
| Create | `server/src/services/cross_ring_cache.rs` |
| Modify | `server/src/services/mod.rs` |
| Modify | `server/src/state.rs` |
| Modify | `server/src/services/super_chat.rs` |
| Modify | `server/src/services/archive_service.rs` |
| Modify | `server/src/routes/graph.rs` |
| Modify | `server/src/routes/ring.rs` |
| Modify | `server/src/routes/member.rs` |
| Modify | `server/tests/integration.rs` |

## Testing

- Unit test: cache miss → compute → cache hit → return same value
- Unit test: TTL expiry → recomputed
- Unit test: `invalidate_ring` clears only matching keys
- Integration test: tools defined and cache round-trips
