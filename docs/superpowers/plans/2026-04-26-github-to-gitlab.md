# GitHub → GitLab 回退 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将上个 session 错误引入的 GitHub 相关代码全部回退为 GitLab（公司内网），保持 StorageBackend 架构不变，只改具体实现和命名。

**Architecture:** StorageBackend trait 保持不变。`LocalBackend` 保持不变。`GitHubBackend` 重命名为 `GitLabBackend`，内部 API 调用改为 GitLab REST API v4。DB 中已有的 `gitlab_*` 列继续使用，删除 014 migration 新增的 `github_*` 列。

**Tech Stack:** Rust + Axum + SQLite + GitLab REST API v4 + React + TypeScript

---

## 改动范围

### 命名映射

| 旧 (GitHub) | 新 (GitLab) |
|---|---|
| `GitHubBackend` | `GitLabBackend` |
| `github.rs` | `gitlab.rs` |
| `storage_mode = "github"` | `storage_mode = "gitlab"` |
| `github_repo_url` (rings) | 复用已有 `gitlab_repo_url` |
| `github_namespace` (rings) | 复用已有 `gitlab_namespace` |
| `github_url` (users) | 复用已有 `gitlab_url` |
| `github_token` (users) | 复用已有 `gitlab_token` |
| 前端 `StorageMode = 'github'` | `StorageMode = 'gitlab'` |
| 前端 `github_repo_url` | `gitlab_repo_url` |
| UI labels "GitHub" | "GitLab" |
| API headers `Bearer token` | `PRIVATE-TOKEN` header |
| `api.github.com/repos/...` | `{gitlab_url}/api/v4/projects/...` |
| PR → Merge Request | MR |

### 文件清单

| 操作 | 文件 |
|---|---|
| 重命名+重写 | `server/src/services/storage/github.rs` → `gitlab.rs` |
| 修改 | `server/src/services/storage/mod.rs` |
| 修改 | `server/src/services/archive_service.rs` |
| 修改 | `server/src/services/setup.rs` |
| 修改 | `server/src/models/ring.rs` |
| 修改 | `server/src/models/user.rs` |
| 修改 | `server/src/state.rs` |
| 修改 | `server/src/routes/archive.rs` |
| 修改 | `server/migrations/014_storage_mode.sql` |
| 修改 | `ui/src/types/ring.ts` |
| 修改 | `ui/src/stores/ring-store.ts` |
| 修改 | `ui/src/stores/chat-store.ts` |
| 修改 | `ui/src/services/api.ts` |
| 修改 | `ui/src/services/mock-data.ts` |
| 修改 | `ui/src/components/sidebar/RingList.tsx` |
| 修改 | `ui/src/components/setup/SetupWizard.tsx` |
| 修改 | `ui/src/components/setup/StepGitLab.tsx` |
| 修改 | `ui/src/components/panels/SuperSettingsPanel.tsx` |

---

### Task 1: 后端 storage 层 — GitHubBackend → GitLabBackend

**Files:**
- Rename: `server/src/services/storage/github.rs` → `server/src/services/storage/gitlab.rs`
- Modify: `server/src/services/storage/mod.rs`

- [ ] **Step 1: 创建 gitlab.rs**

将 `github.rs` 复制为 `gitlab.rs`，全部替换：

- `GitHubBackend` → `GitLabBackend`
- `github_token` → `gitlab_token`
- `github_repo` → `gitlab_project`
- `extract_repo` 改为 `extract_project_id`：从 URL 提取 `owner/repo` 格式，然后 URL-encode 为 GitLab project path（`owner%2Frepo`）
- `api_url()` 改为使用用户配置的 gitlab_url（从 new() 传入），拼接 `/api/v4/projects/{encoded_project}/...`
- `new()` 签名改为 `new(gitlab_url: &str, gitlab_token: &str, repo_url: &str)`，存储 gitlab_url
- HTTP headers: `Authorization: Bearer` → `PRIVATE-TOKEN`
- `Accept` header: 去掉 `application/vnd.github.v3+json`，改为标准 JSON
- `create_review`: POST `/api/v4/projects/{pid}/merge_requests`，body 使用 `source_branch`/`target_branch`/`title`/`description`
- `merge_review`: PUT `/api/v4/projects/{pid}/merge_requests/{iid}/merge`
- `reject_review`: PUT `/api/v4/projects/{pid}/merge_requests/{iid}`，body `{"state_event": "close"}`
- `get_review_diffs`: GET `/api/v4/projects/{pid}/merge_requests/{iid}/changes`，解析 `changes` 数组，取 `old_path`/`new_path`/`diff`
- 错误信息中 `GitHub` → `GitLab`

