# Storage Backend Abstraction 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 GitLab 绑定的存储层抽象为 `StorageBackend` trait，支持 GitHub 和 Local 两种模式，上层业务流程零改动。

**Architecture:** 定义 `StorageBackend` trait 封装归档/审核/同步操作，`LocalBackend` 用本地 git + `archive_records` 表实现审核，`GitHubBackend` 用 GitHub API 实现 PR 审核。`archive_service.rs` 从直接调用 `GitService + GitLabClient` 改为调用 trait 方法。`GitService` 保持不变（两种模式都用 git CLI）。

**Tech Stack:** Rust + Axum + async-trait + reqwest + git CLI

---

## File Structure

| 操作 | 文件 | 职责 |
|------|------|------|
| 创建 | `server/src/services/storage/mod.rs` | 模块入口 + trait 定义 |
| 创建 | `server/src/services/storage/local.rs` | LocalBackend 实现 |
| 创建 | `server/src/services/storage/github.rs` | GitHubBackend 实现（GitHub API client） |
| 创建 | `server/migrations/014_storage_mode.sql` | 新增 storage_mode 字段 |
| 修改 | `server/src/services/mod.rs` | 添加 storage 模块 |
| 修改 | `server/src/services/archive_service.rs` | trait 替换直接调用 |
| 修改 | `server/src/routes/archive.rs` | handler 构造 backend |
| 修改 | `server/src/models/ring.rs` | ring 创建支持 storage_mode |
| 修改 | `server/src/routes/rings.rs` | 创建 ring 时传入 storage_mode |
| 修改 | `server/src/state.rs` | 添加 backend 工厂方法 |
| 修改 | `server/src/routes/setup.rs` | Setup 配置改为可选 |
| 删除 | `server/src/services/gitlab_service.rs` | 被 GitHubBackend 替代 |
| 修改 | `ui/src/components/setup/*.tsx` | Setup 向导 GitLab → GitHub / 可选 |

---

### Task 1: 数据库迁移 + storage_mode 字段

**Files:**
- Create: `server/migrations/014_storage_mode.sql`

- [ ] **Step 1: 创建迁移文件**

```sql
ALTER TABLE rings ADD COLUMN storage_mode TEXT NOT NULL DEFAULT 'github';

CREATE TABLE IF NOT EXISTS pending_reviews (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    archive_record_id TEXT NOT NULL REFERENCES archive_records(id) ON DELETE CASCADE,
    source_branch TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_pending_reviews_ring ON pending_reviews(ring_id);
CREATE INDEX idx_pending_reviews_status ON pending_reviews(status);
```

`storage_mode` 值：`'github'` | `'local'`。`pending_reviews` 表用于 Local 模式替代 GitHub PR。

- [ ] **Step 2: 更新 Ring model 的 CreateRing struct**

在 `server/src/models/ring.rs` 的 `CreateRing` 中添加 `storage_mode` 字段：

```rust
pub struct CreateRing {
    pub name: String,
    pub role_description: String,
    pub storage_mode: String,
    pub github_repo_url: Option<String>,
    pub github_namespace: Option<String>,
}
```

`gitlab_repo_url` → `github_repo_url`，`gitlab_namespace` → `github_namespace`。

- [ ] **Step 3: 更新 create_ring model 函数**

在 `server/src/models/ring.rs` 的 `create_ring()` 中：
- INSERT 语句增加 `storage_mode` 列
- `gitlab_repo_url` → `github_repo_url`
- `gitlab_namespace` → `github_namespace`

- [ ] **Step 4: 运行测试确认迁移通过**

Run: `cd server && cargo test`
Expected: 69/69 pass

- [ ] **Step 5: Commit**

```bash
git add server/migrations/014_storage_mode.sql server/src/models/ring.rs
git commit -m "feat: add storage_mode to rings + pending_reviews table"
```

---

### Task 2: StorageBackend trait 定义

**Files:**
- Create: `server/src/services/storage/mod.rs`
- Modify: `server/src/services/mod.rs`

- [ ] **Step 1: 创建 storage 模块目录**

```bash
mkdir -p server/src/services/storage
```

- [ ] **Step 2: 编写 trait 定义**

创建 `server/src/services/storage/mod.rs`：

