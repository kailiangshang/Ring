use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;

use crate::error::{Result, RingError};
use crate::models::invite::{self, CreateInviteToken, InviteTokenRow};
use crate::models::ring;
use crate::state::AppState;

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

async fn check_admin(pool: &sqlx::SqlitePool, ring_id: &str, user_id: &str) -> Result<String> {
    let role = ring::get_user_role(pool, ring_id, user_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden(
            "only creator or admin can manage invite tokens".into(),
        ));
    }
    Ok(role)
}

pub async fn create_token(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    input: &CreateInviteToken,
) -> Result<InviteTokenRow> {
    check_admin(&state.db, ring_id, user_id).await?;

    if input.r#type != "open" && input.r#type != "audit" {
        return Err(RingError::BadRequest(
            "type must be 'open' or 'audit'".into(),
        ));
    }
    if input.role != "member" && input.role != "readonly" {
        return Err(RingError::BadRequest(
            "role must be 'member' or 'readonly'".into(),
        ));
    }
    if input.expires_in_hours <= 0 {
        return Err(RingError::BadRequest(
            "expires_in_hours must be positive".into(),
        ));
    }

    let token = generate_token();
    let expires_at = Utc::now() + chrono::Duration::hours(input.expires_in_hours);

    let row = InviteTokenRow {
        token,
        ring_id: ring_id.to_string(),
        r#type: input.r#type.clone(),
        role: input.role.clone(),
        max_uses: input.max_uses,
        use_count: 0,
        max_members: input.max_members,
        expires_at: expires_at.to_rfc3339(),
        revoked_at: None,
        created_by: user_id.to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    invite::insert_token(&state.db, &row).await?;
    Ok(row)
}

pub async fn list_tokens(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    include_expired: bool,
    include_revoked: bool,
) -> Result<Vec<InviteTokenRow>> {
    check_admin(&state.db, ring_id, user_id).await?;
    invite::list_tokens(&state.db, ring_id, include_expired, include_revoked).await
}

pub async fn revoke_token(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    token: &str,
) -> Result<String> {
    check_admin(&state.db, ring_id, user_id).await?;
    invite::revoke_token(&state.db, ring_id, token).await?;
    Ok(Utc::now().to_rfc3339())
}
