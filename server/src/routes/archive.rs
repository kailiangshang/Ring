use async_stream::stream;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use serde::Serialize;
use std::convert::Infallible;

use crate::error::{Result, RingError};
use crate::extractors::auth::AuthUser;
use crate::models::archive::{self, CreateArchiveInput, ReviewInput};
use crate::models::ring;
use crate::services::archive_service::{self, ArchiveStep};
use crate::services::git_service::GitService;
use crate::services::gitlab_service::GitLabClient;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ArchiveListResponse {
    pub archives: Vec<archive::ArchiveRecord>,
}

#[derive(Debug, Serialize)]
pub struct ArchiveQueueResponse {
    pub queue: Vec<archive::ArchiveRecord>,
}

#[derive(Debug, Serialize)]
pub struct RepoStatusResponse {
    pub initialized: bool,
    pub has_remote: bool,
}

pub async fn trigger_archive(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateArchiveInput>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let is_creator = role == "creator" || role == "admin";

    let node_id = match &body.node_suggestion {
        archive::NodeSuggestionInput::CreateNew { .. } => None,
        archive::NodeSuggestionInput::AttachExisting { node_id } => Some(node_id.clone()),
        archive::NodeSuggestionInput::UpdateExisting { node_id } => Some(node_id.clone()),
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ArchiveStep>(16);

    let pool = state.db.clone();
    let rings_dir = state.rings_dir.clone();
    let title = body.suggested_title.clone();
    let content = body.content.clone();
    let session_id = body.session_id.clone();
    let token_id = user.token_id.clone();
    let ring_id_c = ring_id.clone();

    tokio::spawn(async move {
        let git = GitService::new();

        let _ = tx.send(ArchiveStep::Pulling).await;
        let _ = tx.send(ArchiveStep::Writing).await;

        if is_creator {
            match archive_service::archive_content_creator(
                &pool,
                &git,
                &rings_dir,
                &ring_id_c,
                session_id.as_deref(),
                node_id.as_deref(),
                &content,
                &title,
                &token_id,
            )
            .await
            {
                Ok(_) => {
                    let _ = tx.send(ArchiveStep::Complete).await;
                }
                Err(e) => {
                    tracing::error!("archive failed: {e}");
                }
            }
        } else {
            let repo_url = sqlx::query_scalar::<_, Option<String>>(
                "SELECT gitlab_repo_url FROM rings WHERE id = ?1",
            )
            .bind(&ring_id_c)
            .fetch_one(&pool)
            .await
            .ok()
            .flatten();

            let user_row = crate::models::user::get_user(&pool, &token_id).await;
            let (gitlab_url, gitlab_token) = match user_row {
                Ok(u) => (u.gitlab_url.clone(), u.gitlab_token.clone()),
                Err(_) => (None, None),
            };

            match (repo_url, gitlab_url, gitlab_token) {
                (Some(url), Some(gl_url), Some(gl_token)) => {
                    let gitlab = GitLabClient::new(&gl_url, &gl_token);
                    let _ = tx.send(ArchiveStep::CreatingMR).await;
                    match archive_service::archive_content_member(
                        &pool,
                        &git,
                        &gitlab,
                        &rings_dir,
                        &ring_id_c,
                        &url,
                        session_id.as_deref(),
                        node_id.as_deref(),
                        &content,
                        &title,
                        &token_id,
                    )
                    .await
                    {
                        Ok(_) => {
                            let _ = tx.send(ArchiveStep::Complete).await;
                        }
                        Err(e) => {
                            tracing::error!("member archive failed: {e}");
                        }
                    }
                }
                _ => {
                    tracing::error!("GitLab not configured for member archive");
                }
            }
        }
    });

    let s = stream! {
        while let Some(step) = rx.recv().await {
            let data = serde_json::json!({
                "step": step.step_name(),
                "message": step.message()
            });
            yield Ok(Event::default().event("progress").data(data.to_string()));
        }
    };

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn list_archives(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<ArchiveListResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let records = archive::list_by_ring(&state.db, &ring_id).await?;
    Ok(Json(ArchiveListResponse { archives: records }))
}

pub async fn get_archive(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, archive_id)): Path<(String, String)>,
) -> Result<Json<archive::ArchiveRecord>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let record = archive::get_record(&state.db, &archive_id).await?;
    Ok(Json(record))
}

pub async fn review_archive(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, archive_id)): Path<(String, String)>,
    Json(body): Json<ReviewInput>,
) -> Result<Json<archive::ArchiveRecord>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden("only creator/admin can review".into()));
    }

    let user_row = crate::models::user::get_user(&state.db, &user.token_id).await?;
    let (gitlab_url, gitlab_token) = match (user_row.gitlab_url, user_row.gitlab_token) {
        (Some(url), Some(token)) => (url, token),
        _ => return Err(RingError::GitlabNotConfigured),
    };

    let repo_url: Option<String> =
        sqlx::query_scalar("SELECT gitlab_repo_url FROM rings WHERE id = ?1")
            .bind(&ring_id)
            .fetch_optional(&state.db)
            .await?
            .flatten();

    let repo_url = repo_url.ok_or(RingError::GitlabNotConfigured)?;

    let git = GitService::new();
    let gitlab = GitLabClient::new(&gitlab_url, &gitlab_token);

    let record = archive_service::review_mr(
        &state.db,
        &git,
        &gitlab,
        &state.rings_dir,
        &archive_id,
        &repo_url,
        body.action,
    )
    .await?;

    Ok(Json(record))
}

pub async fn archive_queue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<ArchiveQueueResponse>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden(
            "only creator/admin can view queue".into(),
        ));
    }
    let records = archive::list_pending_reviews(&state.db, &ring_id).await?;
    Ok(Json(ArchiveQueueResponse { queue: records }))
}

pub async fn repo_status(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<RepoStatusResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let repo_path = state.rings_dir.join(&ring_id);
    let initialized = repo_path.join(".git").exists();
    let has_remote = if initialized {
        GitService::new().has_remote(&repo_path)
    } else {
        false
    };
    Ok(Json(RepoStatusResponse {
        initialized,
        has_remote,
    }))
}

pub async fn init_repo(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<RepoStatusResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let repo_url: Option<String> =
        sqlx::query_scalar("SELECT gitlab_repo_url FROM rings WHERE id = ?1")
            .bind(&ring_id)
            .fetch_optional(&state.db)
            .await?
            .flatten();

    let git = GitService::new();
    let repo_path =
        archive_service::init_ring_repo(&git, &state.rings_dir, &ring_id, repo_url.as_deref())?;
    let has_remote = git.has_remote(&repo_path);
    Ok(Json(RepoStatusResponse {
        initialized: true,
        has_remote,
    }))
}