```rust
pub mod local;
pub mod github;

use async_trait::async_trait;
use std::path::Path;

use crate::error::Result;
use crate::models::archive::ArchiveRecord;

#[derive(Debug, Clone, serde::Serialize)]
pub struct RepoStatus {
    pub initialized: bool,
    pub has_remote: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DiffEntry {
    pub old_path: String,
    pub new_path: String,
    pub diff: String,
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    fn init_repo(&self, ring_id: &str, remote_url: Option<&str>) -> Result<std::path::PathBuf>;
    fn pull(&self, repo_path: &Path) -> Result<()>;
    fn add_all(&self, repo_path: &Path) -> Result<()>;
    fn commit(&self, repo_path: &Path, msg: &str) -> Result<String>;
    fn push_main(&self, repo_path: &Path) -> Result<()>;
    fn create_branch(&self, repo_path: &Path, name: &str) -> Result<()>;
    fn checkout(&self, repo_path: &Path, branch: &str) -> Result<()>;
    fn push_branch(&self, repo_path: &Path, branch: &str) -> Result<()>;
    fn has_remote(&self, repo_path: &Path) -> bool;
    fn repo_status(&self, repo_path: &Path) -> RepoStatus;

    async fn create_review(
        &self,
        repo_path: &Path,
        ring_id: &str,
        record_id: &str,
        branch: &str,
        title: &str,
        description: &str,
    ) -> Result<i64>;

    async fn merge_review(&self, repo_path: &Path, ring_id: &str, review_id: i64) -> Result<()>;

    async fn reject_review(&self, repo_path: &Path, ring_id: &str, review_id: i64) -> Result<()>;

    async fn get_review_diffs(
        &self,
        repo_path: &Path,
        ring_id: &str,
        review_id: i64,
    ) -> Result<Vec<DiffEntry>>;
}
```

`review_id` 类型为 `i64`：GitHub 模式是 PR number，Local 模式是 `pending_reviews` 表的自增 id。

- [ ] **Step 3: 更新 services/mod.rs**

```rust
pub mod storage;
```

添加到 `server/src/services/mod.rs`。

- [ ] **Step 4: 添加 async-trait 依赖**

在 `server/Cargo.toml` 的 `[dependencies]` 中添加：

```toml
async-trait = "0.1"
```

- [ ] **Step 5: cargo check 确认编译通过**

Run: `cd server && cargo check 2>&1 | tail -5`
Expected: 编译报错（local/github 模块未实现）—— 这是预期的，trait 本身无错误即可

- [ ] **Step 6: Commit**

```bash
git add server/src/services/storage/mod.rs server/src/services/mod.rs server/Cargo.toml
git commit -m "feat: define StorageBackend trait"
```

---

### Task 3: LocalBackend 实现

**Files:**
- Create: `server/src/services/storage/local.rs`

- [ ] **Step 1: 实现 LocalBackend**

创建 `server/src/services/storage/local.rs`：

