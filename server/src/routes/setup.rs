use axum::extract::State;
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::services::setup::{self, SetupResponse};
use crate::state::AppState;

pub async fn get_status(State(state): State<AppState>) -> Result<Json<setup::SetupStatusResponse>> {
    let status = setup::get_status(&state).await?;
    Ok(Json(status))
}

pub async fn submit_setup(
    State(state): State<AppState>,
    Json(body): Json<setup::SetupRequest>,
) -> Result<Json<SetupResponse>> {
    let result = setup::submit_setup(&state, body).await?;
    let token_path = state.hub_dir.join("token");
    let _ = std::fs::write(&token_path, &result.token_id);
    Ok(Json(result))
}

pub async fn update_setup(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<setup::SetupRequest>,
) -> Result<Json<SetupResponse>> {
    let result = setup::update_setup(&state, &user.token_id, body).await?;
    let token_path = state.hub_dir.join("token");
    let _ = std::fs::write(&token_path, &result.token_id);
    Ok(Json(result))
}

pub async fn recover_token(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    let token_path = state.hub_dir.join("token");
    let token = std::fs::read_to_string(&token_path)
        .map_err(|_| crate::error::RingError::NotFound("no recovery token found".into()))?;
    let done = crate::models::config::get_setup_done(&state.db).await?;
    if !done {
        return Err(crate::error::RingError::BadRequest(
            "setup not complete".into(),
        ));
    }
    Ok(Json(serde_json::json!({ "token_id": token })))
}
