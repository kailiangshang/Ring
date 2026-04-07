use crate::error::{Result, RingError};
use crate::models::ring::{NewRing, Ring};

#[derive(sqlx::FromRow)]
pub(crate) struct RingRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub creator_id: String,
    pub gitlab_repo: String,
    pub local_path: String,
    pub next_token_id: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    pub async fn create_ring_inner(&self, new_ring: NewRing) -> Result<Ring> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let local_path = format!(".ring/repos/ring-{}", new_ring.name);

        sqlx::query(
            "INSERT INTO rings (id, name, description, creator_id, gitlab_repo, local_path, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?)",
        )
        .bind(&id)
        .bind(&new_ring.name)
        .bind(&new_ring.description)
        .bind(&new_ring.creator_id)
        .bind(&new_ring.gitlab_repo)
        .bind(&local_path)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(Ring {
            id,
            name: new_ring.name,
            description: new_ring.description,
            creator_id: new_ring.creator_id,
            gitlab_repo: new_ring.gitlab_repo,
            local_path,
            next_token_id: 2,
            status: "active".into(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn get_ring_inner(&self, id: &str) -> Result<Option<Ring>> {
        let row = sqlx::query_as::<_, RingRow>(
            "SELECT id, name, description, creator_id, gitlab_repo, local_path, next_token_id, status, created_at, updated_at FROM rings WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| Ring {
            id: r.id,
            name: r.name,
            description: r.description,
            creator_id: r.creator_id,
            gitlab_repo: r.gitlab_repo,
            local_path: r.local_path,
            next_token_id: r.next_token_id,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn list_rings_by_user_inner(&self, user_id: &str) -> Result<Vec<Ring>> {
        let rows = sqlx::query_as::<_, RingRow>(
            "SELECT DISTINCT r.id, r.name, r.description, r.creator_id, r.gitlab_repo, r.local_path, r.next_token_id, r.status, r.created_at, r.updated_at \
             FROM rings r \
             LEFT JOIN members m ON m.ring_id = r.id \
             WHERE r.creator_id = ? OR m.user_id = ?",
        )
        .bind(user_id)
        .bind(user_id)
        .fetch_all(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| Ring {
                id: r.id,
                name: r.name,
                description: r.description,
                creator_id: r.creator_id,
                gitlab_repo: r.gitlab_repo,
                local_path: r.local_path,
                next_token_id: r.next_token_id,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn update_ring_inner(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Ring> {
        let existing = self
            .get_ring_inner(id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", id)))?;

        let new_name = name.unwrap_or(existing.name);
        let new_description = description.or(existing.description);
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query("UPDATE rings SET name = ?, description = ?, updated_at = ? WHERE id = ?")
            .bind(&new_name)
            .bind(&new_description)
            .bind(&now)
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;

        Ok(Ring {
            name: new_name,
            description: new_description,
            updated_at: now,
            ..existing
        })
    }

    pub async fn delete_ring_inner(&self, id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM rings WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;

        if result.rows_affected() == 0 {
            return Err(RingError::NotFound(format!("ring {}", id)));
        }
        Ok(())
    }

    pub async fn update_ring_status_inner(&self, id: &str, status: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let result = sqlx::query("UPDATE rings SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;

        if result.rows_affected() == 0 {
            return Err(RingError::NotFound(format!("ring {}", id)));
        }
        Ok(())
    }
}
