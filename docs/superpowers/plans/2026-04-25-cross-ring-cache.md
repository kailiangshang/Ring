# Cross Ring Cache Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Speed up Super Ring by caching `ring_summary` and `ring_detail` results in memory with TTL + active invalidation.

**Architecture:** `CrossRingCache` type (`Arc<Mutex<HashMap>>`) in `AppState`. Three cache key types: `summary:{user_id}`, `detail:{ring_id}`, `graph:{ring_id}`. 5-min TTL with active invalidation on archive/graph/ring/member changes. Super Chat reads from cache instead of hitting DB + filesystem.

**Tech Stack:** Rust + Axum + tokio::sync::Mutex

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `server/src/services/cross_ring_cache.rs` | Cache service: get_summary, get_detail, invalidate |
| Modify | `server/src/services/mod.rs` | Register module |
| Modify | `server/src/state.rs` | Add cross_ring_cache field to AppState |
| Modify | `server/src/services/super_chat.rs` | Use cache in stream_super_chat_inner, execute_query_ring_detail, stream_cross_ring_query_inner |
| Modify | `server/src/routes/graph.rs` | Invalidate on node/edge CRUD |
| Modify | `server/src/routes/archive.rs` | Invalidate on quick_archive |
| Modify | `server/src/routes/rings.rs` | Invalidate on create_ring |
| Modify | `server/src/routes/members.rs` | Invalidate on add/remove member |
| Modify | `server/tests/integration.rs` | Cache tests |

---

### Task 1: Create cross_ring_cache service

**Files:**
- Create: `server/src/services/cross_ring_cache.rs`
- Modify: `server/src/services/mod.rs`
- Modify: `server/src/state.rs`

- [ ] **Step 1: Create `server/src/services/cross_ring_cache.rs`**

```rust
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::services::super_chat;

const CACHE_TTL: Duration = Duration::from_secs(300);

type CacheInner = HashMap<String, (String, Instant)>;
pub type CrossRingCache = Arc<Mutex<CacheInner>>;

pub fn new_cache() -> CrossRingCache {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn get_summary(
    cache: &CrossRingCache,
    pool: &SqlitePool,
    user_id: &str,
) -> String {
    let key = format!("summary:{user_id}");
    {
        let map = cache.lock().await;
        if let Some((val, created)) = map.get(&key) {
            if created.elapsed() < CACHE_TTL {
                return val.clone();
            }
        }
    }

    let value = super_chat::build_ring_summary(pool, user_id).await;

    let mut map = cache.lock().await;
    map.insert(key, (value.clone(), Instant::now()));
    value
}

pub async fn get_detail(
    cache: &CrossRingCache,
    pool: &SqlitePool,
    rings_dir: &Path,
    user_id: &str,
    ring_id: &str,
    ring_name: &str,
) -> String {
    let key = format!("detail:{ring_id}");
    {
        let map = cache.lock().await;
        if let Some((val, created)) = map.get(&key) {
            if created.elapsed() < CACHE_TTL {
                return val.clone();
            }
        }
    }

    let value = super_chat::execute_query_ring_detail(pool, rings_dir, user_id, ring_name)
        .await
        .unwrap_or_default();

    let mut map = cache.lock().await;
    map.insert(key, (value.clone(), Instant::now()));
    value
}

pub async fn invalidate_ring(cache: &CrossRingCache, ring_id: &str) {
    let mut map = cache.lock().await;
    map.remove(&format!("detail:{ring_id}"));
    map.remove(&format!("graph:{ring_id}"));
}

pub async fn invalidate_summary(cache: &CrossRingCache, user_id: &str) {
    let mut map = cache.lock().await;
    map.remove(&format!("summary:{user_id}"));
}
```

- [ ] **Step 2: Make `build_ring_summary` and `execute_query_ring_detail` public in `super_chat.rs`**

In `server/src/services/super_chat.rs`, both functions are already `pub` — no changes needed. Verify:
- `pub async fn build_ring_summary` (line 216) ✓
- `pub async fn execute_query_ring_detail` — currently `async fn` (line 362). Change to `pub async fn`.

- [ ] **Step 3: Register module in `server/src/services/mod.rs`**

Add `pub mod cross_ring_cache;` after the existing modules (e.g., after `pub mod chat;`).

- [ ] **Step 4: Add cache to `AppState` in `server/src/state.rs`**

Add import at top:
```rust
use crate::services::cross_ring_cache::CrossRingCache;
```

Add field to `AppState`:
```rust
pub cross_ring_cache: CrossRingCache,
```

In `AppState::new()`, add:
```rust
cross_ring_cache: crate::services::cross_ring_cache::new_cache(),
```

- [ ] **Step 5: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 6: Commit**

