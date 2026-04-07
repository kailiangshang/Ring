use crate::error::{Result, RingError};
use crate::models::session_model::{Session, SessionMember, SessionMessage};

#[derive(sqlx::FromRow)]
pub(crate) struct SessionRow {
    pub id: String,
    pub ring_id: String,
    pub title: Option<String>,
    pub scenario: String,
    pub created_by: String,
    pub archive_enabled: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(sqlx::FromRow)]
pub(crate) struct SessionMemberRow {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub role: String,
    pub status: String,
    pub joined_at: String,
    pub left_at: Option<String>,
}

#[derive(sqlx::FromRow)]
pub(crate) struct SessionMessageRow {
    pub id: String,
    pub session_id: String,
    pub sender_id: String,
    pub role: String,
    pub content: String,
    pub seq_num: i64,
    pub created_at: String,
}

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    pub async fn create_session_inner(
        &self,
        ring_id: &str,
        title: Option<&str>,
        scenario: &str,
        created_by: &str,
        archive_enabled: bool,
    ) -> Result<Session> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?)",
        )
        .bind(&id)
        .bind(ring_id)
        .bind(title)
        .bind(scenario)
        .bind(created_by)
        .bind(archive_enabled)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(Session {
            id,
            ring_id: ring_id.to_string(),
            title: title.map(|s| s.to_string()),
            scenario: scenario.to_string(),
            created_by: created_by.to_string(),
            archive_enabled,
            status: "active".to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn get_session_inner(&self, id: &str) -> Result<Option<Session>> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at FROM sessions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| Session {
            id: r.id,
            ring_id: r.ring_id,
            title: r.title,
            scenario: r.scenario,
            created_by: r.created_by,
            archive_enabled: r.archive_enabled,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    pub async fn list_sessions_by_ring_inner(
        &self,
        ring_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<Session>> {
        let rows = if let Some(s) = status {
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at FROM sessions WHERE ring_id = ? AND status = ? ORDER BY created_at DESC",
            )
            .bind(ring_id)
            .bind(s)
            .fetch_all(self.pool())
            .await
            .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at FROM sessions WHERE ring_id = ? ORDER BY created_at DESC",
            )
            .bind(ring_id)
            .fetch_all(self.pool())
            .await
            .map_err(RingError::Database)?
        };

        Ok(rows
            .into_iter()
            .map(|r| Session {
                id: r.id,
                ring_id: r.ring_id,
                title: r.title,
                scenario: r.scenario,
                created_by: r.created_by,
                archive_enabled: r.archive_enabled,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    pub async fn update_session_status_inner(&self, id: &str, status: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE sessions SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    pub async fn update_session_archive_inner(&self, id: &str, enabled: bool) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE sessions SET archive_enabled = ?, updated_at = ? WHERE id = ?")
            .bind(enabled)
            .bind(&now)
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    pub async fn delete_session_inner(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM session_messages WHERE session_id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        sqlx::query("DELETE FROM session_members WHERE session_id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    pub async fn create_session_member_inner(
        &self,
        session_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<SessionMember> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO session_members (id, session_id, user_id, role, status, joined_at) VALUES (?, ?, ?, ?, 'active', ?)",
        )
        .bind(&id)
        .bind(session_id)
        .bind(user_id)
        .bind(role)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(SessionMember {
            id,
            session_id: session_id.to_string(),
            user_id: user_id.to_string(),
            role: role.to_string(),
            status: "active".to_string(),
            joined_at: now,
            left_at: None,
        })
    }

    pub async fn list_session_members_inner(&self, session_id: &str) -> Result<Vec<SessionMember>> {
        let rows = sqlx::query_as::<_, SessionMemberRow>(
            "SELECT id, session_id, user_id, role, status, joined_at, left_at FROM session_members WHERE session_id = ? AND status = 'active'",
        )
        .bind(session_id)
        .fetch_all(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| SessionMember {
                id: r.id,
                session_id: r.session_id,
                user_id: r.user_id,
                role: r.role,
                status: r.status,
                joined_at: r.joined_at,
                left_at: r.left_at,
            })
            .collect())
    }

    pub async fn leave_session_member_inner(&self, session_id: &str, user_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE session_members SET status = 'left', left_at = ? WHERE session_id = ? AND user_id = ?",
        )
        .bind(&now)
        .bind(session_id)
        .bind(user_id)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }

    pub async fn create_session_message_inner(
        &self,
        session_id: &str,
        sender_id: &str,
        role: &str,
        content: &str,
        seq_num: i64,
    ) -> Result<SessionMessage> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO session_messages (id, session_id, sender_id, role, content, seq_num, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(session_id)
        .bind(sender_id)
        .bind(role)
        .bind(content)
        .bind(seq_num)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(SessionMessage {
            id,
            session_id: session_id.to_string(),
            sender_id: sender_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            seq_num,
            created_at: now,
        })
    }

    pub async fn get_session_messages_inner(
        &self,
        session_id: &str,
        after_seq: Option<i64>,
        limit: i64,
    ) -> Result<Vec<SessionMessage>> {
        let rows = if let Some(seq) = after_seq {
            sqlx::query_as::<_, SessionMessageRow>(
                "SELECT id, session_id, sender_id, role, content, seq_num, created_at FROM session_messages WHERE session_id = ? AND seq_num > ? ORDER BY seq_num ASC LIMIT ?",
            )
            .bind(session_id)
            .bind(seq)
            .bind(limit)
            .fetch_all(self.pool())
            .await
            .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, SessionMessageRow>(
                "SELECT id, session_id, sender_id, role, content, seq_num, created_at FROM session_messages WHERE session_id = ? ORDER BY seq_num ASC LIMIT ?",
            )
            .bind(session_id)
            .bind(limit)
            .fetch_all(self.pool())
            .await
            .map_err(RingError::Database)?
        };

        Ok(rows
            .into_iter()
            .map(|r| SessionMessage {
                id: r.id,
                session_id: r.session_id,
                sender_id: r.sender_id,
                role: r.role,
                content: r.content,
                seq_num: r.seq_num,
                created_at: r.created_at,
            })
            .collect())
    }
}
