# Phase 4 Implementation Plan — Git Integration (TDD)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 Git 仓库关联、git2 操作封装、归档流程（创建者直接 commit / 成员提交 PR）、GitLab API 集成（创建仓库、MR 管理）、前端 PR 审核界面。

**Architecture:** GitService 封装 git2 本地操作（clone/pull/push/commit/diff），GitlabService 封装 GitLab API HTTP 调用（创建仓库、创建/合并/关闭 MR、获取 diff），ArchiveService 编排归档流程（创建者 → 直接 commit push；成员 → 创建分支 → commit → push → 创建 MR → 进入审核队列）。

**Tech Stack:** git2 (Rust Git 库), reqwest (GitLab API HTTP), AES-256-GCM (凭证加密)

**Reference docs:**
- `docs/technical/git-integration.md` — Git 集成方案
- `docs/technical/api-design.md` section 6 (归档 API) + section 7 (Git API)
- `docs/technical/developer-guide.md` section 2 (依赖) + section 7.2 (凭证管理)

---

## File Structure

```
ring-server/src/
├── services/
│   ├── git_service.rs           # git2 wrapper: clone, pull, push, commit, diff, log, create_branch
│   ├── gitlab_service.rs        # GitLab API client: create repo, create/merge/close MR, get diff
│   ├── archive_service.rs       # Archive orchestration: creator flow + member PR flow
│   └── credential_service.rs    # AES-256-GCM encrypt/decrypt for GitLab tokens
├── handlers/
│   ├── git.rs                   # Git API endpoints: PR list, diff, merge, reject, commit history
│   └── archive.rs               # Archive API endpoints: archive, queue, confirm
├── models/
│   ├── git_model.rs             # Git/PR/Archive request/response types
│   └── archive.rs               # Archive record model
└── (routes.rs updated with git + archive routes)

ring-frontend/src/
├── components/git/
│   └── DiffView.tsx             # Diff rendering component
├── pages/RingSpace/
│   ├── PrList.tsx               # PR list page
│   └── PrDetail.tsx             # PR detail + diff + merge/reject
├── api/client.ts                # Add git/archive API functions
├── stores/
│   └── gitStore.ts              # Git/PR state management
└── App.tsx                      # Add routes
```

---

## Module 1: Credential Service + Git Service Core

**Files:**
- Create: `ring-server/src/services/credential_service.rs`
- Create: `ring-server/src/services/git_service.rs`

- [ ] **Step 1: Write failing tests for CredentialService**

Test against in-memory key:

- `encrypt_decrypt_roundtrip` — encrypt text, decrypt, verify matches
- `decrypt_wrong_key_fails` — encrypt with key A, decrypt with key B, verify fails
- `encrypt_empty_string` — encrypt empty, decrypt, verify empty

- [ ] **Step 2: Write failing tests for GitService**

Test against a temporary directory with a real git repo (git2 can init local repos):

- `init_and_clone_repo` — init bare repo, clone it, verify .git exists
- `commit_and_get_log` — create file, commit, verify log has one entry
- `create_branch_and_get_diff` — create branch, modify file, get diff, verify changes
- `pull_detect_changes` — clone repo twice, commit in one, pull in other, verify change

- [ ] **Step 3: Run test to verify it fails**

Run: `cd ring-server && cargo test --lib services::credential_service services::git_service`
Expected: FAIL

- [ ] **Step 4: Implement**

`services/credential_service.rs`:
```rust
pub struct CredentialService {
    key: [u8; 32], // AES-256 key
}

impl CredentialService {
    pub fn new(key: [u8; 32]) -> Self;
    pub fn encrypt(&self, plaintext: &str) -> Result<String>;  // base64(nonce + ciphertext + tag)
    pub fn decrypt(&self, encrypted: &str) -> Result<String>;
}
```
Use `aes_gcm::{Aes256Gcm, KeyInit, Nonce}` and `base64` for encoding.

