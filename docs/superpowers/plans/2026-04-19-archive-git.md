# Archive + Git/GitLab Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement conversation → Markdown archive → Git commit/push → GitLab MR review queue for Ring.

**Architecture:** Shell `git` commands wrapped in `git_service.rs`, GitLab REST API via `reqwest` in `gitlab_service.rs`, orchestration in `archive_service.rs`. Archives stored as flat Markdown files in `~/.ring/rings/<id>/archives/`. Creator/admin commits directly to main; members create branches + MRs.

**Tech Stack:** Rust + Axum 0.8, SQLite (sqlx), shell git, reqwest for GitLab API, SSE for progress streaming. Frontend: React + TypeScript + Zustand.

**Spec:** `docs/superpowers/specs/2026-04-19-archive-git-design.md`

---

## File Map

**Create:**
- `server/migrations/006_archive.sql` — archive_records table + graph_nodes.markdown_path
- `server/src/models/archive.rs` — ArchiveRecord model + CRUD
- `server/src/services/git_service.rs` — Shell git command wrappers
- `server/src/services/gitlab_service.rs` — GitLab REST API client
- `server/src/services/archive_service.rs` — Archive business logic orchestration
- `server/src/routes/archive.rs` — HTTP endpoints for archive operations
- `ui/src/types/archive.ts` — Frontend archive type definitions
- `ui/src/stores/archive-store.ts` — Archive Zustand store
- `ui/src/components/panels/ArchivePanel.tsx` — Archive UI panel

**Modify:**
- `server/Cargo.toml` — Add `reqwest` + `chrono` dependencies
- `server/src/models/mod.rs` — Register `archive` module
- `server/src/services/mod.rs` — Register new service modules
- `server/src/routes/mod.rs` — Register archive routes
- `server/src/error.rs` — Add git/gitlab error variants
- `server/src/state.rs` — Add rings data dir path to AppState
- `server/src/main.rs` — Pass data dir to AppState
- `server/src/models/graph.rs` — Add markdown_path to GraphNodeRow + update_node_markdown_path query
- `ui/src/services/api.ts` — Add archive API calls

---

## Task 6-0: Database Migration 006

**Files:**
- Create: `server/migrations/006_archive.sql`

- [ ] **Step 1: Write migration file**

```sql
CREATE TABLE IF NOT EXISTS archive_records (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    node_id TEXT REFERENCES graph_nodes(id) ON DELETE SET NULL,
    file_name TEXT NOT NULL,
    commit_sha TEXT,
    branch TEXT,
    merge_request_iid INTEGER,
    status TEXT NOT NULL DEFAULT 'pending',
    archived_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_archive_records_ring ON archive_records(ring_id);
CREATE INDEX IF NOT EXISTS idx_archive_records_status ON archive_records(status);
CREATE INDEX IF NOT EXISTS idx_archive_records_archived_by ON archive_records(archived_by);

ALTER TABLE graph_nodes ADD COLUMN markdown_path TEXT;
```

- [ ] **Step 2: Verify migration runs**

Run: `cd server && cargo test`
Expected: All existing tests pass (migration applies cleanly).

- [ ] **Step 3: Commit**

```bash
git add server/migrations/006_archive.sql
git commit -m "feat: add migration 006 — archive_records table + graph_nodes.markdown_path"
```

---

## Task 6-1: Archive Model + CRUD Queries

**Files:**
- Create: `server/src/models/archive.rs`
- Modify: `server/src/models/mod.rs` — add `pub mod archive;`

- [ ] **Step 1: Write archive model file**

