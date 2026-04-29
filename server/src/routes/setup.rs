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
    let _ = tokio::fs::write(&token_path, &result.token_id).await;
    Ok(Json(result))
}

pub async fn update_setup(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<setup::SetupRequest>,
) -> Result<Json<SetupResponse>> {
    let result = setup::update_setup(&state, &user.token_id, body).await?;
    let token_path = state.hub_dir.join("token");
    let _ = tokio::fs::write(&token_path, &result.token_id).await;
    Ok(Json(result))
}

pub async fn recover_token(
    State(state): State<AppState>,
    user: crate::extractors::OptionalUser,
) -> Result<Json<serde_json::Value>> {
    let done = crate::models::config::get_setup_done(&state.db).await?;
    if !done {
        return Err(crate::error::RingError::NotFound(
            "setup not complete".into(),
        ));
    }

    if let Some(token) = user.token_id {
        let exists: bool =
            sqlx::query_scalar::<_, bool>("SELECT COUNT(*) > 0 FROM users WHERE token_id = ?1")
                .bind(&token)
                .fetch_one(&state.db)
                .await
                .map_err(|e: sqlx::Error| crate::error::RingError::Internal(e.to_string()))?;
        if !exists {
            return Err(crate::error::RingError::Unauthorized("invalid token".into()));
        }
    } else {
        return Err(crate::error::RingError::Unauthorized(
            "missing token".into(),
        ));
    }

    let token_path = state.hub_dir.join("token");
    let token = tokio::fs::read_to_string(&token_path).await
        .map_err(|_| crate::error::RingError::NotFound("no recovery token found".into()))?;
    Ok(Json(serde_json::json!({ "token_id": token })))
}