`services/git_service.rs`:
```rust
pub struct GitService;

pub struct PullResult {
    pub had_changes: bool,
    pub changed_files: Vec<String>,
}

pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}

pub struct DiffResult {
    pub files: Vec<FileDiff>,
}

pub struct FileDiff {
    pub path: String,
    pub status: String,  // added, modified, deleted
    pub additions: i64,
    pub deletions: i64,
    pub content: String, // unified diff text
}

impl GitService {
    pub fn new() -> Self;
    pub async fn init_repo(&self, path: &Path) -> Result<()>;            // git init
    pub async fn clone_repo(&self, url: &str, path: &Path) -> Result<()>; // git clone
    pub async fn add_all(&self, path: &Path) -> Result<()>;
    pub async fn commit(&self, path: &Path, message: &str) -> Result<String>;  // returns commit sha
    pub async fn push(&self, path: &Path, branch: &str) -> Result<()>;   // uses stored credentials
    pub async fn pull(&self, path: &Path, branch: &str) -> Result<PullResult>;
    pub async fn create_branch(&self, path: &Path, name: &str) -> Result<()>;
    pub async fn checkout_branch(&self, path: &Path, name: &str) -> Result<()>;
    pub async fn get_diff(&self, path: &Path, from: &str, to: &str) -> Result<DiffResult>;
    pub async fn get_log(&self, path: &Path, limit: usize) -> Result<Vec<CommitInfo>>;
    pub async fn has_changes(&self, path: &Path) -> Result<bool>;
}
```
All git2 operations MUST run in `tokio::task::spawn_blocking` since git2 is sync.
For tests: use local file paths (no auth needed for local repos). Auth integration comes in Module 2.

- [ ] **Step 5: Run test**

Run: `cd ring-server && cargo test --lib services::credential_service services::git_service`
Expected: ALL PASS

- [ ] **Step 6: Commit**

```bash
git add ring-server/src/services/credential_service.rs ring-server/src/services/git_service.rs
git commit -m "feat(phase4): add credential encryption and git service with tests"
```

---

## Module 2: GitLab API Service

**Files:**
- Create: `ring-server/src/services/gitlab_service.rs`
- Modify: `ring-server/src/config.rs` — add gitlab fields

- [ ] **Step 1: Write failing tests**

Test using mock HTTP (or skip HTTP tests, test request building):

- `create_repo_builds_correct_request` — verify request URL and body structure
- `create_mr_builds_correct_request` — verify MR creation payload
- `parse_merge_response` — verify parsing of successful merge

For integration-level tests, use a mock server or just test request construction.

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`services/gitlab_service.rs`:
```rust
pub struct GitlabService {
    base_url: String,
    token: String,
    client: reqwest::Client,
}

pub struct CreateRepoResponse {
    pub id: i64,
    pub url: String,
    pub ssh_url: String,
}

pub struct MergeRequestInfo {
    pub id: i64,
    pub iid: i64,         // project-level MR number
    pub title: String,
    pub author: String,
    pub state: String,
    pub web_url: String,
}

pub struct MrDiff {
    pub old_path: String,
    pub new_path: String,
    pub diff: String,
}

impl GitlabService {
    pub fn new(base_url: &str, token: &str) -> Self;
    pub async fn create_repo(&self, name: &str, namespace: Option<&str>) -> Result<CreateRepoResponse>;
    pub async fn create_mr(&self, project_id: i64, source_branch: &str, target_branch: &str, title: &str) -> Result<MergeRequestInfo>;
    pub async fn merge_mr(&self, project_id: i64, mr_iid: i64) -> Result<MergeRequestInfo>;
    pub async fn close_mr(&self, project_id: i64, mr_iid: i64) -> Result<MergeRequestInfo>;
    pub async fn list_mrs(&self, project_id: i64, state: &str) -> Result<Vec<MergeRequestInfo>>;
    pub async fn get_mr_diff(&self, project_id: i64, mr_iid: i64) -> Result<Vec<MrDiff>>;
    pub async fn get_repo_url(&self, project_path: &str) -> Result<String>;
}
```

All GitLab API calls use reqwest with PRIVATE-TOKEN header.
Base URL defaults to `https://gitlab.com` (or custom internal GitLab URL).
Token comes from CredentialService (decrypted from SQLite settings).

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/services/gitlab_service.rs
git commit -m "feat(phase4): add GitLab API service for repo and MR management"
```

---

## Module 3: Archive Service + Models

**Files:**
- Create: `ring-server/src/services/archive_service.rs`
- Create: `ring-server/src/models/git_model.rs`
- Modify: `ring-server/src/db/traits.rs` — add archive query methods
- Modify: `ring-server/src/db/sqlite.rs` — implement archive queries
- Modify: `ring-server/src/services/ai_service.rs` — update MockRepo if needed

- [ ] **Step 1: Write failing tests**

- `creator_archive_commits_directly` — archive as creator, verify git commit created
- `member_archive_creates_branch_and_mr` — archive as member, verify branch + MR created (mock git/gitlab)
- `archive_queue_ordering` — multiple archives, verify queue order
- `confirm_archive_updates_status` — confirm archive, verify status changed

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`models/git_model.rs` — request/response types:
```rust
pub struct ArchiveRequest {
    pub message_ids: Vec<String>,
    pub conversation_id: String,
    pub graph_id: String,
    pub target_node_id: Option<String>,
    pub label: String,
}