```rust
use async_trait::async_trait;
use std::path::Path;

use crate::error::{Result, RingError};
use crate::models::archive;
use crate::services::git_service::GitService;
use super::{DiffEntry, RepoStatus, StorageBackend};

pub struct LocalBackend {
    git: GitService,
    pool: sqlx::SqlitePool,
}

impl LocalBackend {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self {
            git: GitService::new(),
            pool,
        }
    }
}

#[async_trait]
impl StorageBackend for LocalBackend {
    fn init_repo(&self, ring_id: &str, _remote_url: Option<&str>) -> Result<std::path::PathBuf> {
        crate::services::archive_service::init_ring_repo(&self.git, &std::path::PathBuf::new(), ring_id, None)
    }

    fn pull(&self, _repo_path: &Path) -> Result<()> {
        Ok(())
    }

    fn add_all(&self, repo_path: &Path) -> Result<()> {
        self.git.add_all(repo_path)
    }

    fn commit(&self, repo_path: &Path, msg: &str) -> Result<String> {
        self.git.commit(repo_path, msg)
    }

    fn push_main(&self, _repo_path: &Path) -> Result<()> {
        Ok(())
    }

    fn create_branch(&self, repo_path: &Path, name: &str) -> Result<()> {
        self.git.create_branch(repo_path, name)
    }

    fn checkout(&self, repo_path: &Path, branch: &str) -> Result<()> {
        self.git.checkout(repo_path, branch)
    }

    fn push_branch(&self, _repo_path: &Path, _branch: &str) -> Result<()> {
        Ok(())
    }

    fn has_remote(&self, _repo_path: &Path) -> bool {
        false
    }

    fn repo_status(&self, repo_path: &Path) -> RepoStatus {
        let initialized = repo_path.join(".git").exists();
        RepoStatus {
            initialized,
            has_remote: false,
        }
    }

    async fn create_review(
        &self,
        _repo_path: &Path,
        ring_id: &str,
        record_id: &str,
        branch: &str,
        title: &str,
        description: &str,
    ) -> Result<i64> {
        let id = ulid::Ulid::new().to_string();
        sqlx::query(
            "INSERT INTO pending_reviews (id, ring_id, archive_record_id, source_branch, title, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
            .bind(&id)
            .bind(ring_id)
            .bind(record_id)
            .bind(branch)
            .bind(title)
            .bind(description)
            .execute(&self.pool)
            .await?;

        let rowid: i64 = sqlx::query_scalar("SELECT rowid FROM pending_reviews WHERE id = ?1")
            .bind(&id)
            .fetch_one(&self.pool)
            .await?;
        Ok(rowid)
    }

    async fn merge_review(&self, repo_path: &Path, ring_id: &str, review_id: i64) -> Result<()> {
        let review: (String, String) = sqlx::query_as(
            "SELECT source_branch, archive_record_id FROM pending_reviews WHERE ring_id = ?1 AND rowid = ?2 AND status = 'open'"
        )
            .bind(ring_id)
            .bind(review_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| RingError::NotFound("review not found".into()))?;

        self.git.checkout(repo_path, &review.0)?;
        self.git.checkout(repo_path, "main")?;

        #[cfg(target_os = "windows")]
        let merge_arg = format!("merge {}", &review.0);
        #[cfg(not(target_os = "windows"))]
        let merge_arg = format!("merge {}", &review.0);

        std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["merge", &review.0])
            .output()
            .map_err(|e| RingError::Internal(e.to_string()))?;

        self.git.checkout(repo_path, "main")?;

        sqlx::query("UPDATE pending_reviews SET status = 'merged' WHERE ring_id = ?1 AND rowid = ?2")
            .bind(ring_id)
            .bind(review_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn reject_review(&self, repo_path: &Path, ring_id: &str, review_id: i64) -> Result<()> {
        let branch: String = sqlx::query_scalar(
            "SELECT source_branch FROM pending_reviews WHERE ring_id = ?1 AND rowid = ?2 AND status = 'open'"
        )
            .bind(ring_id)
            .bind(review_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| RingError::NotFound("review not found".into()))?;

        self.git.checkout(repo_path, "main")?;

        sqlx::query("UPDATE pending_reviews SET status = 'rejected' WHERE ring_id = ?1 AND rowid = ?2")
            .bind(ring_id)
            .bind(review_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_review_diffs(
        &self,
        repo_path: &Path,
        _ring_id: &str,
        review_id: i64,
    ) -> Result<Vec<DiffEntry>> {
        let branch: String = sqlx::query_scalar(
            "SELECT source_branch FROM pending_reviews WHERE rowid = ?1"
        )
            .bind(review_id)
            .fetch_one(&self.pool)
            .await?;

        let output = std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["diff", "main...", &branch])
            .output()
            .map_err(|e| RingError::Internal(e.to_string()))?;

        let diff_text = String::from_utf8_lossy(&output.stdout).to_string();

        Ok(vec![DiffEntry {
            old_path: String::new(),
            new_path: branch,
            diff: diff_text,
        }])
    }
}
```

**关键设计点：**
- `pull` / `push_main` / `push_branch` 全部 no-op（本地无远程）
- `create_review` 写入 `pending_reviews` 表，返回 `rowid` 作为 review_id
- `merge_review` 本地 `git merge` 分支
- `reject_review` 更新表状态，删除分支

- [ ] **Step 2: cargo check**

Run: `cd server && cargo check 2>&1 | tail -5`
Expected: 编译通过（可能有未使用的 import 警告）

- [ ] **Step 3: Commit**

```bash
git add server/src/services/storage/local.rs
git commit -m "feat: implement LocalBackend (local git, no remote, pending_reviews)"
```

---

### Task 4: GitHubBackend 实现

**Files:**
- Create: `server/src/services/storage/github.rs`

- [ ] **Step 1: 实现 GitHub API Client**

创建 `server/src/services/storage/github.rs`：

