use axum::extract::{Path, State};
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::services::mode::{self, UpdateModeRequest};
use crate::state::AppState;

pub async fn get_mode(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<mode::ModeResponse>> {
    let result = mode::get_mode(&state, &ring_id, &user.token_id).await?;
    Ok(Json(result))
}

pub async fn update_mode(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<UpdateModeRequest>,
) -> Result<Json<mode::ModeResponse>> {
    let result = mode::update_mode(&state, &ring_id, &user.token_id, &body).await?;
    Ok(Json(result))
}
