use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;

use crate::error::Result;
use crate::extractors::auth::AuthUser;
use crate::services::skill;
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct ListSkillsResponse {
    pub skills: Vec<skill::SkillInfo>,
}

pub async fn list_skills(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<ListSkillsResponse>> {
    let skills = skill::list_skills(&state.skills_dir);
    Ok(Json(ListSkillsResponse { skills }))
}

#[derive(Debug, Deserialize)]
pub struct InstallSkillRequest {
    pub name: String,
    pub source_url: String,
}

#[derive(Debug, serde::Serialize)]
pub struct InstallSkillResponse {
    pub ok: bool,
    pub name: String,
    pub description: String,
}

pub async fn install_skill_handler(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<InstallSkillRequest>,
) -> Result<Json<InstallSkillResponse>> {
    let info = skill::install_skill(&state.skills_dir, &body.name, &body.source_url).await?;
    Ok(Json(InstallSkillResponse {
        ok: true,
        name: info.name,
        description: info.description,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct SkillDetailResponse {
    pub name: String,
    pub description: String,
    pub source: String,
    pub content: String,
}

pub async fn get_skill_detail(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(name): Path<String>,
) -> Result<Json<SkillDetailResponse>> {
    let resolved = skill::get_skill_resolved(&name, &state.skills_dir)
        .ok_or_else(|| crate::error::RingError::NotFound(format!("Skill '{name}' not found")))?;
    Ok(Json(SkillDetailResponse {
        name: resolved.name,
        description: resolved.description,
        source: resolved.source,
        content: resolved.content,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct RemoveSkillResponse {
    pub ok: bool,
    pub name: String,
}

pub async fn remove_skill(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(name): Path<String>,
) -> Result<Json<RemoveSkillResponse>> {
    skill::remove_skill(&state.skills_dir, &name)?;
    Ok(Json(RemoveSkillResponse { ok: true, name }))
}
