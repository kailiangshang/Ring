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
    let owned_sessions: Vec<(String, String)> = sqlx::query_as(
        "SELECT s.id, s.title FROM sessions s
         JOIN session_participants sp ON sp.session_id = s.id
         WHERE s.ring_id = ?1 AND sp.token_id = ?2 AND sp.role = 'owner' AND s.phase != 'closed'",
    )
    .bind(&ring_id)
    .bind(&target_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;

    if !owned_sessions.is_empty() {
        return Err(crate::error::RingError::BadRequest(format!(
            "User owns {} active session(s). Transfer ownership first: {}",
            owned_sessions.len(),
            owned_sessions.iter().map(|(id, t)| format!("{} ({})", id, t)).collect::<Vec<_>>().join(", ")
        )));
    }

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
        .await
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;
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
        .await
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
