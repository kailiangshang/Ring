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
    let node = graph::create_node(&state.db, &id, &g.id, ring_id, input).await?;
    let ring_name = crate::services::search::get_ring_name(&state.db, &node.ring_id)
        .await
        .unwrap_or_default();
    let content = format!("{} {}", &node.content, &node.tags);
    let metadata =
        serde_json::json!({"node_type": &node.node_type, "graph_id": &node.graph_id}).to_string();
    let _ = crate::services::search::upsert_search_index(
        &state.db,
        "graph_node",
        &node.id,
        &node.ring_id,
        &ring_name,
        &node.label,
        &content,
        &metadata,
    )
    .await;
    Ok(node)
}

pub async fn update_node(
    state: &AppState,
    node_id: &str,
    input: &graph::UpdateNodeInput,
) -> Result<graph::GraphNodeRow> {
    let node = graph::update_node(&state.db, node_id, input).await?;
    let ring_name = crate::services::search::get_ring_name(&state.db, &node.ring_id)
        .await
        .unwrap_or_default();
    let content = format!("{} {}", &node.content, &node.tags);
    let metadata =
        serde_json::json!({"node_type": &node.node_type, "graph_id": &node.graph_id}).to_string();
    let _ = crate::services::search::upsert_search_index(
        &state.db,
        "graph_node",
        &node.id,
        &node.ring_id,
        &ring_name,
        &node.label,
        &content,
        &metadata,
    )
    .await;
    Ok(node)
}

pub async fn delete_node(state: &AppState, node_id: &str) -> Result<()> {
    let _ = crate::services::search::delete_search_index(&state.db, "graph_node", node_id).await;
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