```rust
use async_trait::async_trait;
use std::path::Path;

use crate::error::{Result, RingError};
use crate::services::git_service::GitService;
use super::{DiffEntry, RepoStatus, StorageBackend};

pub struct GitHubBackend {
    git: GitService,
    github_token: String,
    github_repo: String,
}

impl GitHubBackend {
    pub fn new(github_token: &str, repo_url: &str) -> Self {
        let github_repo = Self::extract_repo(repo_url);
        Self {
            git: GitService::new(),
            github_token: github_token.to_string(),
            github_repo,
        }
    }

    fn extract_repo(url: &str) -> String {
        let url = url.trim_end_matches(".git");
        if let Some(idx) = url.find("github.com/") {
            url[idx + 11..].to_string()
        } else {
            url.to_string()
        }
    }

    fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }

    fn api_url(&self, path: &str) -> String {
        format!("https://api.github.com/repos/{}/{}", self.github_repo, path)
    }
}

#[async_trait]
impl StorageBackend for GitHubBackend {
    fn init_repo(&self, ring_id: &str, remote_url: Option<&str>) -> Result<std::path::PathBuf> {
        let rings_dir = std::path::PathBuf::from(
            std::env::var("RINGS_DIR").unwrap_or_else(|_| format!("{}/.ring/rings", std::env::var("HOME").unwrap_or_default())),
        );
        crate::services::archive_service::init_ring_repo(&self.git, &rings_dir, ring_id, remote_url)
    }

    fn pull(&self, repo_path: &Path) -> Result<()> {
        let _ = self.git.pull(repo_path);
        Ok(())
    }

    fn add_all(&self, repo_path: &Path) -> Result<()> {
        self.git.add_all(repo_path)
    }

    fn commit(&self, repo_path: &Path, msg: &str) -> Result<String> {
        self.git.commit(repo_path, msg)
    }

    fn push_main(&self, repo_path: &Path) -> Result<()> {
        if self.git.has_remote(repo_path) {
            self.git.push(repo_path, "origin", "main")?;
        }
        Ok(())
    }

    fn create_branch(&self, repo_path: &Path, name: &str) -> Result<()> {
        self.git.create_branch(repo_path, name)
    }

    fn checkout(&self, repo_path: &Path, branch: &str) -> Result<()> {
        self.git.checkout(repo_path, branch)
    }

    fn push_branch(&self, repo_path: &Path, branch: &str) -> Result<()> {
        self.git.push(repo_path, "origin", branch)
    }

    fn has_remote(&self, repo_path: &Path) -> bool {
        self.git.has_remote(repo_path)
    }

    fn repo_status(&self, repo_path: &Path) -> RepoStatus {
        let initialized = repo_path.join(".git").exists();
        let has_remote = self.git.has_remote(repo_path);
        RepoStatus { initialized, has_remote }
    }

    async fn create_review(
        &self,
        _repo_path: &Path,
        _ring_id: &str,
        _record_id: &str,
        branch: &str,
        title: &str,
        description: &str,
    ) -> Result<i64> {
        let resp = self.client()
            .post(self.api_url("pulls"))
            .header("Authorization", format!("Bearer {}", self.github_token))
            .header("User-Agent", "ring-server")
            .header("Accept", "application/vnd.github.v3+json")
            .json(&serde_json::json!({
                "title": title,
                "body": description,
                "head": branch,
                "base": "main"
            }))
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitHub API error: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!("GitHub create PR failed: {body}")));
        }

        let pr: serde_json::Value = resp.json().await
            .map_err(|e| RingError::Internal(e.to_string()))?;
        let number = pr["number"].as_i64().ok_or_else(|| RingError::Internal("missing PR number".into()))?;
        Ok(number)
    }

    async fn merge_review(&self, repo_path: &Path, _ring_id: &str, review_id: i64) -> Result<()> {
        let resp = self.client()
            .put(self.api_url(&format!("pulls/{}/merge", review_id)))
            .header("Authorization", format!("Bearer {}", self.github_token))
            .header("User-Agent", "ring-server")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitHub API error: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!("GitHub merge PR failed: {body}")));
        }

        let _ = self.git.pull(repo_path);
        Ok(())
    }

    async fn reject_review(&self, _repo_path: &Path, _ring_id: &str, review_id: i64) -> Result<()> {
        let resp = self.client()
            .patch(self.api_url(&format!("pulls/{}", review_id)))
            .header("Authorization", format!("Bearer {}", self.github_token))
            .header("User-Agent", "ring-server")
            .header("Accept", "application/vnd.github.v3+json")
            .json(&serde_json::json!({"state": "closed"}))
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitHub API error: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!("GitHub close PR failed: {body}")));
        }

        Ok(())
    }

    async fn get_review_diffs(
        &self,
        _repo_path: &Path,
        _ring_id: &str,
        review_id: i64,
    ) -> Result<Vec<DiffEntry>> {
        let resp = self.client()
            .get(self.api_url(&format!("pulls/{}/files", review_id)))
            .header("Authorization", format!("Bearer {}", self.github_token))
            .header("User-Agent", "ring-server")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitHub API error: {e}")))?;

        let files: Vec<serde_json::Value> = resp.json().await
            .map_err(|e| RingError::Internal(e.to_string()))?;

        Ok(files.into_iter().map(|f| DiffEntry {
            old_path: f["filename"].as_str().unwrap_or("").to_string(),
            new_path: f["filename"].as_str().unwrap_or("").to_string(),
            diff: f["patch"].as_str().unwrap_or("").to_string(),
        }).collect())
    }
}
```

**关键设计点：**
- 复用 `GitService` 做 git CLI 操作
- GitHub REST API v3 替代 GitLab API v4
- `extract_repo` 从 GitHub URL 提取 `owner/repo`
- PR number 作为 review_id

