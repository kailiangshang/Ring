use crate::error::{Result, RingError};
use crate::models::user::{NewUser, User};
#[derive(sqlx::FromRow)]
pub(crate) struct UserRow {
    pub id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub ip_address: Option<String>,
    pub setup_completed: bool,
    pub created_at: String,
}

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    pub async fn create_user_inner(&self, new_user: NewUser) -> Result<User> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO users (id, display_name, setup_completed, created_at) VALUES (?, ?, FALSE, ?)",
        )
        .bind(&id)
        .bind(&new_user.display_name)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(User {
            id,
            display_name: new_user.display_name,
            avatar_url: None,
            ip_address: None,
            setup_completed: false,
            created_at: now,
        })
    }

    pub async fn get_user_inner(&self, id: &str) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>("SELECT id, display_name, avatar_url, ip_address, setup_completed, created_at FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(RingError::Database)?;

        Ok(row.map(|r| User {
            id: r.id,
            display_name: r.display_name,
            avatar_url: r.avatar_url,
            ip_address: r.ip_address,
            setup_completed: r.setup_completed,
            created_at: r.created_at,
        }))
    }

    pub async fn list_all_users_inner(&self) -> Result<Vec<User>> {
        let rows = sqlx::query_as::<_, UserRow>(
            "SELECT id, display_name, avatar_url, ip_address, setup_completed, created_at FROM users",
        )
        .fetch_all(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| User {
                id: r.id,
                display_name: r.display_name,
                avatar_url: r.avatar_url,
                ip_address: r.ip_address,
                setup_completed: r.setup_completed,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn is_setup_completed_inner(&self) -> Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as("SELECT setup_completed FROM users LIMIT 1")
            .fetch_optional(self.pool())
            .await
            .map_err(RingError::Database)?;

        match row {
            Some((setup_completed,)) => Ok(setup_completed),
            None => Ok(false),
        }
    }

    pub async fn complete_setup_inner(&self, user_id: &str) -> Result<()> {
        sqlx::query("UPDATE users SET setup_completed = TRUE WHERE id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }
}
