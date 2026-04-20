use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::user;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ContentRequest {
    pub content: String,
}

pub async fn get_identity(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = get_self_dir(&state, &user).await?;
    let (content, exists) = crate::services::self_data::read_self_file(&self_dir, "identity")?;
    Ok(Json(
        serde_json::json!({ "content": content, "exists": exists }),
    ))
}

pub async fn update_identity(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ContentRequest>,
) -> Result<Json<serde_json::Value>> {
    let self_dir = get_self_dir(&state, &user).await?;
    crate::services::self_data::write_self_file(&self_dir, "identity", &body.content)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_style(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = get_self_dir(&state, &user).await?;
    let (content, exists) = crate::services::self_data::read_self_file(&self_dir, "style")?;
    Ok(Json(
        serde_json::json!({ "content": content, "exists": exists }),
    ))
}

pub async fn update_style(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ContentRequest>,
) -> Result<Json<serde_json::Value>> {
    let self_dir = get_self_dir(&state, &user).await?;
    crate::services::self_data::write_self_file(&self_dir, "style", &body.content)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_metrics(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = get_self_dir(&state, &user).await?;
    let metrics = crate::services::self_data::read_metrics(&self_dir);
    Ok(Json(metrics))
}

async fn get_self_dir(
    state: &AppState,
    user: &AuthUser,
) -> crate::error::Result<std::path::PathBuf> {
    let _user_row = user::get_user(&state.db, &user.token_id).await?;
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    Ok(std::path::PathBuf::from(format!("{home}/.ring/self")))
}