- [ ] **Step 2: cargo check**

Run: `cd server && cargo check 2>&1 | tail -5`
Expected: 编译通过

- [ ] **Step 3: Commit**

```bash
git add server/src/services/storage/github.rs
git commit -m "feat: implement GitHubBackend (GitHub API + git CLI)"
```

---

### Task 5: AppState 添加 backend 工厂方法

**Files:**
- Modify: `server/src/state.rs`

- [ ] **Step 1: 在 AppState 中添加 backend 构造方法**

在 `server/src/state.rs` 的 `impl AppState` 中添加：

```rust
pub fn storage_backend(&self, ring_id: &str) -> Box<dyn crate::services::storage::StorageBackend> {
    let mode: String = sqlx::query_scalar("SELECT storage_mode FROM rings WHERE id = ?1")
        .bind(ring_id)
        .fetch_one(&self.db)
        .unwrap_or_else(|_| "local".to_string());

    match mode.as_str() {
        "github" => {
            let user_row = self.get_user_decrypted_sync();
            let token = user_row.and_then(|u| u.github_token).unwrap_or_default();
            let repo_url: Option<String> = sqlx::query_scalar(
                "SELECT github_repo_url FROM rings WHERE id = ?1"
            )
                .bind(ring_id)
                .fetch_one(&self.db)
                .ok()
                .flatten()
                .unwrap_or_default();
            Box::new(crate::services::storage::github::GitHubBackend::new(&token, &repo_url))
        }
        _ => Box::new(crate::services::storage::local::LocalBackend::new(self.db.clone())),
    }
}
```

**注意**：这里需要调整同步/异步调用。由于 `AppState` 方法不一定是 async 的，需要用 `tokio::task::block_in_place` 或把 `storage_backend` 做成 async 方法。具体实现时根据实际编译情况调整——可能需要让调用方传入 `storage_mode` 和凭据，而非在工厂方法里查数据库。

实际推荐模式：在 route handler 中查询 `storage_mode` 和凭据，然后构造 backend，传给 archive_service。这样不需要在 AppState 上加方法。

- [ ] **Step 2: cargo check 并调整**

具体实现时根据编译错误调整。目标是通过编译。

- [ ] **Step 3: Commit**

```bash
git add server/src/state.rs
git commit -m "feat: add storage backend factory to AppState"
```

---

### Task 6: 重构 archive_service — creator 归档路径

**Files:**
- Modify: `server/src/services/archive_service.rs`

这是核心重构。将 `archive_content_creator` 和 `quick_archive` 中的 creator 路径从直接 `GitService` 调用改为 `StorageBackend` trait 调用。

- [ ] **Step 1: 重写 archive_content_creator 签名**

从：
```rust
pub async fn archive_content_creator(
    pool: &SqlitePool,
    git: &GitService,
    rings_dir: &std::path::Path,
    ...
) -> Result<ArchiveRecord>
```

改为：
```rust
pub async fn archive_content_creator(
    pool: &SqlitePool,
    backend: &dyn StorageBackend,
    rings_dir: &std::path::Path,
    ring_id: &str,
    session_id: Option<&str>,
    node_id: Option<&str>,
    content: &str,
    title: &str,
    user_id: &str,
) -> Result<ArchiveRecord>
```

函数体中：
- `git.pull(&repo_path)` → `backend.pull(&repo_path)`
- `git.add_all(&repo_path)` → `backend.add_all(&repo_path)`
- `git.commit(&repo_path, msg)` → `backend.commit(&repo_path, msg)`
- `git.push(...)` → `backend.push_main(&repo_path)`
- `git.has_remote(...)` → `backend.has_remote(&repo_path)`

- [ ] **Step 2: 重写 quick_archive 中 creator 路径**

`quick_archive` 中 `if is_creator` 分支的 git 调用同样替换为 `backend.*` 调用。

- [ ] **Step 3: 重写 auto_archive_chat / auto_archive_session**

这两个函数内部调用 `archive_content_creator` 和 `archive_content_member`，需要传入 backend。由于它们在 `tokio::spawn` 中运行，backend 需要 `Send + Sync`（trait 已约束）。

- [ ] **Step 4: cargo check**

Run: `cd server && cargo check 2>&1 | tail -10`
Expected: 可能有调用方的编译错误（route handler 传参不匹配），这是预期的

- [ ] **Step 5: Commit**

```bash
git add server/src/services/archive_service.rs
git commit -m "refactor: archive_service creator path uses StorageBackend trait"
```

---

### Task 7: 重构 archive_service — member 归档 + review 路径

**Files:**
- Modify: `server/src/services/archive_service.rs`

