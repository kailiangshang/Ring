use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMember {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub role: String,
    pub status: String,
    pub joined_at: String,
    pub left_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub session_id: String,
    pub sender_id: String,
    pub role: String,
    pub content: String,
    pub seq_num: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    pub scenario: String,
    pub archive_enabled: Option<bool>,
    pub invite_member_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetailResponse {
    pub id: String,
    pub ring_id: String,
    pub title: Option<String>,
    pub scenario: String,
    pub created_by: String,
    pub archive_enabled: bool,
    pub status: String,
    pub members: Vec<SessionMemberBrief>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMemberBrief {
    pub user_id: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListItem {
    pub id: String,
    pub title: Option<String>,
    pub created_by: String,
    pub member_count: i64,
    pub archive_enabled: bool,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteSessionRequest {
    pub member_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveToggleRequest {
    pub archive_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessagesResponse {
    pub messages: Vec<SessionMessage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_serializes_snake_case() {
        let session = Session {
            id: "s-1".into(),
            ring_id: "r-1".into(),
            title: Some("test".into()),
            scenario: "discussion".into(),
            created_by: "u-1".into(),
            archive_enabled: false,
            status: "active".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_value(&session).unwrap();
        assert_eq!(json["ring_id"], "r-1");
        assert_eq!(json["archive_enabled"], false);
    }

    #[test]
    fn create_session_request_deserializes() {
        let json = r#"{"scenario":"deep_research"}"#;
        let req: CreateSessionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.scenario, "deep_research");
        assert!(req.title.is_none());
        assert!(req.archive_enabled.is_none());
    }
}
