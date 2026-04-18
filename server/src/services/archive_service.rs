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
        git.init(&repo_path)?;
    }

    for d in &["archives", "graphs", ".group", "assets", ".ring-local"] {
        let dir = repo_path.join(d);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
    }

    let gitignore_path = repo_path.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, ".ring-local/
assets/
")?;
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
        return Err(RingError::RepoNotFound { ring_id: ring_id.to_string() });
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
        return Err(RingError::RepoNotFound { ring_id: ring_id.to_string() });
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
            let _ = git.pull(&repo_path);
            archive::update_status(pool, record_id, "merged", None, None, None).await
        }
        archive::ReviewAction::Reject => {
            gitlab.close_mr(gitlab_repo_url, mr_iid).await?;
            archive::update_status(pool, record_id, "rejected", None, None, None).await
        }
    }
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
