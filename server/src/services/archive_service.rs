use std::path::PathBuf;

use chrono::Utc;
use sqlx::SqlitePool;

use crate::error::{Result, RingError};
use crate::models::archive;
use crate::models::archive::ArchiveRecord;
use crate::models::graph;
use crate::services::git_service::GitService;
use crate::services::gitlab_service::GitLabClient;
use crate::state::AppState;

pub async fn quick_archive(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    content: &str,
) -> Result<()> {
    let role = sqlx::query_scalar::<_, String>(
        "SELECT role FROM ring_members WHERE ring_id = ?1 AND user_id = ?2",
    )
    .bind(ring_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?
    .ok_or_else(|| RingError::Forbidden("not a ring member".into()))?;

    let is_creator = role == "creator" || role == "admin";
    let title = if content.len() > 40 {
        let s: String = content.chars().take(40).collect();
        format!("{s}...")
    } else {
        content.to_string()
    };

    let git = GitService::new();
    let repo_path = ring_repo_path(&state.rings_dir, ring_id);

    if !repo_path.join(".git").exists() {
        return Err(RingError::RepoNotFound {
            ring_id: ring_id.to_string(),
        });
    }

    let _ = git.pull(&repo_path);

    let file_name = sanitize_filename(&title);
    let file_path = repo_path.join("archives").join(&file_name);
    std::fs::write(&file_path, content)?;

    let record_id = ulid::Ulid::new().to_string();

    if is_creator {
        git.add_all(&repo_path)?;
        let sha = git.commit(&repo_path, &format!("archive: {title}"))?;

        let has_remote = git.has_remote(&repo_path);
        if has_remote {
            git.push(&repo_path, "origin", "main")?;
        }

        archive::insert_record(
            &state.db, &record_id, ring_id, None, None, &file_name, user_id,
        )
        .await?;

        let status = if has_remote { "pushed" } else { "committed" };
        archive::update_status(&state.db, &record_id, status, Some(&sha), None, None).await?;
    } else {
        let repo_url = sqlx::query_scalar::<_, Option<String>>(
            "SELECT gitlab_repo_url FROM rings WHERE id = ?1",
        )
        .bind(ring_id)
        .fetch_one(&state.db)
        .await
        .ok()
        .flatten();

        let user_row = state.get_user_decrypted(user_id).await?;
        let (gitlab_url, gitlab_token) = (user_row.gitlab_url.clone(), user_row.gitlab_token.clone());

        match (repo_url, gitlab_url, gitlab_token) {
            (Some(url), Some(gl_url), Some(gl_token)) => {
                let gitlab = GitLabClient::new(&gl_url, &gl_token);
                let branch_name = format!("archive/{record_id}");

                git.create_branch(&repo_path, &branch_name)?;
                git.add_all(&repo_path)?;
                let sha = git.commit(&repo_path, &format!("archive: {title}"))?;
                git.push(&repo_path, "origin", &branch_name)?;
                git.checkout(&repo_path, "main")?;

                archive::insert_record(
                    &state.db, &record_id, ring_id, None, None, &file_name, user_id,
                )
                .await?;
                archive::update_status(
                    &state.db, &record_id, "committed", Some(&sha), Some(&branch_name), None,
                )
                .await?;

                let mr = gitlab
                    .create_mr(
                        &url,
                        &branch_name,
                        "main",
                        &format!("归档: {title}"),
                        &format!("由 {user_id} 提交的归档请求"),
                    )
                    .await?;

                archive::update_status(&state.db, &record_id, "mr_opened", None, None, Some(mr.iid)).await?;
            }
            _ => {
                return Err(RingError::GitlabNotConfigured);
            }
        }
    }

    Ok(())
}

pub fn ring_repo_path(rings_dir: &std::path::Path, ring_id: &str) -> PathBuf {
    rings_dir.join(ring_id)
}

pub fn sanitize_filename(title: &str) -> String {
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let safe_title: String = title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
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
        std::fs::write(
            &gitignore_path,
            ".ring-local/
assets/
",
        )?;
    }

    if let Some(url) = gitlab_url {
        if !git.has_remote(&repo_path) {
            git.set_remote(&repo_path, "origin", url)?;
        }
    }

    Ok(repo_path)
}

#[allow(clippy::too_many_arguments)]
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
    archive::insert_record(
        pool, &record_id, ring_id, session_id, node_id, &file_name, user_id,
    )
    .await?;

    let status = if has_remote { "pushed" } else { "committed" };
    archive::update_status(pool, &record_id, status, Some(&sha), None, None).await
}

#[allow(clippy::too_many_arguments)]
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

    archive::insert_record(
        pool, &record_id, ring_id, session_id, node_id, &file_name, user_id,
    )
    .await?;
    archive::update_status(
        pool,
        &record_id,
        "committed",
        Some(&sha),
        Some(&branch_name),
        None,
    )
    .await?;

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

