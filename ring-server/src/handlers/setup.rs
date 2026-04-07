use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::RingError;
use crate::models::user::NewUser;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatus {
    pub setup_completed: bool,
    pub step: String,
    pub     user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameRequest {
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitlabRequest {
    pub repo_url: String,
    pub auth_type: String,
    pub ssh_key_path: Option<String>,
    pub auto_create: bool,
}

pub async fn get_status(State(state): State<AppState>) -> Result<Json<SetupStatus>, RingError> {
    let completed = state.db.is_setup_completed().await?;
    let (step, user_id) = if completed {
        let users = state.db.list_all_users().await?;
        let uid = users.into_iter().next().map(|u| u.id);
        ("completed".to_string(), uid)
    } else {
        ("username".to_string(), None)
    };
    Ok(Json(SetupStatus {
        setup_completed: completed,
        step,
        user_id,
    }))
}

pub async fn set_username(
    State(state): State<AppState>,
    Json(req): Json<UsernameRequest>,
) -> Result<(StatusCode, Json<UserResponse>), RingError> {
    if state.db.is_setup_completed().await? {
        return Err(RingError::Conflict("setup already completed".into()));
    }
    let name = req.display_name.trim();
    if name.is_empty() {
        return Err(RingError::Validation(
            "display_name must not be empty".into(),
        ));
    }
    if name.len() > 50 {
        return Err(RingError::Validation(
            "display_name must be 50 characters or less".into(),
        ));
    }
    let user = state
        .db
        .create_user(NewUser {
            display_name: name.to_string(),
        })
        .await?;
    Ok((
        StatusCode::OK,
        Json(UserResponse {
            user_id: user.id,
            display_name: user.display_name,
        }),
    ))
}

pub async fn set_llm(
    State(state): State<AppState>,
    Json(req): Json<LlmRequest>,
) -> Result<StatusCode, RingError> {
    if state.db.is_setup_completed().await? {
        return Err(RingError::Conflict("setup already completed".into()));
    }
    let value = serde_json::to_string(&req).map_err(RingError::Serialization)?;
    state.db.set_setting("llm_config", &value).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn set_gitlab(
    State(state): State<AppState>,
    Json(req): Json<GitlabRequest>,
) -> Result<StatusCode, RingError> {
    if state.db.is_setup_completed().await? {
        return Err(RingError::Conflict("setup already completed".into()));
    }
    let value = serde_json::to_string(&req).map_err(RingError::Serialization)?;
    state.db.set_setting("gitlab_config", &value).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn complete(State(state): State<AppState>) -> Result<StatusCode, RingError> {
    if state.db.is_setup_completed().await? {
        return Err(RingError::Conflict("setup already completed".into()));
    }
    let users = state.db.list_all_users().await?;
    let user = users
        .into_iter()
        .next()
        .ok_or_else(|| RingError::Validation("must set username before completing setup".into()))?;
    state.db.complete_setup(&user.id).await?;
    Ok(StatusCode::NO_CONTENT)
}
