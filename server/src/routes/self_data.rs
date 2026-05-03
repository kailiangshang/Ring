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
    let files = self_memory::list_memory_files(&self_dir).await?;
    Ok(Json(files))
}

pub async fn get_memory(
    State(_state): State<AppState>,
    user: AuthUser,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let (content, exists) = self_memory::read_memory_file(&self_dir, &name).await?;
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
    self_memory::write_memory_file(&self_dir, &name, content).await?;
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
    self_memory::delete_memory_file(&self_dir, &name).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_greeting(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>> {
    let self_dir = self_data::get_self_dir(&user.token_id);
    let first_today = self_data::check_first_today(&self_dir);
    if !first_today {
        return Ok(Json(
            serde_json::json!({ "greeting": null, "first_today": false }),
        ));
    }
    let user_row = state.get_user_decrypted(&user.token_id).await?;
    let metrics = self_data::read_metrics(&self_dir);
    let ctx = self_data::build_greeting_context(&self_dir, &metrics);
    let most_active_ring = if !ctx.most_active_ring.is_empty() {
        sqlx::query_scalar::<_, String>("SELECT name FROM rings WHERE id = ?1")
            .bind(&ctx.most_active_ring)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten()
            .unwrap_or_default()
    } else {
        String::new()
    };
    let profile_summary = if ctx.user_profile.is_empty() {
        "暂无".to_string()
    } else if ctx.user_profile.len() > 100 {
        format!("{}...", &ctx.user_profile[..100])
    } else {
        ctx.user_profile.clone()
    };
    let goals_summary = if ctx.active_goals.is_empty() {
        "暂无".to_string()
    } else if ctx.active_goals.len() > 100 {
        format!("{}...", &ctx.active_goals[..100])
    } else {
        ctx.active_goals.clone()
    };
    let prompt = format!(
        "基于以下信息生成一句个性化问候（中文，30字以内，自然亲切，不要用引号）：\n\
         - 日期：{}\n\
         - 用户画像：{}\n\
         - 当前目标：{}\n\
         - 最活跃 Ring：{}",
        ctx.date,
        profile_summary,
        goals_summary,
        if most_active_ring.is_empty() {
            "无".to_string()
        } else {
            most_active_ring
        },
    );
    let llm = crate::services::llm::LlmClient::from_user(&user_row)?;
    let greeting = llm
        .chat_complete("你是一个温暖的朋友，用一句话打招呼。".into(), prompt)
        .await
        .ok();
    Ok(Json(
        serde_json::json!({ "greeting": greeting, "first_today": true }),
    ))
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