Create `server/src/models/archive.rs`:

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct ArchiveRecord {
    pub id: String,
    pub ring_id: String,
    pub session_id: Option<String>,
    pub node_id: Option<String>,
    pub file_name: String,
    pub commit_sha: Option<String>,
    pub branch: Option<String>,
    pub merge_request_iid: Option<i64>,
    pub status: String,
    pub archived_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateArchiveInput {
    pub session_id: Option<String>,
    pub content: String,
    pub suggested_title: String,
    pub node_suggestion: NodeSuggestionInput,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum NodeSuggestionInput {
    #[serde(rename = "create_new")]
    CreateNew {
        parent_id: Option<String>,
        node_title: String,
    },
    #[serde(rename = "attach_existing")]
    AttachExisting { node_id: String },
    #[serde(rename = "update_existing")]
    UpdateExisting { node_id: String },
}

#[derive(Debug, Deserialize)]
pub struct ReviewInput {
    pub action: ReviewAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewAction {
    Merge,
    Reject,
}

pub async fn insert_record(
    pool: &sqlx::SqlitePool,
    id: &str,
    ring_id: &str,
    session_id: Option<&str>,
    node_id: Option<&str>,
    file_name: &str,
    archived_by: &str,
) -> Result<ArchiveRecord> {
    sqlx::query_as::<_, ArchiveRecord>(
        "INSERT INTO archive_records (id, ring_id, session_id, node_id, file_name, status, archived_by)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)
         RETURNING *",
    )
    .bind(id)
    .bind(ring_id)
    .bind(session_id)
    .bind(node_id)
    .bind(file_name)
    .bind(archived_by)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn update_status(
    pool: &sqlx::SqlitePool,
    id: &str,
    status: &str,
    commit_sha: Option<&str>,
    branch: Option<&str>,
    merge_request_iid: Option<i64>,
) -> Result<ArchiveRecord> {
    sqlx::query_as::<_, ArchiveRecord>(
        "UPDATE archive_records
         SET status = ?1, commit_sha = COALESCE(?2, commit_sha),
             branch = COALESCE(?3, branch), merge_request_iid = COALESCE(?4, merge_request_iid),
             updated_at = datetime('now')
         WHERE id = ?5
         RETURNING *",
    )
    .bind(status)
    .bind(commit_sha)
    .bind(branch)
    .bind(merge_request_iid)
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound(format!("archive record {id} not found")))
}

pub async fn get_record(
    pool: &sqlx::SqlitePool,
    id: &str,
) -> Result<ArchiveRecord> {
    sqlx::query_as::<_, ArchiveRecord>("SELECT * FROM archive_records WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("archive record {id} not found")))
}

pub async fn list_by_ring(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
) -> Result<Vec<ArchiveRecord>> {
    sqlx::query_as::<_, ArchiveRecord>(
        "SELECT * FROM archive_records WHERE ring_id = ?1 ORDER BY created_at DESC",
    )
    .bind(ring_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn list_pending_reviews(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
) -> Result<Vec<ArchiveRecord>> {
    sqlx::query_as::<_, ArchiveRecord>(
        "SELECT * FROM archive_records WHERE ring_id = ?1 AND status = 'mr_opened' ORDER BY created_at ASC",
    )
    .bind(ring_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}
```

- [ ] **Step 2: Register module in `models/mod.rs`**

Add `pub mod archive;` to `server/src/models/mod.rs` after the existing module declarations.

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add server/src/models/archive.rs server/src/models/mod.rs
git commit -m "feat: add archive model + CRUD queries"
```

---

## Task 6-2: Error Variants + Dependencies

**Files:**
- Modify: `server/src/error.rs` — add git/gitlab error variants
- Modify: `server/Cargo.toml` — add `reqwest` + `chrono` dependencies
- Modify: `server/src/state.rs` — add `rings_dir` field
- Modify: `server/src/main.rs` — compute and pass `rings_dir`

- [ ] **Step 1: Add error variants**

Add to `RingError` enum in `server/src/error.rs` before the closing brace:

```rust
    #[error("Git command failed: {cmd}: {stderr}")]
    GitCommandFailed { cmd: String, stderr: String },

    #[error("GitLab API error ({status}): {message}")]
    GitlabApiError { status: u16, message: String },

    #[error("GitLab not configured")]
    GitlabNotConfigured,

    #[error("Repository not found for ring: {ring_id}")]
    RepoNotFound { ring_id: String },

    #[error("Archive conflict: {record_id}")]
    ArchiveConflict { record_id: String },

    #[error("Invalid archive state: record {record_id} is {current}, expected {expected}")]
    InvalidArchiveState {
        record_id: String,
        current: String,
        expected: String,
    },
```

- [ ] **Step 2: Add Cargo dependencies**

Add to `server/Cargo.toml` under `[dependencies]`:

```toml
reqwest = { version = "0.12", features = ["json"] }
chrono = "0.4"
```

- [ ] **Step 3: Update AppState with rings_dir**

Replace `server/src/state.rs`:

```rust
use std::path::PathBuf;

use sqlx::SqlitePool;

use crate::ws_hub::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub ws_hub: WsHub,
    pub rings_dir: PathBuf,
}

impl AppState {
    pub fn new(db: SqlitePool, rings_dir: PathBuf) -> Self {
        Self {
            db,
            ws_hub: WsHub::new(),
            rings_dir,
        }
    }
}
```

- [ ] **Step 4: Update main.rs to pass rings_dir**

In `server/src/main.rs`, update the `main` function. After the `data_dir` line, add:

```rust
    let rings_dir = std::path::PathBuf::from(format!("{data_dir}/rings"));
    std::fs::create_dir_all(&rings_dir).expect("failed to create rings dir");
```

And change `AppState::new(pool)` to:

```rust
    let state = AppState::new(pool, rings_dir);
```

The full `main` function becomes:

```rust
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("ring_server=debug,tower_http=debug")
        .init();

    let data_dir = dirs_data_dir();
    std::fs::create_dir_all(&data_dir).expect("failed to create data dir");

    let db_url = format!("sqlite:{}/ring.db?mode=rwc", data_dir);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("failed to connect to SQLite");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let rings_dir = std::path::PathBuf::from(format!("{data_dir}/rings"));
    std::fs::create_dir_all(&rings_dir).expect("failed to create rings dir");

    let state = AppState::new(pool, rings_dir);
    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7420")
        .await
        .expect("failed to bind to port 7420");

    tracing::info!("ring-server listening on http://localhost:7420");
    axum::serve(listener, app).await.expect("server error");
}
```

- [ ] **Step 5: Verify compilation + tests**

Run: `cd server && cargo build && cargo test`
Expected: All 9 existing tests pass.

- [ ] **Step 6: Commit**

```bash
git add server/src/error.rs server/Cargo.toml server/src/state.rs server/src/main.rs
git commit -m "feat: add git/gitlab error variants, reqwest dep, rings_dir in AppState"
```

---

## Task 6-3: git_service.rs — Shell Git Command Wrapper

**Files:**
- Create: `server/src/services/git_service.rs`
- Modify: `server/src/services/mod.rs` — add `pub mod git_service;`

- [ ] **Step 1: Write git_service.rs**

Create `server/src/services/git_service.rs`:

```rust
use std::path::Path;
use std::process::Command;

use crate::error::{Result, RingError};

pub struct GitService;

impl GitService {
    pub fn new() -> Self {
        Self
    }

    fn run_git(repo_path: &Path, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|e| RingError::Internal(format!("failed to execute git: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(RingError::GitCommandFailed {
                cmd: args.join(" "),
                stderr,
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn init(path: &Path) -> Result<()> {
        Self::run_git(path, &["init"])?;
        Ok(())
    }

    pub fn clone(url: &str, path: &Path) -> Result<()> {
        Self::run_git(path, &["clone", url, &path.to_string_lossy()])?;
        Ok(())
    }

    pub fn pull(repo_path: &Path) -> Result<()> {
        Self::run_git(repo_path, &["pull", "--rebase"])?;
        Ok(())
    }

    pub fn add_all(repo_path: &Path) -> Result<()> {
        Self::run_git(repo_path, &["add", "."])?;
        Ok(())
    }

    pub fn commit(repo_path: &Path, msg: &str) -> Result<String> {
        Self::run_git(repo_path, &["commit", "-m", msg])?;
        Self::run_git(repo_path, &["rev-parse", "HEAD"])
    }

    pub fn push(repo_path: &Path, remote: &str, branch: &str) -> Result<()> {
        Self::run_git(repo_path, &["push", remote, branch])?;
        Ok(())
    }

    pub fn create_branch(repo_path: &Path, name: &str) -> Result<()> {
        Self::run_git(repo_path, &["checkout", "-b", name])?;
        Ok(())
    }

    pub fn checkout(repo_path: &Path, branch: &str) -> Result<()> {
        Self::run_git(repo_path, &["checkout", branch])?;
        Ok(())
    }

    pub fn log(repo_path: &Path, n: usize) -> Result<Vec<LogEntry>> {
        let format = "--pretty=format:%H|%s|%an|%ai";
        let output = Self::run_git(repo_path, &["log", format, "-n", &n.to_string()])?;
        let entries = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(4, '|').collect();
                if parts.len() == 4 {
                    Some(LogEntry {
                        sha: parts[0].to_string(),
                        subject: parts[1].to_string(),
                        author: parts[2].to_string(),
                        date: parts[3].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok(entries)
    }

    pub fn has_remote(path: &Path) -> bool {
        Self::run_git(path, &["remote"]).map(|r| !r.is_empty()).unwrap_or(false)
    }

    pub fn set_remote(path: &Path, name: &str, url: &str) -> Result<()> {
        let has_origin = Self::run_git(path, &["remote"])
            .map(|r| r.lines().any(|l| l == name))
            .unwrap_or(false);
        if has_origin {
            Self::run_git(path, &["remote", "set-url", name, url])?;
        } else {
            Self::run_git(path, &["remote", "add", name, url])?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEntry {
    pub sha: String,
    pub subject: String,
    pub author: String,
    pub date: String,
}
```

- [ ] **Step 2: Register module**

Add `pub mod git_service;` to `server/src/services/mod.rs`.

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add server/src/services/git_service.rs server/src/services/mod.rs
git commit -m "feat: add git_service — shell git command wrapper"
```

---

## Task 6-4: gitlab_service.rs — GitLab REST API Client

**Files:**
- Create: `server/src/services/gitlab_service.rs`
- Modify: `server/src/services/mod.rs` — add `pub mod gitlab_service;`

- [ ] **Step 1: Write gitlab_service.rs**

Create `server/src/services/gitlab_service.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::error::{Result, RingError};

#[derive(Clone)]
pub struct GitLabClient {
    base_url: String,
    token: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeRequest {
    pub iid: i64,
    pub title: String,
    pub description: Option<String>,
    pub source_branch: String,
    pub target_branch: String,
    pub state: String,
    pub web_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffRef {
    pub old_path: String,
    pub new_path: String,
    pub diff: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabUser {
    pub id: i64,
    pub username: String,
}

impl GitLabClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            client: reqwest::Client::new(),
        }
    }

    fn project_id_from_url(repo_url: &str) -> String {
        let url = url::Url::parse(repo_url).unwrap_or_else(|_| {
            url::Url::parse(&format!("https://{}", repo_url)).unwrap()
        });
        let path = url.path().trim_start_matches('/').trim_end_matches(".git");
        urlencoding::encode(path).to_string()
    }

    pub async fn get_current_user(&self) -> Result<GitLabUser> {
        let url = format!("{}/api/v4/user", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError {
                status,
                message: body,
            });
        }

        resp.json::<GitLabUser>()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("parse error: {e}"),
            })
    }

    pub async fn create_mr(
        &self,
        project_url: &str,
        source_branch: &str,
        target_branch: &str,
        title: &str,
        description: &str,
    ) -> Result<MergeRequest> {
        let project_id = Self::project_id_from_url(project_url);
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests",
            self.base_url, project_id
        );

        let resp = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .json(&serde_json::json!({
                "source_branch": source_branch,
                "target_branch": target_branch,
                "title": title,
                "description": description,
            }))
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        let status = resp.status().as_u16();
        if status == 409 {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError {
                status,
                message: format!("conflict: {body}"),
            });
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError {
                status,
                message: body,
            });
        }

        resp.json::<MergeRequest>()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("parse error: {e}"),
            })
    }

    pub async fn merge_mr(
        &self,
        project_url: &str,
        mr_iid: i64,
    ) -> Result<MergeRequest> {
        let project_id = Self::project_id_from_url(project_url);
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}/merge",
            self.base_url, project_id, mr_iid
        );

        let resp = self
            .client
            .put(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        let status = resp.status().as_u16();
        if status == 405 {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::ArchiveConflict {
                record_id: format!("mr-{}", mr_iid),
            });
        }

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError {
                status,
                message: body,
            });
        }

        resp.json::<MergeRequest>()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("parse error: {e}"),
            })
    }

    pub async fn close_mr(
        &self,
        project_url: &str,
        mr_iid: i64,
    ) -> Result<MergeRequest> {
        let project_id = Self::project_id_from_url(project_url);
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}",
            self.base_url, project_id, mr_iid
        );

        let resp = self
            .client
            .put(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .json(&serde_json::json!({
                "state_event": "close"
            }))
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError {
                status,
                message: body,
            });
        }

        resp.json::<MergeRequest>()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("parse error: {e}"),
            })
    }

    pub async fn get_mr_diffs(
        &self,
        project_url: &str,
        mr_iid: i64,
    ) -> Result<Vec<DiffRef>> {
        let project_id = Self::project_id_from_url(project_url);
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests/{}/diffs",
            self.base_url, project_id, mr_iid
        );

        let resp = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("connection failed: {e}"),
            })?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::GitlabApiError {
                status,
                message: body,
            });
        }

        #[derive(Deserialize)]
        struct MrDiffsResponse {
            diffs: Vec<DiffRef>,
        }

        let result: MrDiffsResponse = resp
            .json()
            .await
            .map_err(|e| RingError::GitlabApiError {
                status: 0,
                message: format!("parse error: {e}"),
            })?;

        Ok(result.diffs)
    }
}
```

- [ ] **Step 2: Add url + urlencoding dependencies to Cargo.toml**

Add to `server/Cargo.toml` under `[dependencies]`:

```toml
url = "2"
urlencoding = "2"
```

- [ ] **Step 3: Register module**

Add `pub mod gitlab_service;` to `server/src/services/mod.rs`.

- [ ] **Step 4: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add server/src/services/gitlab_service.rs server/src/services/mod.rs server/Cargo.toml
git commit -m "feat: add gitlab_service — GitLab REST API client"
```

