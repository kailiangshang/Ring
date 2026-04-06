use std::sync::Arc;

use crate::db::traits::Repository;
use crate::error::Result;
use crate::models::notification_model::Notification;

pub struct NotificationService {
    db: Arc<dyn Repository>,
}

impl NotificationService {
    pub fn new(db: Arc<dyn Repository>) -> Self {
        NotificationService { db }
    }

    pub async fn list_for_user(
        &self,
        user_id: &str,
        unread_only: bool,
    ) -> Result<Vec<Notification>> {
        self.db
            .list_notifications_by_user(user_id, unread_only)
            .await
    }

    pub async fn mark_read(&self, notification_id: &str) -> Result<()> {
        self.db.mark_notification_read(notification_id).await
    }
}
