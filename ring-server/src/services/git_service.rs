use std::path::Path;

use crate::error::{Result, RingError};

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
    pub status: String,
    pub additions: i64,
    pub deletions: i64,
    pub content: String,
}

#[derive(Default)]
pub struct GitService;

impl GitService {
    pub fn new() -> Self {
        GitService
    }

    pub async fn init_repo(&self, path: &Path) -> Result<()> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            git2::Repository::init(&path)?;
            Ok(())
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }

    pub async fn clone_repo(&self, url: &str, to_path: &Path) -> Result<()> {
        let url = url.to_string();
        let to_path = to_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            git2::Repository::clone(&url, &to_path)?;
            Ok(())
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }

    pub async fn add_all(&self, repo_path: &Path) -> Result<()> {
        let repo_path = repo_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&repo_path)?;
            let mut index = repo.index()?;
            index.add_all(["."], git2::IndexAddOption::DEFAULT, None)?;
            index.write()?;
            Ok(())
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }

    pub async fn commit(&self, repo_path: &Path, message: &str) -> Result<String> {
        let repo_path = repo_path.to_path_buf();
        let message = message.to_string();
        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&repo_path)?;
            let sig = repo.signature()?;
            let mut index = repo.index()?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;

            let maybe_parent = repo
                .head()
                .ok()
                .and_then(|h| h.target())
                .map(|oid| repo.find_commit(oid))
                .transpose()?;

            let commit_id = match &maybe_parent {
                Some(parent) => {
                    repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &[parent])?
                }
                None => repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &[])?,
            };
            Ok(commit_id.to_string())
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }

    pub async fn create_branch(&self, repo_path: &Path, name: &str) -> Result<()> {
        let repo_path = repo_path.to_path_buf();
        let name = name.to_string();
        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&repo_path)?;
            let head_commit = repo.head()?.peel_to_commit()?;
            repo.branch(&name, &head_commit, false)?;
            Ok(())
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }

    pub async fn get_current_branch(&self, repo_path: &Path) -> Result<String> {
        let repo_path = repo_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&repo_path)?;
            let head = repo.head()?;
            let name = head
                .shorthand()
                .ok_or_else(|| RingError::Internal("no branch name".into()))?
                .to_string();
            Ok(name)
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }

    pub async fn get_diff(&self, repo_path: &Path, from: &str, to: &str) -> Result<DiffResult> {
        let repo_path = repo_path.to_path_buf();
        let from = from.to_string();
        let to = to.to_string();
        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&repo_path)?;
            let from_oid = repo.revparse_single(&from)?.id();
            let to_oid = repo.revparse_single(&to)?.id();

            let from_commit = repo.find_commit(from_oid)?;
            let to_commit = repo.find_commit(to_oid)?;

            let from_tree = from_commit.tree()?;
            let to_tree = to_commit.tree()?;

            let diff = repo.diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)?;

            let mut file_stats: std::collections::HashMap<String, (i64, i64)> =
                std::collections::HashMap::new();

            for (i, delta) in diff.deltas().enumerate() {
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(|p| p.to_string_lossy().to_string());
                if let Some(path) = path.clone() {
                    let mut adds: i64 = 0;
                    let mut dels: i64 = 0;
                    if let Some(patch) = git2::Patch::from_diff(&diff, i).ok().flatten() {
                        for h in 0..patch.num_hunks() {
                            if let Ok((_header, hunk_lines)) = patch.hunk(h) {
                                for l in 0..hunk_lines {
                                    if let Ok(line) = patch.line_in_hunk(h, l) {
                                        match line.origin() {
                                            '+' => adds += 1,
                                            '-' => dels += 1,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    file_stats.insert(path, (adds, dels));
                }
            }

            let mut files = Vec::new();
            for delta in diff.deltas() {
                if let (Some(old_path), Some(new_path)) =
                    (delta.old_file().path(), delta.new_file().path())
                {
                    let path = new_path.to_string_lossy().to_string();
                    let old_path_str = old_path.to_string_lossy().to_string();
                    let status = match delta.status() {
                        git2::Delta::Added => "added",
                        git2::Delta::Deleted => "deleted",
                        git2::Delta::Modified => "modified",
                        git2::Delta::Renamed => "renamed",
                        _ => "unknown",
                    };
                    let (additions, deletions) = file_stats.get(&path).copied().unwrap_or((0, 0));
                    files.push(FileDiff {
                        path,
                        status: status.to_string(),
                        additions,
                        deletions,
                        content: format!("{} -> {}", old_path_str, new_path.to_string_lossy()),
                    });
                }
            }

            Ok(DiffResult { files })
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }

    pub async fn get_log(&self, repo_path: &Path, limit: usize) -> Result<Vec<CommitInfo>> {
        let repo_path = repo_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&repo_path)?;
            let mut revwalk = repo.revwalk()?;
            revwalk.push_head()?;
            revwalk.set_sorting(git2::Sort::TIME)?;

            let mut commits = Vec::new();
            for oid in revwalk.take(limit) {
                let oid = oid?;
                let commit = repo.find_commit(oid)?;
                let author = commit.author();
                let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default();
                commits.push(CommitInfo {
                    id: commit.id().to_string(),
                    message: commit.message().unwrap_or("").to_string(),
                    author: author.name().unwrap_or("").to_string(),
                    timestamp,
                });
            }
            Ok(commits)
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }

    pub async fn has_changes(&self, repo_path: &Path) -> Result<bool> {
        let repo_path = repo_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&repo_path)?;
            let mut opts = git2::StatusOptions::new();
            opts.include_untracked(true);
            let statuses = repo.statuses(Some(&mut opts))?;
            Ok(!statuses.is_empty())
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }

    pub async fn status_files(&self, repo_path: &Path) -> Result<Vec<String>> {
        let repo_path = repo_path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(&repo_path)?;
            let mut opts = git2::StatusOptions::new();
            opts.include_untracked(true);
            let statuses = repo.statuses(Some(&mut opts))?;
            let mut files = Vec::new();
            for entry in statuses.iter() {
                if let Some(path) = entry.path() {
                    files.push(path.to_string());
                }
            }
            Ok(files)
        })
        .await
        .map_err(|e| RingError::Internal(format!("spawn blocking failed: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn init_repo_creates_git_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = GitService::new();
        svc.init_repo(dir.path()).await.unwrap();
        assert!(dir.path().join(".git").exists());
    }

    #[tokio::test]
    async fn commit_creates_log_entry() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = GitService::new();
        svc.init_repo(dir.path()).await.unwrap();
        fs::write(dir.path().join("hello.txt"), "world").unwrap();
        svc.add_all(dir.path()).await.unwrap();
        let id = svc.commit(dir.path(), "first commit").await.unwrap();
        assert!(!id.is_empty());

        let log = svc.get_log(dir.path(), 10).await.unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].message, "first commit");
    }

    #[tokio::test]
    async fn create_branch_switches() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = GitService::new();
        svc.init_repo(dir.path()).await.unwrap();
        fs::write(dir.path().join("f.txt"), "x").unwrap();
        svc.add_all(dir.path()).await.unwrap();
        svc.commit(dir.path(), "init").await.unwrap();

        svc.create_branch(dir.path(), "feature-x").await.unwrap();

        let repo = git2::Repository::open(dir.path()).unwrap();
        let branch = repo
            .find_branch("feature-x", git2::BranchType::Local)
            .unwrap();
        assert!(branch.get().target().is_some());
    }

    #[tokio::test]
    async fn get_diff_detects_changes() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = GitService::new();
        svc.init_repo(dir.path()).await.unwrap();
        fs::write(dir.path().join("a.txt"), "v1").unwrap();
        svc.add_all(dir.path()).await.unwrap();
        svc.commit(dir.path(), "first").await.unwrap();

        fs::write(dir.path().join("a.txt"), "v2").unwrap();
        fs::write(dir.path().join("b.txt"), "new").unwrap();
        svc.add_all(dir.path()).await.unwrap();
        svc.commit(dir.path(), "second").await.unwrap();

        let log = svc.get_log(dir.path(), 10).await.unwrap();
        let head = &log[0].id;
        let prev = &log[1].id;

        let diff = svc.get_diff(dir.path(), prev, head).await.unwrap();
        assert!(!diff.files.is_empty());
    }

    #[tokio::test]
    async fn has_changes_detects_uncommitted() {
        let dir = tempfile::TempDir::new().unwrap();
        let svc = GitService::new();
        svc.init_repo(dir.path()).await.unwrap();

        assert!(!svc.has_changes(dir.path()).await.unwrap());

        fs::write(dir.path().join("new.txt"), "content").unwrap();
        assert!(svc.has_changes(dir.path()).await.unwrap());
    }
}