---

## Task 6-5: graph.rs — Add markdown_path Support

**Files:**
- Modify: `server/src/models/graph.rs` — add `markdown_path` field to `GraphNodeRow` + new query

- [ ] **Step 1: Add markdown_path field to GraphNodeRow**

In `server/src/models/graph.rs`, add `markdown_path` field to `GraphNodeRow` struct after `tags`:

```rust
    pub markdown_path: Option<String>,
```

- [ ] **Step 2: Add update_node_markdown_path query**

Add to end of `server/src/models/graph.rs`:

```rust
pub async fn update_node_markdown_path(
    pool: &sqlx::SqlitePool,
    node_id: &str,
    markdown_path: &str,
) -> Result<GraphNodeRow> {
    sqlx::query_as::<_, GraphNodeRow>(
        "UPDATE graph_nodes SET markdown_path = ?1, updated_at = datetime('now')
         WHERE id = ?2 RETURNING *",
    )
    .bind(markdown_path)
    .bind(node_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound("node not found".into()))
}
```

- [ ] **Step 3: Verify compilation + tests**

Run: `cd server && cargo test`
Expected: All existing tests pass (the new column defaults to NULL for existing data).

- [ ] **Step 4: Commit**

```bash
git add server/src/models/graph.rs
git commit -m "feat: add markdown_path to GraphNodeRow + update query"
```

---

## Task 6-6: archive_service.rs — Core Orchestration

**Files:**
- Create: `server/src/services/archive_service.rs`
- Modify: `server/src/services/mod.rs` — add `pub mod archive_service;`

- [ ] **Step 1: Write archive_service.rs**

Create `server/src/services/archive_service.rs`:

