use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::services::self_data;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ContentRequest {
    pub content: String,
}

pub async fn get_identity(
    State(_state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let (content, exists) = self_data::read_self_file(&self_dir, "identity")?;
    Ok(Json(
        serde_json::json!({ "content": content, "exists": exists }),
    ))
}

pub async fn update_identity(
    State(_state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ContentRequest>,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    self_data::write_self_file(&self_dir, "identity", &body.content)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_style(
    State(_state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let (content, exists) = self_data::read_self_file(&self_dir, "style")?;
    Ok(Json(
        serde_json::json!({ "content": content, "exists": exists }),
    ))
}

pub async fn update_style(
    State(_state): State<AppState>,
    user: AuthUser,
    Json(body): Json<ContentRequest>,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    self_data::write_self_file(&self_dir, "style", &body.content)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_personality(
    State(_state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let (content, exists) = self_data::read_self_file(&self_dir, "personality")?;
    let default = serde_json::json!({
        "tone": "friendly",
        "proactivity": true,
        "suggestions": true,
    });
    let data: serde_json::Value = if exists && !content.is_empty() {
        serde_json::from_str(&content).unwrap_or(default)
    } else {
        default
    };
    Ok(Json(data))
}

pub async fn update_personality(
    State(_state): State<AppState>,
    user: AuthUser,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let content = serde_json::to_string_pretty(&body)?;
    self_data::write_self_file(&self_dir, "personality", &content)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_privacy(
    State(_state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let (content, exists) = self_data::read_self_file(&self_dir, "privacy")?;
    let default = serde_json::json!({
        "level": "standard",
        "share_metrics": false,
        "allow_proactive": true,
    });
    let data: serde_json::Value = if exists && !content.is_empty() {
        serde_json::from_str(&content).unwrap_or(default)
    } else {
        default
    };
    Ok(Json(data))
}

pub async fn update_privacy(
    State(_state): State<AppState>,
    user: AuthUser,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let content = serde_json::to_string_pretty(&body)?;
    self_data::write_self_file(&self_dir, "privacy", &content)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn get_metrics(
    State(_state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let metrics = self_data::read_metrics(&self_dir);
    Ok(Json(metrics))
}

pub async fn export_data(
    State(_state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let data = self_data::export_all_data(&self_dir)?;
    Ok(Json(data))
}

pub async fn reset_data(
    State(_state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    self_data::reset_all_data(&self_dir)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
