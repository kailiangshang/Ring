use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::error::RingError;
use crate::models::graph_model::{
    CreateEdgeRequest, CreateNodeRequest, EdgeResponse, GraphDetailResponse, NodeContentResponse,
    NodeResponse, UpdateNodeRequest,
};
use crate::services::graph_service::GraphService;
use crate::state::AppState;

fn make_graph_service(state: &AppState) -> GraphService {
    GraphService::new(
        state.graph_store.clone(),
        state.search_service.clone(),
        state.config.data_dir.clone(),
    )
}

pub async fn list_graphs(
    State(state): State<AppState>,
    Path(_ring_id): Path<String>,
) -> Result<Json<Vec<String>>, RingError> {
    let service = make_graph_service(&state);
    let graphs = service.list_graphs(&_ring_id).await?;
    Ok(Json(graphs))
}

pub async fn get_graph(
    State(state): State<AppState>,
    Path((_ring_id, graph_id)): Path<(String, String)>,
) -> Result<Json<GraphDetailResponse>, RingError> {
    let graph_data = state.graph_store.export_graph_json(&graph_id).await?;
    let nodes: Vec<NodeResponse> = graph_data
        .nodes
        .into_iter()
        .map(NodeResponse::from)
        .collect();
    let edges: Vec<EdgeResponse> = graph_data
        .edges
        .into_iter()
        .map(EdgeResponse::from)
        .collect();
    Ok(Json(GraphDetailResponse {
        graph_id,
        nodes,
        edges,
    }))
}

pub async fn create_node(
    State(state): State<AppState>,
    Path((_ring_id, graph_id)): Path<(String, String)>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<(StatusCode, Json<NodeResponse>), RingError> {
    let service = make_graph_service(&state);
    let node = service.create_node(&graph_id, req).await?;
    Ok((StatusCode::CREATED, Json(NodeResponse::from(node))))
}

pub async fn get_node(
    State(state): State<AppState>,
    Path((_ring_id, graph_id, node_id)): Path<(String, String, String)>,
) -> Result<Json<NodeResponse>, RingError> {
    let service = make_graph_service(&state);
    let node = service
        .get_node(&graph_id, &node_id)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("node {} not found", node_id)))?;
    Ok(Json(NodeResponse::from(node)))
}

pub async fn update_node(
    State(state): State<AppState>,
    Path((_ring_id, graph_id, node_id)): Path<(String, String, String)>,
    Json(req): Json<UpdateNodeRequest>,
) -> Result<Json<NodeResponse>, RingError> {
    let service = make_graph_service(&state);
    let node = service.update_node(&graph_id, &node_id, req).await?;
    Ok(Json(NodeResponse::from(node)))
}

pub async fn delete_node(
    State(state): State<AppState>,
    Path((_ring_id, graph_id, node_id)): Path<(String, String, String)>,
) -> Result<StatusCode, RingError> {
    let service = make_graph_service(&state);
    service.delete_node(&graph_id, &node_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_node_content(
    State(state): State<AppState>,
    Path((_ring_id, graph_id, node_id)): Path<(String, String, String)>,
) -> Result<Json<NodeContentResponse>, RingError> {
    let service = make_graph_service(&state);
    let content = service.get_node_content(&graph_id, &node_id).await?;
    Ok(Json(content))
}

pub async fn create_edge(
    State(state): State<AppState>,
    Path((_ring_id, graph_id)): Path<(String, String)>,
    Json(req): Json<CreateEdgeRequest>,
) -> Result<(StatusCode, Json<EdgeResponse>), RingError> {
    let new_edge = crate::graph::types::NewEdge {
        source_id: req.source_id,
        target_id: req.target_id,
        relation: req.relation,
        label: req.label,
    };
    let edge = state.graph_store.create_edge(&graph_id, new_edge).await?;
    Ok((StatusCode::CREATED, Json(EdgeResponse::from(edge))))
}

pub async fn delete_edge(
    State(state): State<AppState>,
    Path((_ring_id, graph_id, edge_id)): Path<(String, String, String)>,
) -> Result<StatusCode, RingError> {
    state.graph_store.delete_edge(&graph_id, &edge_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
