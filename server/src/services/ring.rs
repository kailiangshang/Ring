use serde::Serialize;
use ulid::Ulid;

use crate::error::Result;
use crate::models::ring::{self, CreateRing, RingDetail, RingListItem};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CreateRingResponse {
    pub id: String,
    pub name: String,
    pub role: String,
    pub blueprint_status: String,
}

pub async fn list_rings(state: &AppState, user_id: &str) -> Result<Vec<RingListItem>> {
    ring::list_rings_for_user(&state.db, user_id).await
}

pub async fn create_ring(
    state: &AppState,
    user_id: &str,
    input: CreateRing,
) -> Result<CreateRingResponse> {
    let id = Ulid::new().to_string();
    let row = ring::create_ring(&state.db, &id, user_id, &input).await?;
    Ok(CreateRingResponse {
        id: row.id,
        name: row.name,
        role: "creator".into(),
        blueprint_status: row.blueprint_status,
    })
}

pub async fn get_ring_detail(state: &AppState, ring_id: &str, user_id: &str) -> Result<RingDetail> {
    ring::get_ring_detail(&state.db, ring_id, user_id).await
}
