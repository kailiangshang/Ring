use async_stream::stream;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::stream::BoxStream;
use futures_util::StreamExt;
use serde::Serialize;
use std::convert::Infallible;

use crate::error::{Result, RingError};
use crate::extractors::auth::AuthUser;
use crate::models::archive::{self, CreateArchiveInput, ReviewAction, ReviewInput};
use crate::models::ring;
use crate::services::archive_service::{self, ArchiveStep};
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

async fn get_backend(
    state: &AppState,
    ring_id: &str,
) -> Result<Box<dyn crate::services::storage::StorageBackend>> {
    archive_service::get_backend(&state.db, ring_id, None, Some(&state.encryption)).await
}

pub async fn quick_archive_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateArchiveInput>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    ring::reject_readonly(&role)?;

    let backend = get_backend(&state, &ring_id).await?;

    let content = body.content.clone();
    let title = if content.len() > 40 {
        let s: String = content.chars().take(40).collect();
        format!("{s}...")
    } else {
        content.clone()
    };

    let repo_path = archive_service::ring_repo_path(&state.rings_dir, &ring_id);

    if !repo_path.join(".git").exists() {
        return Err(RingError::RepoNotFound {
            ring_id: ring_id.to_string(),
        });
    }

    let _ = backend.pull(&repo_path);

    let file_name = archive_service::sanitize_filename(&title);
    let file_path = repo_path.join("archives").join(&file_name);
    std::fs::write(&file_path, &content)?;

    let record_id = ulid::Ulid::new().to_string();

    let is_creator = role == "creator" || role == "admin";

    if is_creator {
        backend.add_all(&repo_path)?;
        let sha = backend.commit(&repo_path, &format!("archive: {title}"))?;

        let has_remote = backend.has_remote(&repo_path);
        if has_remote {
            backend.push_main(&repo_path)?;
        }

        archive::insert_record(
            &state.db,
            &record_id,
            &ring_id,
            body.session_id.as_deref(),
            None,
            &file_name,
            &user.token_id,
        )
        .await?;

        let status = if has_remote { "pushed" } else { "committed" };
        archive::update_status(&state.db, &record_id, status, Some(&sha), None, None).await?;
    } else {
        let record_id_for_desc = record_id.clone();
        let title_for_desc = title.clone();
        let user_id_for_desc = user.token_id.clone();

        let branch_name = format!("archive/{record_id}");

        backend.create_branch(&repo_path, &branch_name)?;
        backend.add_all(&repo_path)?;
        let sha = backend.commit(&repo_path, &format!("archive: {title}"))?;
        backend.push_branch(&repo_path, &branch_name)?;
        backend.checkout(&repo_path, "main")?;

        archive::insert_record(
            &state.db,
            &record_id,
            &ring_id,
            body.session_id.as_deref(),
            None,
            &file_name,
            &user.token_id,
        )
        .await?;
        archive::update_status(
            &state.db,
            &record_id,
            "committed",
            Some(&sha),
            Some(&branch_name),
            None,
        )
        .await?;

        let mr_iid = backend
            .create_review(
                &repo_path,
                &ring_id,
                &record_id_for_desc,
                &branch_name,
                &format!("归档: {title_for_desc}"),
                &format!("由 {} 提交的归档请求", user_id_for_desc),
            )
            .await?;

        archive::update_status(&state.db, &record_id, "mr_opened", None, None, Some(mr_iid))
            .await?;
    }

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    if let Err(e) = crate::services::self_data::record_tool_usage(&self_dir, "archive") {
        tracing::warn!("failed to record tool usage: {e}");
    }
    if let Err(e) =
        crate::services::self_data::record_archive_operation(&self_dir, &ring_id, &file_name)
    {
        tracing::warn!("failed to record archive operation: {e}");
    }

    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        let uid = user.token_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
            crate::services::cross_ring_cache::invalidate_summary(&cache, &uid).await;
        });
    }

    Ok(Json(
        serde_json::json!({ "ok": true, "record_id": record_id }),
    ))
}

