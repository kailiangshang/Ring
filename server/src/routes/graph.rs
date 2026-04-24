use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;

use crate::error::{Result, RingError};
use crate::extractors::auth::AuthUser;
use crate::models::graph::{CreateEdgeInput, CreateNodeInput, UpdateNodeInput};
use crate::models::ring;
use crate::services;
use crate::state::AppState;

pub async fn get_graph(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<services::graph::GraphResponse>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let graph = services::graph::get_full_graph(&state, &ring_id).await?;
    Ok(Json(graph))
}

pub async fn create_node_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateNodeInput>,
) -> Result<Json<crate::models::graph::GraphNodeRow>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let node = services::graph::create_node(&state, &ring_id, &body).await?;

    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
        });
    }

    let state_c = state.clone();
    let ring_id_c = ring_id.clone();
    let token_id_c = user.token_id.clone();
    tokio::spawn(async move {
        if let Ok(user_row) = state_c.get_user_decrypted(&token_id_c).await {
            let _ = crate::services::group_doc_maintenance::update_knowledge_summary(
                &state_c, &ring_id_c, &user_row,
            )
            .await;
        }
    });

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    let _ = crate::services::self_data::record_tool_usage(&self_dir, "graph_edit");

    Ok(Json(node))
}

pub async fn update_node(
    State(state): State<AppState>,
    _user: AuthUser,
    Path((_ring_id, node_id)): Path<(String, String)>,
    Json(body): Json<UpdateNodeInput>,
) -> Result<Json<crate::models::graph::GraphNodeRow>> {
    let node = services::graph::update_node(&state, &node_id, &body).await?;

    let self_dir = crate::services::self_data::get_self_dir(&_user.token_id);
    let _ = crate::services::self_data::record_tool_usage(&self_dir, "graph_edit");

    Ok(Json(node))
}

pub async fn delete_node(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, node_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden(
            "only creator/admin can delete nodes".into(),
        ));
    }
    services::graph::delete_node(&state, &node_id).await?;

    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
        });
    }

    let state_c = state.clone();
    let ring_id_c = ring_id.clone();
    let user_id = user.token_id.clone();
    tokio::spawn(async move {
        if let Ok(user_row) = state_c.get_user_decrypted(&user_id).await {
            let _ = crate::services::group_doc_maintenance::update_knowledge_summary(
                &state_c, &ring_id_c, &user_row,
            )
            .await;
        }
    });

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    let _ = crate::services::self_data::record_tool_usage(&self_dir, "graph_edit");

    Ok(Json(serde_json::json!({"status": "deleted"})))
}

pub async fn create_edge_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateEdgeInput>,
) -> Result<Json<crate::models::graph::GraphEdgeRow>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let edge = services::graph::create_edge(&state, &ring_id, &body).await?;

    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
        });
    }

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    let _ = crate::services::self_data::record_tool_usage(&self_dir, "graph_edit");

    Ok(Json(edge))
}

pub async fn delete_edge(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, edge_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden(
            "only creator/admin can delete edges".into(),
        ));
    }
    services::graph::delete_edge(&state, &edge_id).await?;

    {
        let cache = state.cross_ring_cache.clone();
        let rid = ring_id.clone();
        tokio::spawn(async move {
            crate::services::cross_ring_cache::invalidate_ring(&cache, &rid).await;
        });
    }

    let self_dir = crate::services::self_data::get_self_dir(&user.token_id);
    let _ = crate::services::self_data::record_tool_usage(&self_dir, "graph_edit");

    Ok(Json(serde_json::json!({"status": "deleted"})))
}

#[derive(Debug, Deserialize)]
pub struct CreateGraphInput {
    pub name: String,
}

pub async fn list_graphs_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let graphs = crate::models::graph::list_graphs(&state.db, &ring_id).await?;
    Ok(Json(serde_json::json!({ "graphs": graphs })))
}

pub async fn create_graph_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateGraphInput>,
) -> Result<Json<crate::models::graph::GraphRow>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" && role != "admin" {
        return Err(RingError::Forbidden(
            "only creator/admin can create graphs".into(),
        ));
    }
    let graph = crate::models::graph::create_graph(&state.db, &ring_id, &body.name).await?;
    Ok(Json(graph))
}

pub async fn delete_graph_handler(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, graph_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if role != "creator" {
        return Err(RingError::Forbidden(
            "only creator can delete graphs".into(),
        ));
    }
    crate::models::graph::delete_graph(&state.db, &graph_id).await?;
    Ok(Json(serde_json::json!({"status": "deleted"})))
}
