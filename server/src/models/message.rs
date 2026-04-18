use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct MessageRow {
    pub id: String,
    pub ring_id: Option<String>,
    pub user_id: String,
    pub role: String,
    pub sender_name: String,
    pub content: String,
    pub node_refs: String,
    pub tag_refs: String,
    pub token_usage: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    #[serde(default)]
    pub node_refs: Vec<String>,
    #[serde(default)]
    pub tag_refs: Vec<String>,
}

pub struct NewMessage<'a> {
    pub id: &'a str,
    pub ring_id: Option<&'a str>,
    pub user_id: &'a str,
    pub role: &'a str,
    pub sender_name: &'a str,
    pub content: &'a str,
    pub node_refs: &'a [String],
    pub tag_refs: &'a [String],
    pub token_usage: Option<&'a str>,
}

pub async fn insert_message(pool: &sqlx::SqlitePool, msg: &NewMessage<'_>) -> Result<MessageRow> {
    sqlx::query_as::<_, MessageRow>(
        "INSERT INTO messages (id, ring_id, user_id, role, sender_name, content, node_refs, tag_refs, token_usage)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         RETURNING *",
    )
    .bind(msg.id)
    .bind(msg.ring_id)
    .bind(msg.user_id)
    .bind(msg.role)
    .bind(msg.sender_name)
    .bind(msg.content)
    .bind(serde_json::to_string(msg.node_refs).unwrap_or_else(|_| "[]".into()))
    .bind(serde_json::to_string(msg.tag_refs).unwrap_or_else(|_| "[]".into()))
    .bind(msg.token_usage)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn list_messages(
    pool: &sqlx::SqlitePool,
    ring_id: Option<&str>,
    user_id: &str,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<MessageRow>> {
    let rows = if let Some(before) = before_id {
        sqlx::query_as::<_, MessageRow>(
            "SELECT * FROM messages
             WHERE (ring_id = ?1 OR (?1 IS NULL AND ring_id IS NULL))
             AND user_id = ?2
             AND created_at < (SELECT created_at FROM messages WHERE id = ?3)
             ORDER BY created_at DESC LIMIT ?4",
        )
        .bind(ring_id)
        .bind(user_id)
        .bind(before)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, MessageRow>(
            "SELECT * FROM messages
             WHERE (ring_id = ?1 OR (?1 IS NULL AND ring_id IS NULL))
             AND user_id = ?2
             ORDER BY created_at DESC LIMIT ?3",
        )
        .bind(ring_id)
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await
    };
    rows.map_err(|e| RingError::Internal(e.to_string()))
}
