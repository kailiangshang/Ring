# Tech Debt Batch 1: Unwrap Removal, State Dedup, Token Unification

## Goal

Fix 3 high-value tech debt items: eliminate 18 `.unwrap()` panics in route handlers, deduplicate `active_ring_id` dual state, and unify token retrieval across the frontend.

## Item 1: Remove `.unwrap()` in Route Handlers

**Problem:** 18 locations use `serde_json::to_value(...).unwrap()` in route handlers. If serialization fails, the server panics and crashes.

**Fix:** Replace `Json(serde_json::to_value(result).unwrap())` with `Json(result)` directly. Axum's `Json<T>` already implements `IntoResponse` for any `T: Serialize` — it handles serialization internally with proper error handling.

**Files affected:**
- `server/src/routes/session.rs` — 8 occurrences
- `server/src/routes/upload.rs` — 3 occurrences
- `server/src/routes/rings.rs` — 2 occurrences
- `server/src/routes/setup.rs` — 2 occurrences
- `server/src/routes/members.rs` — 1 occurrence
- `server/src/routes/export.rs` — 2 occurrences (uses `to_string_pretty`, different pattern)

**Return type changes:**
- Most handlers currently return `Result<Json<serde_json::Value>>` → change to return the concrete type wrapped in `Json`. For example:
  - `Result<Json<Value>>` → `Result<Json<SomeStruct>>`
  - Or if the handler returns different types, keep `Result<Json<Value>>` but use `.map_err(|e| RingError::Internal(e.to_string()))?` instead of `.unwrap()`
- For `export.rs`: the `to_string_pretty` calls are for file content, not JSON responses. Replace `.unwrap()` with `.map_err(|e| RingError::Internal(e.to_string()))?`.

**Verification:** `cargo test` passes, `cargo clippy` clean.

## Item 2: Deduplicate `active_ring_id`

**Problem:** `active_ring_id` exists in both `app-store.ts` and `ring-store.ts`. 3 write sites only update `app-store`, causing divergence.

**Fix:**
1. Remove `active_ring_id` from `app-store.ts`. Keep only `current_context` (which is `app-store`'s real concern).
2. Rename `setActiveRing` to `setContext` — it sets `current_context` based on whether a ring is active: `setContext: (ring_id: string | null) => set({ current_context: ring_id ? 'ring' : 'super' })`.
3. All callers that previously called both `selectRing(id)` + `setActiveRing(id)` now call `selectRing(id)` + `setContext(id)`.
4. All callers that previously called only `setActiveRing(id)` (3 buggy sites) now call both `selectRing(id)` + `setContext(id)`.
5. All reads of `app-store.active_ring_id` switch to `ring-store.active_ring_id`.

**Files affected:**
- `ui/src/stores/app-store.ts` — remove `active_ring_id` state, rename `setActiveRing` → `setContext`
- `ui/src/stores/ring-store.ts` — keep `active_ring_id` + `selectRing`
- `ui/src/components/sidebar/RingList.tsx` — update calls
- `ui/src/components/sidebar/SuperRingEntry.tsx` — fix missing `selectRing` call
- `ui/src/components/chat/InputArea.tsx` — update `setActiveRing` → `setContext`
- `ui/src/components/chat/MessageItem.tsx` — update calls
- `ui/src/components/NotificationBell.tsx` — fix missing `selectRing` call
- Any other file that reads `app-store.active_ring_id`

**Verification:** `npm run build` clean. Search for `setActiveRing` and `app-store.*active_ring_id` to confirm zero remaining references.

## Item 3: Unify Token Retrieval

**Problem:** 6 call sites bypass the centralized `getToken()` in `api.ts`, using direct `localStorage.getItem('ring_token')` with inconsistent header styles.

**Fix:**
1. Export `getToken()` from `api.ts` (currently it's a module-level `async function`, not exported).
2. Replace all direct `localStorage.getItem('ring_token')` calls in stores/services with `getToken()` from `api.ts`.
3. Standardize header style: always include `X-Ring-Token` header, with empty string `''` when no token (matches the most common existing pattern).

**Files affected:**
- `ui/src/services/api.ts` — export `getToken()`
- `ui/src/services/sse.ts` — use `getToken()` instead of direct localStorage
- `ui/src/stores/chat-store.ts` — use `getToken()` in `loadHistory()`
- `ui/src/stores/self-chat-store.ts` — use `getToken()` in `loadHistory()`
- `ui/src/stores/ws-store.ts` — use `getToken()` for WebSocket URL

**Verification:** `npm run build` clean. Search for `localStorage.getItem('ring_token')` to confirm only `api.ts` and `auth-store.ts` remain.

## Constraints

- No behavior changes — pure refactoring
- Each item must pass its verification independently
- Commit each item separately with clear messages
- Do NOT touch batch 2 items (SSE dedup, chat-store split, AbortController)
