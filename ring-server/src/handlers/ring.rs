use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::RingError;
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
    pub role_description: String,
    pub gitlab_repo: String,
    pub namespace: Option<String>,
}

async fn get_first_user_id(state: &AppState) -> Result<String, RingError> {
    let users = state.db.list_all_users().await?;
    users
        .into_iter()
        .next()
        .map(|u| u.id)
        .ok_or_else(|| RingError::Validation("no user found, run setup first".into()))
}

pub async fn list_rings(State(state): State<AppState>) -> Result<Json<RingsResponse>, RingError> {
    let user_id = get_first_user_id(&state).await?;
    let service = RingService::new(state.db.clone());
    let rings = service.list_rings(&user_id).await?;
    Ok(Json(RingsResponse { rings }))
}

pub async fn create_ring(
    State(state): State<AppState>,
    Json(req): Json<CreateRingHandlerRequest>,
) -> Result<(StatusCode, Json<Ring>), RingError> {
    if req.name.trim().is_empty() {
        return Err(RingError::Validation("name must not be empty".into()));
    }
    let user_id = get_first_user_id(&state).await?;
    let service = RingService::new(state.db.clone());
    let ring = service
        .create_ring(CreateRingRequest {
            name: req.name,
            description: req.description,
            role_description: req.role_description,
            creator_id: user_id,
            gitlab_repo: req.gitlab_repo,
            namespace: req.namespace,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(ring)))
}

pub async fn get_ring(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> Result<Json<Ring>, RingError> {
    let service = RingService::new(state.db.clone());
    let ring = service.get_ring(&ring_id).await?;
    Ok(Json(ring))
}

pub async fn update_ring(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Json(req): Json<UpdateRingRequest>,
) -> Result<Json<Ring>, RingError> {
    let service = RingService::new(state.db.clone());
    let ring = service
        .update_ring(&ring_id, req.name, req.description)
        .await?;
    Ok(Json(ring))
}

pub async fn delete_ring(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> Result<StatusCode, RingError> {
    let service = RingService::new(state.db.clone());
    service.delete_ring(&ring_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