```rust
use std::path::PathBuf;

use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::{Result, RingError};
use crate::models::archive;
use crate::models::archive::ArchiveRecord;
use crate::models::graph;
use crate::services::git_service::GitService;
use crate::services::gitlab_service::GitLabClient;

pub fn ring_repo_path(rings_dir: &std::path::Path, ring_id: &str) -> PathBuf {
    rings_dir.join(ring_id)
}

pub fn sanitize_filename(title: &str) -> String {
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let safe_title: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    let safe_title = safe_title.trim_matches('-');
    format!("{date}_{safe_title}.md")
}

pub fn init_ring_repo(
    git: &GitService,
    rings_dir: &std::path::Path,
    ring_id: &str,
    gitlab_url: Option<&str>,
) -> Result<PathBuf> {
    let repo_path = ring_repo_path(rings_dir, ring_id);
    std::fs::create_dir_all(&repo_path)?;

    if !repo_path.join(".git").exists() {
        GitService::init(&repo_path)?;
    }

    let archives_dir = repo_path.join("archives");
    if !archives_dir.exists() {
        std::fs::create_dir_all(&archives_dir)?;
    }

    let graphs_dir = repo_path.join("graphs");
    if !graphs_dir.exists() {
        std::fs::create_dir_all(&graphs_dir)?;
    }

    let group_dir = repo_path.join(".group");
    if !group_dir.exists() {
        std::fs::create_dir_all(&group_dir)?;
    }

    let assets_dir = repo_path.join("assets");
    if !assets_dir.exists() {
        std::fs::create_dir_all(&assets_dir)?;
    }

    let gitignore_path = repo_path.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, ".ring-local/\nassets/\n")?;
    }

    let ring_local_dir = repo_path.join(".ring-local");
    if !ring_local_dir.exists() {
        std::fs::create_dir_all(&ring_local_dir)?;
    }

    if let Some(url) = gitlab_url {
        if !git.has_remote(&repo_path) {
            git.set_remote(&repo_path, "origin", url)?;
        }
    }

    Ok(repo_path)
}

pub async fn archive_content_creator(
    pool: &SqlitePool,
    git: &GitService,
    rings_dir: &std::path::Path,
    ring_id: &str,
    session_id: Option<&str>,
    node_id: Option<&str>,
    content: &str,
    title: &str,
    user_id: &str,
) -> Result<ArchiveRecord> {
    let repo_path = ring_repo_path(rings_dir, ring_id);

    if !repo_path.join(".git").exists() {
        return Err(RingError::RepoNotFound {
            ring_id: ring_id.to_string(),
        });
    }

    if git.has_remote(&repo_path) {
        let _ = git.pull(&repo_path);
    }

    let file_name = sanitize_filename(title);
    let file_path = repo_path.join("archives").join(&file_name);
    std::fs::write(&file_path, content)?;

    if let Some(nid) = node_id {
        let relative = format!("archives/{file_name}");
        let _ = graph::update_node_markdown_path(pool, nid, &relative).await;
    }

    git.add_all(&repo_path)?;
    let sha = git.commit(&repo_path, &format!("archive: {title}"))?;

    if git.has_remote(&repo_path) {
        git.push(&repo_path, "origin", "main")?;
        archive::insert_record(pool, &ulid::Ulid::new().to_string(), ring_id, session_id, node_id, &file_name, user_id)
            .await?
            .id
    } else {
        archive::insert_record(pool, &ulid::Ulid::new().to_string(), ring_id, session_id, node_id, &file_name, user_id)
            .await?
            .id
    };

    let status = if git.has_remote(&repo_path) {
        "pushed"
    } else {
        "committed"
    };

    archive::update_status(pool, &archive::insert_record(pool, &ulid::Ulid::new().to_string(), ring_id, session_id, node_id, &file_name, user_id).await?.id, status, Some(&sha), None, None).await
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ArchiveStep {
    Pulling,
    Generating,
    Writing,
    Committing,
    Pushing,
    CreatingMR,
    Complete,
}

impl ArchiveStep {
    pub fn message(&self) -> &str {
        match self {
            ArchiveStep::Pulling => "正在拉取最新内容...",
            ArchiveStep::Generating => "AI 正在生成归档内容...",
            ArchiveStep::Writing => "写入 Markdown 文件...",
            ArchiveStep::Committing => "提交到 Git...",
            ArchiveStep::Pushing => "推送到远程仓库...",
            ArchiveStep::CreatingMR => "创建 Merge Request...",
            ArchiveStep::Complete => "归档完成",
        }
    }

    pub fn step_name(&self) -> &str {
        match self {
            ArchiveStep::Pulling => "pulling",
            ArchiveStep::Generating => "generating",
            ArchiveStep::Writing => "writing",
            ArchiveStep::Committing => "committing",
            ArchiveStep::Pushing => "pushing",
            ArchiveStep::CreatingMR => "creating_mr",
            ArchiveStep::Complete => "complete",
        }
    }
}
```