#[derive(Debug, serde::Deserialize)]
pub struct ArchiveUnit {
    pub title: String,
    pub content: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn auto_archive_session(
    pool: &SqlitePool,
    git: &GitService,
    rings_dir: &std::path::Path,
    ring_id: &str,
    session_id: &str,
    session_title: &str,
    session_skill: &str,
    creator_user: &crate::models::user::UserRow,
) {
    tracing::info!("auto_archive started: session={session_id}, ring={ring_id}");

    let messages =
        match crate::models::session::get_all_messages_ordered(pool, session_id, 100).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("auto_archive failed to load messages: {e}");
                return;
            }
        };

    if messages.is_empty() {
        tracing::info!("auto_archive: no messages in session {session_id}, skipping");
        return;
    }

    let messages_text = messages
        .iter()
        .map(|m| format!("[{}]: {}", m.sender_name, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = crate::prompts::archive::EXTRACT_SYSTEM;

    let user_message = format!(
        "Session 标题: {session_title}\nSkill: {session_skill}\n\n讨论记录：\n{messages_text}"
    );

    let llm = match crate::services::llm::LlmClient::from_user(creator_user) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("auto_archive failed to create LLM client: {e}");
            return;
        }
    };

    let response = match llm
        .chat_complete(system_prompt.to_string(), user_message)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("auto_archive LLM call failed: {e}");
            return;
        }
    };

    let cleaned = response.trim();
    let json_str = if cleaned.starts_with("```") {
        cleaned
            .lines()
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        cleaned.to_string()
    };

    let units: Vec<ArchiveUnit> = match serde_json::from_str(&json_str) {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!("auto_archive failed to parse LLM JSON: {e}\nraw: {json_str}");
            return;
        }
    };

    tracing::info!("auto_archive extracted {} units", units.len());

    if units.is_empty() {
        return;
    }

    let mut success_count = 0u32;
    for unit in &units {
        let title_with_ts = format!("{}_{}", chrono::Utc::now().format("%H%M%S"), unit.title);
        match archive_content_creator(
            pool,
            git,
            rings_dir,
            ring_id,
            Some(session_id),
            None,
            &unit.content,
            &title_with_ts,
            &creator_user.token_id,
        )
        .await
        {
            Ok(_) => success_count += 1,
            Err(e) => {
                tracing::warn!(
                    "auto_archive unit failed: title={}, error={}",
                    unit.title,
                    e
                );
            }
        }
    }

    tracing::info!(
        "auto_archive completed: session={session_id}, {success_count}/{} files created",
        units.len()
    );
}

#[allow(clippy::too_many_arguments)]
pub async fn auto_archive_chat(
    pool: &SqlitePool,
    git: &GitService,
    rings_dir: &std::path::Path,
    ring_id: &str,
    user_message: &str,
    ai_response: &str,
    user_id: &str,
    user_row: &crate::models::user::UserRow,
) {
    tracing::info!("auto_archive_chat started: ring={ring_id}");

    let system_prompt = crate::prompts::archive::JUDGE_SYSTEM;

    let user_prompt = format!("用户消息：\n{}\n\nAI回复：\n{}", user_message, ai_response);

    let llm = match crate::services::llm::LlmClient::from_user(user_row) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("auto_archive_chat failed to create LLM client: {e}");
            return;
        }
    };

    let response = match llm
        .chat_complete(system_prompt.to_string(), user_prompt)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("auto_archive_chat LLM call failed: {e}");
            return;
        }
    };

    let cleaned = response.trim();
    let json_str = if cleaned.starts_with("```") {
        cleaned
            .lines()
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        cleaned.to_string()
    };

    #[derive(Debug, serde::Deserialize)]
    struct ArchiveDecision {
        should_archive: bool,
        title: Option<String>,
        content: Option<String>,
    }

    let decision: ArchiveDecision = match serde_json::from_str(&json_str) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("auto_archive_chat failed to parse LLM JSON: {e}\nraw: {json_str}");
            return;
        }
    };

    if !decision.should_archive {
        tracing::info!("auto_archive_chat: content not worth archiving");
        return;
    }

    let title = match decision.title {
        Some(t) => t,
        None => {
            tracing::warn!("auto_archive_chat: missing title in decision");
            return;
        }
    };

    let content = match decision.content {
        Some(c) => c,
        None => {
            tracing::warn!("auto_archive_chat: missing content in decision");
            return;
        }
    };

    let role = match crate::models::ring::get_user_role(pool, ring_id, user_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("auto_archive_chat failed to get user role: {e}");
            return;
        }
    };

    let is_creator = role == "creator" || role == "admin";
    let title_with_ts = format!("{}_{}", chrono::Utc::now().format("%H%M%S"), title);

    if is_creator {
        match archive_content_creator(
            pool,
            git,
            rings_dir,
            ring_id,
            None,
            None,
            &content,
            &title_with_ts,
            user_id,
        )
        .await
        {
            Ok(_) => {
                tracing::info!("auto_archive_chat: archived '{}'", title);
            }
            Err(e) => {
                tracing::warn!("auto_archive_chat failed to archive: {e}");
            }
        }
    } else {
        let repo_url = match sqlx::query_scalar::<_, Option<String>>(
            "SELECT gitlab_repo_url FROM rings WHERE id = ?1",
        )
        .bind(ring_id)
        .fetch_one(pool)
        .await
        {
            Ok(url) => url,
            Err(e) => {
                tracing::warn!("auto_archive_chat failed to get repo url: {e}");
                return;
            }
        };

        let (gitlab_url, gitlab_token) = match (&user_row.gitlab_url, &user_row.gitlab_token) {
            (Some(url), Some(token)) => (url.clone(), token.clone()),
            _ => {
                tracing::warn!("auto_archive_chat: GitLab not configured for member");
                return;
            }
        };

        match repo_url {
            Some(url) => {
                let gitlab =
                    crate::services::gitlab_service::GitLabClient::new(&gitlab_url, &gitlab_token);
                match archive_content_member(
                    pool,
                    git,
                    &gitlab,
                    rings_dir,
                    ring_id,
                    &url,
                    None,
                    None,
                    &content,
                    &title_with_ts,
                    user_id,
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("auto_archive_chat: created MR for '{}'", title);
                    }
                    Err(e) => {
                        tracing::warn!("auto_archive_chat failed to create MR: {e}");
                    }
                }
            }
            None => {
                tracing::warn!("auto_archive_chat: no GitLab repo configured");
            }
        }
    }
}
