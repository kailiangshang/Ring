pub mod github;
pub mod local;

use async_trait::async_trait;
use serde::Serialize;
use std::path::Path;

use crate::error::Result;

#[derive(Debug, Clone, Serialize)]
pub struct RepoStatus {
    pub initialized: bool,
    pub has_remote: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffEntry {
    pub old_path: String,
    pub new_path: String,
    pub diff: String,
}

#[async_trait]
pub trait StorageBackend: Send + Sync {
    fn init_repo(
        &self,
        rings_dir: &Path,
        ring_id: &str,
        remote_url: Option<&str>,
    ) -> Result<std::path::PathBuf>;
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
