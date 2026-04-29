use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::services::{self_data, self_memory};
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

pub async fn list_memories(
    State(_state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<serde_json::Value>>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let files = self_memory::list_memory_files(&self_dir)?;
    Ok(Json(files))
}

pub async fn get_memory(
    State(_state): State<AppState>,
    user: AuthUser,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let (content, exists) = self_memory::read_memory_file(&self_dir, &name)?;
    Ok(Json(
        serde_json::json!({ "name": name, "content": content, "exists": exists }),
    ))
}

pub async fn update_memory(
    State(_state): State<AppState>,
    user: AuthUser,
    Path(name): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let content = body.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let self_dir = self_data::get_self_dir(&user.token_id);
    self_memory::write_memory_file(&self_dir, &name, content)?;
    Ok(Json(
        serde_json::json!({ "name": name, "content": content }),
    ))
}

pub async fn delete_memory(
    State(_state): State<AppState>,
    user: AuthUser,
    Path(name): Path<String>,
) -> Result<StatusCode> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    self_memory::delete_memory_file(&self_dir, &name)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct HeartbeatRequest {
    pub view: String,
    #[serde(default)]
    pub dwell_time: u64,
    #[allow(dead_code)]
    pub ring_id: Option<String>,
}

pub async fn heartbeat(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<HeartbeatRequest>,
) -> Result<StatusCode> {
    let _ = user;
    let valid_views = ["self_panel", "ring_chat", "graph", "session", "archive"];
    if !valid_views.contains(&body.view.as_str()) {
        return Ok(StatusCode::BAD_REQUEST);
    }
    let mut buf = state.dwell_buffer.lock().await;
    let user_buf = buf.entry(user.token_id.clone()).or_default();
    let entry = user_buf.entry(body.view).or_insert(0);
    *entry += if body.dwell_time > 0 {
        body.dwell_time
    } else {
        30
    };
    Ok(StatusCode::NO_CONTENT)
}
