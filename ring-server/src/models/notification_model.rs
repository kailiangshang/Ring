use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub ring_id: String,
    pub user_id: String,
    pub r#type: String,
    pub title: String,
    pub body: Option<String>,
    pub related_id: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewNotification {
    pub ring_id: String,
    pub user_id: String,
    pub n_type: String,
    pub title: String,
    pub body: Option<String>,
    pub related_id: Option<String>,
}