```rust
use async_trait::async_trait;
use std::path::Path;

use crate::error::{Result, RingError};
use crate::services::git_service::GitService;

use super::{DiffEntry, RepoStatus, StorageBackend};

pub struct GitLabBackend {
    git: GitService,
    gitlab_url: String,
    gitlab_token: String,
    project_id: String,
}

impl GitLabBackend {
    pub fn new(gitlab_url: &str, gitlab_token: &str, repo_url: &str) -> Self {
        let project_id = Self::extract_project_id(gitlab_url, repo_url);
        Self {
            git: GitService::new(),
            gitlab_url: gitlab_url.trim_end_matches('/').to_string(),
            gitlab_token: gitlab_token.to_string(),
            project_id,
        }
    }

    fn extract_project_id(gitlab_url: &str, repo_url: &str) -> String {
        let base = gitlab_url.trim_end_matches('/');
        let url = repo_url.trim_end_matches('/');
        if let Some(path) = url.strip_prefix(base) {
            let path = path.trim_start_matches('/').trim_end_matches(".git");
            urlencoding::encode(path).to_string()
        } else {
            urlencoding::encode(repo_url).to_string()
        }
    }

    fn api_url(&self, path: &str) -> String {
        format!(
            "{}/api/v4/projects/{}/{}",
            self.gitlab_url, self.project_id, path
        )
    }

    fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

#[async_trait]
impl StorageBackend for GitLabBackend {
    fn init_repo(
        &self,
        rings_dir: &Path,
        ring_id: &str,
        remote_url: Option<&str>,
    ) -> Result<std::path::PathBuf> {
        crate::services::archive_service::init_ring_repo(&self.git, rings_dir, ring_id, remote_url)
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
        RepoStatus {
            initialized,
            has_remote,
        }
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
        let resp = self
            .client()
            .post(self.api_url("merge_requests"))
            .header("PRIVATE-TOKEN", &self.gitlab_token)
            .json(&serde_json::json!({
                "source_branch": branch,
                "target_branch": "main",
                "title": title,
                "description": description,
            }))
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitLab API error: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!(
                "GitLab create MR failed: {body}"
            )));
        }

        let mr: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;
        let iid = mr["iid"]
            .as_i64()
            .ok_or_else(|| RingError::Internal("missing MR iid".into()))?;
        Ok(iid)
    }

    async fn merge_review(&self, repo_path: &Path, _ring_id: &str, review_id: i64) -> Result<()> {
        let resp = self
            .client()
            .put(self.api_url(&format!("merge_requests/{}/merge", review_id)))
            .header("PRIVATE-TOKEN", &self.gitlab_token)
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitLab API error: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!(
                "GitLab merge MR failed: {body}"
            )));
        }

        let _ = self.git.pull(repo_path);
        Ok(())
    }

    async fn reject_review(&self, _repo_path: &Path, _ring_id: &str, review_id: i64) -> Result<()> {
        let resp = self
            .client()
            .put(self.api_url(&format!("merge_requests/{}", review_id)))
            .header("PRIVATE-TOKEN", &self.gitlab_token)
            .json(&serde_json::json!({"state_event": "close"}))
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitLab API error: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!(
                "GitLab close MR failed: {body}"
            )));
        }

        Ok(())
    }

    async fn get_review_diffs(
        &self,
        _repo_path: &Path,
        _ring_id: &str,
        review_id: i64,
    ) -> Result<Vec<DiffEntry>> {
        let resp = self
            .client()
            .get(self.api_url(&format!("merge_requests/{}/changes", review_id)))
            .header("PRIVATE-TOKEN", &self.gitlab_token)
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitLab API error: {e}")))?;

        let mr: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;

        let changes = mr["changes"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        Ok(changes
            .into_iter()
            .map(|c| DiffEntry {
                old_path: c["old_path"].as_str().unwrap_or("").to_string(),
                new_path: c["new_path"].as_str().unwrap_or("").to_string(),
                diff: c["diff"].as_str().unwrap_or("").to_string(),
            })
            .collect())
    }
}
```

