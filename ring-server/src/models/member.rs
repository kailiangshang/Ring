use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub ring_id: String,
    pub user_id: String,
    pub token_id: i64,
    pub display_name: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMember {
    pub ring_id: String,
    pub user_id: String,
    pub display_name: String,
    pub role: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_serializes_snake_case() {
        let member = Member {
            id: "uuid-1".into(),
            ring_id: "ring-1".into(),
            user_id: "user-1".into(),
            token_id: 2,
            display_name: "张三".into(),
            role: "member".into(),
            joined_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_value(&member).unwrap();
        assert_eq!(json["ring_id"], "ring-1");
        assert_eq!(json["user_id"], "user-1");
        assert_eq!(json["token_id"], 2);
        assert_eq!(json["display_name"], "张三");
        assert_eq!(json["joined_at"], "2026-01-01T00:00:00Z");
    }

    #[test]
    fn new_member_deserializes_with_default_role() {
        let json = r#"{"ring_id":"ring-1","user_id":"user-1","display_name":"张三"}"#;
        let member: NewMember = serde_json::from_str(json).unwrap();
        assert!(member.role.is_none());
    }
}
