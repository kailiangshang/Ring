use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::error::{Result, RingError};
use crate::models::config::{get_setup_done, set_setup_done};
use crate::models::user;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SetupRequest {
    pub display_name: String,
    pub avatar: Option<String>,
    pub llm_provider: String,
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub llm_base_url: Option<String>,
    pub gitlab_url: Option<String>,
    pub gitlab_token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SetupStatusResponse {
    pub is_setup: bool,
    pub step: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SetupResponse {
    pub token_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
}

pub async fn get_status(state: &AppState) -> Result<SetupStatusResponse> {
    let is_setup = get_setup_done(&state.db).await?;
    Ok(SetupStatusResponse {
        is_setup,
        step: if is_setup {
            None
        } else {
            Some("identity".into())
        },
    })
}

pub async fn submit_setup(state: &AppState, input: SetupRequest) -> Result<SetupResponse> {
    let done = get_setup_done(&state.db).await?;
    if done {
        return Err(RingError::Conflict("setup already completed".into()));
    }

    if input.llm_provider != "ollama" && input.llm_api_key.is_none() {
        return Err(RingError::BadRequest(
            "llm_api_key required for non-ollama providers".into(),
        ));
    }

    let token_id = format!("user-{}", Ulid::new());
    let encrypted_api_key = input.llm_api_key.map(|k| state.encryption.encrypt(&k));
    let encrypted_gitlab_token = input.gitlab_token.map(|t| state.encryption.encrypt(&t));

    let create_input = user::CreateUser {
        display_name: input.display_name,
        avatar: input.avatar,
        llm_provider: input.llm_provider,
        llm_api_key: encrypted_api_key,
        llm_model: input.llm_model,
        llm_base_url: input.llm_base_url,
        gitlab_url: input.gitlab_url,
        gitlab_token: encrypted_gitlab_token,
        privacy_filters: None,
    };
    let user = user::create_user(&state.db, &token_id, &create_input).await?;
    sqlx::query("UPDATE users SET token_created_at = datetime('now') WHERE token_id = ?1")
        .bind(&user.token_id)
        .execute(&state.db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
    set_setup_done(&state.db).await?;

    Ok(SetupResponse {
        token_id: user.token_id,
        display_name: user.display_name,
        avatar: user.avatar,
    })
}

pub async fn update_setup(
    state: &AppState,
    token_id: &str,
    input: SetupRequest,
) -> Result<SetupResponse> {
    let encrypted_api_key = input.llm_api_key.map(|k| state.encryption.encrypt(&k));
    let encrypted_gitlab_token = input.gitlab_token.map(|t| state.encryption.encrypt(&t));

    let update_input = user::UpdateUser {
        display_name: Some(input.display_name),
        avatar: input.avatar,
        llm_provider: Some(input.llm_provider),
        llm_api_key: encrypted_api_key,
        llm_model: input.llm_model,
        llm_base_url: input.llm_base_url,
        gitlab_url: input.gitlab_url,
        gitlab_token: encrypted_gitlab_token,
        privacy_filters: None,
    };
    let user = user::update_user(&state.db, token_id, &update_input).await?;
    Ok(SetupResponse {
        token_id: user.token_id,
        display_name: user.display_name,
        avatar: user.avatar,
    })
}
