use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::member::MemberResponse;
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
) -> Result<Json<serde_json::Value>> {
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
    {
        let cache = state.cross_ring_cache.clone();
        let uid = user.token_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_summary(&cache, &uid).await;
        });
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct AddMemberRequest {
    #[serde(alias = "token_id")]
    pub user_id: String,
}

pub async fn add_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<AddMemberRequest>,
) -> Result<Json<MemberResponse>> {
    let result =
        member::add_member_service(&state, &ring_id, &user.token_id, &body.user_id).await?;
    {
        let cache = state.cross_ring_cache.clone();
        let uid = user.token_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_summary(&cache, &uid).await;
        });
    }
    Ok(Json(result))
}

pub async fn grant_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, target_id)): Path<(String, String)>,
) -> Result<axum::http::StatusCode> {
    let role = crate::models::ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(crate::error::RingError::Forbidden(
            "only creator/admin can grant session".into(),
        ));
    }
    sqlx::query("UPDATE members SET session_grant = 1 WHERE ring_id = ?1 AND user_id = ?2")
        .bind(&ring_id)
        .bind(&target_id)
        .execute(&state.db)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn revoke_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, target_id)): Path<(String, String)>,
) -> Result<axum::http::StatusCode> {
    let role = crate::models::ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" {
        return Err(crate::error::RingError::Forbidden(
            "only creator can revoke session grant".into(),
        ));
    }
    sqlx::query("UPDATE members SET session_grant = 0 WHERE ring_id = ?1 AND user_id = ?2")
        .bind(&ring_id)
        .bind(&target_id)
        .execute(&state.db)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
