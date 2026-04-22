use axum::extract::{Path, State};
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::ring;
use crate::services::blueprint_service::{
    confirm_blueprint, get_blueprint, preview_from_template, FromTemplateRequest,
};
use crate::state::AppState;

pub async fn get_blueprint_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<crate::services::blueprint_service::BlueprintResponse>> {
    let _ = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let blueprint = get_blueprint(&state, &ring_id).await?;
    Ok(Json(blueprint))
}

pub async fn preview_template(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<FromTemplateRequest>,
) -> Result<Json<crate::services::blueprint_service::BlueprintPreview>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(crate::error::RingError::Forbidden(
            "only creator/admin can manage blueprint".into(),
        ));
    }
    let preview = preview_from_template(&state, &body.template).await?;
    Ok(Json(preview))
}

pub async fn confirm_blueprint_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(crate::error::RingError::Forbidden(
            "only creator/admin can manage blueprint".into(),
        ));
    }
    confirm_blueprint(&state, &ring_id).await?;
    Ok(Json(serde_json::json!({ "status": "confirmed" })))
}
