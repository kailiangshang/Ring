use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

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

#[derive(Debug, Serialize)]
pub struct JoinInfoResponse {
    pub valid: bool,
    pub reason: Option<String>,
    pub ring_id: Option<String>,
    pub ring_name: Option<String>,
    pub member_count: Option<i64>,
    pub role: Option<String>,
    pub token_type: Option<String>,
}

pub async fn verify_join_token(state: &AppState, token_str: &str) -> Result<JoinInfoResponse> {
    let row = invite::find_token_by_value(&state.db, token_str)
        .await?
        .ok_or_else(|| RingError::NotFound("invite token not found".into()))?;

    if row.revoked_at.is_some() {
        return Ok(JoinInfoResponse {
            valid: false,
            reason: Some("token revoked".into()),
            ring_id: None,
            ring_name: None,
            member_count: None,
            role: None,
            token_type: None,
        });
    }

    let now = Utc::now().to_rfc3339();
    if row.expires_at < now {
        return Ok(JoinInfoResponse {
            valid: false,
            reason: Some("token expired".into()),
            ring_id: None,
            ring_name: None,
            member_count: None,
            role: None,
            token_type: None,
        });
    }

    let ring: Option<(String,)> = sqlx::query_as("SELECT name FROM rings WHERE id = ?1")
        .bind(&row.ring_id)
        .fetch_optional(&state.db)
        .await?;

    let ring_name = ring.map(|r| r.0).unwrap_or_default();
    let member_count = invite::get_member_count(&state.db, &row.ring_id)
        .await
        .unwrap_or(0);

    Ok(JoinInfoResponse {
        valid: true,
        reason: None,
        ring_id: Some(row.ring_id.clone()),
        ring_name: Some(ring_name),
        member_count: Some(member_count),
        role: Some(row.role.clone()),
        token_type: Some(row.r#type.clone()),
    })
}

#[derive(Debug, Deserialize)]
pub struct JoinRequest {
    pub invite_token: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct JoinResponse {
    pub token_id: String,
    pub ring_id: String,
    pub ring_name: String,
    pub role: String,
    pub gitlab_repo_url: Option<String>,
}

pub async fn execute_join(state: &AppState, input: &JoinRequest) -> Result<JoinResponse> {
    if input.display_name.trim().is_empty() {
        return Err(RingError::BadRequest("display_name is required".into()));
    }

    let row = invite::find_token_by_value(&state.db, &input.invite_token)
        .await?
        .ok_or_else(|| RingError::NotFound("invite token not found".into()))?;

    if row.r#type != "open" {
        return Err(RingError::BadRequest(
            "this token is not an open invite".into(),
        ));
    }

    if row.revoked_at.is_some() {
        return Err(RingError::Gone("token has been revoked".into()));
    }

    let now = Utc::now().to_rfc3339();
    if row.expires_at < now {
        return Err(RingError::Gone("token has expired".into()));
    }

    if row.max_uses > 0 && row.use_count >= row.max_uses {
        return Err(RingError::Forbidden("token has reached max uses".into()));
    }

    if let Some(max) = row.max_members {
        let count = invite::get_member_count(&state.db, &row.ring_id).await?;
        if count >= max {
            return Err(RingError::Forbidden("ring has reached max members".into()));
        }
    }

    let token_id = format!("user-{}", Ulid::new());

    if crate::models::member::is_member(&state.db, &row.ring_id, &token_id).await {
        return Err(RingError::Conflict("already a member".into()));
    }

    crate::models::user::create_joiner_user(&state.db, &token_id, &input.display_name).await?;

    crate::models::member::add_member(&state.db, &row.ring_id, &token_id, &row.role).await?;

    invite::increment_use_count(&state.db, &input.invite_token).await?;

    let ring_name: String = sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
        .bind(&row.ring_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    let gitlab_repo_url: Option<String> =
        sqlx::query_scalar("SELECT gitlab_repo_url FROM rings WHERE id = ?1")
            .bind(&row.ring_id)
            .fetch_optional(&state.db)
            .await?
            .flatten();

    Ok(JoinResponse {
        token_id,
        ring_id: row.ring_id,
        ring_name,
        role: row.role,
        gitlab_repo_url,
    })
}

#[derive(Debug, Deserialize)]
pub struct LocalJoinRequest {
    pub invite_token: String,
    pub creator_ip: String,
}

pub async fn local_join(
    state: &AppState,
    user_id: &str,
    input: &LocalJoinRequest,
) -> Result<serde_json::Value> {
    let base_url = format!("http://{}:7420/api", input.creator_ip);

    let info_url = format!("{}/join/info?token={}", base_url, input.invite_token);
    let info_resp = reqwest::get(&info_url)
        .await
        .map_err(|e| RingError::Internal(format!("failed to contact creator: {e}")))?;

    if !info_resp.status().is_success() {
        return Err(RingError::BadGateway);
    }

    let info: serde_json::Value = info_resp
        .json()
        .await
        .map_err(|e| RingError::Internal(format!("failed to parse creator response: {e}")))?;

    if !info["valid"].as_bool().unwrap_or(false) {
        let reason = info["reason"].as_str().unwrap_or("unknown");
        return Err(RingError::BadRequest(format!("invite invalid: {reason}")));
    }

    let user = crate::models::user::get_user(&state.db, user_id).await?;

    let join_url = format!("{}/join", base_url);
    let join_body = serde_json::json!({
        "invite_token": input.invite_token,
        "display_name": user.display_name,
    });
    let join_resp = reqwest::Client::new()
        .post(&join_url)
        .json(&join_body)
        .send()
        .await
        .map_err(|e| RingError::Internal(format!("failed to join via creator: {e}")))?;

    if !join_resp.status().is_success() {
        let status = join_resp.status();
        let body: serde_json::Value = join_resp.json().await.unwrap_or(serde_json::json!({}));
        let msg = body["error"]["message"].as_str().unwrap_or("unknown error");
        return Err(RingError::Internal(format!(
            "creator join failed ({status}): {msg}"
        )));
    }

    let join_result: serde_json::Value = join_resp
        .json()
        .await
        .map_err(|e| RingError::Internal(format!("failed to parse join result: {e}")))?;

    if let Some(repo_url) = join_result["gitlab_repo_url"].as_str() {
        if !repo_url.is_empty() {
            let rings_dir = state.rings_dir.clone();
            let ring_id = join_result["ring_id"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let repo_url = repo_url.to_string();
            tokio::spawn(async move {
                let repo_path = rings_dir.join(&ring_id);
                if !repo_path.join(".git").exists() {
                    if let Err(e) =
                        crate::services::git_service::GitService::clone(&repo_url, &repo_path)
                    {
                        tracing::warn!("git clone failed for ring {ring_id}: {e}");
                        return;
                    }
                    let _ = std::fs::create_dir_all(repo_path.join("archives"));
                    let _ = std::fs::create_dir_all(repo_path.join("graphs"));
                    let _ = std::fs::create_dir_all(repo_path.join(".group"));
                    tracing::info!("git clone completed: ring={ring_id}");
                }
            });
        }
    }

    Ok(join_result)
}

#[derive(Debug, Deserialize)]
pub struct ApplyRequest {
    pub invite_token: String,
    pub display_name: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApplyResponse {
    pub request_id: String,
    pub status: String,
    pub ring_name: String,
}

pub async fn submit_apply(state: &AppState, input: &ApplyRequest) -> Result<ApplyResponse> {
    if input.display_name.trim().is_empty() {
        return Err(RingError::BadRequest("display_name is required".into()));
    }

    let row = invite::find_token_by_value(&state.db, &input.invite_token)
        .await?
        .ok_or_else(|| RingError::NotFound("invite token not found".into()))?;

    if row.r#type != "audit" {
        return Err(RingError::BadRequest(
            "this token is not an audit invite".into(),
        ));
    }

    if row.revoked_at.is_some() {
        return Err(RingError::Gone("token has been revoked".into()));
    }

    let now = Utc::now().to_rfc3339();
    if row.expires_at < now {
        return Err(RingError::Gone("token has expired".into()));
    }

    let request_id = format!("req-{}", Ulid::new());
    let ring_name: String = sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
        .bind(&row.ring_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    let req_row = invite::JoinRequestRow {
        id: request_id.clone(),
        ring_id: row.ring_id.clone(),
        invite_token: input.invite_token.clone(),
        display_name: input.display_name.clone(),
        message: input.message.clone(),
        status: "pending".to_string(),
        reviewer_id: None,
        review_note: None,
        reviewed_at: None,
        created_at: now,
    };

    invite::insert_join_request(&state.db, &req_row).await?;

    Ok(ApplyResponse {
        request_id,
        status: "pending".to_string(),
        ring_name,
    })
}

#[derive(Debug, Serialize)]
pub struct ApplyStatusResponse {
    pub request_id: String,
    pub status: String,
    pub ring_name: Option<String>,
    pub ring_id: Option<String>,
    pub role: Option<String>,
    pub review_note: Option<String>,
    pub token_id: Option<String>,
}

pub async fn check_apply_status(state: &AppState, request_id: &str) -> Result<ApplyStatusResponse> {
    let req = invite::find_join_request(&state.db, request_id)
        .await?
        .ok_or_else(|| RingError::NotFound("join request not found".into()))?;

    let ring_name: String = sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
        .bind(&req.ring_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    let (ring_id, role, token_id) = if req.status == "approved" {
        let token_row = invite::find_token_by_value(&state.db, &req.invite_token)
            .await?
            .map(|t| t.role.clone())
            .unwrap_or_default();
        let user_id: Option<String> = sqlx::query_scalar(
            "SELECT user_id FROM members WHERE ring_id = ?1 AND user_id IN (SELECT token_id FROM users WHERE display_name = ?2) LIMIT 1",
        )
        .bind(&req.ring_id)
        .bind(&req.display_name)
        .fetch_optional(&state.db)
        .await?;
        (Some(req.ring_id.clone()), Some(token_row), user_id)
    } else {
        (None, None, None)
    };

    Ok(ApplyStatusResponse {
        request_id: req.id,
        status: req.status,
        ring_name: Some(ring_name),
        ring_id,
        role,
        review_note: req.review_note,
        token_id,
    })
}

pub async fn list_join_requests(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    status_filter: &str,
) -> Result<Vec<invite::JoinRequestRow>> {
    check_admin(&state.db, ring_id, user_id).await?;
    invite::list_pending_requests(&state.db, ring_id, status_filter).await
}

pub async fn approve_join_request(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    request_id: &str,
) -> Result<serde_json::Value> {
    check_admin(&state.db, ring_id, user_id).await?;

    let req = invite::find_join_request(&state.db, request_id)
        .await?
        .ok_or_else(|| RingError::NotFound("join request not found".into()))?;

    if req.ring_id != ring_id {
        return Err(RingError::NotFound("join request not found".into()));
    }

    if req.status != "pending" {
        return Err(RingError::Conflict("request is not pending".into()));
    }

    let token_row = invite::find_token_by_value(&state.db, &req.invite_token)
        .await?
        .ok_or_else(|| RingError::Gone("invite token no longer exists".into()))?;

    if token_row.revoked_at.is_some() {
        return Err(RingError::Gone("token has been revoked".into()));
    }

    let now = Utc::now().to_rfc3339();
    if token_row.expires_at < now {
        return Err(RingError::Gone("token has expired".into()));
    }

    if token_row.max_uses > 0 && token_row.use_count >= token_row.max_uses {
        return Err(RingError::Forbidden("token has reached max uses".into()));
    }

    if let Some(max) = token_row.max_members {
        let count = invite::get_member_count(&state.db, &token_row.ring_id).await?;
        if count >= max {
            return Err(RingError::Forbidden("ring has reached max members".into()));
        }
    }

    let new_token_id = format!("user-{}", Ulid::new());

    crate::models::user::create_joiner_user(&state.db, &new_token_id, &req.display_name).await?;

    crate::models::member::add_member(&state.db, ring_id, &new_token_id, &token_row.role).await?;

    invite::increment_use_count(&state.db, &req.invite_token).await?;

    invite::update_join_request_status(&state.db, request_id, "approved", user_id, None).await?;

    let ring_name: String = sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
        .bind(ring_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or_default();

    Ok(serde_json::json!({
        "ok": true,
        "token_id": new_token_id,
        "ring_name": ring_name,
        "role": token_row.role,
    }))
}

pub async fn reject_join_request(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    request_id: &str,
    note: Option<&str>,
) -> Result<serde_json::Value> {
    check_admin(&state.db, ring_id, user_id).await?;

    let req = invite::find_join_request(&state.db, request_id)
        .await?
        .ok_or_else(|| RingError::NotFound("join request not found".into()))?;

    if req.ring_id != ring_id {
        return Err(RingError::NotFound("join request not found".into()));
    }

    invite::update_join_request_status(&state.db, request_id, "rejected", user_id, note).await?;

    Ok(serde_json::json!({ "ok": true }))
}
