use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveRequest {
    pub message_ids: Vec<String>,
    pub conversation_id: String,
    pub graph_id: String,
    pub target_node_id: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveResponse {
    pub archive_id: String,
    pub markdown_path: String,
    pub git_status: String,
    pub pr_url: Option<String>,
    pub queue_position: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveQueueResponse {
    pub current_review: Option<QueueItem>,
    pub queue: Vec<QueueItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub pr_id: i64,
    pub author: String,
    pub title: String,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrResponse {
    pub pr_id: i64,
    pub title: String,
    pub author: String,
    pub state: String,
    pub changes: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub file: String,
    pub status: String,
    pub additions: i64,
    pub deletions: i64,
    pub diff: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitLogResponse {
    pub commits: Vec<CommitEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitEntry {
    pub id: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ArchiveRecord {
    pub id: String,
    pub ring_id: String,
    pub node_id: Option<String>,
    pub conversation_id: Option<String>,
    pub message_ids: Option<String>,
    pub markdown_path: String,
    pub archived_by: String,
    pub git_commit_sha: Option<String>,
    pub pr_status: Option<String>,
    pub pr_url: Option<String>,
    pub created_at: String,
}