pub struct ArchiveResponse {
    pub archive_id: String,
    pub markdown_path: String,
    pub git_status: String,       // committed | pr_pending
    pub pr_url: Option<String>,
    pub queue_position: Option<i64>,
}

pub struct ArchiveQueueResponse {
    pub current_review: Option<QueueItem>,
    pub queue: Vec<QueueItem>,
}

pub struct QueueItem {
    pub pr_id: i64,
    pub author: String,
    pub title: String,
    pub position: i64,
}

pub struct PrResponse {
    pub pr_id: i64,
    pub title: String,
    pub author: String,
    pub state: String,
    pub changes: Vec<FileChange>,
}

pub struct FileChange {
    pub file: String,
    pub status: String,
    pub additions: i64,
    pub deletions: i64,
    pub diff: String,
}

pub struct CommitLogResponse {
    pub commits: Vec<CommitEntry>,
}

pub struct CommitEntry {
    pub id: String,
    pub message: String,
    pub author: String,
    pub date: String,
}
```

`services/archive_service.rs`:
```rust
pub struct ArchiveService {
    repo: Arc<dyn Repository>,
    graph_store: Arc<RwLock<PetgraphStore>>,
    git_service: Arc<GitService>,
    gitlab_service: Arc<GitlabService>,
    credential_service: Arc<CredentialService>,
}

impl ArchiveService {
    pub fn new(...) -> Self;

    pub async fn archive(&self, ring_id: &str, req: ArchiveRequest, is_creator: bool) -> Result<ArchiveResponse>;
    pub async fn get_queue(&self, ring_id: &str) -> Result<ArchiveQueueResponse>;
    pub async fn confirm_archive(&self, ring_id: &str, archive_id: &str) -> Result<()>;
    pub async fn list_prs(&self, ring_id: &str, state: &str) -> Result<Vec<PrResponse>>;
    pub async fn get_pr_diff(&self, ring_id: &str, pr_id: i64) -> Result<PrResponse>;
    pub async fn merge_pr(&self, ring_id: &str, pr_id: i64) -> Result<()>;
    pub async fn reject_pr(&self, ring_id: &str, pr_id: i64) -> Result<()>;
    pub async fn get_commit_log(&self, ring_id: &str, limit: usize) -> Result<CommitLogResponse>;
}
```

Creator flow:
1. Generate Markdown content from messages
2. Write to `nodes/{slug}.md`
3. Update graph.json via PetgraphStore
4. Export graph.json → write to repo
5. `git add . && git commit && git push origin main`

Member flow:
1. `git pull origin main` (ensure latest)
2. Generate Markdown content
3. `git checkout -b archive/{member-id}/{timestamp}`
4. Write files + update graph.json
5. `git add . && git commit && git push origin {branch}`
6. GitLab API `create_mr`
7. Record archive with `pr_pending` status

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/services/archive_service.rs ring-server/src/models/git_model.rs ring-server/src/db/traits.rs ring-server/src/db/sqlite.rs
git commit -m "feat(phase4): add archive service with creator/member flows and PR queue"
```

---

## Module 4: Git + Archive Handlers + Routes

**Files:**
- Create: `ring-server/src/handlers/git.rs`
- Create: `ring-server/src/handlers/archive.rs`
- Create: `ring-server/tests/git_integration.rs`
- Modify: `ring-server/src/routes.rs`
- Modify: `ring-server/src/handlers/mod.rs`

- [ ] **Step 1: Write integration tests**

```
archive_as_creator_returns_committed — POST archive, verify git_status=committed
archive_queue_returns_empty_initially — GET queue, verify empty
list_prs_returns_empty — GET prs, verify empty list
get_commit_log_returns_empty — GET commits, verify empty list
```

Since real git/gitlab operations need infrastructure, these tests use mocked ArchiveService or simplified flows.

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`handlers/archive.rs`:
```
POST /rings/{ringId}/archive → archive
GET  /rings/{ringId}/archive/queue → get_queue
POST /rings/{ringId}/archive/{archiveId}/confirm → confirm_archive
```

`handlers/git.rs`:
```
GET  /rings/{ringId}/prs → list_prs
GET  /rings/{ringId}/prs/{prId}/diff → get_pr_diff
POST /rings/{ringId}/prs/{prId}/merge → merge_pr
POST /rings/{ringId}/prs/{prId}/reject → reject_pr
GET  /rings/{ringId}/commits → get_commit_log
```

