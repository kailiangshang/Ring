use serde::Serialize;
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize)]
pub struct MemberRow {
    pub user_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub token_id: String,
    pub display_name: String,
    pub avatar: Option<String>,
    pub role: String,
    pub joined_at: String,
    pub online: bool,
}

pub async fn list_members(pool: &sqlx::SqlitePool, ring_id: &str) -> Result<Vec<MemberResponse>> {
    let rows = sqlx::query_as::<_, MemberRow>(
        "SELECT m.user_id, u.display_name, u.avatar, m.role, m.joined_at
         FROM members m
         JOIN users u ON u.token_id = m.user_id
         WHERE m.ring_id = ?1
         ORDER BY m.joined_at",
    )
    .bind(ring_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| MemberResponse {
            token_id: r.user_id,
            display_name: r.display_name,
            avatar: r.avatar,
            role: r.role,
            joined_at: r.joined_at,
            online: false,
        })
        .collect())
}

pub async fn update_role(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    user_id: &str,
    new_role: &str,
) -> Result<()> {
    let result = sqlx::query("UPDATE members SET role = ?1 WHERE ring_id = ?2 AND user_id = ?3")
        .bind(new_role)
        .bind(ring_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("member not found".into()));
    }
    Ok(())
}

pub async fn remove_member(pool: &sqlx::SqlitePool, ring_id: &str, user_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM members WHERE ring_id = ?1 AND user_id = ?2")
        .bind(ring_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("member not found".into()));
    }
    Ok(())
}
