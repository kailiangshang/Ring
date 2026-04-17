use crate::error::{Result, RingError};
use crate::models::member::{self, MemberResponse};
use crate::models::ring;
use crate::state::AppState;

pub async fn list_members(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
) -> Result<Vec<MemberResponse>> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    member::list_members(&state.db, ring_id).await
}

pub async fn update_member_role(
    state: &AppState,
    ring_id: &str,
    caller_id: &str,
    target_id: &str,
    new_role: &str,
) -> Result<()> {
    let caller_role = ring::get_user_role(&state.db, ring_id, caller_id).await?;
    if caller_role != "creator" && caller_role != "admin" {
        return Err(RingError::Forbidden(
            "only creator or admin can change roles".into(),
        ));
    }
    if new_role != "admin" && new_role != "member" && new_role != "readonly" {
        return Err(RingError::BadRequest("invalid role".into()));
    }
    member::update_role(&state.db, ring_id, target_id, new_role).await
}

pub async fn remove_member(
    state: &AppState,
    ring_id: &str,
    caller_id: &str,
    target_id: &str,
) -> Result<()> {
    let caller_role = ring::get_user_role(&state.db, ring_id, caller_id).await?;
    if caller_role != "creator" && caller_role != "admin" {
        return Err(RingError::Forbidden(
            "only creator or admin can remove members".into(),
        ));
    }
    member::remove_member(&state.db, ring_id, target_id).await
}
