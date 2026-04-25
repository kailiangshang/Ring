use async_trait::async_trait;
use std::path::Path;

use crate::error::{Result, RingError};
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
    fn init_repo(
        &self,
        rings_dir: &Path,
        ring_id: &str,
        _remote_url: Option<&str>,
    ) -> Result<std::path::PathBuf> {
        crate::services::archive_service::init_ring_repo(&self.git, rings_dir, ring_id, None)
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
        let review: (String,) = sqlx::query_as(
            "SELECT source_branch FROM pending_reviews WHERE ring_id = ?1 AND rowid = ?2 AND status = 'open'"
        )
            .bind(ring_id)
            .bind(review_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| RingError::NotFound("review not found".into()))?;

        self.git.checkout(repo_path, "main")?;

        std::process::Command::new("git")
            .current_dir(repo_path)
            .args(["merge", &review.0])
            .output()
            .map_err(|e| RingError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE pending_reviews SET status = 'merged' WHERE ring_id = ?1 AND rowid = ?2",
        )
        .bind(ring_id)
        .bind(review_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn reject_review(&self, repo_path: &Path, ring_id: &str, review_id: i64) -> Result<()> {
        let _branch: String = sqlx::query_scalar(
            "SELECT source_branch FROM pending_reviews WHERE ring_id = ?1 AND rowid = ?2 AND status = 'open'"
        )
            .bind(ring_id)
            .bind(review_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| RingError::NotFound("review not found".into()))?;

        self.git.checkout(repo_path, "main")?;

        sqlx::query(
            "UPDATE pending_reviews SET status = 'rejected' WHERE ring_id = ?1 AND rowid = ?2",
        )
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
        let branch: String =
            sqlx::query_scalar("SELECT source_branch FROM pending_reviews WHERE rowid = ?1")
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