```rust
pub async fn archive_content_creator(
    pool: &SqlitePool,
    git: &GitService,
    rings_dir: &std::path::Path,
    ring_id: &str,
    session_id: Option<&str>,
    node_id: Option<&str>,
    content: &str,
    title: &str,
    user_id: &str,
) -> Result<ArchiveRecord> {
    let repo_path = ring_repo_path(rings_dir, ring_id);

    if !repo_path.join(".git").exists() {
        return Err(RingError::RepoNotFound {
            ring_id: ring_id.to_string(),
        });
    }

    if git.has_remote(&repo_path) {
        let _ = git.pull(&repo_path);
    }

    let file_name = sanitize_filename(title);
    let file_path = repo_path.join("archives").join(&file_name);
    std::fs::write(&file_path, content)?;

    if let Some(nid) = node_id {
        let relative = format!("archives/{file_name}");
        let _ = graph::update_node_markdown_path(pool, nid, &relative).await;
    }

    git.add_all(&repo_path)?;
    let sha = git.commit(&repo_path, &format!("archive: {title}"))?;

    let has_remote = git.has_remote(&repo_path);
    if has_remote {
        git.push(&repo_path, "origin", "main")?;
    }

    let record_id = ulid::Ulid::new().to_string();
    archive::insert_record(pool, &record_id, ring_id, session_id, node_id, &file_name, user_id).await?;

    let status = if has_remote { "pushed" } else { "committed" };
    archive::update_status(pool, &record_id, status, Some(&sha), None, None).await
}

pub async fn archive_content_member(
    pool: &SqlitePool,
    git: &GitService,
    gitlab: &GitLabClient,
    rings_dir: &std::path::Path,
    ring_id: &str,
    gitlab_repo_url: &str,
    session_id: Option<&str>,
    node_id: Option<&str>,
    content: &str,
    title: &str,
    user_id: &str,
) -> Result<ArchiveRecord> {
    let repo_path = ring_repo_path(rings_dir, ring_id);

    if !repo_path.join(".git").exists() {
        return Err(RingError::RepoNotFound {
            ring_id: ring_id.to_string(),
        });
    }

    let _ = git.pull(&repo_path);

    let file_name = sanitize_filename(title);
    let file_path = repo_path.join("archives").join(&file_name);
    std::fs::write(&file_path, content)?;

    if let Some(nid) = node_id {
        let relative = format!("archives/{file_name}");
        let _ = graph::update_node_markdown_path(pool, nid, &relative).await;
    }

    let record_id = ulid::Ulid::new().to_string();
    let branch_name = format!("archive/{record_id}");

    git.create_branch(&repo_path, &branch_name)?;
    git.add_all(&repo_path)?;
    let sha = git.commit(&repo_path, &format!("archive: {title}"))?;
    git.push(&repo_path, "origin", &branch_name)?;
    git.checkout(&repo_path, "main")?;

    archive::insert_record(pool, &record_id, ring_id, session_id, node_id, &file_name, user_id).await?;
    archive::update_status(pool, &record_id, "committed", Some(&sha), Some(&branch_name), None).await?;

    let mr = gitlab
        .create_mr(
            gitlab_repo_url,
            &branch_name,
            "main",
            &format!("归档: {title}"),
            &format!("由 {user_id} 提交的归档请求"),
        )
        .await?;

    archive::update_status(pool, &record_id, "mr_opened", None, None, Some(mr.iid)).await
}

pub async fn review_mr(
    pool: &SqlitePool,
    git: &GitService,
    gitlab: &GitLabClient,
    rings_dir: &std::path::Path,
    record_id: &str,
    gitlab_repo_url: &str,
    action: archive::ReviewAction,
) -> Result<ArchiveRecord> {
    let record = archive::get_record(pool, record_id).await?;

    if record.status != "mr_opened" {
        return Err(RingError::InvalidArchiveState {
            record_id: record_id.to_string(),
            current: record.status,
            expected: "mr_opened".to_string(),
        });
    }

    let mr_iid = record
        .merge_request_iid
        .ok_or_else(|| RingError::Internal("MR IID missing".into()))?;

    let repo_path = ring_repo_path(rings_dir, &record.ring_id);

    match action {
        archive::ReviewAction::Merge => {
            gitlab.merge_mr(gitlab_repo_url, mr_iid).await?;
            git.pull(&repo_path)?;
            archive::update_status(pool, record_id, "merged", None, None, None).await
        }
        archive::ReviewAction::Reject => {
            gitlab.close_mr(gitlab_repo_url, mr_iid).await?;
            archive::update_status(pool, record_id, "rejected", None, None, None).await
        }
    }
}
```

- [ ] **Step 2: Register module**

Add `pub mod archive_service;` to `server/src/services/mod.rs`.

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add server/src/services/archive_service.rs server/src/services/mod.rs
git commit -m "feat: add archive_service — orchestration for archive/git/gitlab"
```

---

## Task 6-7: routes/archive.rs — HTTP Endpoints

**Files:**
- Create: `server/src/routes/archive.rs`
- Modify: `server/src/routes/mod.rs` — add archive routes

- [ ] **Step 1: Write routes/archive.rs**

Create `server/src/routes/archive.rs`:

```rust
use async_stream::stream;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use serde::Serialize;
use std::convert::Infallible;

use crate::error::{Result, RingError};
use crate::extractors::auth::AuthUser;
use crate::models::archive::{self, CreateArchiveInput, ReviewInput};
use crate::models::ring;
use crate::services::archive_service::{self, ArchiveStep};
use crate::services::git_service::GitService;
use crate::services::gitlab_service::GitLabClient;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ArchiveListResponse {
    pub archives: Vec<archive::ArchiveRecord>,
}

#[derive(Debug, Serialize)]
pub struct ArchiveQueueResponse {
    pub queue: Vec<archive::ArchiveRecord>,
}

#[derive(Debug, Serialize)]
pub struct RepoStatusResponse {
    pub initialized: bool,
    pub has_remote: bool,
}

pub async fn trigger_archive(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateArchiveInput>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let is_creator = role == "creator" || role == "admin";

    let record_id = ulid::Ulid::new().to_string();
    let ring_id_c = ring_id.clone();
    let token_id = user.token_id.clone();

    let git = GitService::new();

    let node_id = match &body.node_suggestion {
        archive::NodeSuggestionInput::CreateNew { .. } => None,
        archive::NodeSuggestionInput::AttachExisting { node_id } => Some(node_id.clone()),
        archive::NodeSuggestionInput::UpdateExisting { node_id } => Some(node_id.clone()),
    };

    let (tx, rx) = tokio::sync::mpsc::channel::<ArchiveStep>(16);

    let pool = state.db.clone();
    let rings_dir = state.rings_dir.clone();
    let title = body.suggested_title.clone();
    let content = body.content.clone();
    let session_id = body.session_id.clone();

    tokio::spawn(async move {
        let _ = tx.send(ArchiveStep::Pulling).await;

        let _ = tx.send(ArchiveStep::Writing).await;

        if is_creator {
            match archive_service::archive_content_creator(
                &pool,
                &git,
                &rings_dir,
                &ring_id_c,
                session_id.as_deref(),
                node_id.as_deref(),
                &content,
                &title,
                &token_id,
            )
            .await
            {
                Ok(_) => { let _ = tx.send(ArchiveStep::Complete).await; }
                Err(e) => { tracing::error!("archive failed: {e}"); }
            }
        } else {
            let repo_url = sqlx::query_scalar::<_, Option<String>>(
                "SELECT gitlab_repo_url FROM rings WHERE id = ?1",
            )
            .bind(&ring_id_c)
            .fetch_one(&pool)
            .await
            .ok()
            .flatten();

            let user_row = crate::models::user::get_user(&pool, &token_id).await;
            let (gitlab_url, gitlab_token) = match user_row {
                Ok(u) => (u.gitlab_url.clone(), u.gitlab_token.clone()),
                Err(_) => (None, None),
            };

            match (repo_url, gitlab_url, gitlab_token) {
                (Some(url), Some(gl_url), Some(gl_token)) => {
                    let gitlab = GitLabClient::new(&gl_url, &gl_token);
                    let _ = tx.send(ArchiveStep::CreatingMR).await;
                    match archive_service::archive_content_member(
                        &pool,
                        &git,
                        &gitlab,
                        &rings_dir,
                        &ring_id_c,
                        &url,
                        session_id.as_deref(),
                        node_id.as_deref(),
                        &content,
                        &title,
                        &token_id,
                    )
                    .await
                    {
                        Ok(_) => { let _ = tx.send(ArchiveStep::Complete).await; }
                        Err(e) => { tracing::error!("member archive failed: {e}"); }
                    }
                }
                _ => { tracing::error!("GitLab not configured for member archive"); }
            }
        }
    });

    let s = stream! {
        while let Some(step) = rx.recv().await {
            let data = serde_json::json!({
                "step": step.step_name(),
                "message": step.message()
            });
            yield Ok(Event::default().event("progress").data(data.to_string()));
        }
    };

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn list_archives(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<ArchiveListResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let records = archive::list_by_ring(&state.db, &ring_id).await?;
    Ok(Json(ArchiveListResponse { archives: records }))
}

pub async fn get_archive(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, archive_id)): Path<(String, String)>,
) -> Result<Json<archive::ArchiveRecord>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let record = archive::get_record(&state.db, &archive_id).await?;
    Ok(Json(record))
}

