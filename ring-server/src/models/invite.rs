use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteToken {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invite_token_serializes_snake_case() {
        let token = InviteToken {
            id: "uuid-1".into(),
            ring_id: "ring-1".into(),
            token: "abc123".into(),
            token_type: "open".into(),
            role: "member".into(),
            inviter_id: "user-1".into(),
            max_uses: 1,
            use_count: 0,
            max_members: None,
            expires_at: "2026-01-02T00:00:00Z".into(),
            used_at: None,
            revoked_at: None,
            created_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_value(&token).unwrap();
        assert_eq!(json["ring_id"], "ring-1");
        assert_eq!(json["token_type"], "open");
        assert_eq!(json["inviter_id"], "user-1");
        assert_eq!(json["max_uses"], 1);
        assert_eq!(json["use_count"], 0);
    }
}