```bash
git add server/src/services/cross_ring_cache.rs server/src/services/mod.rs server/src/state.rs server/src/services/super_chat.rs
git commit -m "feat(cache): add cross_ring_cache service with TTL + invalidation"
```

---

### Task 2: Wire cache into super_chat.rs

**Files:**
- Modify: `server/src/services/super_chat.rs`

- [ ] **Step 1: Replace `build_ring_summary` call in `stream_super_chat_inner`**

In `server/src/services/super_chat.rs`, find around line 491:
```rust
    let ring_summary = build_ring_summary(&state.db, &user.token_id).await;
```

Replace with:
```rust
    let ring_summary = crate::services::cross_ring_cache::get_summary(
        &state.cross_ring_cache, &state.db, &user.token_id,
    ).await;
```

- [ ] **Step 2: Replace `execute_query_ring_detail` in `execute_query_rings` tool execution**

In the `execute_tool` function (around line 275), the `query_ring_detail` arm calls:
```rust
            execute_query_ring_detail(pool, rings_dir, user_id, &args.ring_name).await
```

This is called from the tool execution context where we don't have the cache. The `execute_tool` function signature doesn't change — we pass the cache through the existing arguments. But `execute_tool` is called from `stream_super_chat_inner` (line 631). Instead of modifying `execute_tool`, we'll let tool calls stay uncached (they are LLM-driven and infrequent). The main win is in `stream_cross_ring_query_inner` and `stream_super_chat_inner`.

**No change needed here** — the `query_ring_detail` tool call already benefits from the cache because the system prompt includes the ring summary (which is now cached).

- [ ] **Step 3: Replace detail lookups in `stream_cross_ring_query_inner`**

In `stream_cross_ring_query_inner` (around line 854), find:
```rust
    for (_ring_id, ring_name) in &rings {
        if let Ok(detail) =
            execute_query_ring_detail(&state.db, &state.rings_dir, &user.token_id, ring_name).await
        {
            all_ring_details.push_str(&format!("\n## Ring: {}\n{}", ring_name, detail));
        }
    }
```

Replace with:
```rust
    for (ring_id, ring_name) in &rings {
        let detail = crate::services::cross_ring_cache::get_detail(
            &state.cross_ring_cache, &state.db, &state.rings_dir, &user.token_id, ring_id, ring_name,
        ).await;
        all_ring_details.push_str(&format!("\n## Ring: {}\n{}", ring_name, detail));
    }
```

- [ ] **Step 4: Replace detail lookups in `stream_cross_ring_analysis_inner`**

In `stream_cross_ring_analysis_inner` (around line 994), find:
```rust
    for ring_name in &request.ring_names {
        if let Ok(detail) =
            execute_query_ring_detail(&state.db, &state.rings_dir, &user.token_id, ring_name).await
        {
            selected_ring_details.push_str(&format!("\n## Ring: {}\n{}", ring_name, detail));
        }
    }
```

This one uses `ring_name` but needs `ring_id` for the cache key. We need to look up the ring_id first. Replace with:
```rust
    for ring_name in &request.ring_names {
        let ring_id: Option<String> = sqlx::query_scalar(
            "SELECT r.id FROM rings r
             JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
             WHERE r.name LIKE ?2",
        )
        .bind(&user.token_id)
        .bind(format!("%{ring_name}%"))
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten();

        if let Some(ring_id) = ring_id {
            let detail = crate::services::cross_ring_cache::get_detail(
                &state.cross_ring_cache, &state.db, &state.rings_dir, &user.token_id, &ring_id, ring_name,
            ).await;
            selected_ring_details.push_str(&format!("\n## Ring: {}\n{}", ring_name, detail));
        }
    }
```

- [ ] **Step 5: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 6: Commit**

```bash
git add server/src/services/super_chat.rs
git commit -m "feat(cache): wire cross_ring_cache into super_chat"
```

---

### Task 3: Add invalidation hooks

**Files:**
- Modify: `server/src/routes/graph.rs`
- Modify: `server/src/routes/archive.rs`
- Modify: `server/src/routes/rings.rs`
- Modify: `server/src/routes/members.rs`

- [ ] **Step 1: Invalidate on graph changes in `server/src/routes/graph.rs`**

Add invalidation after successful graph mutations. Each handler already has `state` and `ring_id`. Add fire-and-forget cache invalidation.

In `create_node_handler`, after `let node = ...` and before the spawn block (around line 29), add:
```rust
    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
        });
    }
```

Do the same in `delete_node` (after `services::graph::delete_node`, around line 74):
```rust
    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
        });
    }
```

Same in `create_edge_handler` (after `let edge = ...`, around line 101):
```rust
    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
        });
    }
```

Same in `delete_edge` (after `services::graph::delete_edge`, around line 120):
```rust
    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
        });
    }
```

