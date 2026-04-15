# Data Storage Structure Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename all `.ring/` references to `.group/` in code, config, and tests to reflect the new Group Ring terminology

**Architecture:** This is a path/directory renaming task. `~/.ring/` remains the user data root. Only the Group Ring subdirectory name changes: `.ring/` → `.group/`

**Tech Stack:** Rust, TypeScript, Shell

---

## 3. Affected Files (Confirmed)

| Location | Files | Count |
|----------|-------|-------|
| Backend | ai_service.rs, context_loader.rs, member_service.rs, ring_repo.rs, ring.rs | 5 |
| Frontend (tests/mocks) | mocks/handlers.ts, chatStore.test.ts, RingList.test.tsx | 3 |

### Backend Files

| File | Line | Content |
|------|------|---------|
| ai_service.rs | 440 | `.ring/repos/ring-TestRing` |
| context_loader.rs | 61 | `.ring/` (comment) |
| member_service.rs | 564 | `.ring/repos/ring-TestRing` |
| ring_repo.rs | 24 | `.ring/repos/ring-` |
| ring.rs (model) | 46 | `.ring/repos/ring-` |

### Frontend Files (Test Mocks)

| File | Lines | Content |
|------|-------|---------|
| mocks/handlers.ts | 32,47,82,83,163,359 | Test mock data paths |
| stores/chatStore.test.ts | 7 | Test mock data path |
| pages/RingHub/RingList.test.tsx | 14,26 | Test mock data paths |

---

## 4. Tasks

### Task 1: Update Backend Files

**Files:**
- Modify: `ring-server/src/services/ai_service.rs`
- Modify: `ring-server/src/services/context_loader.rs`
- Modify: `ring-server/src/services/member_service.rs`
- Modify: `ring-server/src/db/sqlite/ring_repo.rs`
- Modify: `ring-server/src/models/ring.rs`

- [ ] **Step 1: Update ai_service.rs:440**

```rust
// Change from:
local_path: ".ring/repos/ring-TestRing".into(),
// To:
local_path: ".group/repos/ring-TestRing".into(),
```

- [ ] **Step 2: Update context_loader.rs:61**

```rust
// Change from:
// 知识图谱不是凭空构建的。每个图谱节点都必须对应 `.ring/` 目录下的一个 Markdown 文档。
// To:
// 知识图谱不是凭空构建的。每个图谱节点都必须对应 `.group/` 目录下的一个 Markdown 文档。
```

- [ ] **Step 3: Update member_service.rs:564**

```rust
// Change from:
local_path: ".ring/repos/ring-TestRing".into(),
// To:
local_path: ".group/repos/ring-TestRing".into(),
```

- [ ] **Step 4: Update ring_repo.rs:24**

```rust
// Change from:
let local_path = format!(".ring/repos/ring-{}", new_ring.name);
// To:
let local_path = format!(".group/repos/ring-{}", new_ring.name);
```

- [ ] **Step 5: Update ring.rs:46**

```rust
// Change from:
local_path: "/home/.ring/repos/ring-竞品分析".into(),
// To:
local_path: "/home/.group/repos/ring-竞品分析".into(),
```

- [ ] **Step 6: Search for any other `.ring/` occurrences**

Run: `grep -r "\.ring/" ring-server/src/ --include="*.rs" | grep -v "\.group/"`
Expected: Only intentional references (none expected)

- [ ] **Step 7: Commit**

```bash
git add ring-server/src/services/ai_service.rs ring-server/src/services/context_loader.rs ring-server/src/services/member_service.rs ring-server/src/db/sqlite/ring_repo.rs ring-server/src/models/ring.rs
git commit -m "refactor: rename .ring/ to .group/ in backend paths"
```

---

### Task 2: Update Frontend Test Mocks

**Files:**
- Modify: `ring-frontend/src/mocks/handlers.ts`
- Modify: `ring-frontend/src/stores/chatStore.test.ts`
- Modify: `ring-frontend/src/pages/RingHub/RingList.test.tsx`

- [ ] **Step 1: Update mocks/handlers.ts**

Replace all `.ring/repos/` with `.group/repos/`:
```typescript
// Lines 32, 47, 163: '/home/.ring/repos/ring-1' → '/home/.group/repos/ring-1'
// Lines 82, 83: '.ring/docs/mock.md' → '.group/docs/mock.md'
// Line 359: '.ring/docs/...' → '.group/docs/...'
```

- [ ] **Step 2: Update stores/chatStore.test.ts:7**

```typescript
// Change from:
markdown_path: '.ring/docs/test.md',
// To:
markdown_path: '.group/docs/test.md',
```

- [ ] **Step 3: Update pages/RingHub/RingList.test.tsx**

```typescript
// Lines 14, 26: '/home/.ring/repos/a' → '/home/.group/repos/a'
```

- [ ] **Step 4: Search frontend for remaining .ring/ references**

Run: `grep -r "\.ring/" ring-frontend/src/ --include="*.ts" --include="*.tsx" | grep -v "\.group/"`
Expected: None

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/src/mocks/handlers.ts ring-frontend/src/stores/chatStore.test.ts ring-frontend/src/pages/RingHub/RingList.test.tsx
git commit -m "refactor: rename .ring/ to .group/ in frontend test mocks"
```

---

### Task 3: Verify No Breaking Changes

- [ ] **Step 1: Run Rust tests**

Run: `cargo test -p ring-server`
Expected: All tests pass

- [ ] **Step 2: Run Frontend tests**

Run: `cd ring-frontend && npm test`
Expected: All tests pass

- [ ] **Step 3: Commit any fixes**

```bash
git add -A
git commit -m "fix: resolve any test failures from .ring/ to .group/ rename"
```

---

### Task 4: Final Verification

- [ ] **Step 1: Final grep for any remaining .ring/ references**

Run: `grep -r "\.ring/" ring-server/src/ ring-frontend/src/ --include="*.rs" --include="*.ts" --include="*.tsx" | grep -v "\.group/"`
Expected: None (only false positives like "string" or similar)

- [ ] **Step 2: Commit final cleanup**

```bash
git add -A
git commit -m "refactor: final cleanup of .ring/ references"
```

---

## Self-Review Checklist

1. **Spec coverage:** All `.ring/` → `.group/` changes complete
   - ✅ 5 backend files updated
   - ✅ 3 frontend test/mock files updated

2. **Placeholder scan:** No "TBD" or "TODO" found

3. **Type consistency:** Path strings consistent - `.group/repos/ring-` format used everywhere

4. **Note on data migration:** This plan assumes new install. If migrating existing data, additional migration script would be needed (not in scope for this plan).

---

## Execution Options

**Plan complete and saved to `docs/superpowers/plans/2026-04-15-data-storage-structure.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?