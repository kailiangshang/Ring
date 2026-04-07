use crate::error::{Result, RingError};
use crate::models::notification_model::{NewNotification, Notification};

#[derive(sqlx::FromRow)]
pub(crate) struct NotificationRow {
    pub id: String,
    pub ring_id: String,
    pub user_id: String,
    #[sqlx(rename = "type")]
    pub type_field: String,
    pub title: String,
    pub body: Option<String>,
    pub related_id: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    pub async fn create_notification_inner(&self, n: NewNotification) -> Result<Notification> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO notifications (id, ring_id, user_id, type, title, body, related_id, is_read, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, FALSE, ?)",
        )
        .bind(&id)
        .bind(&n.ring_id)
        .bind(&n.user_id)
        .bind(&n.n_type)
        .bind(&n.title)
        .bind(&n.body)
        .bind(&n.related_id)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;

        Ok(Notification {
            id,
            ring_id: n.ring_id,
            user_id: n.user_id,
            r#type: n.n_type,
            title: n.title,
            body: n.body,
            related_id: n.related_id,
            is_read: false,
            created_at: now,
        })
    }

    pub async fn list_notifications_by_user_inner(
        &self,
        user_id: &str,
        unread_only: bool,
    ) -> Result<Vec<Notification>> {
        let rows = if unread_only {
            sqlx::query_as::<_, NotificationRow>(
                "SELECT id, ring_id, user_id, type, title, body, related_id, is_read, created_at FROM notifications WHERE user_id = ? AND is_read = FALSE ORDER BY created_at DESC",
            )
            .bind(user_id)
            .fetch_all(self.pool())
            .await
            .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, NotificationRow>(
                "SELECT id, ring_id, user_id, type, title, body, related_id, is_read, created_at FROM notifications WHERE user_id = ? ORDER BY created_at DESC",
            )
            .bind(user_id)
            .fetch_all(self.pool())
            .await
            .map_err(RingError::Database)?
        };

        Ok(rows
            .into_iter()
            .map(|r| Notification {
                id: r.id,
                ring_id: r.ring_id,
                user_id: r.user_id,
                r#type: r.type_field,
                title: r.title,
                body: r.body,
                related_id: r.related_id,
                is_read: r.is_read,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn mark_notification_read_inner(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE notifications SET is_read = TRUE WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }
}