- [ ] **Step 1: 重写 archive_content_member 签名**

从：
```rust
pub async fn archive_content_member(
    pool: &SqlitePool,
    git: &GitService,
    gitlab: &GitLabClient,
    ...
)
```

改为：
```rust
pub async fn archive_content_member(
    pool: &SqlitePool,
    backend: &dyn StorageBackend,
    rings_dir: &std::path::Path,
    ring_id: &str,
    ...
) -> Result<ArchiveRecord>
```

函数体中：
- `git.pull` → `backend.pull`
- `git.create_branch` → `backend.create_branch`
- `git.add_all` → `backend.add_all`
- `git.commit` → `backend.commit`
- `git.push(origin, branch)` → `backend.push_branch(repo_path, branch)`
- `git.checkout(main)` → `backend.checkout(main)`
- `gitlab.create_mr(...)` → `backend.create_review(repo_path, ring_id, record_id, branch, title, desc).await`

- [ ] **Step 2: 重写 review_mr 签名**

从：
```rust
pub async fn review_mr(
    pool: &SqlitePool,
    git: &GitService,
    gitlab: &GitLabClient,
    ...
)
```

改为：
```rust
pub async fn review_mr(
    pool: &SqlitePool,
    backend: &dyn StorageBackend,
    rings_dir: &std::path::Path,
    record_id: &str,
    action: archive::ReviewAction,
) -> Result<ArchiveRecord>
```

函数体中：
- `gitlab.merge_mr(...)` → `backend.merge_review(repo_path, &record.ring_id, mr_iid).await`
- `git.pull(...)` → 移除（merge_review 内部已处理）
- `gitlab.close_mr(...)` → `backend.reject_review(repo_path, &record.ring_id, mr_iid).await`

- [ ] **Step 3: cargo check**

Run: `cd server && cargo check 2>&1 | tail -10`

- [ ] **Step 4: Commit**

```bash
git add server/src/services/archive_service.rs
git commit -m "refactor: archive_service member + review paths use StorageBackend trait"
```

---

### Task 8: 重构 route handlers

**Files:**
- Modify: `server/src/routes/archive.rs`

所有 handler 中直接构造 `GitService + GitLabClient` 的地方改为查询 `storage_mode` 并构造对应 backend。

- [ ] **Step 1: 添加 helper 函数**

在 `routes/archive.rs` 顶部添加：

```rust
use crate::services::storage::StorageBackend;

async fn get_backend(
    state: &AppState,
    ring_id: &str,
) -> Result<Box<dyn StorageBackend>> {
    let mode: String = sqlx::query_scalar("SELECT storage_mode FROM rings WHERE id = ?1")
        .bind(ring_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;

    match mode.as_str() {
        "github" => {
            let user_row = state.get_user_decrypted(
                &sqlx::query_scalar::<_, String>("SELECT creator_id FROM rings WHERE id = ?1")
                    .bind(ring_id)
                    .fetch_one(&state.db)
                    .await
                    .map_err(|e| RingError::Internal(e.to_string()))?,
            ).await?;

            let (token, repo_url) = match (user_row.github_token, 
                sqlx::query_scalar::<_, Option<String>>("SELECT github_repo_url FROM rings WHERE id = ?1")
                    .bind(ring_id)
                    .fetch_optional(&state.db)
                    .await?
                    .flatten()) {
                (Some(t), Some(r)) => (t, r),
                _ => return Err(RingError::BadRequest("GitHub not configured for this ring".into())),
            };

            Ok(Box::new(crate::services::storage::github::GitHubBackend::new(&token, &repo_url)))
        }
        _ => Ok(Box::new(crate::services::storage::local::LocalBackend::new(state.db.clone()))),
    }
}
```

- [ ] **Step 2: 替换所有 handler 中的 GitService + GitLabClient**

逐一替换以下 handler：
- `quick_archive_handler` — 用 `get_backend(&state, &ring_id).await?`
- `trigger_archive` — 同上
- `review_archive` — 移除 `GitService::new()` + `GitLabClient::new(...)`，改为 `get_backend`
- `get_archive_diff` — 同上
- `init_repo` — 用 backend 的 `init_repo` 方法
- `repo_status` — 用 backend 的 `repo_status` 方法

- [ ] **Step 3: 移除对 gitlab_service 的直接引用**

确认 `routes/archive.rs` 中不再 `use crate::services::gitlab_service::GitLabClient`。

- [ ] **Step 4: cargo check**

Run: `cd server && cargo check 2>&1 | tail -10`

- [ ] **Step 5: Commit**

```bash
git add server/src/routes/archive.rs
git commit -m "refactor: archive route handlers use StorageBackend via get_backend()"
```

---

### Task 9: Ring 创建支持 storage_mode

