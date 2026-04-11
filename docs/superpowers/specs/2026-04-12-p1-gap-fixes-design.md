# P1 Gap Fixes Design

Date: 2026-04-12

## GAP-08: SSE Stream Abort on Conversation Switch

### Problem

`chatStore` and `sessionChatStore` use raw `fetch()` + `reader` with no `AbortController`. Switching conversations or sessions while streaming leaves the old stream running, causing state corruption.

### Solution

Add `AbortController` to both stores. Before each new stream, abort the previous one. Pass `signal` to `fetch()`.

### Files

| File | Change |
|------|--------|
| `ring-frontend/src/stores/chatStore.ts` | Add `_abort_controller` field; abort on `send_message` start and `reset`; pass signal to `api.send_message` |
| `ring-frontend/src/stores/sessionChatStore.ts` | Same pattern |
| `ring-frontend/src/api/client.ts` | `send_message` and `send_session_message` accept optional `signal?: AbortSignal` |

### Design

The `AbortController` is stored as a private field (not in Zustand state — not reactive). Pattern:

```typescript
// In store creator closure
let abort_controller: AbortController | null = null

// In send_message:
if (abort_controller) abort_controller.abort()
abort_controller = new AbortController()
const res = await api.send_message(ring_id, conv_id, content, active_tools, abort_controller.signal)

// In reset:
if (abort_controller) { abort_controller.abort(); abort_controller = null }
```

When `abort()` fires, `fetch()` throws an `AbortError`. The catch block sets `is_streaming: false` — same as current error handling. No backend changes needed.

---

## GAP-06: .ring/ Template Directory Initialization

### Problem

`create_ring` creates `repos/ring-{name}/nodes/` and `graph.json` but never creates the `.ring/` template directory with the 6 template files specified in `docs/technical/ring-templates.md`. The AI service hardcodes placeholder strings instead of reading from these files.

### Solution

1. `create_ring` creates `repos/ring-{name}/.ring/` with all 6 template files
2. `ai_service.rs` reads `role.md`, `conventions.md`, `active-context.md` from disk, falls back to placeholders

### Files

| File | Change |
|------|--------|
| `ring-server/src/services/ring_service.rs` | After graph.json, create `.ring/` dir with 6 template files |
| `ring-server/src/services/ai_service.rs` | `group_ring_chat` reads `.ring/role.md` and `.ring/conventions.md` from ring's repo path |

### Design

Template content for each file comes from `docs/technical/ring-templates.md`. The `role.md` embeds `{role_description}` from the `CreateRingRequest`.

For AI service, add a helper function:

```rust
fn read_ring_file(data_dir: &Path, ring_name: &str, filename: &str) -> Option<String> {
    let path = data_dir.join("repos").join(format!("ring-{}", ring_name)).join(".ring").join(filename);
    std::fs::read_to_string(path).ok()
}
```

Used in `group_ring_chat`:

```rust
let role_md = read_ring_file(&self.data_dir, &ring.name, "role.md").unwrap_or_else(|| "(未设置角色定义)".into());
let conventions_md = read_ring_file(&self.data_dir, &ring.name, "conventions.md").unwrap_or_else(|| "(未设置团队约定)".into());
```

`AiService` needs to hold `data_dir: PathBuf` (currently doesn't have it).

---

## GAP-07: Archive Git Merge Implementation

### Problem

`merge_pr()` only updates DB status. `git_service` has no merge method. Handler hardcodes `is_creator: true` and passes `None` for `gitlab_service`. Archive records don't store the branch name needed for merge.

### Solution

1. Add `merge_branch` and `checkout` methods to `GitService`
2. Store branch name in archive record (use existing `markdown_path` field to derive it, or add a column)
3. Rewrite `merge_pr` to: git merge branch → update DB
4. Fix handler to pass real `is_creator` and `gitlab_service`

### Files

| File | Change |
|------|--------|
| `ring-server/src/services/git_service.rs` | Add `merge_branch(repo_path, branch)` and `checkout(repo_path, branch)` |
| `ring-server/src/services/archive_service.rs` | Store branch name; rewrite `merge_pr` to merge branch then update DB |
| `ring-server/src/handlers/archive.rs` | Look up member role for `is_creator`; pass `state.gitlab_service` |
| `ring-server/src/state.rs` | Expose `gitlab_service` (verify it's accessible) |

### Design

**GitService.merge_branch**: Checkout main → merge branch:

```rust
pub async fn merge_branch(&self, repo_path: &Path, branch: &str) -> Result<()> {
    // 1. Open repo
    // 2. Find branch reference
    // 3. Checkout HEAD (main)
    // 4. Merge branch into HEAD using git2::merge()
    // 5. Commit merge
}
```

**archive_service.merge_pr**: Get record → get ring → merge branch → update status.

**handler**: Use `state.db.get_member_by_user_and_ring()` to determine `is_creator`.

**Branch name storage**: Instead of a DB migration, derive branch name from the archive record's commit timestamp (already encoded as `archive/{timestamp}`). Alternatively, store it in the `pr_url` field metadata. Simplest: use the existing pattern `archive/{timestamp}` where timestamp is derived from the record.

## Execution Order

1. GAP-08 (frontend-only, smallest, no backend deps)
2. GAP-06 (backend, medium)
3. GAP-07 (backend, largest)
