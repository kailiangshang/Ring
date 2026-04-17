use axum::extract::State;
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::config::UpdateLLMConfig;
use crate::services::config;
use crate::state::AppState;

pub async fn get_llm_config(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<crate::models::config::LLMConfigResponse>> {
    let cfg = config::get_llm_config(&state, &user.token_id).await?;
    Ok(Json(cfg))
}

pub async fn update_llm_config(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateLLMConfig>,
) -> Result<Json<crate::models::config::LLMConfigResponse>> {
    let cfg = config::update_llm_config(&state, &user.token_id, body).await?;
    Ok(Json(cfg))
}
