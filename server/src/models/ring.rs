use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize)]
pub struct RingRow {
    pub id: String,
    pub name: String,
    pub creator_id: String,
    pub role_description: Option<String>,
    pub interaction_mode: String,
    pub skill_permission_mode: String,
    pub blueprint_status: String,
    pub gitlab_repo_url: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRing {
    pub name: String,
    pub role_description: String,
    pub gitlab_repo_url: Option<String>,
    pub gitlab_namespace: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
pub struct RingListItem {
    pub id: String,
    pub name: String,
    pub role: String,
    pub member_count: i64,
    pub node_count: i64,
    pub last_activity_at: String,
    pub has_active_session: bool,
}

#[derive(Debug, Serialize)]
pub struct RingDetail {
    pub id: String,
    pub name: String,
    pub role: String,
    pub role_description: Option<String>,
    pub member_count: i64,
    pub node_count: i64,
    pub blueprint_status: String,
    pub interaction_mode: String,
    pub skill_permission_mode: String,
    pub created_at: String,
}

pub async fn create_ring(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    creator_id: &str,
    input: &CreateRing,
) -> Result<RingRow> {
    let ring = sqlx::query_as::<_, RingRow>(
        "INSERT INTO rings (id, name, creator_id, role_description, gitlab_repo_url, gitlab_namespace)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         RETURNING *"
    )
        .bind(ring_id)
        .bind(&input.name)
        .bind(creator_id)
        .bind(&input.role_description)
        .bind(&input.gitlab_repo_url)
        .bind(&input.gitlab_namespace)
        .fetch_one(pool)
        .await?;

    sqlx::query("INSERT INTO members (ring_id, user_id, role) VALUES (?1, ?2, 'creator')")
        .bind(ring_id)
        .bind(creator_id)
        .execute(pool)
        .await?;

    Ok(ring)
}

pub async fn list_rings_for_user(
    pool: &sqlx::SqlitePool,
    user_id: &str,
) -> Result<Vec<RingListItem>> {
    let rings = sqlx::query_as::<_, (String, String, String, i64, String)>(
        "SELECT r.id, r.name, m.role,
                (SELECT COUNT(*) FROM members m2 WHERE m2.ring_id = r.id) as member_count,
                r.created_at as last_activity_at
         FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?1
         ORDER BY r.created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(rings.len());
    for (id, name, role, member_count, last_activity_at) in rings {
        let node_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM graph_nodes WHERE ring_id = ?1")
                .bind(&id)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

        result.push(RingListItem {
            id,
            name,
            role,
            member_count,
            node_count,
            last_activity_at,
            has_active_session: false,
        });
    }
    Ok(result)
}

pub async fn get_ring_detail(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    user_id: &str,
) -> Result<RingDetail> {
    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            String,
        ),
    >(
        "SELECT r.id, r.name, r.role_description, r.blueprint_status,
                r.interaction_mode, r.skill_permission_mode, r.created_at
         FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?2
         WHERE r.id = ?1",
    )
    .bind(ring_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound(format!("ring {ring_id} not found")))?;

    let member_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM members WHERE ring_id = ?1")
        .bind(ring_id)
        .fetch_one(pool)
        .await?;

    let node_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM graph_nodes WHERE ring_id = ?1")
        .bind(ring_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    Ok(RingDetail {
        id: row.0,
        name: row.1,
        role: get_user_role(pool, ring_id, user_id).await?,
        role_description: row.2,
        member_count,
        node_count,
        blueprint_status: row.3,
        interaction_mode: row.4,
        skill_permission_mode: row.5,
        created_at: row.6,
    })
}

pub async fn get_user_role(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    user_id: &str,
) -> Result<String> {
    sqlx::query_scalar::<_, String>("SELECT role FROM members WHERE ring_id = ?1 AND user_id = ?2")
        .bind(ring_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound("not a member".into()))
}