pub async fn trigger_archive(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateArchiveInput>,
) -> Result<
    Sse<
        axum::response::sse::KeepAliveStream<
            BoxStream<'static, std::result::Result<Event, Infallible>>,
        >,
    >,
> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    ring::reject_readonly(&role)?;
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
    let state_c = state.clone();

    let backend = get_backend(&state, &ring_id).await?;

    tokio::spawn(async move {
        let _ = tx.send(ArchiveStep::Pulling).await;
        let _ = tx.send(ArchiveStep::Writing).await;

        if is_creator {
            match archive_service::archive_content_creator(
                &pool,
                backend.as_ref(),
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

                    let self_dir = crate::services::self_data::get_self_dir(&token_id);
                    if let Err(e) = crate::services::self_data::record_archive_operation(
                        &self_dir, &ring_id_c, &title,
                    ) {
                        tracing::warn!("failed to record archive operation: {e}");
                    }

                    let state = state_c.clone();
                    let ring_id = ring_id_c.clone();
                    let user_row = match state.get_user_decrypted(&token_id).await {
                        Ok(u) => u,
                        Err(_) => return,
                    };
                    let archive_detail = format!(
                        "Title: {}\nContent preview: {}...",
                        title,
                        content.chars().take(200).collect::<String>()
                    );
                    tokio::spawn(async move {
                        if let Err(e) =
                            crate::services::group_doc_maintenance::update_archive_patterns(
                                &state,
                                &ring_id,
                                &user_row,
                                &archive_detail,
                            )
                            .await
                        {
                            tracing::warn!("failed to update archive patterns: {e}");
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("archive failed: {e}");
                }
            }
        } else {
            let _ = tx.send(ArchiveStep::CreatingMR).await;
            match archive_service::archive_content_member(
                &pool,
                backend.as_ref(),
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
                    tracing::error!("member archive failed: {e}");
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
    }
    .boxed();

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

    let backend = get_backend(&state, &ring_id).await?;

    let record = archive_service::review_mr(
        &state.db,
        backend.as_ref(),
        &state.rings_dir,
        &archive_id,
        body.action,
    )
    .await?;

    if let ReviewAction::Reject = body.action {
        let detail = format!("归档 {} 被拒绝（文件: {}）", archive_id, record.file_name);
        if let Err(e) =
            crate::services::group_doc_maintenance::add_correction(&state, &ring_id, &detail).await
        {
            tracing::warn!("failed to add correction: {e}");
        }
    }

    Ok(Json(record))
}

pub async fn get_archive_diff(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, archive_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let record = archive::get_record(&state.db, &archive_id).await?;
    let mr_iid = record
        .merge_request_iid
        .ok_or_else(|| RingError::BadRequest("archive has no merge request".into()))?;

    let backend = get_backend(&state, &ring_id).await?;
    let repo_path = archive_service::ring_repo_path(&state.rings_dir, &ring_id);
    let diffs = backend
        .get_review_diffs(&repo_path, &ring_id, mr_iid)
        .await?;

    Ok(Json(serde_json::json!({ "diffs": diffs })))
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
    let backend = get_backend(&state, &ring_id).await?;
    let repo_path = state.rings_dir.join(&ring_id);
    let status = backend.repo_status(&repo_path);
    Ok(Json(RepoStatusResponse {
        initialized: status.initialized,
        has_remote: status.has_remote,
    }))
}

pub async fn init_repo(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<RepoStatusResponse>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    ring::reject_readonly(&role)?;

    let backend = get_backend(&state, &ring_id).await?;
    let repo_url: Option<String> = sqlx::query_scalar(
        "SELECT COALESCE(github_repo_url, gitlab_repo_url) FROM rings WHERE id = ?1",
    )
    .bind(&ring_id)
    .fetch_optional(&state.db)
    .await?
    .flatten();

    let repo_path = backend.init_repo(&state.rings_dir, &ring_id, repo_url.as_deref())?;
    let has_remote = backend.has_remote(&repo_path);
    Ok(Json(RepoStatusResponse {
        initialized: true,
        has_remote,
    }))
}
