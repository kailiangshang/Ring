use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewConversation {
    pub ring_id: String,
    pub title: Option<String>,
    pub context_mode: Option<String>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sender_id: Option<String>,
    pub tool_calls: Option<String>,
    pub archived: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMessage {
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sender_id: Option<String>,
}
