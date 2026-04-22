use axum::extract::State;
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::config::UpdateLLMConfig;
use crate::models::conversation_token::UpdateAutoCompact;
use crate::services::config;
use crate::state::AppState;

pub async fn get_llm_config(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<crate::models::config::LLMConfigResponse>> {
    let cfg = config::get_llm_config(&state, &user.token_id).await?;
    Ok(Json(cfg))
}

pub async fn update_llm_config(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateLLMConfig>,
) -> Result<Json<crate::models::config::LLMConfigResponse>> {
    let cfg = config::update_llm_config(&state, &user.token_id, body).await?;
    Ok(Json(cfg))
}

pub async fn get_auto_compact(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let auto_compact = crate::models::conversation_token::get_auto_compact(&state.db, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "auto_compact": auto_compact })))
}

pub async fn update_auto_compact(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<UpdateAutoCompact>,
) -> Result<Json<serde_json::Value>> {
    let auto_compact = crate::models::conversation_token::update_auto_compact(&state.db, &user.token_id, body.auto_compact).await?;
    Ok(Json(serde_json::json!({ "auto_compact": auto_compact })))
}

pub async fn test_llm_config(
    Json(body): Json<crate::models::config::TestLLMRequest>,
) -> Result<Json<serde_json::Value>> {
    let (ok, message) = crate::services::llm::test_connection(
        &body.provider,
        &body.model,
        body.api_key.as_deref(),
        body.base_url.as_deref(),
    )
    .await?;
    Ok(Json(serde_json::json!({ "ok": ok, "message": message })))
}

pub async fn test_gitlab_config(
    Json(body): Json<crate::models::config::TestGitLabRequest>,
) -> Result<Json<serde_json::Value>> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v4/user", body.url.trim_end_matches('/'));
    let res = client
        .get(&url)
        .header("PRIVATE-TOKEN", &body.token)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                Ok(Json(
                    serde_json::json!({ "ok": true, "message": "GitLab connection successful" }),
                ))
            } else {
                let status = resp.status().as_u16();
                Ok(Json(
                    serde_json::json!({ "ok": false, "message": format!("GitLab returned status {}", status) }),
                ))
            }
        }
        Err(e) => Ok(Json(
            serde_json::json!({ "ok": false, "message": format!("{}", e) }),
        )),
    }
}
