use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ring {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub creator_id: String,
    pub gitlab_repo: String,
    pub local_path: String,
    pub next_token_id: i64,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRing {
    pub name: String,
    pub description: Option<String>,
    pub creator_id: String,
    pub gitlab_repo: String,
    pub namespace: Option<String>,
    pub role_description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ring_requires_name() {
        let json = r#"{"name": "test", "creator_id": "user-1", "gitlab_repo": "auto", "role_description": "expert"}"#;
        let ring: NewRing = serde_json::from_str(json).unwrap();
        assert_eq!(ring.name, "test");
    }

    #[test]
    fn ring_serializes_snake_case() {
        let ring = Ring {
            id: "uuid-1".into(),
            name: "竞品分析".into(),
            description: Some("desc".into()),
            creator_id: "user-1".into(),
            gitlab_repo: "git@gitlab.corp:user/ring.git".into(),
            local_path: "/home/.ring/repos/ring-竞品分析".into(),
            next_token_id: 2,
            status: "active".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_value(&ring).unwrap();
        assert_eq!(json["creator_id"], "user-1");
        assert_eq!(json["gitlab_repo"], "git@gitlab.corp:user/ring.git");
        assert_eq!(json["next_token_id"], 2);
        assert_eq!(json["created_at"], "2026-01-01T00:00:00Z");
    }

    #[test]
    fn new_ring_deserializes_with_optional_fields() {
        let json = r#"{"name":"test","creator_id":"user-1","gitlab_repo":"auto_create","namespace":null,"role_description":"expert","description":"a desc"}"#;
        let ring: NewRing = serde_json::from_str(json).unwrap();
        assert_eq!(ring.name, "test");
        assert_eq!(ring.description, Some("a desc".into()));
        assert!(ring.namespace.is_none());
    }
}
