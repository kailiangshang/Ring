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
        let repo_url = repo_url.trim_end_matches(".git");
        let base = gitlab_url.trim_end_matches('/');
        let path = if let Some(stripped) = repo_url.strip_prefix(base) {
            stripped.trim_start_matches('/')
        } else {
            repo_url
        };
        path.replace('/', "%2F")
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

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;

        let changes = body["changes"]
            .as_array()
            .ok_or_else(|| RingError::Internal("missing changes array".into()))?;

        Ok(changes
            .iter()
            .map(|c| DiffEntry {
                old_path: c["old_path"].as_str().unwrap_or("").to_string(),
                new_path: c["new_path"].as_str().unwrap_or("").to_string(),
                diff: c["diff"].as_str().unwrap_or("").to_string(),
            })
            .collect())
    }
}
