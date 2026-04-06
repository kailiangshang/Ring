use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::RingError;
use crate::models::git_model::{CommitLogResponse, PrResponse};
use crate::services::archive_service::ArchiveService;
use crate::services::git_service::GitService;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListPrsQuery {
    pub state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PrsResponse {
    pub prs: Vec<PrResponse>,
}

pub async fn list_prs(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Query(query): Query<ListPrsQuery>,
) -> Result<Json<PrsResponse>, RingError> {
    let git_service = std::sync::Arc::new(GitService::new());
    let service = ArchiveService::new(
        state.db.clone(),
        git_service,
        state.graph_store.clone(),
        None,
    );
    let pr_state = query.state.as_deref().unwrap_or("opened");
    let prs = service.list_prs(&ring_id, pr_state).await?;
    Ok(Json(PrsResponse { prs }))
}

pub async fn get_pr_diff(
    State(state): State<AppState>,
    Path((ring_id, pr_id)): Path<(String, i64)>,
) -> Result<Json<PrResponse>, RingError> {
    let git_service = std::sync::Arc::new(GitService::new());
    let service = ArchiveService::new(
        state.db.clone(),
        git_service,
        state.graph_store.clone(),
        None,
    );
    let pr = service.get_pr_diff(&ring_id, pr_id).await?;
    Ok(Json(pr))
}

pub async fn merge_pr(
    State(state): State<AppState>,
    Path((ring_id, pr_id)): Path<(String, i64)>,
) -> Result<StatusCode, RingError> {
    let git_service = std::sync::Arc::new(GitService::new());
    let service = ArchiveService::new(
        state.db.clone(),
        git_service,
        state.graph_store.clone(),
        None,
    );
    service.merge_pr(&ring_id, &pr_id.to_string()).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reject_pr(
    State(state): State<AppState>,
    Path((ring_id, pr_id)): Path<(String, i64)>,
) -> Result<StatusCode, RingError> {
    let git_service = std::sync::Arc::new(GitService::new());
    let service = ArchiveService::new(
        state.db.clone(),
        git_service,
        state.graph_store.clone(),
        None,
    );
    service.reject_pr(&ring_id, &pr_id.to_string()).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct CommitLogQuery {
    pub limit: Option<usize>,
}

pub async fn get_commit_log(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Query(query): Query<CommitLogQuery>,
) -> Result<Json<CommitLogResponse>, RingError> {
    let git_service = std::sync::Arc::new(GitService::new());
    let service = ArchiveService::new(
        state.db.clone(),
        git_service,
        state.graph_store.clone(),
        None,
    );
    let log = service
        .get_commit_log(&ring_id, query.limit.unwrap_or(20))
        .await?;
    Ok(Json(log))
}
