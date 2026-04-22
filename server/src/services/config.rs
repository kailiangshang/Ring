use crate::error::Result;
use crate::models::config::{self, LLMConfigResponse, UpdateLLMConfig};
use crate::state::AppState;

pub async fn get_llm_config(state: &AppState, user_id: &str) -> Result<LLMConfigResponse> {
    config::get_llm_config(&state.db, user_id).await
}

pub async fn update_llm_config(
    state: &AppState,
    user_id: &str,
    input: UpdateLLMConfig,
) -> Result<LLMConfigResponse> {
    config::update_llm_config(&state.db, user_id, &input).await
}

pub async fn get_privacy_filters(
    state: &AppState,
    user_id: &str,
) -> Result<config::PrivacyFiltersResponse> {
    config::get_privacy_filters(&state.db, user_id).await
}

pub async fn update_privacy_filters(
    state: &AppState,
    user_id: &str,
    input: config::UpdatePrivacyFilters,
) -> Result<config::PrivacyFiltersResponse> {
    config::update_privacy_filters(&state.db, user_id, &input).await
}