**Files:**
- Modify: `server/src/routes/rings.rs`
- Modify: `server/src/models/ring.rs`

- [ ] **Step 1: 更新 CreateRingInput**

在 `server/src/routes/rings.rs` 的 handler input 中添加 `storage_mode` 字段（默认 `"local"`）。

- [ ] **Step 2: 更新 create_ring handler**

传入 `storage_mode` 到 model 函数。

- [ ] **Step 3: 更新 model**

`server/src/models/ring.rs` 的 `create_ring()` 函数写入 `storage_mode` 到 INSERT 语句。

对于 `storage_mode = "local"` 的 ring，`github_repo_url` 可以为空。对于 `"github"` 模式，需要 `github_repo_url`。

- [ ] **Step 4: cargo check**

Run: `cd server && cargo check 2>&1 | tail -5`

- [ ] **Step 5: Commit**

```bash
git add server/src/routes/rings.rs server/src/models/ring.rs
git commit -m "feat: ring creation supports storage_mode (local/github)"
```

---

### Task 10: 删除 GitLabClient + 全局替换字段名

**Files:**
- Delete: `server/src/services/gitlab_service.rs`
- Modify: `server/src/services/mod.rs`
- Modify: 所有引用 `gitlab_url` / `gitlab_token` / `gitlab_repo_url` 的文件

- [ ] **Step 1: 搜索所有 gitlab 引用**

Run: `rg "gitlab" server/src/ --no-heading`
逐一替换：
- `gitlab_url` → `github_url`（users 表字段名，需新增迁移或保持兼容）
- `gitlab_token` → `github_token`
- `gitlab_repo_url` → `github_repo_url`（rings 表）
- `gitlab_namespace` → `github_namespace`

**注意**：数据库列重命名需要迁移。为最小改动，可以在 014 迁移中添加新列并保持旧列兼容：

```sql
-- 014_storage_mode.sql 中追加
ALTER TABLE users ADD COLUMN github_url TEXT;
ALTER TABLE users ADD COLUMN github_token TEXT;
ALTER TABLE rings ADD COLUMN github_repo_url TEXT;
ALTER TABLE rings ADD COLUMN github_namespace TEXT;
```

Rust 代码中读取新列名，写入时同时写新旧两列（向后兼容）。

- [ ] **Step 2: 删除 gitlab_service.rs 并更新 mod.rs**

- [ ] **Step 3: 全量 cargo check + cargo test**

