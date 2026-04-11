use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::RingError;
use crate::middleware::auth::AuthUser;
use crate::models::git_model::{ArchiveQueueResponse, ArchiveRequest, ArchiveResponse};
use crate::services::archive_service::ArchiveService;
use crate::services::git_service::GitService;
use crate::state::AppState;

pub async fn archive(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(ring_id): Path<String>,
    Json(req): Json<ArchiveRequest>,
) -> Result<(StatusCode, Json<ArchiveResponse>), RingError> {
    let ring = state
        .db
        .get_ring(&ring_id)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;
    let is_creator = ring.creator_id == auth_user.user_id;
    let git_service = std::sync::Arc::new(GitService::new());
    let service = ArchiveService::new(
        state.db.clone(),
        git_service,
        state.graph_store.clone(),
        None,
    );
    let resp = service
        .archive(&ring_id, &req, &auth_user.user_id, is_creator)
        .await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn get_queue(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> Result<Json<ArchiveQueueResponse>, RingError> {
    let git_service = std::sync::Arc::new(GitService::new());
    let service = ArchiveService::new(
        state.db.clone(),
        git_service,
        state.graph_store.clone(),
        None,
    );
    let queue = service.get_queue(&ring_id).await?;
    Ok(Json(queue))
}

pub async fn confirm_archive(
    State(state): State<AppState>,
    Path((ring_id, archive_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let git_service = std::sync::Arc::new(GitService::new());
    let service = ArchiveService::new(
        state.db.clone(),
        git_service,
        state.graph_store.clone(),
        None,
    );
    service.confirm_archive(&ring_id, &archive_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
