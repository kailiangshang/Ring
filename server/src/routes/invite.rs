use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::invite::CreateInviteToken;
use crate::services::invite;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListTokensQuery {
    #[serde(default)]
    pub include_expired: Option<bool>,
    #[serde(default)]
    pub include_revoked: Option<bool>,
}

pub async fn create_invite_token(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateInviteToken>,
) -> Result<Json<serde_json::Value>> {
    let token = invite::create_token(&state, &ring_id, &user.token_id, &body).await?;
    Ok(Json(json!({
        "token": token.token,
        "type": token.r#type,
        "role": token.role,
        "max_uses": token.max_uses,
        "max_members": token.max_members,
        "expires_at": token.expires_at,
        "created_at": token.created_at,
    })))
}

pub async fn list_invite_tokens(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<ListTokensQuery>,
) -> Result<Json<serde_json::Value>> {
    let include_expired = query.include_expired.unwrap_or(false);
    let include_revoked = query.include_revoked.unwrap_or(false);
    let tokens = invite::list_tokens(
        &state,
        &ring_id,
        &user.token_id,
        include_expired,
        include_revoked,
    )
    .await?;
    Ok(Json(json!({ "tokens": tokens })))
}

pub async fn revoke_invite_token(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, token)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let revoked_at = invite::revoke_token(&state, &ring_id, &user.token_id, &token).await?;
    Ok(Json(json!({ "ok": true, "revoked_at": revoked_at })))
}

#[derive(Debug, Deserialize)]
pub struct JoinInfoQuery {
    pub token: String,
}

pub async fn join_info(
    State(state): State<AppState>,
    Query(query): Query<JoinInfoQuery>,
) -> Result<Json<serde_json::Value>> {
    let info = invite::verify_join_token(&state, &query.token).await?;
    Ok(Json(json!({
        "valid": info.valid,
        "reason": info.reason,
        "ring_id": info.ring_id,
        "ring_name": info.ring_name,
        "member_count": info.member_count,
        "role": info.role,
        "token_type": info.token_type,
    })))
}

#[derive(Debug, Deserialize)]
pub struct JoinBody {
    pub invite_token: String,
    pub display_name: String,
}

pub async fn join_ring(
    State(state): State<AppState>,
    Json(body): Json<JoinBody>,
) -> Result<Json<serde_json::Value>> {
    let req = invite::JoinRequest {
        invite_token: body.invite_token,
        display_name: body.display_name,
    };
    let result = invite::execute_join(&state, &req).await?;
    Ok(Json(json!({
        "token_id": result.token_id,
        "ring_id": result.ring_id,
        "ring_name": result.ring_name,
        "role": result.role,
        "gitlab_repo_url": result.gitlab_repo_url,
    })))
}

#[derive(Debug, Deserialize)]
pub struct LocalJoinBody {
    pub invite_token: String,
    pub creator_ip: String,
}

pub async fn local_join_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<LocalJoinBody>,
) -> Result<Json<serde_json::Value>> {
    let req = invite::LocalJoinRequest {
        invite_token: body.invite_token,
        creator_ip: body.creator_ip,
    };
    let result = invite::local_join(&state, &user.token_id, &req).await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct ApplyBody {
    pub invite_token: String,
    pub display_name: String,
    pub message: Option<String>,
}

pub async fn apply_join(
    State(state): State<AppState>,
    Json(body): Json<ApplyBody>,
) -> Result<Json<serde_json::Value>> {
    let req = invite::ApplyRequest {
        invite_token: body.invite_token,
        display_name: body.display_name,
        message: body.message,
    };
    let result = invite::submit_apply(&state, &req).await?;
    Ok(Json(json!({
        "request_id": result.request_id,
        "status": result.status,
        "ring_name": result.ring_name,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ApplyStatusQuery {
    pub id: String,
}

pub async fn apply_status(
    State(state): State<AppState>,
    Query(query): Query<ApplyStatusQuery>,
) -> Result<Json<serde_json::Value>> {
    let result = invite::check_apply_status(&state, &query.id).await?;
    Ok(Json(json!({
        "request_id": result.request_id,
        "status": result.status,
        "ring_name": result.ring_name,
        "ring_id": result.ring_id,
        "role": result.role,
        "review_note": result.review_note,
        "token_id": result.token_id,
    })))
}

#[derive(Debug, Deserialize)]
pub struct ListRequestsQuery {
    #[serde(default)]
    pub status: Option<String>,
}

pub async fn list_join_requests_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<ListRequestsQuery>,
) -> Result<Json<serde_json::Value>> {
    let status_filter = query.status.as_deref().unwrap_or("pending");
    let requests =
        invite::list_join_requests(&state, &ring_id, &user.token_id, status_filter).await?;
    Ok(Json(json!({ "requests": requests })))
}

pub async fn approve_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, request_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let result =
        invite::approve_join_request(&state, &ring_id, &user.token_id, &request_id).await?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct RejectBody {
    pub note: Option<String>,
}

pub async fn reject_request(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, request_id)): Path<(String, String)>,
    Json(body): Json<RejectBody>,
) -> Result<Json<serde_json::Value>> {
    let result = invite::reject_join_request(
        &state,
        &ring_id,
        &user.token_id,
        &request_id,
        body.note.as_deref(),
    )
    .await?;
    Ok(Json(result))
}
