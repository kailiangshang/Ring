use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Serialize)]
pub struct LLMConfigResponse {
    pub provider: String,
    pub model: String,
    pub api_key_set: bool,
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLLMConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestLLMRequest {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TestGitLabRequest {
    pub url: String,
    pub token: String,
}

pub async fn get_llm_config(pool: &sqlx::SqlitePool, user_id: &str) -> Result<LLMConfigResponse> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        "SELECT llm_provider, llm_model, llm_api_key, llm_base_url FROM users WHERE token_id = ?1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| crate::error::RingError::NotFound("user not found".into()))?;

    Ok(LLMConfigResponse {
        provider: row.0,
        model: row.1,
        api_key_set: row.2.is_some(),
        base_url: row.3,
    })
}

pub async fn update_llm_config(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    input: &UpdateLLMConfig,
) -> Result<LLMConfigResponse> {
    let current = get_llm_config(pool, user_id).await?;

    let provider = input.provider.as_deref().unwrap_or(&current.provider);
    let model = input.model.as_deref().unwrap_or(&current.model);
    let api_key = input
        .api_key
        .as_deref()
        .or(if current.api_key_set { None } else { Some("") });

    if let Some(key) = api_key {
        sqlx::query("UPDATE users SET llm_provider = ?1, llm_model = ?2, llm_api_key = ?3, llm_base_url = ?4 WHERE token_id = ?5")
            .bind(provider)
            .bind(model)
            .bind(if key.is_empty() { None as Option<&str> } else { Some(key) })
            .bind(input.base_url.as_deref().or(current.base_url.as_deref()))
            .bind(user_id)
            .execute(pool)
            .await?;
    } else {
        sqlx::query("UPDATE users SET llm_provider = ?1, llm_model = ?2, llm_base_url = ?3 WHERE token_id = ?4")
            .bind(provider)
            .bind(model)
            .bind(input.base_url.as_deref().or(current.base_url.as_deref()))
            .bind(user_id)
            .execute(pool)
            .await?;
    }

    get_llm_config(pool, user_id).await
}

#[derive(Debug, Serialize)]
pub struct PrivacyFiltersResponse {
    pub phone: bool,
    pub id_card: bool,
    pub email: bool,
    pub bank_card: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePrivacyFilters {
    pub phone: Option<bool>,
    pub id_card: Option<bool>,
    pub email: Option<bool>,
    pub bank_card: Option<bool>,
}

pub async fn get_privacy_filters(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> Result<PrivacyFiltersResponse> {
    let row = sqlx::query_scalar::<_, Option<String>>(
        "SELECT privacy_filters FROM users WHERE token_id = ?1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    let filters = match row {
        Some(json) => crate::services::privacy_filter::PrivacyFilters::from_json(&json),
        None => crate::services::privacy_filter::PrivacyFilters::default(),
    };

    Ok(PrivacyFiltersResponse {
        phone: filters.phone,
        id_card: filters.id_card,
        email: filters.email,
        bank_card: filters.bank_card,
    })
}

pub async fn update_privacy_filters(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    input: &UpdatePrivacyFilters,
) -> Result<PrivacyFiltersResponse> {
    let current = get_privacy_filters(pool, user_id).await?;

    let filters = crate::services::privacy_filter::PrivacyFilters {
        phone: input.phone.unwrap_or(current.phone),
        id_card: input.id_card.unwrap_or(current.id_card),
        email: input.email.unwrap_or(current.email),
        bank_card: input.bank_card.unwrap_or(current.bank_card),
    };

    sqlx::query("UPDATE users SET privacy_filters = ?1 WHERE token_id = ?2")
        .bind(filters.to_json())
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(PrivacyFiltersResponse {
        phone: filters.phone,
        id_card: filters.id_card,
        email: filters.email,
        bank_card: filters.bank_card,
    })
}

pub async fn get_setup_done(pool: &sqlx::SqlitePool) -> Result<bool> {
    let done = sqlx::query_scalar::<_, bool>("SELECT is_setup FROM setup_state WHERE id = 1")
        .fetch_one(pool)
        .await?;
    Ok(done)
}

pub async fn set_setup_done(pool: &sqlx::SqlitePool) -> Result<()> {
    sqlx::query("UPDATE setup_state SET is_setup = 1 WHERE id = 1")
        .execute(pool)
        .await?;
    Ok(())
}
