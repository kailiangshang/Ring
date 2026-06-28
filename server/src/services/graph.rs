use crate::error::Result;
use crate::models::graph;
use crate::services::git_service::GitService;
use crate::state::AppState;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DocRef {
    pub path: String,
    pub title: String,
    #[serde(rename = "type")]
    pub doc_type: String,
}

pub fn get_node_doc_refs(metadata: &str) -> Vec<DocRef> {
    let val: serde_json::Value = serde_json::from_str(metadata).unwrap_or_default();
    val.get("doc_refs")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

pub fn resolve_doc_content(rings_dir: &Path, ring_id: &str, doc_ref: &DocRef) -> Option<String> {
    let path = rings_dir.join(ring_id).join(&doc_ref.path);
    if path.exists() {
        let content = std::fs::read_to_string(&path).ok()?;
        Some(content.chars().take(5000).collect())
    } else {
        None
    }
}

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

async fn persist_graph_snapshot(state: &AppState, ring_id: &str) {
    let graphs = match graph::list_graphs(&state.db, ring_id).await {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("persist_graph_snapshot: list_graphs failed: {e}");
            return;
        }
    };

    let mut graph_data = Vec::new();
    for g in &graphs {
        let nodes = graph::list_nodes(&state.db, &g.id)
            .await
            .unwrap_or_default();
        let edges = graph::list_edges(&state.db, &g.id)
            .await
            .unwrap_or_default();
        graph_data.push(serde_json::json!({
            "graph": g,
            "nodes": nodes,
            "edges": edges,
        }));
    }

    let snapshot = serde_json::json!({
        "version": "1.0",
        "ring_id": ring_id,
        "graphs": graph_data,
    });

    let json_str = match serde_json::to_string_pretty(&snapshot) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("persist_graph_snapshot: serialization failed: {e}");
            return;
        }
    };

    let graphs_dir = state.rings_dir.join(ring_id).join("graphs");
    if let Err(e) = tokio::fs::create_dir_all(&graphs_dir).await {
        tracing::error!("persist_graph_snapshot: create_dir_all failed: {e}");
        return;
    }

    let file_path = graphs_dir.join("main.json");
    if let Err(e) = tokio::fs::write(&file_path, &json_str).await {
        tracing::error!("persist_graph_snapshot: write failed: {e}");
        return;
    }

    let ring_path: PathBuf = state.rings_dir.join(ring_id);
    if ring_path.join(".git").exists() {
        let git = GitService::new();
        if let Err(e) = git.add_all(&ring_path) {
            tracing::error!("persist_graph_snapshot: git add failed: {e}");
            return;
        }
        if let Err(e) = git.commit(
            &ring_path,
            &format!("sync: update graph snapshot for ring {ring_id}"),
        ) {
            tracing::error!("persist_graph_snapshot: git commit failed: {e}");
        }
    }
}

pub async fn get_full_graph(
    state: &AppState,
    ring_id: &str,
    graph_id: Option<&str>,
) -> Result<GraphResponse> {
    let g = if let Some(gid) = graph_id {
        graph::get_graph_by_id(&state.db, gid).await?
    } else {
        graph::ensure_default_graph(&state.db, ring_id).await?
    };
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
    let g = if let Some(ref gid) = input.graph_id {
        graph::get_graph_by_id(&state.db, gid).await?
    } else {
        graph::ensure_default_graph(&state.db, ring_id).await?
    };
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
    persist_graph_snapshot(state, ring_id).await;
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
    persist_graph_snapshot(state, &node.ring_id).await;
    Ok(node)
}

pub async fn delete_node(state: &AppState, node_id: &str) -> Result<()> {
    let ring_id: String = sqlx::query_scalar("SELECT ring_id FROM graph_nodes WHERE id = ?1")
        .bind(node_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;
    let _ = crate::services::search::delete_search_index(&state.db, "graph_node", node_id).await;
    graph::delete_node(&state.db, node_id).await?;
    persist_graph_snapshot(state, &ring_id).await;
    Ok(())
}

pub async fn create_edge(
    state: &AppState,
    ring_id: &str,
    input: &graph::CreateEdgeInput,
) -> Result<graph::GraphEdgeRow> {
    let g = if let Some(ref gid) = input.graph_id {
        graph::get_graph_by_id(&state.db, gid).await?
    } else {
        graph::ensure_default_graph(&state.db, ring_id).await?
    };
    let id = ulid::Ulid::new().to_string();
    let edge = graph::create_edge(&state.db, &id, &g.id, ring_id, input).await?;
    persist_graph_snapshot(state, ring_id).await;
    Ok(edge)
}

pub async fn delete_edge(state: &AppState, edge_id: &str) -> Result<()> {
    let ring_id: String = sqlx::query_scalar("SELECT ring_id FROM graph_edges WHERE id = ?1")
        .bind(edge_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| crate::error::RingError::Internal(e.to_string()))?;
    graph::delete_edge(&state.db, edge_id).await?;
    persist_graph_snapshot(state, &ring_id).await;
    Ok(())
}
