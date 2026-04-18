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

    pub fn init(&self, path: &Path) -> Result<()> {
        Self::run_git(path, &["init"])?;
        Ok(())
    }

    pub fn clone(url: &str, path: &Path) -> Result<()> {
        Self::run_git(path, &["clone", url, &path.to_string_lossy()])?;
        Ok(())
    }

    pub fn pull(&self, repo_path: &Path) -> Result<()> {
        Self::run_git(repo_path, &["pull", "--rebase"])?;
        Ok(())
    }

    pub fn add_all(&self, repo_path: &Path) -> Result<()> {
        Self::run_git(repo_path, &["add", "."])?;
        Ok(())
    }

    pub fn commit(&self, repo_path: &Path, msg: &str) -> Result<String> {
        Self::run_git(repo_path, &["commit", "-m", msg])?;
        Self::run_git(repo_path, &["rev-parse", "HEAD"])
    }

    pub fn push(&self, repo_path: &Path, remote: &str, branch: &str) -> Result<()> {
        Self::run_git(repo_path, &["push", remote, branch])?;
        Ok(())
    }

    pub fn create_branch(&self, repo_path: &Path, name: &str) -> Result<()> {
        Self::run_git(repo_path, &["checkout", "-b", name])?;
        Ok(())
    }

    pub fn checkout(&self, repo_path: &Path, branch: &str) -> Result<()> {
        Self::run_git(repo_path, &["checkout", branch])?;
        Ok(())
    }

    pub fn log(&self, repo_path: &Path, n: usize) -> Result<Vec<LogEntry>> {
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

    pub fn has_remote(&self, path: &Path) -> bool {
        Self::run_git(path, &["remote"])
            .map(|r| !r.is_empty())
            .unwrap_or(false)
    }

    pub fn set_remote(&self, path: &Path, name: &str, url: &str) -> Result<()> {
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
