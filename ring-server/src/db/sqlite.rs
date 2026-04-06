use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::models::invite::InviteToken;
use crate::models::ring::{NewRing, Ring};
use crate::models::user::{NewUser, User};
use sqlx::SqlitePool;

pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        SqliteRepository { pool }
    }
}

#[async_trait::async_trait]
impl Repository for SqliteRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO users (id, display_name, setup_completed, created_at) VALUES (?, ?, FALSE, ?)",
        )
        .bind(&id)
        .bind(&new_user.display_name)
        .bind(&now)
        .execute(&self.pool)
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

    async fn get_user(&self, id: &str) -> Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>("SELECT id, display_name, avatar_url, ip_address, setup_completed, created_at FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
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

    async fn list_all_users(&self) -> Result<Vec<User>> {
        let rows = sqlx::query_as::<_, UserRow>(
            "SELECT id, display_name, avatar_url, ip_address, setup_completed, created_at FROM users",
        )
        .fetch_all(&self.pool)
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

    async fn is_setup_completed(&self) -> Result<bool> {
        let row: Option<(bool,)> = sqlx::query_as("SELECT setup_completed FROM users LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(RingError::Database)?;

        match row {
            Some((setup_completed,)) => Ok(setup_completed),
            None => Ok(false),
        }
    }

    async fn complete_setup(&self, user_id: &str) -> Result<()> {
        sqlx::query("UPDATE users SET setup_completed = TRUE WHERE id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn create_ring(&self, new_ring: NewRing) -> Result<Ring> {
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
        .execute(&self.pool)
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

    async fn get_ring(&self, id: &str) -> Result<Option<Ring>> {
        let row = sqlx::query_as::<_, RingRow>(
            "SELECT id, name, description, creator_id, gitlab_repo, local_path, next_token_id, status, created_at, updated_at FROM rings WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
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

    async fn list_rings_by_user(&self, user_id: &str) -> Result<Vec<Ring>> {
        let rows = sqlx::query_as::<_, RingRow>(
            "SELECT id, name, description, creator_id, gitlab_repo, local_path, next_token_id, status, created_at, updated_at FROM rings WHERE creator_id = ?",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
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

    async fn update_ring(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Ring> {
        let existing = self
            .get_ring(id)
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
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;

        Ok(Ring {
            name: new_name,
            description: new_description,
            updated_at: now,
            ..existing
        })
    }

    async fn delete_ring(&self, id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM rings WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;

        if result.rows_affected() == 0 {
            return Err(RingError::NotFound(format!("ring {}", id)));
        }
        Ok(())
    }

    async fn create_invite_token(
        &self,
        ring_id: &str,
        token: &str,
        token_type: &str,
        inviter_id: &str,
    ) -> Result<InviteToken> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

        sqlx::query(
            "INSERT INTO invite_tokens (id, ring_id, token, token_type, inviter_id, expires_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(ring_id)
        .bind(token)
        .bind(token_type)
        .bind(inviter_id)
        .bind(expires_at.to_rfc3339())
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(InviteToken {
            id,
            ring_id: ring_id.to_string(),
            token: token.to_string(),
            token_type: token_type.to_string(),
            role: "member".into(),
            inviter_id: inviter_id.to_string(),
            max_uses: 1,
            use_count: 0,
            max_members: None,
            expires_at: expires_at.to_rfc3339(),
            used_at: None,
            revoked_at: None,
            created_at: now,
        })
    }

    async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(row.map(|(v,)| v))
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }

    async fn count_members_by_ring(&self, ring_id: &str) -> Result<i64> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM members WHERE ring_id = ?")
            .bind(ring_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(row.map(|(c,)| c).unwrap_or(0))
    }

    async fn get_invite_token(&self, token: &str) -> Result<Option<InviteToken>> {
        let row = sqlx::query_as::<_, InviteTokenRow>(
            "SELECT id, ring_id, token, token_type, role, inviter_id, max_uses, use_count, max_members, expires_at, used_at, revoked_at, created_at FROM invite_tokens WHERE token = ?",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| InviteToken {
            id: r.id,
            ring_id: r.ring_id,
            token: r.token,
            token_type: r.token_type,
            role: r.role,
            inviter_id: r.inviter_id,
            max_uses: r.max_uses,
            use_count: r.use_count,
            max_members: r.max_members,
            expires_at: r.expires_at,
            used_at: r.used_at,
            revoked_at: r.revoked_at,
            created_at: r.created_at,
        }))
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: String,
    display_name: String,
    avatar_url: Option<String>,
    ip_address: Option<String>,
    setup_completed: bool,
    created_at: String,
}

#[derive(sqlx::FromRow)]
struct RingRow {
    id: String,
    name: String,
    description: Option<String>,
    creator_id: String,
    gitlab_repo: String,
    local_path: String,
    next_token_id: i64,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct InviteTokenRow {
    id: String,
    ring_id: String,
    token: String,
    token_type: String,
    role: String,
    inviter_id: String,
    max_uses: i64,
    use_count: i64,
    max_members: Option<i64>,
    expires_at: String,
    used_at: Option<String>,
    revoked_at: Option<String>,
    created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ring::NewRing;
    use crate::models::user::NewUser;

    async fn setup_test_db() -> SqliteRepository {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        SqliteRepository::new(pool)
    }

    #[tokio::test]
    async fn create_and_get_user() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        assert_eq!(user.display_name, "张三");
        assert!(!user.id.is_empty());

        let fetched = repo.get_user(&user.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, user.id);
    }

    #[tokio::test]
    async fn create_and_list_rings() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();

        let ring = repo
            .create_ring(NewRing {
                name: "竞品分析".into(),
                description: Some("desc".into()),
                creator_id: user.id.clone(),
                gitlab_repo: "auto_create".into(),
                namespace: None,
                role_description: "产品专家".into(),
            })
            .await
            .unwrap();
        assert_eq!(ring.name, "竞品分析");

        let rings = repo.list_rings_by_user(&user.id).await.unwrap();
        assert_eq!(rings.len(), 1);
    }

    #[tokio::test]
    async fn setup_status_defaults_to_false() {
        let repo = setup_test_db().await;
        let status = repo.is_setup_completed().await.unwrap();
        assert!(!status);
    }

    #[tokio::test]
    async fn complete_setup_sets_flag() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        repo.complete_setup(&user.id).await.unwrap();
        let status = repo.is_setup_completed().await.unwrap();
        assert!(status);
    }

    #[tokio::test]
    async fn create_and_get_invite_token() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        let ring = repo
            .create_ring(NewRing {
                name: "竞品分析".into(),
                description: None,
                creator_id: user.id.clone(),
                gitlab_repo: "auto_create".into(),
                namespace: None,
                role_description: "专家".into(),
            })
            .await
            .unwrap();

        let invite = repo
            .create_invite_token(&ring.id, "test-token-123", "open", &user.id)
            .await
            .unwrap();
        assert_eq!(invite.token, "test-token-123");
        assert_eq!(invite.token_type, "open");

        let fetched = repo
            .get_invite_token("test-token-123")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.id, invite.id);
    }

    #[tokio::test]
    async fn delete_ring_removes_from_list() {
        let repo = setup_test_db().await;
        let user = repo
            .create_user(NewUser {
                display_name: "张三".into(),
            })
            .await
            .unwrap();
        let ring = repo
            .create_ring(NewRing {
                name: "竞品分析".into(),
                description: None,
                creator_id: user.id.clone(),
                gitlab_repo: "auto_create".into(),
                namespace: None,
                role_description: "专家".into(),
            })
            .await
            .unwrap();

        repo.delete_ring(&ring.id).await.unwrap();

        let fetched = repo.get_ring(&ring.id).await.unwrap();
        assert!(fetched.is_none());

        let rings = repo.list_rings_by_user(&user.id).await.unwrap();
        assert!(rings.is_empty());
    }
}
