use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct UserRow {
    pub token_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub is_creator: bool,
    pub llm_provider: String,
    pub llm_api_key: Option<String>,
    pub llm_model: String,
    pub llm_base_url: Option<String>,
    pub gitlab_url: Option<String>,
    pub gitlab_token: Option<String>,
    pub github_url: Option<String>,
    pub github_token: Option<String>,
    pub privacy_filters: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub display_name: String,
    pub avatar: Option<String>,
    pub llm_provider: String,
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub llm_base_url: Option<String>,
    pub gitlab_url: Option<String>,
    pub gitlab_token: Option<String>,
    pub privacy_filters: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub llm_provider: Option<String>,
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub llm_base_url: Option<String>,
    pub gitlab_url: Option<String>,
    pub gitlab_token: Option<String>,
    pub privacy_filters: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub token_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
}

impl UserRow {
    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            token_id: self.token_id.clone(),
            display_name: self.display_name.clone(),
            avatar: self.avatar.clone(),
        }
    }
}

pub async fn create_user(
    pool: &sqlx::SqlitePool,
    token_id: &str,
    input: &CreateUser,
) -> Result<UserRow> {
    let model = input.llm_model.as_deref().unwrap_or("gpt-4o");
    sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, llm_base_url, gitlab_url, gitlab_token, privacy_filters)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         RETURNING *"
    )
        .bind(token_id)
        .bind(&input.display_name)
        .bind(&input.avatar)
        .bind(&input.llm_provider)
        .bind(&input.llm_api_key)
        .bind(model)
        .bind(&input.llm_base_url)
        .bind(&input.gitlab_url)
        .bind(&input.gitlab_token)
        .bind(&input.privacy_filters)
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

pub async fn create_joiner_user(
    pool: &sqlx::SqlitePool,
    token_id: &str,
    display_name: &str,
) -> Result<UserRow> {
    sqlx::query_as::<_, UserRow>(
        "INSERT INTO users (token_id, display_name, avatar, is_creator, llm_provider, llm_api_key, llm_model, llm_base_url, gitlab_url, gitlab_token, privacy_filters)
         VALUES (?1, ?2, NULL, 0, 'openai', NULL, 'gpt-4o', NULL, NULL, NULL, NULL)
         RETURNING *",
    )
    .bind(token_id)
    .bind(display_name)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn get_user(pool: &sqlx::SqlitePool, token_id: &str) -> Result<UserRow> {
    sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE token_id = ?1")
        .bind(token_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("user {token_id} not found")))
}

pub async fn update_user(
    pool: &sqlx::SqlitePool,
    token_id: &str,
    input: &UpdateUser,
) -> Result<UserRow> {
    let current = get_user(pool, token_id).await?;
    sqlx::query_as::<_, UserRow>(
        "UPDATE users SET
            display_name = ?1, avatar = ?2, llm_provider = ?3, llm_api_key = ?4,
            llm_model = ?5, llm_base_url = ?6, gitlab_url = ?7, gitlab_token = ?8,
            privacy_filters = ?9
         WHERE token_id = ?10
         RETURNING *",
    )
    .bind(
        input
            .display_name
            .as_deref()
            .unwrap_or(&current.display_name),
    )
    .bind(input.avatar.as_ref().or(current.avatar.as_ref()))
    .bind(
        input
            .llm_provider
            .as_deref()
            .unwrap_or(&current.llm_provider),
    )
    .bind(input.llm_api_key.as_ref().or(current.llm_api_key.as_ref()))
    .bind(input.llm_model.as_deref().unwrap_or(&current.llm_model))
    .bind(
        input
            .llm_base_url
            .as_ref()
            .or(current.llm_base_url.as_ref()),
    )
    .bind(input.gitlab_url.as_ref().or(current.gitlab_url.as_ref()))
    .bind(
        input
            .gitlab_token
            .as_ref()
            .or(current.gitlab_token.as_ref()),
    )
    .bind(
        input
            .privacy_filters
            .as_ref()
            .or(current.privacy_filters.as_ref()),
    )
    .bind(token_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}