pub async fn review_archive(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, archive_id)): Path<(String, String)>,
    Json(body): Json<ReviewInput>,
) -> Result<Json<archive::ArchiveRecord>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden("only creator/admin can review".into()));
    }

    let user_row = crate::models::user::get_user(&state.db, &user.token_id).await?;
    let (gitlab_url, gitlab_token) = match (user_row.gitlab_url, user_row.gitlab_token) {
        (Some(url), Some(token)) => (url, token),
        _ => return Err(RingError::GitlabNotConfigured),
    };

    let repo_url: Option<String> =
        sqlx::query_scalar("SELECT gitlab_repo_url FROM rings WHERE id = ?1")
            .bind(&ring_id)
            .fetch_optional(&state.db)
            .await?
            .flatten();

    let repo_url = repo_url.ok_or_else(|| RingError::GitlabNotConfigured)?;

    let git = GitService::new();
    let gitlab = GitLabClient::new(&gitlab_url, &gitlab_token);

    let record = archive_service::review_mr(
        &state.db,
        &git,
        &gitlab,
        &state.rings_dir,
        &archive_id,
        &repo_url,
        body.action,
    )
    .await?;

    Ok(Json(record))
}

pub async fn archive_queue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<ArchiveQueueResponse>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden("only creator/admin can view queue".into()));
    }

    let records = archive::list_pending_reviews(&state.db, &ring_id).await?;
    Ok(Json(ArchiveQueueResponse { queue: records }))
}

pub async fn repo_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<RepoStatusResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let repo_path = state.rings_dir.join(&ring_id);
    let initialized = repo_path.join(".git").exists();
    let has_remote = if initialized {
        GitService::new().has_remote(&repo_path)
    } else {
        false
    };

    Ok(Json(RepoStatusResponse {
        initialized,
        has_remote,
    }))
}

pub async fn init_repo(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<RepoStatusResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let repo_url: Option<String> =
        sqlx::query_scalar("SELECT gitlab_repo_url FROM rings WHERE id = ?1")
            .bind(&ring_id)
            .fetch_optional(&state.db)
            .await?
            .flatten();

    let git = GitService::new();
    let repo_path = archive_service::init_ring_repo(
        &git,
        &state.rings_dir,
        &ring_id,
        repo_url.as_deref(),
    )?;

    let has_remote = git.has_remote(&repo_path);

    Ok(Json(RepoStatusResponse {
        initialized: true,
        has_remote,
    }))
}
```

- [ ] **Step 2: Register archive routes in routes/mod.rs**

Add `mod archive;` to the module declarations in `server/src/routes/mod.rs`.

Add these routes to the `api` Router (after the existing session routes, before `.with_state(state)`):

```rust
        .route(
            "/rings/{ring_id}/archive",
            post(archive::trigger_archive),
        )
        .route(
            "/rings/{ring_id}/archives",
            get(archive::list_archives),
        )
        .route(
            "/rings/{ring_id}/archives/{archive_id}",
            get(archive::get_archive),
        )
        .route(
            "/rings/{ring_id}/archives/{archive_id}/review",
            post(archive::review_archive),
        )
        .route(
            "/rings/{ring_id}/archive-queue",
            get(archive::archive_queue),
        )
        .route(
            "/rings/{ring_id}/repo/status",
            get(archive::repo_status),
        )
        .route(
            "/rings/{ring_id}/repo/init",
            post(archive::init_repo),
        )
```

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles without errors.

- [ ] **Step 4: Run all tests**

Run: `cd server && cargo test`
Expected: All existing tests pass.

- [ ] **Step 5: Commit**

```bash
git add server/src/routes/archive.rs server/src/routes/mod.rs
git commit -m "feat: add archive HTTP endpoints — trigger, list, review, repo status"
```

---

## Task 6-8: Frontend Archive Types + Store

**Files:**
- Create: `ui/src/types/archive.ts`
- Create: `ui/src/stores/archive-store.ts`
- Modify: `ui/src/services/api.ts` — add archive API calls

- [ ] **Step 1: Create archive types**

Create `ui/src/types/archive.ts`:

```typescript
export interface ArchiveRecord {
  id: string
  ring_id: string
  session_id: string | null
  node_id: string | null
  file_name: string
  commit_sha: string | null
  branch: string | null
  merge_request_iid: number | null
  status: ArchiveStatus
  archived_by: string
  created_at: string
  updated_at: string
}

export type ArchiveStatus =
  | "pending"
  | "committed"
  | "pushed"
  | "mr_opened"
  | "merged"
  | "rejected"

export type NodeSuggestionAction =
  | "create_new"
  | "attach_existing"
  | "update_existing"

export interface NodeSuggestion {
  action: NodeSuggestionAction
  parent_id?: string
  node_id?: string
  node_title?: string
}

export interface CreateArchiveInput {
  session_id?: string
  content: string
  suggested_title: string
  node_suggestion: NodeSuggestion
}

export interface ReviewInput {
  action: "merge" | "reject"
}

export interface ArchiveProgressEvent {
  step: string
  message: string
}

export interface RepoStatus {
  initialized: boolean
  has_remote: boolean
}
```

- [ ] **Step 2: Add archive API calls to api.ts**

Read the existing `ui/src/services/api.ts` to find the pattern, then append these functions:

```typescript
export async function triggerArchive(
  ringId: string,
  input: CreateArchiveInput,
): Promise<EventSource> {
  const token = getToken()
  const es = new EventSourceWithHeaders(
    `${API_BASE}/rings/${ringId}/archive`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Ring-Token": token || "",
      },
      body: JSON.stringify(input),
    },
  )
  return es
}
```

Actually, since POST + SSE doesn't work with native EventSource, we need to use fetch + ReadableStream. Append to `ui/src/services/api.ts`:

```typescript
export async function triggerArchiveSSE(
  ringId: string,
  input: import("../types/archive").CreateArchiveInput,
  onProgress: (event: import("../types/archive").ArchiveProgressEvent) => void,
  onComplete: () => void,
  onError: (err: string) => void,
): Promise<void> {
  const token = getToken()
  const resp = await fetch(`${API_BASE}/rings/${ringId}/archive`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Ring-Token": token || "",
    },
    body: JSON.stringify(input),
  })

  if (!resp.ok || !resp.body) {
    onError(`archive failed: ${resp.status}`)
    return
  }

  const reader = resp.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ""

  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    buffer += decoder.decode(value, { stream: true })
    const lines = buffer.split("\n")
    buffer = lines.pop() || ""
    for (const line of lines) {
      if (line.startsWith("data:")) {
        const data = line.slice(5).trim()
        if (!data) continue
        try {
          const parsed = JSON.parse(data)
          if (parsed.step && parsed.message) {
            onProgress(parsed)
          }
        } catch {}
      }
      if (line.startsWith("event:")) {
        const evt = line.slice(6).trim()
        if (evt === "complete") {
          onComplete()
        }
      }
    }
  }
  onComplete()
}