注意：GitLab 的 `extract_project_id` 需要 URL encode `owner/repo` → `owner%2Frepo`。需要在 `Cargo.toml` 添加 `urlencoding` crate（如果还没有的话）。如果不想加依赖，可以用手动替换：`path.replace('/', "%2F")`。

- [ ] **Step 2: 更新 mod.rs**

```rust
pub mod gitlab;
pub mod local;
```

`github` → `gitlab`

- [ ] **Step 3: 删除 github.rs**

```bash
rm server/src/services/storage/github.rs
```

- [ ] **Step 4: cargo check 验证编译**

```bash
cd server && cargo check 2>&1 | head -20
```

预期：有编译错误（因为其他文件还引用 github），这是正常的，后续 task 修复。

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "refactor: rename GitHubBackend to GitLabBackend with GitLab API v4"
```

---

### Task 2: 后端 models + services — 去掉 github 字段，复用 gitlab

**Files:**
- Modify: `server/src/models/ring.rs`
- Modify: `server/src/models/user.rs`
- Modify: `server/src/services/setup.rs`
- Modify: `server/src/services/archive_service.rs`
- Modify: `server/src/state.rs`
- Modify: `server/migrations/014_storage_mode.sql`

- [ ] **Step 1: 修改 014 migration**

014 migration 改为不添加 `github_*` 列（它们已存在 `gitlab_*` 列），只添加 `storage_mode`、`pending_reviews` 和 `github_namespace`（如果需要兼容）。实际上只需保留：

```sql
ALTER TABLE rings ADD COLUMN storage_mode TEXT NOT NULL DEFAULT 'local';

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

CREATE INDEX IF NOT EXISTS idx_pending_reviews_ring ON pending_reviews(ring_id);
CREATE INDEX IF NOT EXISTS idx_pending_reviews_status ON pending_reviews(status);
```

**注意：** 由于 migration 已经执行过（对于已有数据库），直接改 migration 文件对已有数据库无影响。但对于全新安装，这是正确的 schema。对于已有数据库，`github_*` 列已存在但不使用，无害。如果要清理，需要新的 015 migration 来 DROP 这些列（SQLite 不支持 DROP COLUMN 低于 3.35.0）。**建议：** 保留已有数据库的 github_* 列不动，只改代码中的引用。

- [ ] **Step 2: 修改 models/ring.rs**

- `CreateRing`: 删除 `github_repo_url`，保留 `gitlab_repo_url`
- `create_ring` SQL: 去掉 `github_repo_url`，改为 `gitlab_repo_url`
- `RingRow` 不变（已有 `gitlab_repo_url`，如果 schema 加了 `storage_mode` 也需要加）

```rust
#[derive(Debug, Deserialize)]
pub struct CreateRing {
    pub name: String,
    pub role_description: String,
    #[serde(default = "default_storage_mode")]
    pub storage_mode: String,
    pub gitlab_repo_url: Option<String>,
    pub gitlab_namespace: Option<String>,
}
```

`create_ring` 的 SQL:
```sql
INSERT INTO rings (id, name, creator_id, role_description, storage_mode, gitlab_repo_url, gitlab_namespace)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
RETURNING *
```

- [ ] **Step 3: 修改 models/user.rs**

删除所有 `github_url` 和 `github_token` 字段。`UserRow`、`CreateUser`、`UpdateUser` 都只保留 `gitlab_url` 和 `gitlab_token`。

`create_user` SQL:
```sql
INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, llm_base_url, gitlab_url, gitlab_token, privacy_filters)
VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
RETURNING *
```

`update_user` SQL:
```sql
UPDATE users SET
    display_name = ?1, avatar = ?2, llm_provider = ?3, llm_api_key = ?4,
    llm_model = ?5, llm_base_url = ?6, gitlab_url = ?7, gitlab_token = ?8,
    privacy_filters = ?9
