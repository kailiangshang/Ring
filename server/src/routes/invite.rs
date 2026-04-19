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
