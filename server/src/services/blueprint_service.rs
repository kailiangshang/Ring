use serde::{Deserialize, Serialize};

use crate::error::{Result, RingError};
use crate::services::blueprint::get_builtin_templates;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct BlueprintResponse {
    pub status: String,
    pub template: Option<String>,
    pub preview: Option<BlueprintPreview>,
}

#[derive(Debug, Serialize)]
pub struct BlueprintPreview {
    pub nodes: Vec<PreviewNode>,
    pub edges: Vec<PreviewEdge>,
}

#[derive(Debug, Serialize)]
pub struct PreviewNode {
    pub label: String,
    pub node_type: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PreviewEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Deserialize)]
pub struct FromTemplateRequest {
    pub template: String,
}

pub async fn get_blueprint(state: &AppState, ring_id: &str) -> Result<BlueprintResponse> {
    let row = sqlx::query_as::<_, (String,)>("SELECT blueprint_status FROM rings WHERE id = ?1")
        .bind(ring_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("ring {} not found", ring_id)))?;

    Ok(BlueprintResponse {
        status: row.0,
        template: None,
        preview: None,
    })
}

pub async fn preview_from_template(
    _state: &AppState,
    template_id: &str,
) -> Result<BlueprintPreview> {
    let templates = get_builtin_templates();
    let template = templates
        .into_iter()
        .find(|t| t.id == template_id)
        .ok_or_else(|| RingError::NotFound(format!("template {} not found", template_id)))?;

    Ok(BlueprintPreview {
        nodes: template
            .nodes
            .into_iter()
            .map(|n| PreviewNode {
                label: n.label,
                node_type: n.node_type,
                tags: n.tags,
            })
            .collect(),
        edges: template
            .edges
            .into_iter()
            .map(|e| PreviewEdge {
                from: e.from,
                to: e.to,
                relation: e.relation,
            })
            .collect(),
    })
}

pub async fn confirm_blueprint(state: &AppState, ring_id: &str) -> Result<()> {
    sqlx::query("UPDATE rings SET blueprint_status = ?1 WHERE id = ?2")
        .bind("confirmed")
        .bind(ring_id)
        .execute(&state.db)
        .await?;
    Ok(())
}