export async function listArchives(
  ringId: string,
): Promise<{ archives: import("../types/archive").ArchiveRecord[] }> {
  const resp = await fetch(`${API_BASE}/rings/${ringId}/archives`, {
    headers: { "X-Ring-Token": getToken() || "" },
  })
  return handleResponse(resp)
}

export async function getArchive(
  ringId: string,
  archiveId: string,
): Promise<import("../types/archive").ArchiveRecord> {
  const resp = await fetch(
    `${API_BASE}/rings/${ringId}/archives/${archiveId}`,
    { headers: { "X-Ring-Token": getToken() || "" } },
  )
  return handleResponse(resp)
}

export async function reviewArchive(
  ringId: string,
  archiveId: string,
  action: "merge" | "reject",
): Promise<import("../types/archive").ArchiveRecord> {
  const resp = await fetch(
    `${API_BASE}/rings/${ringId}/archives/${archiveId}/review`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Ring-Token": getToken() || "",
      },
      body: JSON.stringify({ action }),
    },
  )
  return handleResponse(resp)
}

export async function getArchiveQueue(
  ringId: string,
): Promise<{ queue: import("../types/archive").ArchiveRecord[] }> {
  const resp = await fetch(`${API_BASE}/rings/${ringId}/archive-queue`, {
    headers: { "X-Ring-Token": getToken() || "" },
  })
  return handleResponse(resp)
}

export async function getRepoStatus(
  ringId: string,
): Promise<import("../types/archive").RepoStatus> {
  const resp = await fetch(`${API_BASE}/rings/${ringId}/repo/status`, {
    headers: { "X-Ring-Token": getToken() || "" },
  })
  return handleResponse(resp)
}

export async function initRepo(
  ringId: string,
): Promise<import("../types/archive").RepoStatus> {
  const resp = await fetch(`${API_BASE}/rings/${ringId}/repo/init`, {
    method: "POST",
    headers: { "X-Ring-Token": getToken() || "" },
  })
  return handleResponse(resp)
}
```

- [ ] **Step 3: Create archive store**

Create `ui/src/stores/archive-store.ts`:

```typescript
import { create } from "zustand"
import type {
  ArchiveRecord,
  ArchiveStatus,
  RepoStatus,
} from "../types/archive"
import * as api from "../services/api"

interface ArchiveState {
  archives: ArchiveRecord[]
  queue: ArchiveRecord[]
  repo_status: RepoStatus | null
  loading: boolean
  archiving: boolean
  progress: string

  fetchArchives: (ringId: string) => Promise<void>
  fetchQueue: (ringId: string) => Promise<void>
  fetchRepoStatus: (ringId: string) => Promise<void>
  triggerArchive: (
    ringId: string,
    content: string,
    title: string,
    sessionId?: string,
  ) => Promise<void>
  reviewArchive: (
    ringId: string,
    archiveId: string,
    action: "merge" | "reject",
  ) => Promise<void>
  initRepo: (ringId: string) => Promise<void>
}

const STATUS_ORDER: Record<ArchiveStatus, string> = {
  pending: "⏳",
  committed: "📝",
  pushed: "✅",
  mr_opened: "🔀",
  merged: "✅",
  rejected: "❌",
}

export const useArchiveStore = create<ArchiveState>((set, get) => ({
  archives: [],
  queue: [],
  repo_status: null,
  loading: false,
  archiving: false,
  progress: "",

  fetchArchives: async (ringId) => {
    set({ loading: true })
    try {
      const data = await api.listArchives(ringId)
      set({ archives: data.archives })
    } finally {
      set({ loading: false })
    }
  },

  fetchQueue: async (ringId) => {
    const data = await api.getArchiveQueue(ringId)
    set({ queue: data.queue })
  },

  fetchRepoStatus: async (ringId) => {
    const status = await api.getRepoStatus(ringId)
    set({ repo_status: status })
  },

  triggerArchive: async (ringId, content, title, sessionId) => {
    set({ archiving: true, progress: "" })
    try {
      await api.triggerArchiveSSE(
        ringId,
        {
          session_id: sessionId,
          content,
          suggested_title: title,
          node_suggestion: { action: "create_new", node_title: title },
        },
        (event) => set({ progress: event.message }),
        () => {},
        () => {},
      )
      await get().fetchArchives(ringId)
    } finally {
      set({ archiving: false, progress: "" })
    }
  },

  reviewArchive: async (ringId, archiveId, action) => {
    await api.reviewArchive(ringId, archiveId, action)
    await Promise.all([
      get().fetchArchives(ringId),
      get().fetchQueue(ringId),
    ])
  },

  initRepo: async (ringId) => {
    const status = await api.initRepo(ringId)
    set({ repo_status: status })
  },
}))
```

- [ ] **Step 4: Verify frontend compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No type errors.

- [ ] **Step 5: Commit**

```bash
git add ui/src/types/archive.ts ui/src/stores/archive-store.ts ui/src/services/api.ts
git commit -m "feat: add frontend archive types, store, and API calls"
```

---

## Task 6-9: Frontend ArchivePanel Component

**Files:**
- Create: `ui/src/components/panels/ArchivePanel.tsx`

- [ ] **Step 1: Write ArchivePanel.tsx**

This component follows the IceChat CLI aesthetic established in the project. Create `ui/src/components/panels/ArchivePanel.tsx`:

```tsx
import { useEffect, useState } from "react"
import { useArchiveStore } from "../../stores/archive-store"
import type { ArchiveRecord } from "../../types/archive"

interface ArchivePanelProps {
  ringId: string
}

const STATUS_LABELS: Record<string, string> = {
  pending: "⏳ 待处理",
  committed: "📝 已提交",
  pushed: "✅ 已推送",
  mr_opened: "🔀 MR 待审核",
  merged: "✅ 已合并",
  rejected: "❌ 已拒绝",
}

