use axum::extract::{Path, State};
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::ring::{CreateRing, RingDetail};
use crate::services::ring::{self, CreateRingResponse};
use crate::state::AppState;

pub async fn list_rings(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let rings = ring::list_rings(&state, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "rings": rings })))
}

pub async fn create_ring(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateRing>,
) -> Result<(axum::http::StatusCode, Json<CreateRingResponse>)> {
    let result = ring::create_ring(&state, &user.token_id, body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(result)))
}

pub async fn get_ring(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<RingDetail>> {
    let detail = ring::get_ring_detail(&state, &ring_id, &user.token_id).await?;
    Ok(Json(detail))
}
