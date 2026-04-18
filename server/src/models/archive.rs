use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct ArchiveRecord {
    pub id: String,
    pub ring_id: String,
    pub session_id: Option<String>,
    pub node_id: Option<String>,
    pub file_name: String,
    pub commit_sha: Option<String>,
    pub branch: Option<String>,
    pub merge_request_iid: Option<i64>,
    pub status: String,
    pub archived_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateArchiveInput {
    pub session_id: Option<String>,
    pub content: String,
    pub suggested_title: String,
    pub node_suggestion: NodeSuggestionInput,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum NodeSuggestionInput {
    #[serde(rename = "create_new")]
    CreateNew {
        parent_id: Option<String>,
        node_title: String,
    },
    #[serde(rename = "attach_existing")]
    AttachExisting { node_id: String },
    #[serde(rename = "update_existing")]
    UpdateExisting { node_id: String },
}

#[derive(Debug, Deserialize)]
pub struct ReviewInput {
    pub action: ReviewAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewAction {
    Merge,
    Reject,
}

pub async fn insert_record(
    pool: &sqlx::SqlitePool,
    id: &str,
    ring_id: &str,
    session_id: Option<&str>,
    node_id: Option<&str>,
    file_name: &str,
    archived_by: &str,
) -> Result<ArchiveRecord> {
    sqlx::query_as::<_, ArchiveRecord>(
        "INSERT INTO archive_records (id, ring_id, session_id, node_id, file_name, status, archived_by)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)
         RETURNING *",
    )
    .bind(id)
    .bind(ring_id)
    .bind(session_id)
    .bind(node_id)
    .bind(file_name)
    .bind(archived_by)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn update_status(
    pool: &sqlx::SqlitePool,
    id: &str,
    status: &str,
    commit_sha: Option<&str>,
    branch: Option<&str>,
    merge_request_iid: Option<i64>,
) -> Result<ArchiveRecord> {
    sqlx::query_as::<_, ArchiveRecord>(
        "UPDATE archive_records
         SET status = ?1, commit_sha = COALESCE(?2, commit_sha),
             branch = COALESCE(?3, branch), merge_request_iid = COALESCE(?4, merge_request_iid),
             updated_at = datetime('now')
         WHERE id = ?5
         RETURNING *",
    )
    .bind(status)
    .bind(commit_sha)
    .bind(branch)
    .bind(merge_request_iid)
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound(format!("archive record {id} not found")))
}

pub async fn get_record(pool: &sqlx::SqlitePool, id: &str) -> Result<ArchiveRecord> {
    sqlx::query_as::<_, ArchiveRecord>("SELECT * FROM archive_records WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("archive record {id} not found")))
}

pub async fn list_by_ring(pool: &sqlx::SqlitePool, ring_id: &str) -> Result<Vec<ArchiveRecord>> {
    sqlx::query_as::<_, ArchiveRecord>(
        "SELECT * FROM archive_records WHERE ring_id = ?1 ORDER BY created_at DESC",
    )
    .bind(ring_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn list_pending_reviews(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
) -> Result<Vec<ArchiveRecord>> {
    sqlx::query_as::<_, ArchiveRecord>(
        "SELECT * FROM archive_records WHERE ring_id = ?1 AND status = 'mr_opened' ORDER BY created_at ASC",
    )
    .bind(ring_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}
