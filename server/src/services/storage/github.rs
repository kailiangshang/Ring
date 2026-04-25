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

    fn api_url(&self, path: &str) -> String {
        format!("https://api.github.com/repos/{}/{}", self.github_repo, path)
    }

    fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }
}

#[async_trait]
impl StorageBackend for GitHubBackend {
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
            return Err(RingError::Internal(format!(
                "GitHub create PR failed: {body}"
            )));
        }

        let pr: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;
        let number = pr["number"]
            .as_i64()
            .ok_or_else(|| RingError::Internal("missing PR number".into()))?;
        Ok(number)
    }

    async fn merge_review(&self, repo_path: &Path, _ring_id: &str, review_id: i64) -> Result<()> {
        let resp = self
            .client()
            .put(self.api_url(&format!("pulls/{}/merge", review_id)))
            .header("Authorization", format!("Bearer {}", self.github_token))
            .header("User-Agent", "ring-server")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitHub API error: {e}")))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(RingError::Internal(format!(
                "GitHub merge PR failed: {body}"
            )));
        }

        let _ = self.git.pull(repo_path);
        Ok(())
    }

    async fn reject_review(&self, _repo_path: &Path, _ring_id: &str, review_id: i64) -> Result<()> {
        let resp = self
            .client()
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
            return Err(RingError::Internal(format!(
                "GitHub close PR failed: {body}"
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
            .get(self.api_url(&format!("pulls/{}/files", review_id)))
            .header("Authorization", format!("Bearer {}", self.github_token))
            .header("User-Agent", "ring-server")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| RingError::Internal(format!("GitHub API error: {e}")))?;

        let files: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;

        Ok(files
            .into_iter()
            .map(|f| DiffEntry {
                old_path: f["filename"].as_str().unwrap_or("").to_string(),
                new_path: f["filename"].as_str().unwrap_or("").to_string(),
                diff: f["patch"].as_str().unwrap_or("").to_string(),
            })
            .collect())
    }
}
