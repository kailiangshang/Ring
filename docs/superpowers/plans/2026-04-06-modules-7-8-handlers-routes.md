# Modules 7-8: Setup Handler, Ring Handler, Routes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add HTTP handlers for setup wizard + ring CRUD, wire routes, and integration tests.

**Architecture:** Handlers parse params -> call service/repo -> return response. No business logic in handlers. Settings (LLM, GitLab) stored as key-value in `settings` table. Router wired in `routes.rs`.

**Tech Stack:** Axum 0.8 extractors (State, Path, Json), SQLite via sqlx, RingService for ring operations.

---

### Task 1: Add settings methods to Repository trait + SqliteRepository

**Files:**
- Modify: `ring-server/src/db/traits.rs`
- Modify: `ring-server/src/db/sqlite.rs`

- [ ] **Step 1: Add `get_setting` and `set_setting` to Repository trait**

Add to `ring-server/src/db/traits.rs`:
```rust
async fn get_setting(&self, key: &str) -> Result<Option<String>>;
async fn set_setting(&self, key: &str, value: &str) -> Result<()>;
```

- [ ] **Step 2: Implement in SqliteRepository**

In `ring-server/src/db/sqlite.rs`, add implementations:
```rust
async fn get_setting(&self, key: &str) -> Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;
    Ok(row.map(|(v,)| v))
}

async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at")
        .bind(key)
        .bind(value)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;
    Ok(())
}
```

- [ ] **Step 3: Run `cargo test` to verify existing tests still pass**

Run: `cargo test`
Expected: All existing tests pass.

---

### Task 2: Create handlers module structure

**Files:**
- Modify: `ring-server/src/lib.rs`
- Create: `ring-server/src/handlers/mod.rs`

- [ ] **Step 1: Add `pub mod handlers;` and `pub mod routes;` to lib.rs**

- [ ] **Step 2: Create `handlers/mod.rs` exporting setup and ring modules**

```rust
pub mod ring;
pub mod setup;
```

---

### Task 3: Create setup handler

**Files:**
- Create: `ring-server/src/handlers/setup.rs`

- [ ] **Step 1: Write setup.rs with 5 handlers + request/response structs**

Handlers:
- `get_status` — GET /api/v1/setup/status
- `set_username` — POST /api/v1/setup/username
- `set_llm` — POST /api/v1/setup/llm
- `set_gitlab` — POST /api/v1/setup/gitlab
- `complete` — POST /api/v1/setup/complete

All handlers (except get_status) check setup_completed first, return 409 if done.

---

### Task 4: Create ring handler

**Files:**
- Create: `ring-server/src/handlers/ring.rs`

- [ ] **Step 1: Write ring.rs with 5 handlers**

Handlers:
- `list_rings` — GET /api/v1/rings
- `create_ring` — POST /api/v1/rings
- `get_ring` — GET /api/v1/rings/{ringId}
- `update_ring` — PUT /api/v1/rings/{ringId}
- `delete_ring` — DELETE /api/v1/rings/{ringId}

Hardcode user_id as first user from DB for Phase 1.

---

### Task 5: Create routes module

**Files:**
- Create: `ring-server/src/routes.rs`

- [ ] **Step 1: Write build_router function wiring all routes**

```
/api/v1/setup/status     -> GET get_status
/api/v1/setup/username   -> POST set_username
/api/v1/setup/llm        -> POST set_llm
/api/v1/setup/gitlab     -> POST set_gitlab
/api/v1/setup/complete   -> POST complete
/api/v1/rings            -> GET list_rings, POST create_ring
/api/v1/rings/{ringId}   -> GET get_ring, PUT update_ring, DELETE delete_ring
```

---

### Task 6: Create integration tests

**Files:**
- Create: `ring-server/tests/setup_integration.rs`
- Create: `ring-server/tests/ring_integration.rs`

- [ ] **Step 1: Write setup_integration.rs** with tests:
- `full_setup_wizard_flow`
- `setup_rejects_empty_username`
- `setup_rejects_long_username`
- `setup_twice_returns_conflict`

- [ ] **Step 2: Write ring_integration.rs** with tests:
- `create_and_list_rings`
- `create_and_get_ring`
- `update_ring_name`
- `delete_ring`
- `create_ring_empty_name`
- `get_nonexistent_ring`

---

### Task 7: Verify everything passes

- [ ] **Step 1: Run `cargo fmt`**
- [ ] **Step 2: Run `cargo clippy -- -D warnings`**
- [ ] **Step 3: Run `cargo test`** — all tests must pass
