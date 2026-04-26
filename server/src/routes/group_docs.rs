use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::{Result, RingError};
use crate::extractors::AuthUser;
use crate::models::ring;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct GroupDocResponse {
    pub doc_name: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGroupDocRequest {
    pub content: String,
}

pub async fn get_group_doc(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, doc_name)): Path<(String, String)>,
) -> Result<Json<GroupDocResponse>> {
    let _ = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let valid_docs = [
        "role",
        "conventions",
        "active-context",
        "archive-patterns",
        "corrections",
        "knowledge-summary",
    ];
    if !valid_docs.contains(&doc_name.as_str()) {
        return Err(RingError::BadRequest(format!(
            "invalid doc_name: {doc_name}"
        )));
    }

    let content: Option<String> =
        sqlx::query_scalar("SELECT content FROM group_docs WHERE ring_id = ?1 AND doc_name = ?2")
            .bind(&ring_id)
            .bind(&doc_name)
            .fetch_optional(&state.db)
            .await?;

    Ok(Json(GroupDocResponse {
        doc_name,
        content: content.unwrap_or_default(),
    }))
}

pub async fn update_group_doc(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, doc_name)): Path<(String, String)>,
    Json(body): Json<UpdateGroupDocRequest>,
) -> Result<Json<GroupDocResponse>> {
    let role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;

    let admin_only_docs = ["role", "conventions"];
    if admin_only_docs.contains(&doc_name.as_str()) && role != "creator" && role != "admin" {
        return Err(RingError::Forbidden(
            "only creator/admin can edit this doc".into(),
        ));
    }

    ring::reject_readonly(&role)?;

    let valid_docs = [
        "role",
        "conventions",
        "active-context",
        "archive-patterns",
        "corrections",
        "knowledge-summary",
    ];
    if !valid_docs.contains(&doc_name.as_str()) {
        return Err(RingError::BadRequest(format!(
            "invalid doc_name: {doc_name}"
        )));
    }

    sqlx::query(
        "INSERT INTO group_docs (ring_id, doc_name, content, updated_at) VALUES (?1, ?2, ?3, datetime('now'))
         ON CONFLICT(ring_id, doc_name) DO UPDATE SET content = ?3, updated_at = datetime('now')"
    )
        .bind(&ring_id)
        .bind(&doc_name)
        .bind(&body.content)
        .execute(&state.db)
        .await?;

    crate::services::group_doc_maintenance::persist_group_doc(
        &state.rings_dir,
        &ring_id,
        &doc_name,
        &body.content,
    );

    let ring_name = crate::services::search::get_ring_name(&state.db, &ring_id)
        .await
        .unwrap_or_default();
    let source_id = format!("{}:{}", &ring_id, &doc_name);
    let _ = crate::services::search::upsert_search_index(
        &state.db,
        "group_doc",
        &source_id,
        &ring_id,
        &ring_name,
        &doc_name,
        &body.content,
        "{}",
    )
    .await;

    Ok(Json(GroupDocResponse {
        doc_name,
        content: body.content,
    }))
}