WHERE token_id = ?10
RETURNING *
```

- [ ] **Step 4: 修改 services/setup.rs**

`SetupRequest`: 删除 `github_url` 和 `github_token`，只保留 `gitlab_url` 和 `gitlab_token`。

`submit_setup` 和 `update_setup`: 去掉 `encrypted_github_token`，`CreateUser`/`UpdateUser` 不传 github 字段。

- [ ] **Step 5: 修改 services/archive_service.rs**

`get_backend` 函数：
- `"github"` → `"gitlab"`
- 读取 `gitlab_token`（不是 `github_token`）
- 读取 `gitlab_repo_url`（不是 COALESCE 的 `github_repo_url`）
- 构造 `GitLabBackend::new(&user_row.gitlab_url.unwrap_or_default(), &gitlab_token, &repo_url)`

注意：`GitLabBackend::new` 现在需要 3 个参数：`gitlab_url`, `gitlab_token`, `repo_url`。用户的 `gitlab_url` 是实例 URL（如 `https://gitlab.company.com`），`repo_url` 是完整项目 URL。

- [ ] **Step 6: 修改 state.rs**

`get_user_decrypted`: 删除 `github_token` 解密块，只保留 `gitlab_token`。

- [ ] **Step 7: 修改 routes/archive.rs**

`init_repo` handler 中的 SQL:
```rust
"SELECT gitlab_repo_url FROM rings WHERE id = ?1"
```

- [ ] **Step 8: cargo check 验证编译**

```bash
cd server && cargo check 2>&1
```

预期：编译通过

- [ ] **Step 9: cargo test 验证测试**

```bash
cd server && cargo test 2>&1 | tail -10
```

- [ ] **Step 10: Commit**

```bash
git add -A && git commit -m "refactor: remove github fields from models/services, use gitlab only"
```

---

### Task 3: 前端 — GitHub → GitLab

**Files:**
- Modify: `ui/src/types/ring.ts`
- Modify: `ui/src/stores/ring-store.ts`
- Modify: `ui/src/stores/chat-store.ts`
- Modify: `ui/src/services/api.ts`
- Modify: `ui/src/services/mock-data.ts`
- Modify: `ui/src/components/sidebar/RingList.tsx`
- Modify: `ui/src/components/setup/SetupWizard.tsx`
- Modify: `ui/src/components/setup/StepGitLab.tsx`
- Modify: `ui/src/components/panels/SuperSettingsPanel.tsx`

- [ ] **Step 1: types/ring.ts**

```typescript
export type StorageMode = 'local' | 'gitlab'
```

- [ ] **Step 2: stores/ring-store.ts**

```typescript
interface CreateRingInput {
  name: string
  role_description: string
  storage_mode: 'local' | 'gitlab'
  gitlab_repo_url?: string
}
```

`createRing` 中:
```typescript
if (input.storage_mode === 'gitlab' && input.gitlab_repo_url) {
    body.gitlab_repo_url = input.gitlab_repo_url
}
```

- [ ] **Step 3: stores/chat-store.ts**

检查是否有 `github` 引用，改为 `gitlab`。

- [ ] **Step 4: services/api.ts**

无 API endpoint 变化，只需确认无 `github` 引用。

- [ ] **Step 5: services/mock-data.ts**

确认 `storage_mode: 'local'`（已正确）。

- [ ] **Step 6: components/sidebar/RingList.tsx**

所有 `github` → `gitlab`:
- `storageMode` type: `'local' | 'gitlab'`
- `githubRepoUrl` → `gitlabRepoUrl`
- `setGithubRepoUrl` → `setGitlabRepoUrl`
- storage_mode `'github'` → `'gitlab'`
- placeholder: `"https://gitlab.company.com/owner/repo"`
- 按钮 label: `gitlab` (小写)