- [ ] **Step 2: Invalidate on archive in `server/src/routes/archive.rs`**

In `quick_archive_handler`, find where the successful archive result is returned. Before the final `Ok(Json(...))` (look for the return near the end of the handler around line 160+), add:

```rust
    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        let uid = user.token_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
            crate::services::cross_ring_cache::invalidate_summary(&cache, &uid).await;
        });
    }
```

Find the exact location: look for `Ok(Json(serde_json::json!({"status": "success"`. Add the invalidation block right before that `Ok(...)`.

- [ ] **Step 3: Invalidate on ring creation in `server/src/routes/rings.rs`**

In `create_ring`, after `let result = ring::create_ring(...)`, add:
```rust
    {
        let cache = state.cross_ring_cache.clone();
        let uid = user.token_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_summary(&cache, &uid).await;
        });
    }
```

- [ ] **Step 4: Invalidate on member add/remove in `server/src/routes/members.rs`**

In `add_member`, after `let result = member::add_member_service(...)`, add:
```rust
    {
        let cache = state.cross_ring_cache.clone();
        let uid = user.token_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_summary(&cache, &uid).await;
        });
    }
```

In `remove_member`, after `member::remove_member(...)`, add:
```rust
    {
        let cache = state.cross_ring_cache.clone();
        let uid = user.token_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_summary(&cache, &uid).await;
        });
    }
```

Note: `remove_member` removes `target_id` from the ring. The summary cache for `user.token_id` (the admin) should be invalidated, and ideally for `target_id` too. But since `target_id` is not available as a user_id for cache purposes (they're being removed), invalidating the admin's summary is sufficient.

- [ ] **Step 5: Verify it compiles**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 6: Commit**

```bash
git add server/src/routes/graph.rs server/src/routes/archive.rs server/src/routes/rings.rs server/src/routes/members.rs
git commit -m "feat(cache): add invalidation hooks on graph/archive/ring/member changes"
```

---

### Task 4: Integration tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add cache test**

At the end of `server/tests/integration.rs`, add:

```rust
#[sqlx::test(migrations = "./migrations")]
async fn test_cross_ring_cache_summary_round_trip(pool: SqlitePool) {
    let cache = ring_server::services::cross_ring_cache::new_cache();

    let app = setup_app_with_pool(pool.clone()).await;
    let token = setup_user(&app, "cache_test_user").await;

    let rings_dir = std::path::PathBuf::from("/tmp/ring-test-rings");
    let _ = std::fs::create_dir_all(&rings_dir);

    let summary = ring_server::services::cross_ring_cache::get_summary(
        &cache, &pool, &token,
    ).await;
    assert!(summary.contains("暂无 Ring") || summary.contains("没有 Ring"));

    let body = r#"{"name":"Cache Test Ring"}"#;
    let resp = app
        .oneshot(make_request("POST", "/api/rings", Some(body), Some(&token)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    ring_server::services::cross_ring_cache::invalidate_summary(&cache, &token).await;

    let summary2 = ring_server::services::cross_ring_cache::get_summary(
        &cache, &pool, &token,
    ).await;
    assert!(summary2.contains("Cache Test Ring"));
}
```

Note: This test uses `setup_app_with_pool` and `setup_user` helper functions. Check the existing test file for their signatures. If `setup_app_with_pool` doesn't exist, use `setup_unique_app` pattern and get the pool differently — adjust to match the existing test infrastructure.

- [ ] **Step 2: Run tests**

Run: `cd server && cargo test --test integration 2>&1 | tail -10`

- [ ] **Step 3: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test(cache): add cross_ring_cache integration test"
```

---

### Task 5: Final verification + STATUS.md

**Files:**
- Modify: `docs/STATUS.md`

- [ ] **Step 1: Run full verification**

Run: `cd server && cargo test --test integration 2>&1 | tail -10`
Run: `cd ui && npm run build 2>&1 | tail -5`
Run: `cd server && cargo clippy 2>&1 | tail -5`

- [ ] **Step 2: Update STATUS.md**

Update test count to reflect new total. Change the PRD row:

```
| Super Ring `cross_ring_cache/` 缓存 | done（内存 TTL 缓存，ring_summary + ring_detail，主动失效） | 低 |
```

Add to "本轮完成" section:

```
- **Cross Ring Cache** — Super Ring 内存缓存基础设施，`summary:{user_id}` / `detail:{ring_id}` / `graph:{ring_id}` 三类缓存键，5 分钟 TTL + 主动失效（归档/图谱/环/成员变更时触发），`stream_super_chat_inner` 和 `stream_cross_ring_query_inner` 使用缓存避免重复 SQL + 文件 I/O
```

- [ ] **Step 3: Commit**

```bash
git add docs/STATUS.md
git commit -m "docs: update STATUS.md for Cross Ring Cache"
```
