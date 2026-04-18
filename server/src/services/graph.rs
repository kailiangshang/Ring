use crate::error::Result;
use crate::models::graph;
use crate::state::AppState;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub id: String,
    pub name: String,
    pub ring_id: String,
    pub nodes: Vec<graph::GraphNodeRow>,
    pub edges: Vec<graph::GraphEdgeRow>,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_full_graph(state: &AppState, ring_id: &str) -> Result<GraphResponse> {
    let g = graph::ensure_default_graph(&state.db, ring_id).await?;
    let nodes = graph::list_nodes(&state.db, &g.id).await?;
    let edges = graph::list_edges(&state.db, &g.id).await?;
    Ok(GraphResponse {
        id: g.id,
        name: g.name,
        ring_id: g.ring_id,
        nodes,
        edges,
        created_at: g.created_at,
        updated_at: g.updated_at,
    })
}

pub async fn create_node(
    state: &AppState,
    ring_id: &str,
    input: &graph::CreateNodeInput,
) -> Result<graph::GraphNodeRow> {
    let g = graph::ensure_default_graph(&state.db, ring_id).await?;
    let id = ulid::Ulid::new().to_string();
    graph::create_node(&state.db, &id, &g.id, ring_id, input).await
}

pub async fn update_node(
    state: &AppState,
    node_id: &str,
    input: &graph::UpdateNodeInput,
) -> Result<graph::GraphNodeRow> {
    graph::update_node(&state.db, node_id, input).await
}

pub async fn delete_node(state: &AppState, node_id: &str) -> Result<()> {
    graph::delete_node(&state.db, node_id).await
}

pub async fn create_edge(
    state: &AppState,
    ring_id: &str,
    input: &graph::CreateEdgeInput,
) -> Result<graph::GraphEdgeRow> {
    let g = graph::ensure_default_graph(&state.db, ring_id).await?;
    let id = ulid::Ulid::new().to_string();
    graph::create_edge(&state.db, &id, &g.id, ring_id, input).await
}

pub async fn delete_edge(state: &AppState, edge_id: &str) -> Result<()> {
    graph::delete_edge(&state.db, edge_id).await
}
