use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

pub const TOKEN_THRESHOLD: i64 = 100_000;
pub const TOKEN_WARNING_THRESHOLD: i64 = 80_000;

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct ConversationTokenRow {
    pub id: String,
    pub user_id: String,
    pub ring_id: Option<String>,
    pub total_tokens: i64,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn get_or_create(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    ring_id: Option<&str>,
) -> Result<ConversationTokenRow> {
    let existing = sqlx::query_as::<_, ConversationTokenRow>(
        "SELECT * FROM conversation_tokens WHERE user_id = ?1 AND (ring_id = ?2 OR (?2 IS NULL AND ring_id IS NULL))"
    )
    .bind(user_id)
    .bind(ring_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = existing {
        return Ok(row);
    }

    let id = ulid::Ulid::new().to_string();
    sqlx::query_as::<_, ConversationTokenRow>(
        "INSERT INTO conversation_tokens (id, user_id, ring_id, total_tokens)
         VALUES (?1, ?2, ?3, 0)
         RETURNING *",
    )
    .bind(&id)
    .bind(user_id)
    .bind(ring_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn add_tokens(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    ring_id: Option<&str>,
    tokens: i64,
) -> Result<ConversationTokenRow> {
    let row = get_or_create(pool, user_id, ring_id).await?;
    let new_total = row.total_tokens + tokens;

    sqlx::query_as::<_, ConversationTokenRow>(
        "UPDATE conversation_tokens
         SET total_tokens = ?1, updated_at = datetime('now')
         WHERE id = ?2
         RETURNING *",
    )
    .bind(new_total)
    .bind(&row.id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn reset_tokens(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    ring_id: Option<&str>,
) -> Result<ConversationTokenRow> {
    let row = get_or_create(pool, user_id, ring_id).await?;

    sqlx::query_as::<_, ConversationTokenRow>(
        "UPDATE conversation_tokens
         SET total_tokens = 0, updated_at = datetime('now')
         WHERE id = ?1
         RETURNING *",
    )
    .bind(&row.id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn get_token_count(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    ring_id: Option<&str>,
) -> Result<i64> {
    let row = get_or_create(pool, user_id, ring_id).await?;
    Ok(row.total_tokens)
}

#[derive(Debug, Deserialize)]
pub struct UpdateAutoCompact {
    pub auto_compact: bool,
}

pub async fn get_auto_compact(pool: &sqlx::SqlitePool, user_id: &str) -> Result<bool> {
    let row: Option<(bool,)> = sqlx::query_as("SELECT auto_compact FROM users WHERE token_id = ?1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some((auto_compact,)) => Ok(auto_compact),
        None => Err(RingError::NotFound("user not found".into())),
    }
}

pub async fn update_auto_compact(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    auto_compact: bool,
) -> Result<bool> {
    let result = sqlx::query("UPDATE users SET auto_compact = ?1 WHERE token_id = ?2")
        .bind(auto_compact)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("user not found".into()));
    }

    Ok(auto_compact)
}
