use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::Json;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::error::RingError;
use crate::graph::types::NewNode;
use crate::models::blueprint::BlueprintTemplate;
use crate::services::ai_service::AiService;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintChatRequest {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewRequest {
    pub graphs: Vec<GraphDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDef {
    pub name: String,
    pub graph_type: String,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResponse {
    pub graphs: Vec<GraphPreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPreview {
    pub name: String,
    pub nodes: Vec<NodePreview>,
    pub edges: Vec<EdgePreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePreview {
    pub id: String,
    pub label: String,
    pub node_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgePreview {
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmRequest {
    pub graphs: Vec<GraphDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmResponse {
    pub blueprint_id: String,
    pub graphs: Vec<GraphInfo>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphInfo {
    pub id: String,
    pub name: String,
    pub graph_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintTemplatesResponse {
    pub templates: Vec<BlueprintTemplate>,
}

pub async fn list_templates(
    State(state): State<AppState>,
    Path(_ring_id): Path<String>,
) -> Result<Json<BlueprintTemplatesResponse>, RingError> {
    let templates = state.db.list_blueprint_templates().await?;
    Ok(Json(BlueprintTemplatesResponse { templates }))
}

pub async fn blueprint_chat(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Json(req): Json<BlueprintChatRequest>,
) -> Result<Sse<ReceiverStream<Result<Event, Infallible>>>, RingError> {
    if req.message.trim().is_empty() {
        return Err(RingError::Validation("message must not be empty".into()));
    }

    let ai = AiService::new(state.db.clone(), state.llm_provider.clone());
    let llm_stream = ai.blueprint_chat(&ring_id, req.message).await?;

    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(32);

    tokio::spawn(async move {
        let mut stream = std::pin::pin!(llm_stream);
        while let Some(event) = stream.next().await {
            let json = serde_json::to_string(&event).unwrap_or_default();
            let sse_event = Event::default().event("message").data(json);
            if tx.send(Ok(sse_event)).await.is_err() {
                break;
            }
        }
    });

    Ok(Sse::new(ReceiverStream::new(rx)))
}

pub async fn preview_blueprint(
    State(state): State<AppState>,
    Path(_ring_id): Path<String>,
    Json(req): Json<PreviewRequest>,
) -> Result<Json<PreviewResponse>, RingError> {
    let store = state.graph_store.read().await;
    let mut graphs = Vec::new();

    for graph_def in &req.graphs {
        let temp_graph_id = Uuid::new_v4().to_string();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let root_id = Uuid::new_v4().to_string();
        nodes.push(NodePreview {
            id: root_id.clone(),
            label: graph_def.name.clone(),
            node_type: graph_def.graph_type.clone(),
        });

        let mut category_node_ids = Vec::new();
        for cat in &graph_def.categories {
            let cat_id = Uuid::new_v4().to_string();
            nodes.push(NodePreview {
                id: cat_id.clone(),
                label: cat.clone(),
                node_type: "category".into(),
            });
            edges.push(EdgePreview {
                source_id: root_id.clone(),
                target_id: cat_id.clone(),
                relation: "contains".into(),
            });
            category_node_ids.push(cat_id);
        }

        for cat_id in &category_node_ids {
            let _ = store
                .create_node(
                    &temp_graph_id,
                    NewNode {
                        label: "placeholder".into(),
                        node_type: "placeholder".into(),
                        parent_id: Some(cat_id.clone()),
                        description: None,
                    },
                )
                .await;
        }

        graphs.push(GraphPreview {
            name: graph_def.name.clone(),
            nodes,
            edges,
        });
    }

    Ok(Json(PreviewResponse { graphs }))
}

pub async fn confirm_blueprint(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Json(req): Json<ConfirmRequest>,
) -> Result<Json<ConfirmResponse>, RingError> {
    let blueprint_id = Uuid::new_v4().to_string();
    let mut graph_infos = Vec::new();

    {
        let store = state.graph_store.read().await;
        for graph_def in &req.graphs {
            let graph_id = Uuid::new_v4().to_string();

            let root = store
                .create_node(
                    &graph_id,
                    NewNode {
                        label: graph_def.name.clone(),
                        node_type: graph_def.graph_type.clone(),
                        parent_id: None,
                        description: None,
                    },
                )
                .await?;

            for cat in &graph_def.categories {
                store
                    .create_node(
                        &graph_id,
                        NewNode {
                            label: cat.clone(),
                            node_type: "category".into(),
                            parent_id: Some(root.id.clone()),
                            description: None,
                        },
                    )
                    .await?;
            }

            graph_infos.push(GraphInfo {
                id: graph_id,
                name: graph_def.name.clone(),
                graph_type: graph_def.graph_type.clone(),
            });
        }
    }

    state.db.update_ring_status(&ring_id, "active").await?;

    Ok(Json(ConfirmResponse {
        blueprint_id,
        graphs: graph_infos,
        status: "confirmed".into(),
    }))
}