`routes.rs` additions:
```rust
let archive_routes = Router::new()
    .route("/", post(archive::archive))
    .route("/queue", get(archive::get_queue))
    .route("/{archiveId}/confirm", post(archive::confirm_archive));

let git_routes = Router::new()
    .route("/prs", get(git::list_prs))
    .route("/prs/{prId}/diff", get(git::get_pr_diff))
    .route("/prs/{prId}/merge", post(git::merge_pr))
    .route("/prs/{prId}/reject", post(git::reject_pr))
    .route("/commits", get(git::get_commit_log));

.nest("/api/v1/rings/{ringId}/archive", archive_routes)
.nest("/api/v1/rings/{ringId}/git", git_routes)
```

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/handlers/git.rs ring-server/src/handlers/archive.rs ring-server/src/routes.rs ring-server/src/handlers/mod.rs ring-server/tests/git_integration.rs
git commit -m "feat(phase4): add git and archive handlers with integration tests"
```

---

## Module 5: Frontend — PR List + Diff View + Archive Button

**Files:**
- Create: `ring-frontend/src/components/git/DiffView.tsx`
- Create: `ring-frontend/src/pages/RingSpace/PrList.tsx`
- Create: `ring-frontend/src/pages/RingSpace/PrDetail.tsx`
- Create: `ring-frontend/src/stores/gitStore.ts`
- Modify: `ring-frontend/src/api/client.ts`
- Modify: `ring-frontend/src/App.tsx`
- Modify: `ring-frontend/src/types/index.ts`

- [ ] **Step 1: Write tests**

- `PrList renders empty state` — no PRs, verify empty message
- `PrList renders pr items` — mock PRs, verify list rendered
- `DiffView renders file changes` — mock diff data, verify rendered

- [ ] **Step 2: Run test to verify it fails**

- [ ] **Step 3: Implement**

`types/index.ts` additions:
```typescript
export interface ArchiveRequest {
  message_ids: string[]
  conversation_id: string
  graph_id: string
  target_node_id?: string
  label: string
}

export interface ArchiveResponse {
  archive_id: string
  markdown_path: string
  git_status: string
  pr_url: string | null
  queue_position: number | null
}

export interface PrListItem {
  pr_id: number
  title: string
  author: string
  state: string
}

export interface PrDetail {
  pr_id: number
  title: string
  author: string
  changes: FileChange[]
}

export interface FileChange {
  file: string
  status: string
  additions: number
  deletions: number
  diff: string
}

export interface CommitLogEntry {
  id: string
  message: string
  author: string
  date: string
}
```

`api/client.ts` additions:
```typescript
export async function archive_content(ring_id: string, req: ArchiveRequest): Promise<ArchiveResponse>
export async function get_archive_queue(ring_id: string): Promise<ArchiveQueueResponse>
export async function list_prs(ring_id: string, state?: string): Promise<PrListItem[]>
export async function get_pr_diff(ring_id: string, pr_id: number): Promise<PrDetail>
export async function merge_pr(ring_id: string, pr_id: number): Promise<void>
export async function reject_pr(ring_id: string, pr_id: number): Promise<void>
export async function get_commit_log(ring_id: string, limit?: number): Promise<CommitLogEntry[]>
```

`DiffView.tsx`:
- Side-by-side diff view for file changes
- Syntax highlighting via basic CSS (no Monaco Editor yet — MVP uses pre-formatted text)
- File list header with additions/deletions counts
- Each file section: file path + status badge + diff content

`PrList.tsx`:
- List of PRs with title, author, state badge
- Click → navigate to PrDetail
- Filter by state (opened/merged/closed)
- Empty state message

`PrDetail.tsx`:
- PR title + author + metadata
- File change list
- DiffView for each file
- Merge/Reject action buttons

`gitStore.ts`:
- State: prs[], current_pr, commit_log[], loading, error
- Actions: load_prs, load_pr_detail, merge_pr, reject_pr, load_commit_log

`App.tsx` routes:
```
/ring/:ringId/prs → PrList
/ring/:ringId/prs/:prId → PrDetail
```

- [ ] **Step 4: Run test**

- [ ] **Step 5: Commit**

```bash
git add ring-frontend/
git commit -m "feat(phase4): add PR list, diff view, and archive button frontend"
```

---

## Module 6: Integration Verification

- [ ] **Step 1: Run all backend tests**

Run: `cd ring-server && cargo test`
Expected: ALL PASS

- [ ] **Step 2: Run all frontend tests**

Run: `cd ring-frontend && npm test`
Expected: ALL PASS

- [ ] **Step 3: Run clippy + fmt**

```bash
cd ring-server && cargo fmt --check && cargo clippy -- -D warnings
```

- [ ] **Step 4: Final commit**

```bash
git commit --allow-empty -m "milestone: Phase 4 complete — git integration"
```
