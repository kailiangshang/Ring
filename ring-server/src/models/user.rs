use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub ip_address: Option<String>,
    pub setup_completed: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub display_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_serializes_snake_case() {
        let user = User {
            id: "uuid-1".into(),
            display_name: "张三".into(),
            avatar_url: None,
            ip_address: Some("192.168.1.1".into()),
            setup_completed: true,
            created_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_value(&user).unwrap();
        assert_eq!(json["display_name"], "张三");
        assert_eq!(json["setup_completed"], true);
    }

    #[test]
    fn user_deserializes_snake_case() {
        let json = r#"{"id":"uuid-1","display_name":"张三","avatar_url":null,"ip_address":"192.168.1.1","setup_completed":true,"created_at":"2026-01-01T00:00:00Z"}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.display_name, "张三");
        assert_eq!(user.setup_completed, true);
        assert_eq!(user.ip_address, Some("192.168.1.1".into()));
    }

    #[test]
    fn new_user_requires_display_name() {
        let json = r#"{"display_name":"李四"}"#;
        let new_user: NewUser = serde_json::from_str(json).unwrap();
        assert_eq!(new_user.display_name, "李四");
    }
}
