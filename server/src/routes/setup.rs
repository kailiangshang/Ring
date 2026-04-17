use axum::extract::State;
use axum::Json;
use serde_json::Value;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::services::setup;
use crate::state::AppState;

pub async fn get_status(State(state): State<AppState>) -> Result<Json<setup::SetupStatusResponse>> {
    let status = setup::get_status(&state).await?;
    Ok(Json(status))
}

pub async fn submit_setup(
    State(state): State<AppState>,
    Json(body): Json<setup::SetupRequest>,
) -> Result<Json<Value>> {
    let result = setup::submit_setup(&state, body).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn update_setup(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<setup::SetupRequest>,
) -> Result<Json<Value>> {
    let result = setup::update_setup(&state, &user.token_id, body).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}
