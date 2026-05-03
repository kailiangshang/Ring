use axum::extract::State;
use axum::Json;
use serde_json::json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::state::AppState;

pub async fn rotate_token(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let new_token = crate::services::auth::rotate_token(&state, &user.token_id).await?;
    let token_path = state.hub_dir.join("token");
    let _ = tokio::fs::write(&token_path, &new_token).await;
    Ok(Json(json!({ "token_id": new_token })))
}