export function ArchivePanel({ ringId }: ArchivePanelProps) {
  const {
    archives,
    queue,
    repo_status,
    loading,
    archiving,
    progress,
    fetchArchives,
    fetchQueue,
    fetchRepoStatus,
    reviewArchive,
    initRepo,
  } = useArchiveStore()

  const [selected, setSelected] = useState<ArchiveRecord | null>(null)

  useEffect(() => {
    fetchArchives(ringId)
    fetchQueue(ringId)
    fetchRepoStatus(ringId)
  }, [ringId])

  if (repo_status && !repo_status.initialized) {
    return (
      <div className="archive-panel">
        <div className="archive-empty">
          <p>Git 仓库未初始化</p>
          <button onClick={() => initRepo(ringId)}>初始化仓库</button>
        </div>
      </div>
    )
  }

  return (
    <div className="archive-panel">
      <div className="archive-header">
        <span className="archive-title">归档</span>
        {archiving && <span className="archive-progress">{progress}</span>}
      </div>

      <div className="archive-content">
        <div className="archive-list">
          {loading ? (
            <div className="archive-loading">加载中...</div>
          ) : archives.length === 0 ? (
            <div className="archive-empty">暂无归档</div>
          ) : (
            archives.map((a) => (
              <div
                key={a.id}
                className={`archive-item ${selected?.id === a.id ? "selected" : ""}`}
                onClick={() => setSelected(a)}
              >
                <span className="archive-item-status">
                  {STATUS_LABELS[a.status] || a.status}
                </span>
                <span className="archive-item-name">{a.file_name}</span>
                <span className="archive-item-date">{a.created_at.slice(0, 10)}</span>
              </div>
            ))
          )}
        </div>

        <div className="archive-detail">
          {selected ? (
            <div>
              <h3>{selected.file_name}</h3>
              <div className="archive-meta">
                <span>状态: {STATUS_LABELS[selected.status]}</span>
                <span>归档者: {selected.archived_by}</span>
                {selected.commit_sha && (
                  <span>Commit: {selected.commit_sha.slice(0, 8)}</span>
                )}
                {selected.merge_request_iid && (
                  <span>MR !{selected.merge_request_iid}</span>
                )}
              </div>
            </div>
          ) : (
            <div className="archive-empty">选择归档查看详情</div>
          )}
        </div>
      </div>

      {queue.length > 0 && (
        <div className="archive-queue">
          <div className="archive-queue-header">
            PR 审核队列 ({queue.length})
          </div>
          {queue.map((mr) => (
            <div key={mr.id} className="archive-queue-item">
              <span>{mr.file_name}</span>
              <div className="archive-queue-actions">
                <button
                  className="btn-merge"
                  onClick={() => reviewArchive(ringId, mr.id, "merge")}
                >
                  合并
                </button>
                <button
                  className="btn-reject"
                  onClick={() => reviewArchive(ringId, mr.id, "reject")}
                >
                  拒绝
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Verify frontend compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No type errors.

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/panels/ArchivePanel.tsx
git commit -m "feat: add ArchivePanel component — list, detail, PR review queue"
```

---

## Task 6-10: Frontend CLI Commands for Archive

**Files:**
- Modify: `ui/src/stores/chat-store.ts` — add `/archive` command handling

- [ ] **Step 1: Read current chat-store.ts to find command handling pattern**

Read `ui/src/stores/chat-store.ts` to find where CLI commands are parsed (e.g., `/node` command handling).

- [ ] **Step 2: Add /archive commands**

In the command handling section of the chat store, add handling for these commands:

- `/archive <title>` — calls `archiveStore.triggerArchive(ringId, recentMessages, title)`
- `/archive list` — opens ArchivePanel (sets active panel)
- `/archive queue` — opens ArchivePanel focused on queue
- `/archive review <id> merge|reject` — calls `archiveStore.reviewArchive`

The exact integration depends on the existing command dispatch pattern in chat-store. Follow the existing pattern (match on command prefix, extract args, call store actions).

- [ ] **Step 3: Verify compilation**

Run: `cd ui && npx tsc --noEmit`
Expected: No type errors.

- [ ] **Step 4: Commit**

```bash
git add ui/src/stores/chat-store.ts
git commit -m "feat: add /archive CLI commands — trigger, list, queue, review"
```

---

## Task 6-11: Integration Test — Archive CRUD + Git Operations

**Files:**
- Modify: `server/tests/integration.rs` — add archive tests

- [ ] **Step 1: Add archive integration tests**

Append to `server/tests/integration.rs`:

```rust
#[tokio::test]
async fn test_archive_repo_init() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/repo/status"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["initialized"], false);

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/repo/init"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["initialized"], true);
}

#[tokio::test]
async fn test_archive_list_empty() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/archives"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["archives"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_archive_queue_empty() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/archive-queue"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["queue"].as_array().unwrap().len(), 0);
}
```

- [ ] **Step 2: Run all tests**

Run: `cd server && cargo test`
Expected: All tests pass including new archive tests.

- [ ] **Step 3: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test: add archive integration tests — repo init, list, queue"
```

---

## Self-Review Checklist

**1. Spec coverage:**

| Spec Section | Task |
|---|---|
| 2.1 Disk directory structure | 6-3, 6-6 (init_ring_repo) |
| 2.2 DB changes (archive_records + markdown_path) | 6-0, 6-1, 6-5 |
| 2.3 Status machine | 6-6 (archive_service functions) |
| 3.2 git_service.rs | 6-3 |
| 3.3 gitlab_service.rs | 6-4 |
| 3.4 archive_service.rs | 6-6 |
| 3.5 Archive flow (creator + member) | 6-6 |
| 4.1 API endpoints | 6-7 |
| 4.4 SSE progress | 6-7 (trigger_archive) |
| 5.1 Setup extension | 6-2 (rings_dir in AppState — Setup already collects gitlab_url/token in users table) |
| 5.2 Ring Git init | 6-6 (init_ring_repo) |
| 5.4 MR creation | 6-4 (create_mr) + 6-6 (archive_content_member) |
| 5.5 MR review | 6-4 (merge/close) + 6-6 (review_mr) + 6-7 (review_archive) |
| 6.1 Frontend types/store | 6-8 |
| 6.2 CLI commands | 6-10 |
| 6.3 ArchivePanel UI | 6-9 |
| 7. Error handling | 6-2 |
| 8. Test | 6-11 |

**2. Placeholder scan:** No TBD/TODO/"fill in details". Task 6-10 has a note to "follow existing pattern" — acceptable as it requires reading the current code first.

**3. Type consistency:** Checked — `ArchiveRecord` fields match between model, service, and route. `ReviewAction` is `merge`/`reject` consistently. `NodeSuggestionInput` matches the API contract.

**4. Missing items noted:** Task 6-10 (CLI commands) and Task 6-11 (tests) reference patterns that need to be checked against actual current code at implementation time. The steps include "read current code" as step 1.
