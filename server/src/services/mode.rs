use serde::{Deserialize, Serialize};

use crate::error::{Result, RingError};
use crate::models::ring;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ModeResponse {
    pub interaction_mode: String,
    pub skill_permission_mode: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModeRequest {
    pub interaction_mode: Option<String>,
    pub skill_permission_mode: Option<String>,
}

pub async fn get_mode(state: &AppState, ring_id: &str, user_id: &str) -> Result<ModeResponse> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT interaction_mode, skill_permission_mode FROM rings WHERE id = ?1",
    )
    .bind(ring_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| RingError::NotFound("ring not found".into()))?;

    Ok(ModeResponse {
        interaction_mode: row.0,
        skill_permission_mode: row.1,
    })
}

pub async fn update_mode(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    input: &UpdateModeRequest,
) -> Result<ModeResponse> {
    let role = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if role == "readonly" {
        return Err(RingError::Forbidden(
            "readonly members cannot change mode".into(),
        ));
    }

    if let Some(ref mode) = input.interaction_mode {
        if mode != "normal" && mode != "auto" {
            return Err(RingError::BadRequest(
                "interaction_mode must be 'normal' or 'auto'".into(),
            ));
        }
    }
    if let Some(ref mode) = input.skill_permission_mode {
        if mode != "auto" && mode != "plan" && mode != "edit" {
            return Err(RingError::BadRequest(
                "skill_permission_mode must be 'auto', 'plan', or 'edit'".into(),
            ));
        }
    }

    let current = get_mode(state, ring_id, user_id).await?;
    let im = input
        .interaction_mode
        .as_deref()
        .unwrap_or(&current.interaction_mode);
    let spm = input
        .skill_permission_mode
        .as_deref()
        .unwrap_or(&current.skill_permission_mode);

    sqlx::query("UPDATE rings SET interaction_mode = ?1, skill_permission_mode = ?2 WHERE id = ?3")
        .bind(im)
        .bind(spm)
        .bind(ring_id)
        .execute(&state.db)
        .await?;

    Ok(ModeResponse {
        interaction_mode: im.to_string(),
        skill_permission_mode: spm.to_string(),
    })
}
