use axum::extract::{Path, State};
use axum::Json;

use crate::error::RingError;
use crate::models::graph_model::{SearchRequest, SearchResponse};
use crate::services::search_service::SearchService;
use crate::state::AppState;

pub async fn search_nodes(
    State(state): State<AppState>,
    Path(_ring_id): Path<String>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, RingError> {
    let service = SearchService::new(state.db.clone(), state.graph_store.clone());
    let limit = req.limit.unwrap_or(20);
    let results = service
        .search_nodes(&req.query, req.graph_ids, limit)
        .await?;
    let total = results.len();
    Ok(Json(SearchResponse { results, total }))
}
