use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::RingError;
use crate::middleware::auth::AuthUser;
use crate::models::ring::Ring;
use crate::services::ring_service::{CreateRingRequest, RingService};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRingRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingsResponse {
    pub rings: Vec<Ring>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRingHandlerRequest {
    pub name: String,
    pub description: Option<String>,
    pub role_description: Option<String>,
    pub gitlab_repo: Option<String>,
    pub namespace: Option<String>,
}

fn make_ring_service(state: &AppState) -> RingService {
    RingService::new(state.db.clone(), state.config.data_dir.clone())
}

pub async fn list_rings(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<RingsResponse>, RingError> {
    let user_id = auth_user.user_id;
    let service = make_ring_service(&state);
    let rings = service.list_rings(&user_id).await?;
    Ok(Json(RingsResponse { rings }))
}

pub async fn create_ring(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(req): Json<CreateRingHandlerRequest>,
) -> Result<(StatusCode, Json<Ring>), RingError> {
    if req.name.trim().is_empty() {
        return Err(RingError::Validation("name must not be empty".into()));
    }
    let user_id = auth_user.user_id;
    let service = make_ring_service(&state);
    let ring = service
        .create_ring(CreateRingRequest {
            name: req.name,
            description: req.description,
            role_description: req.role_description.unwrap_or_default(),
            creator_id: user_id,
            gitlab_repo: req.gitlab_repo.unwrap_or_default(),
            namespace: req.namespace,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(ring)))
}

pub async fn get_ring(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> Result<Json<Ring>, RingError> {
    let service = make_ring_service(&state);
    let ring = service.get_ring(&ring_id).await?;
    Ok(Json(ring))
}

pub async fn update_ring(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Json(req): Json<UpdateRingRequest>,
) -> Result<Json<Ring>, RingError> {
    let service = make_ring_service(&state);
    let ring = service
        .update_ring(&ring_id, req.name, req.description)
        .await?;
    Ok(Json(ring))
}

pub async fn delete_ring(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> Result<StatusCode, RingError> {
    let service = make_ring_service(&state);
    service.delete_ring(&ring_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