- [ ] **Step 7: components/setup/SetupWizard.tsx**

`SetupData` 删除 `github_url` 和 `github_token`。`handleSubmit` 中去掉 github 字段。

- [ ] **Step 8: components/setup/StepGitLab.tsx**

全部恢复为 GitLab:
- 标题: "Step 3: GitLab Config"
- 描述: GitLab 用于归档...
- Label: "GitLab URL"
- placeholder: `"https://gitlab.company.com"`
- onChange: 只设 `gitlab_url`/`gitlab_token`
- Token 说明: "在 GitLab Settings → Access Tokens 中创建，需勾选 api 权限"
- placeholder: `"glpat-xxx"`

- [ ] **Step 9: components/panels/SuperSettingsPanel.tsx**

- Section title: "GitLab Config"
- Labels: "GitLab URL"
- placeholder: `"https://gitlab.company.com"`
- `handleGitlabTest`: body 中去掉 `github_url`/`github_token`

- [ ] **Step 10: npm run build 验证**

```bash
cd ui && npm run build 2>&1 | tail -20
```

- [ ] **Step 11: Commit**

```bash
git add -A && git commit -m "refactor: frontend GitHub → GitLab for company internal usage"
```

---

### Task 4: 全量验证 + docs 更新

- [ ] **Step 1: cargo test**

```bash
cd server && cargo test 2>&1 | tail -10
```

- [ ] **Step 2: cargo fmt + clippy**

```bash
cd server && cargo fmt && cargo clippy 2>&1 | tail -5
```

- [ ] **Step 3: npm run build**

```bash
cd ui && npm run build 2>&1 | tail -10
```

- [ ] **Step 4: grep 检查残留**

```bash
cd /Users/kaiiangs/Desktop/open-source-project/Ring
grep -r "github" --include="*.rs" --include="*.ts" --include="*.tsx" --include="*.sql" server/ ui/src/
```

预期：0 匹配（除了 `raw.githubusercontent.com` 在 skill.rs 中用于 URL 下载，和 `join_page.rs` 中 GitHub release 下载链接——这些保留）。

- [ ] **Step 5: 更新 docs/STATUS.md**

将 StorageBackend 条目中的 GitHub 引用改为 GitLab。

- [ ] **Step 6: Final commit**

```bash
git add -A && git commit -m "docs: update STATUS.md for GitLab storage backend"
```

---

## Self-Review

### 1. Spec Coverage
- ✅ 后端 GitHubBackend → GitLabBackend（API v4，PRIVATE-TOKEN）
- ✅ 后端 models 去掉 github 字段，复用 gitlab
- ✅ 后端 services/setup/state 跟进
- ✅ 前端全部 GitHub → GitLab
- ✅ DB migration 不再添加 github 列
- ✅ Ring 创建默认 local

### 2. Placeholder Scan
- 无 TBD/TODO

### 3. Type Consistency
- `GitLabBackend::new(gitlab_url, gitlab_token, repo_url)` — 3 参数
- `archive_service::get_backend` 传 3 参数 ✅
- 前端 `StorageMode = 'local' | 'gitlab'` 一致
- 前端 `CreateRingInput.gitlab_repo_url` 一致

### 注意事项

1. **已有数据库兼容性**：如果用户已运行过 014 migration，DB 中会有 `github_*` 列和 `storage_mode DEFAULT 'github'`。需要 015 migration 将 `storage_mode` 从 `'github'` 更新为 `'local'`，或直接在代码中把 `'github'` 也当作 `'gitlab'` 处理（向后兼容）。

2. **urlencoding 依赖**：`GitLabBackend::extract_project_id` 需要 URL encode `owner/repo` → `owner%2Frepo`。可以手动 `path.replace('/', "%2F")` 避免新依赖。

3. **skill.rs 和 join_page.rs 中的 github.com 引用**：这些是 GitHub release 下载 URL 和 raw.githubusercontent.com URL 下载，与存储后端无关，保留。
