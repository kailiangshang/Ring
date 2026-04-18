use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::Value;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::services::member;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub async fn list_members(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<Value>> {
    let members = member::list_members(&state, &ring_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}

pub async fn update_role(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, target_id)): Path<(String, String)>,
    Json(body): Json<UpdateRoleRequest>,
) -> Result<axum::http::StatusCode> {
    member::update_member_role(&state, &ring_id, &user.token_id, &target_id, &body.role).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn remove_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, target_id)): Path<(String, String)>,
) -> Result<axum::http::StatusCode> {
    member::remove_member(&state, &ring_id, &user.token_id, &target_id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
}

pub async fn add_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<AddMemberRequest>,
) -> Result<Json<Value>> {
    let result =
        member::add_member_service(&state, &ring_id, &user.token_id, &body.user_id).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}
