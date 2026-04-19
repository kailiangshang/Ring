use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize)]
pub struct InviteTokenRow {
    pub token: String,
    pub ring_id: String,
    pub r#type: String,
    pub role: String,
    pub max_uses: i64,
    pub use_count: i64,
    pub max_members: Option<i64>,
    pub expires_at: String,
    pub revoked_at: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteToken {
    #[serde(default = "default_type")]
    pub r#type: String,
    #[serde(default = "default_role")]
    pub role: String,
    #[serde(default = "default_max_uses")]
    pub max_uses: i64,
    pub max_members: Option<i64>,
    #[serde(default = "default_expires_hours")]
    pub expires_in_hours: i64,
}

fn default_type() -> String {
    "open".to_string()
}

fn default_role() -> String {
    "member".to_string()
}

fn default_max_uses() -> i64 {
    1
}

fn default_expires_hours() -> i64 {
    24
}

pub async fn insert_token(pool: &sqlx::SqlitePool, row: &InviteTokenRow) -> Result<()> {
    sqlx::query(
        "INSERT INTO invite_tokens (token, ring_id, type, role, max_uses, use_count, max_members, expires_at, revoked_at, created_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )
    .bind(&row.token)
    .bind(&row.ring_id)
    .bind(&row.r#type)
    .bind(&row.role)
    .bind(row.max_uses)
    .bind(row.use_count)
    .bind(row.max_members)
    .bind(&row.expires_at)
    .bind(&row.revoked_at)
    .bind(&row.created_by)
    .bind(&row.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_tokens(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    include_expired: bool,
    include_revoked: bool,
) -> Result<Vec<InviteTokenRow>> {
    let mut query = String::from("SELECT * FROM invite_tokens WHERE ring_id = ?1");
    if !include_expired {
        query.push_str(" AND expires_at > datetime('now')");
    }
    if !include_revoked {
        query.push_str(" AND revoked_at IS NULL");
    }
    query.push_str(" ORDER BY created_at DESC");

    let rows = sqlx::query_as::<_, InviteTokenRow>(&query)
        .bind(ring_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn revoke_token(pool: &sqlx::SqlitePool, ring_id: &str, token: &str) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE invite_tokens SET revoked_at = datetime('now') WHERE ring_id = ?1 AND token = ?2 AND revoked_at IS NULL",
    )
    .bind(ring_id)
    .bind(token)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM invite_tokens WHERE ring_id = ?1 AND token = ?2)",
        )
        .bind(ring_id)
        .bind(token)
        .fetch_one(pool)
        .await?;

        if !exists {
            return Err(RingError::NotFound("invite token not found".into()));
        }
    }
    Ok(true)
}

pub async fn find_token_by_value(
    pool: &sqlx::SqlitePool,
    token: &str,
) -> Result<Option<InviteTokenRow>> {
    sqlx::query_as::<_, InviteTokenRow>("SELECT * FROM invite_tokens WHERE token = ?1")
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn increment_use_count(pool: &sqlx::SqlitePool, token: &str) -> Result<()> {
    sqlx::query("UPDATE invite_tokens SET use_count = use_count + 1 WHERE token = ?1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_member_count(pool: &sqlx::SqlitePool, ring_id: &str) -> Result<i64> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM members WHERE ring_id = ?1")
        .bind(ring_id)
        .fetch_one(pool)
        .await?;
    Ok(count)
}
