use crate::error::{Result, RingError};
use crate::models::conversation::{Conversation, Message};

#[derive(sqlx::FromRow)]
pub(crate) struct ConversationRow {
    pub id: String,
    pub ring_id: String,
    pub title: Option<String>,
    pub mode: String,
    pub context_mode: String,
    pub token_count: i64,
    pub token_limit: i64,
    pub auto_compact: bool,
    pub summary: Option<String>,
    pub compacted_at: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl ConversationRow {
    pub fn into_model(self) -> Conversation {
        Conversation {
            id: self.id,
            ring_id: self.ring_id,
            title: self.title,
            mode: self.mode,
            context_mode: self.context_mode,
            token_count: self.token_count,
            token_limit: self.token_limit,
            auto_compact: self.auto_compact,
            summary: self.summary,
            compacted_at: self.compacted_at,
            created_by: self.created_by,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sender_id: Option<String>,
    pub tool_calls: Option<String>,
    pub archived: bool,
    pub created_at: String,
}

impl MessageRow {
    pub fn into_model(self) -> Message {
        Message {
            id: self.id,
            conversation_id: self.conversation_id,
            role: self.role,
            content: self.content,
            sender_id: self.sender_id,
            tool_calls: self.tool_calls,
            archived: self.archived,
            created_at: self.created_at,
        }
    }
}

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    pub async fn create_conversation_inner(
        &self,
        ring_id: &str,
        title: Option<String>,
        context_mode: &str,
        created_by: &str,
    ) -> Result<Conversation> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO conversations (id, ring_id, title, mode, context_mode, created_by, created_at, updated_at) VALUES (?, ?, ?, 'chat', ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(ring_id)
        .bind(&title)
        .bind(context_mode)
        .bind(created_by)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(Conversation {
            id,
            ring_id: ring_id.to_string(),
            title,
            mode: "chat".into(),
            context_mode: context_mode.to_string(),
            token_count: 0,
            token_limit: 100000,
            auto_compact: false,
            summary: None,
            compacted_at: None,
            created_by: created_by.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn list_conversations_inner(&self, ring_id: &str) -> Result<Vec<Conversation>> {
        let rows = sqlx::query_as::<_, ConversationRow>(
            "SELECT id, ring_id, title, mode, context_mode, token_count, token_limit, auto_compact, summary, compacted_at, created_by, created_at, updated_at FROM conversations WHERE ring_id = ? ORDER BY created_at DESC",
        )
        .bind(ring_id)
        .fetch_all(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(rows.into_iter().map(|r| r.into_model()).collect())
    }

    pub async fn get_conversation_inner(&self, id: &str) -> Result<Option<Conversation>> {
        let row = sqlx::query_as::<_, ConversationRow>(
            "SELECT id, ring_id, title, mode, context_mode, token_count, token_limit, auto_compact, summary, compacted_at, created_by, created_at, updated_at FROM conversations WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| r.into_model()))
    }

    pub async fn create_message_inner(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        sender_id: Option<&str>,
    ) -> Result<Message> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content, sender_id, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .bind(sender_id)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(Message {
            id,
            conversation_id: conversation_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            sender_id: sender_id.map(|s| s.to_string()),
            tool_calls: None,
            archived: false,
            created_at: now,
        })
    }

    pub async fn get_messages_inner(
        &self,
        conversation_id: &str,
        limit: i64,
        before_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        let rows = match before_id {
            Some(bid) => {
                let before_created = sqlx::query_as::<_, (String,)>(
                    "SELECT created_at FROM messages WHERE id = ?",
                )
                .bind(bid)
                .fetch_optional(self.pool())
                .await
                .map_err(RingError::Database)?;

                match before_created {
                    Some((ts,)) => {
                        sqlx::query_as::<_, MessageRow>(
                            "SELECT id, conversation_id, role, content, sender_id, tool_calls, archived, created_at FROM messages WHERE conversation_id = ? AND created_at < ? ORDER BY created_at ASC LIMIT ?",
                        )
                        .bind(conversation_id)
                        .bind(&ts)
                        .bind(limit)
                        .fetch_all(self.pool())
                        .await
                        .map_err(RingError::Database)?
                    }
                    None => vec![],
                }
            }
            None => {
                sqlx::query_as::<_, MessageRow>(
                    "SELECT id, conversation_id, role, content, sender_id, tool_calls, archived, created_at FROM messages WHERE conversation_id = ? ORDER BY created_at ASC LIMIT ?",
                )
                .bind(conversation_id)
                .bind(limit)
                .fetch_all(self.pool())
                .await
                .map_err(RingError::Database)?
            }
        };

        Ok(rows.into_iter().map(|r| r.into_model()).collect())
    }
}