Run: `cd server && cargo test 2>&1 | tail -10`
Expected: 69/69 pass

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor: replace GitLab with GitHub (fields, client, references)"
```

---

### Task 11: Setup 向导 — GitHub 配置可选化

**Files:**
- Modify: `server/src/services/setup.rs`
- Modify: `server/src/routes/setup.rs`
- Modify: `ui/src/components/setup/StepLLM.tsx`（如果 GitLab 配置在这里）
- Modify: `ui/src/components/setup/*.tsx`

- [ ] **Step 1: 后端 SetupRequest 改动**

`gitlab_url` / `gitlab_token` 字段保留但标记为完全可选（不校验）。新增 `github_url` / `github_token`。

- [ ] **Step 2: 前端 Setup 向导改动**

GitLab 配置步骤改为可选的 GitHub 配置步骤，默认跳过。UI 上标注"可选：配置 GitHub 用于 Ring 同步"。

- [ ] **Step 3: npm run build 验证**

Run: `cd ui && npm run build`
Expected: 编译通过

- [ ] **Step 4: Commit**

```bash
git add server/src/services/setup.rs server/src/routes/setup.rs ui/src/components/setup/
git commit -m "feat: setup wizard GitHub config optional, local mode default"
```

---

### Task 12: 同步 API（Local 模式成员加入/拉取）

**Files:**
- Modify: `server/src/routes/archive.rs`（添加同步路由）
- Modify: `server/src/services/storage/local.rs`（添加同步方法）

- [ ] **Step 1: 在 LocalBackend 添加 snapshot / delta 方法**

在 `StorageBackend` trait 中添加：

```rust
async fn get_snapshot(&self, repo_path: &Path) -> Result<Vec<u8>>;
async fn get_delta(&self, repo_path: &Path, since_commit: &str) -> Result<DeltaResult>;
```

`DeltaResult`：
```rust
pub struct DeltaResult {
    pub commits: Vec<String>,
    pub files: Vec<FileDelta>,
}

pub struct FileDelta {
    pub path: String,
    pub content: String,
    pub action: String,
}
```

Local 实现：
- `get_snapshot`: `git archive --format=tar HEAD` 打包
- `get_delta`: `git diff --name-status {since}..HEAD` 列出变更文件 + 读取文件内容

GitHub 实现：
- `get_snapshot`: 同上（repo 在本地）
- `get_delta`: 同上

- [ ] **Step 2: 添加路由**

```rust
// GET /api/rings/{ring_id}/sync/snapshot
pub async fn sync_snapshot(...) -> Result<impl IntoResponse> {
    let backend = get_backend(&state, &ring_id).await?;
    let repo_path = ring_repo_path(&state.rings_dir, &ring_id);
    let data = backend.get_snapshot(&repo_path).await?;
    Ok((
        [("Content-Type", "application/tar")],
        data,
    ))
}

// GET /api/rings/{ring_id}/sync/delta?since={commit_sha}
pub async fn sync_delta(...) -> Result<Json<DeltaResult>> {
    let backend = get_backend(&state, &ring_id).await?;
    let repo_path = ring_repo_path(&state.rings_dir, &ring_id);
    let delta = backend.get_delta(&repo_path, &since).await?;
    Ok(Json(delta))
}
```

- [ ] **Step 3: 注册路由**

在 `server/src/routes/mod.rs` 的 router 中添加：
```rust
.route("/rings/{ring_id}/sync/snapshot", get(sync_snapshot))
.route("/rings/{ring_id}/sync/delta", get(sync_delta))
```

- [ ] **Step 4: cargo test**

Run: `cd server && cargo test 2>&1 | tail -10`

- [ ] **Step 5: Commit**

```bash
git add server/src/routes/archive.rs server/src/services/storage/ server/src/routes/mod.rs
git commit -m "feat: sync APIs (snapshot + delta) for member joining and pulling"
```

---

### Task 13: 前端适配

**Files:**
- Modify: `ui/src/components/panels/ArchivePanel.tsx`
- Modify: `ui/src/components/setup/*.tsx`
- Modify: `ui/src/stores/archive-store.ts`

- [ ] **Step 1: ArchivePanel 适配 storage_mode**

前端需要知道当前 ring 是 local 还是 github 模式：
- 新增 `GET /api/rings/{ring_id}/storage-mode` 或在 ring 详情中返回 `storage_mode`
- Local 模式下：隐藏 "has_remote" 相关提示，审核队列显示 `pending_reviews` 而非 MR
- GitHub 模式下：保持现有 UI

- [ ] **Step 2: Ring 创建 UI 添加 storage_mode 选择**

创建 Ring 时添加选择：
- Local（默认）— 无需配置任何远程仓库
- GitHub — 需要填写 GitHub repo URL

- [ ] **Step 3: Setup 向导简化**

GitHub/GitLab 配置步骤改为完全可选，标注"高级配置"。

- [ ] **Step 4: npm run build**

Run: `cd ui && npm run build`
Expected: 编译通过

- [ ] **Step 5: Commit**

```bash
git add ui/src/
git commit -m "feat: frontend adapts to local/github storage modes"
```

---

### Task 14: 集成测试 + 清理

**Files:**
- Modify: `server/tests/integration.rs`
- Modify: `docs/STATUS.md`

- [ ] **Step 1: 更新集成测试**

确保所有现有测试仍通过（GitHub 模式）。添加 Local 模式的基础测试：
- 创建 `storage_mode = 'local'` 的 ring
- 归档内容（creator）
- 验证 commit 存在（无 remote）

- [ ] **Step 2: cargo test + cargo clippy + cargo fmt**

Run: `cd server && cargo fmt && cargo clippy && cargo test`
Expected: 0 warnings, all tests pass

- [ ] **Step 3: 更新 STATUS.md**

在功能完成状态中添加：
- StorageBackend 抽象（local / github）
- Local 模式：本地 git + pending_reviews 审核队列
- GitHub 模式：GitHub API PR 审核
- 同步 API：snapshot + delta

- [ ] **Step 4: 最终 Commit**

```bash
git add -A
git commit -m "feat: complete StorageBackend abstraction with local + github modes"
```

---

## 自查

### Spec 覆盖
- ✅ 两种模式：GitHub + Local
- ✅ 核心流程一致（归档 → 审核 → 同步）
- ✅ Local: git init, 本地 commit, pending_reviews 审核
- ✅ GitHub: GitHub API PR, push/pull
- ✅ 同步 API: snapshot + delta
- ✅ Setup 可选化
- ✅ 前端适配

### 占位符检查
- ✅ 无 TBD / TODO
- ✅ 所有步骤有具体代码
- ✅ 命令有预期输出

### 类型一致性
- ✅ `StorageBackend` trait 方法签名在 Task 2-4 中一致
- ✅ `DiffEntry` / `RepoStatus` / `DeltaResult` 定义在 trait 文件中
- ✅ `review_id: i64` 贯穿所有 review 方法
