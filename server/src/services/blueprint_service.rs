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

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewNode {
    pub label: String,
    pub node_type: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Deserialize)]
pub struct FromTemplateRequest {
    pub template: String,
}

#[derive(Debug, Deserialize)]
pub struct BlueprintGraphInput {
    pub name: String,
    pub nodes: Vec<PreviewNode>,
    pub edges: Vec<PreviewEdge>,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmBlueprintRequest {
    pub blueprint: Option<BlueprintGraphsInput>,
}

#[derive(Debug, Deserialize)]
pub struct BlueprintGraphsInput {
    pub graphs: Vec<BlueprintGraphInput>,
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

pub async fn confirm_with_blueprint(
    state: &AppState,
    ring_id: &str,
    req: &ConfirmBlueprintRequest,
) -> Result<()> {
    if let Some(ref bp) = req.blueprint {
        for graph_input in &bp.graphs {
            let graph_id = ulid::Ulid::new().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO graphs (id, ring_id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .bind(&graph_id)
            .bind(ring_id)
            .bind(&graph_input.name)
            .bind(&now)
            .bind(&now)
            .execute(&state.db)
            .await?;

            let mut label_to_id: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for node in &graph_input.nodes {
                let node_id = ulid::Ulid::new().to_string();
                let tags_json = serde_json::to_string(&node.tags)?;
                let node_now = chrono::Utc::now().to_rfc3339();
                sqlx::query(
                    "INSERT INTO graph_nodes (id, graph_id, ring_id, label, parent_id, node_type, content, tags, markdown_path, metadata, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, NULL, ?5, '', ?6, NULL, '{}', ?7, ?8)",
                )
                .bind(&node_id)
                .bind(&graph_id)
                .bind(ring_id)
                .bind(&node.label)
                .bind(&node.node_type)
                .bind(&tags_json)
                .bind(&node_now)
                .bind(&node_now)
                .execute(&state.db)
                .await?;
                label_to_id.insert(node.label.clone(), node_id);
            }

            for edge in &graph_input.edges {
                if let (Some(src), Some(tgt)) =
                    (label_to_id.get(&edge.from), label_to_id.get(&edge.to))
                {
                    let edge_id = ulid::Ulid::new().to_string();
                    let edge_now = chrono::Utc::now().to_rfc3339();
                    sqlx::query(
                        "INSERT INTO graph_edges (id, graph_id, ring_id, source_id, target_id, relation, label, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', ?7)",
                    )
                    .bind(&edge_id)
                    .bind(&graph_id)
                    .bind(ring_id)
                    .bind(src)
                    .bind(tgt)
                    .bind(&edge.relation)
                    .bind(&edge_now)
                    .execute(&state.db)
                    .await?;
                }
            }
        }
    }
    confirm_blueprint(state, ring_id).await
}
