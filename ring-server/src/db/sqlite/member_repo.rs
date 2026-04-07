use crate::error::{Result, RingError};
use crate::models::invite::InviteToken;
use crate::models::member::{Member, NewMember};

#[derive(sqlx::FromRow)]
pub(crate) struct MemberRow {
    pub id: String,
    pub ring_id: String,
    pub user_id: String,
    pub token_id: i64,
    pub display_name: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(sqlx::FromRow)]
pub(crate) struct InviteTokenRow {
    pub id: String,
    pub ring_id: String,
    pub token: String,
    pub token_type: String,
    pub role: String,
    pub inviter_id: String,
    pub max_uses: i64,
    pub use_count: i64,
    pub max_members: Option<i64>,
    pub expires_at: String,
    pub used_at: Option<String>,
    pub revoked_at: Option<String>,
    pub created_at: String,
}

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    pub async fn create_member_inner(&self, new_member: NewMember) -> Result<Member> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let token_id: (i64,) =
            sqlx::query_as("SELECT COALESCE(MAX(token_id), 0) + 1 FROM members WHERE ring_id = ?")
                .bind(&new_member.ring_id)
                .fetch_one(self.pool())
                .await
                .map_err(RingError::Database)?;

        sqlx::query(
            "INSERT INTO members (id, ring_id, user_id, token_id, display_name, role, joined_at) VALUES (?, ?, ?, ?, ?, COALESCE(?, 'member'), ?)",
        )
        .bind(&id)
        .bind(&new_member.ring_id)
        .bind(&new_member.user_id)
        .bind(token_id.0)
        .bind(&new_member.display_name)
        .bind(&new_member.role)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(Member {
            id,
            ring_id: new_member.ring_id,
            user_id: new_member.user_id,
            token_id: token_id.0,
            display_name: new_member.display_name,
            role: new_member.role.unwrap_or_else(|| "member".into()),
            joined_at: now,
        })
    }

    pub async fn get_member_inner(&self, id: &str) -> Result<Option<Member>> {
        let row = sqlx::query_as::<_, MemberRow>(
            "SELECT id, ring_id, user_id, token_id, display_name, role, joined_at FROM members WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| Member {
            id: r.id,
            ring_id: r.ring_id,
            user_id: r.user_id,
            token_id: r.token_id,
            display_name: r.display_name,
            role: r.role,
            joined_at: r.joined_at,
        }))
    }

    pub async fn list_members_by_ring_inner(&self, ring_id: &str) -> Result<Vec<Member>> {
        let rows = sqlx::query_as::<_, MemberRow>(
            "SELECT id, ring_id, user_id, token_id, display_name, role, joined_at FROM members WHERE ring_id = ? ORDER BY joined_at",
        )
        .bind(ring_id)
        .fetch_all(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| Member {
                id: r.id,
                ring_id: r.ring_id,
                user_id: r.user_id,
                token_id: r.token_id,
                display_name: r.display_name,
                role: r.role,
                joined_at: r.joined_at,
            })
            .collect())
    }

    pub async fn get_member_by_user_and_ring_inner(
        &self,
        user_id: &str,
        ring_id: &str,
    ) -> Result<Option<Member>> {
        let row = sqlx::query_as::<_, MemberRow>(
            "SELECT id, ring_id, user_id, token_id, display_name, role, joined_at FROM members WHERE user_id = ? AND ring_id = ?",
        )
        .bind(user_id)
        .bind(ring_id)
        .fetch_optional(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| Member {
            id: r.id,
            ring_id: r.ring_id,
            user_id: r.user_id,
            token_id: r.token_id,
            display_name: r.display_name,
            role: r.role,
            joined_at: r.joined_at,
        }))
    }

    pub async fn update_member_role_inner(&self, id: &str, role: &str) -> Result<()> {
        sqlx::query("UPDATE members SET role = ? WHERE id = ?")
            .bind(role)
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    pub async fn delete_member_inner(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM members WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    pub async fn get_next_token_id_inner(&self, ring_id: &str) -> Result<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COALESCE(MAX(token_id), 0) + 1 FROM members WHERE ring_id = ?")
                .bind(ring_id)
                .fetch_one(self.pool())
                .await
                .map_err(RingError::Database)?;
        Ok(row.0)
    }

    pub async fn create_invite_token_inner(
        &self,
        ring_id: &str,
        token: &str,
        token_type: &str,
        role: &str,
        inviter_id: &str,
    ) -> Result<InviteToken> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

        sqlx::query(
            "INSERT INTO invite_tokens (id, ring_id, token, token_type, role, inviter_id, expires_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(ring_id)
        .bind(token)
        .bind(token_type)
        .bind(role)
        .bind(inviter_id)
        .bind(expires_at.to_rfc3339())
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(InviteToken {
            id,
            ring_id: ring_id.to_string(),
            token: token.to_string(),
            token_type: token_type.to_string(),
            role: role.to_string(),
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

    pub async fn get_invite_token_inner(&self, token: &str) -> Result<Option<InviteToken>> {
        let row = sqlx::query_as::<_, InviteTokenRow>(
            "SELECT id, ring_id, token, token_type, role, inviter_id, max_uses, use_count, max_members, expires_at, used_at, revoked_at, created_at FROM invite_tokens WHERE token = ?",
        )
        .bind(token)
        .fetch_optional(self.pool())
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

    pub async fn count_members_by_ring_inner(&self, ring_id: &str) -> Result<i64> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM members WHERE ring_id = ?")
            .bind(ring_id)
            .fetch_optional(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(row.map(|(c,)| c).unwrap_or(0))
    }
}
