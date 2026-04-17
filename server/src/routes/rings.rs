use axum::extract::{Path, State};
use axum::Json;
use serde_json::Value;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::ring::CreateRing;
use crate::services::ring;
use crate::state::AppState;

pub async fn list_rings(State(state): State<AppState>, user: AuthUser) -> Result<Json<Value>> {
    let rings = ring::list_rings(&state, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "rings": rings })))
}

pub async fn create_ring(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateRing>,
) -> Result<(axum::http::StatusCode, Json<Value>)> {
    let result = ring::create_ring(&state, &user.token_id, body).await?;
    Ok((
        axum::http::StatusCode::CREATED,
        Json(serde_json::to_value(result).unwrap()),
    ))
}

pub async fn get_ring(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<Value>> {
    let detail = ring::get_ring_detail(&state, &ring_id, &user.token_id).await?;
    Ok(Json(serde_json::to_value(detail).unwrap()))
}
