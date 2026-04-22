use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct NotificationRow {
    pub id: String,
    pub user_id: String,
    pub ring_id: Option<String>,
    pub notification_type: String,
    pub title: String,
    pub content: Option<String>,
    pub is_read: bool,
    pub related_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotification {
    pub user_id: String,
    pub ring_id: Option<String>,
    pub notification_type: String,
    pub title: String,
    pub content: Option<String>,
    pub related_id: Option<String>,
}

pub async fn create_notification(
    pool: &sqlx::SqlitePool,
    id: &str,
    input: &CreateNotification,
) -> Result<NotificationRow> {
    sqlx::query_as::<_, NotificationRow>(
        "INSERT INTO notifications (id, user_id, ring_id, type, title, content, related_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         RETURNING *",
    )
    .bind(id)
    .bind(&input.user_id)
    .bind(&input.ring_id)
    .bind(&input.notification_type)
    .bind(&input.title)
    .bind(&input.content)
    .bind(&input.related_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn list_notifications(
    pool: &sqlx::SqlitePool,
    user_id: &str,
    unread_only: bool,
) -> Result<Vec<NotificationRow>> {
    let rows = if unread_only {
        sqlx::query_as::<_, NotificationRow>(
            "SELECT * FROM notifications WHERE user_id = ?1 AND is_read = 0 ORDER BY created_at DESC LIMIT 50",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, NotificationRow>(
            "SELECT * FROM notifications WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 50",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    };
    rows.map_err(Into::into)
}

pub async fn mark_as_read(
    pool: &sqlx::SqlitePool,
    notification_id: &str,
    user_id: &str,
) -> Result<()> {
    let result = sqlx::query("UPDATE notifications SET is_read = 1 WHERE id = ?1 AND user_id = ?2")
        .bind(notification_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("notification not found".into()));
    }
    Ok(())
}

pub async fn mark_all_as_read(pool: &sqlx::SqlitePool, user_id: &str) -> Result<()> {
    sqlx::query("UPDATE notifications SET is_read = 1 WHERE user_id = ?1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_unread_count(pool: &sqlx::SqlitePool, user_id: &str) -> Result<i64> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = ?1 AND is_read = 0")
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(count)
}

pub async fn delete_notification(
    pool: &sqlx::SqlitePool,
    notification_id: &str,
    user_id: &str,
) -> Result<()> {
    let result = sqlx::query("DELETE FROM notifications WHERE id = ?1 AND user_id = ?2")
        .bind(notification_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("notification not found".into()));
    }
    Ok(())
}
