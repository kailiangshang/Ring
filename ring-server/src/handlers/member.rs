use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::error::RingError;
use crate::models::invite::InviteToken;
use crate::models::member::Member;
use crate::services::MemberService;
use crate::state::AppState;

pub async fn list_members(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> Result<Json<serde_json::Value>, RingError> {
    let service = MemberService::new(state.db.clone());
    let members = service.list_members(&ring_id).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}

#[derive(Deserialize)]
pub struct GenerateInviteRequest {
    pub token_type: String,
    pub role: String,
    pub max_uses: i64,
    pub max_members: Option<i64>,
}

pub async fn generate_invite(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Json(req): Json<GenerateInviteRequest>,
) -> Result<Json<InviteToken>, RingError> {
    let service = MemberService::new(state.db.clone());
    let token = service
        .generate_invite(
            &ring_id,
            "user-1",
            &req.token_type,
            &req.role,
            req.max_uses,
            req.max_members,
            86400,
        )
        .await?;
    Ok(Json(token))
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub async fn update_role(
    State(state): State<AppState>,
    Path((ring_id, member_id)): Path<(String, String)>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<StatusCode, RingError> {
    let service = MemberService::new(state.db.clone());
    service
        .update_role(&ring_id, &member_id, &req.role, "user-1")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_member(
    State(state): State<AppState>,
    Path((ring_id, member_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = MemberService::new(state.db.clone());
    service
        .remove_member(&ring_id, &member_id, "user-1")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct JoinRequest {
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct JoinQueryParams {
    pub token: String,
}

pub async fn join_ring(
    State(state): State<AppState>,
    Query(params): Query<JoinQueryParams>,
    Json(req): Json<JoinRequest>,
) -> Result<Json<Member>, RingError> {
    let service = MemberService::new(state.db.clone());
    let member = service
        .join_ring(&params.token, "user-1", &req.display_name)
        .await?;
    Ok(Json(member))
}
