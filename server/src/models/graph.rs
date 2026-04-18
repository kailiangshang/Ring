use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct GraphRow {
    pub id: String,
    pub ring_id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct GraphNodeRow {
    pub id: String,
    pub graph_id: String,
    pub ring_id: String,
    pub label: String,
    pub parent_id: Option<String>,
    pub node_type: String,
    pub content: String,
    pub tags: String,
    pub markdown_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct GraphEdgeRow {
    pub id: String,
    pub graph_id: String,
    pub ring_id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNodeInput {
    pub label: String,
    pub parent_id: Option<String>,
    #[serde(default = "default_node_type")]
    pub node_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content: String,
}

fn default_node_type() -> String {
    "topic".into()
}

#[derive(Debug, Deserialize)]
pub struct UpdateNodeInput {
    pub label: Option<String>,
    pub tags: Option<Vec<String>>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEdgeInput {
    pub source_id: String,
    pub target_id: String,
    #[serde(default = "default_relation")]
    pub relation: String,
    #[serde(default)]
    pub label: String,
}

fn default_relation() -> String {
    "related_to".into()
}

pub async fn ensure_default_graph(pool: &sqlx::SqlitePool, ring_id: &str) -> Result<GraphRow> {
    if let Some(graph) =
        sqlx::query_as::<_, GraphRow>("SELECT * FROM graphs WHERE ring_id = ?1 LIMIT 1")
            .bind(ring_id)
            .fetch_optional(pool)
            .await?
    {
        return Ok(graph);
    }

    let id = format!("graph-{ring_id}");
    sqlx::query_as::<_, GraphRow>(
        "INSERT INTO graphs (id, ring_id, name) VALUES (?1, ?2, 'main') RETURNING *",
    )
    .bind(&id)
    .bind(ring_id)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn list_nodes(pool: &sqlx::SqlitePool, graph_id: &str) -> Result<Vec<GraphNodeRow>> {
    sqlx::query_as::<_, GraphNodeRow>(
        "SELECT * FROM graph_nodes WHERE graph_id = ?1 ORDER BY created_at",
    )
    .bind(graph_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn list_edges(pool: &sqlx::SqlitePool, graph_id: &str) -> Result<Vec<GraphEdgeRow>> {
    sqlx::query_as::<_, GraphEdgeRow>(
        "SELECT * FROM graph_edges WHERE graph_id = ?1 ORDER BY created_at",
    )
    .bind(graph_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn create_node(
    pool: &sqlx::SqlitePool,
    id: &str,
    graph_id: &str,
    ring_id: &str,
    input: &CreateNodeInput,
) -> Result<GraphNodeRow> {
    sqlx::query_as::<_, GraphNodeRow>(
        "INSERT INTO graph_nodes (id, graph_id, ring_id, label, parent_id, node_type, content, tags)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         RETURNING *",
    )
    .bind(id)
    .bind(graph_id)
    .bind(ring_id)
    .bind(&input.label)
    .bind(&input.parent_id)
    .bind(&input.node_type)
    .bind(&input.content)
    .bind(serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".into()))
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn update_node(
    pool: &sqlx::SqlitePool,
    node_id: &str,
    input: &UpdateNodeInput,
) -> Result<GraphNodeRow> {
    let current = sqlx::query_as::<_, GraphNodeRow>("SELECT * FROM graph_nodes WHERE id = ?1")
        .bind(node_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound("node not found".into()))?;

    let label = input.label.as_deref().unwrap_or(&current.label);
    let tags = input
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_else(|_| "[]".into()))
        .unwrap_or(current.tags);
    let content = input.content.as_deref().unwrap_or(&current.content);

    sqlx::query_as::<_, GraphNodeRow>(
        "UPDATE graph_nodes SET label = ?1, tags = ?2, content = ?3, updated_at = datetime('now')
         WHERE id = ?4 RETURNING *",
    )
    .bind(label)
    .bind(tags)
    .bind(content)
    .bind(node_id)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn delete_node(pool: &sqlx::SqlitePool, node_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM graph_nodes WHERE id = ?1")
        .bind(node_id)
        .execute(pool)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("node not found".into()));
    }
    Ok(())
}

pub async fn create_edge(
    pool: &sqlx::SqlitePool,
    id: &str,
    graph_id: &str,
    ring_id: &str,
    input: &CreateEdgeInput,
) -> Result<GraphEdgeRow> {
    sqlx::query_as::<_, GraphEdgeRow>(
        "INSERT INTO graph_edges (id, graph_id, ring_id, source_id, target_id, relation, label)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         RETURNING *",
    )
    .bind(id)
    .bind(graph_id)
    .bind(ring_id)
    .bind(&input.source_id)
    .bind(&input.target_id)
    .bind(&input.relation)
    .bind(&input.label)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn delete_edge(pool: &sqlx::SqlitePool, edge_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM graph_edges WHERE id = ?1")
        .bind(edge_id)
        .execute(pool)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("edge not found".into()));
    }
    Ok(())
}

pub async fn update_node_markdown_path(
    pool: &sqlx::SqlitePool,
    node_id: &str,
    markdown_path: &str,
) -> Result<GraphNodeRow> {
    sqlx::query_as::<_, GraphNodeRow>(
        "UPDATE graph_nodes SET markdown_path = ?1, updated_at = datetime('now')
         WHERE id = ?2 RETURNING *",
    )
    .bind(markdown_path)
    .bind(node_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound("node not found".into()))
}
