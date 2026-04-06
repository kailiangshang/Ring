use std::sync::Arc;

use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::models::invite::InviteToken;
use crate::models::member::{Member, NewMember};

pub struct MemberService {
    db: Arc<dyn Repository>,
}

impl MemberService {
    pub fn new(db: Arc<dyn Repository>) -> Self {
        MemberService { db }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn generate_invite(
        &self,
        ring_id: &str,
        inviter_id: &str,
        token_type: &str,
        _role: &str,
        _max_uses: i64,
        max_members: Option<i64>,
        _expires_in_seconds: i64,
    ) -> Result<InviteToken> {
        let ring = self
            .db
            .get_ring(ring_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;
        if ring.creator_id != inviter_id {
            return Err(RingError::Forbidden("only creator can invite".into()));
        }
        if let Some(mm) = max_members {
            let count = self.db.count_members_by_ring(ring_id).await?;
            if count >= mm {
                return Err(RingError::Conflict("ring member limit reached".into()));
            }
        }
        let token_bytes = uuid::Uuid::new_v4().to_string();
        self.db
            .create_invite_token(ring_id, &token_bytes, token_type, inviter_id)
            .await
    }

    pub async fn join_ring(
        &self,
        token_str: &str,
        user_id: &str,
        display_name: &str,
    ) -> Result<Member> {
        let token = self
            .db
            .get_invite_token(token_str)
            .await?
            .ok_or_else(|| RingError::NotFound("invalid invite token".into()))?;

        if token.revoked_at.is_some() {
            return Err(RingError::Forbidden("token has been revoked".into()));
        }
        let now = chrono::Utc::now().to_rfc3339();
        if token.expires_at < now {
            return Err(RingError::Forbidden("token has expired".into()));
        }
        if token.max_uses > 0 && token.use_count >= token.max_uses {
            return Err(RingError::Forbidden("token usage limit reached".into()));
        }

        let existing = self
            .db
            .get_member_by_user_and_ring(user_id, &token.ring_id)
            .await?;
        if existing.is_some() {
            return Err(RingError::Conflict("already a member".into()));
        }

        let new_member = NewMember {
            ring_id: token.ring_id.clone(),
            user_id: user_id.to_string(),
            display_name: display_name.to_string(),
            role: Some(token.role.clone()),
        };
        let member = self.db.create_member(new_member).await?;
        Ok(member)
    }

    pub async fn list_members(&self, ring_id: &str) -> Result<Vec<Member>> {
        self.db.list_members_by_ring(ring_id).await
    }

    pub async fn update_role(
        &self,
        ring_id: &str,
        member_id: &str,
        new_role: &str,
        caller_id: &str,
    ) -> Result<()> {
        let ring = self
            .db
            .get_ring(ring_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;
        if ring.creator_id != caller_id {
            return Err(RingError::Forbidden("only creator can change roles".into()));
        }
        let member = self
            .db
            .get_member(member_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("member {}", member_id)))?;
        if member.ring_id != ring_id {
            return Err(RingError::NotFound(format!(
                "member not in ring {}",
                ring_id
            )));
        }
        if member.role == "creator" {
            return Err(RingError::Forbidden("cannot change creator role".into()));
        }
        self.db.update_member_role(member_id, new_role).await
    }

    pub async fn remove_member(
        &self,
        ring_id: &str,
        member_id: &str,
        caller_id: &str,
    ) -> Result<()> {
        let ring = self
            .db
            .get_ring(ring_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;
        if ring.creator_id != caller_id {
            return Err(RingError::Forbidden(
                "only creator can remove members".into(),
            ));
        }
        let member = self
            .db
            .get_member(member_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("member {}", member_id)))?;
        if member.role == "creator" {
            return Err(RingError::Forbidden("cannot remove creator".into()));
        }
        self.db.delete_member(member_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ring::Ring;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    struct MockMemberRepo {
        rings: Mutex<HashMap<String, Ring>>,
        members: Mutex<Vec<Member>>,
        tokens: Mutex<Vec<InviteToken>>,
    }

    impl MockMemberRepo {
        fn new() -> Self {
            MockMemberRepo {
                rings: Mutex::new(HashMap::new()),
                members: Mutex::new(Vec::new()),
                tokens: Mutex::new(Vec::new()),
            }
        }

        async fn add_ring(&self, ring: Ring) {
            self.rings.lock().await.insert(ring.id.clone(), ring);
        }

        async fn add_token(&self, token: InviteToken) {
            self.tokens.lock().await.push(token);
        }
    }

    #[async_trait::async_trait]
    impl Repository for MockMemberRepo {
        async fn create_user(
            &self,
            _new_user: crate::models::user::NewUser,
        ) -> crate::error::Result<crate::models::user::User> {
            unimplemented!()
        }
        async fn get_user(
            &self,
            _id: &str,
        ) -> crate::error::Result<Option<crate::models::user::User>> {
            unimplemented!()
        }
        async fn list_all_users(&self) -> crate::error::Result<Vec<crate::models::user::User>> {
            unimplemented!()
        }
        async fn is_setup_completed(&self) -> crate::error::Result<bool> {
            Ok(true)
        }
        async fn complete_setup(&self, _user_id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_ring(
            &self,
            _new_ring: crate::models::ring::NewRing,
        ) -> crate::error::Result<crate::models::ring::Ring> {
            unimplemented!()
        }
        async fn get_ring(
            &self,
            id: &str,
        ) -> crate::error::Result<Option<crate::models::ring::Ring>> {
            let rings = self.rings.lock().await;
            Ok(rings.get(id).cloned())
        }
        async fn list_rings_by_user(
            &self,
            _user_id: &str,
        ) -> crate::error::Result<Vec<crate::models::ring::Ring>> {
            unimplemented!()
        }
        async fn update_ring(
            &self,
            _id: &str,
            _name: Option<String>,
            _description: Option<String>,
        ) -> crate::error::Result<crate::models::ring::Ring> {
            unimplemented!()
        }
        async fn delete_ring(&self, _id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_invite_token(
            &self,
            ring_id: &str,
            token: &str,
            token_type: &str,
            inviter_id: &str,
        ) -> crate::error::Result<crate::models::invite::InviteToken> {
            let now = chrono::Utc::now().to_rfc3339();
            let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24)).to_rfc3339();
            let t = InviteToken {
                id: uuid::Uuid::new_v4().to_string(),
                ring_id: ring_id.to_string(),
                token: token.to_string(),
                token_type: token_type.to_string(),
                role: "member".into(),
                inviter_id: inviter_id.to_string(),
                max_uses: 1,
                use_count: 0,
                max_members: None,
                expires_at,
                used_at: None,
                revoked_at: None,
                created_at: now,
            };
            self.tokens.lock().await.push(t.clone());
            Ok(t)
        }
        async fn get_invite_token(
            &self,
            token: &str,
        ) -> crate::error::Result<Option<crate::models::invite::InviteToken>> {
            let tokens = self.tokens.lock().await;
            Ok(tokens.iter().find(|t| t.token == token).cloned())
        }
        async fn get_setting(&self, _key: &str) -> crate::error::Result<Option<String>> {
            unimplemented!()
        }
        async fn set_setting(&self, _key: &str, _value: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn count_members_by_ring(&self, ring_id: &str) -> crate::error::Result<i64> {
            let members = self.members.lock().await;
            Ok(members.iter().filter(|m| m.ring_id == ring_id).count() as i64)
        }
        async fn create_conversation(
            &self,
            _ring_id: &str,
            _title: Option<String>,
            _context_mode: &str,
            _created_by: &str,
        ) -> crate::error::Result<crate::models::conversation::Conversation> {
            unimplemented!()
        }
        async fn list_conversations(
            &self,
            _ring_id: &str,
        ) -> crate::error::Result<Vec<crate::models::conversation::Conversation>> {
            unimplemented!()
        }
        async fn get_conversation(
            &self,
            _id: &str,
        ) -> crate::error::Result<Option<crate::models::conversation::Conversation>> {
            unimplemented!()
        }
        async fn create_message(
            &self,
            _conversation_id: &str,
            _role: &str,
            _content: &str,
            _sender_id: Option<&str>,
        ) -> crate::error::Result<crate::models::conversation::Message> {
            unimplemented!()
        }
        async fn get_messages(
            &self,
            _conversation_id: &str,
            _limit: i64,
            _before_id: Option<&str>,
        ) -> crate::error::Result<Vec<crate::models::conversation::Message>> {
            unimplemented!()
        }
        async fn update_ring_status(&self, _id: &str, _status: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn list_blueprint_templates(
            &self,
        ) -> crate::error::Result<Vec<crate::models::blueprint::BlueprintTemplate>> {
            unimplemented!()
        }
        async fn create_blueprint_template(
            &self,
            _id: &str,
            _name: &str,
            _description: Option<&str>,
            _graphs_json: &str,
            _is_system: bool,
        ) -> crate::error::Result<crate::models::blueprint::BlueprintTemplate> {
            unimplemented!()
        }
        async fn index_node_search(
            &self,
            _node_id: &str,
            _graph_id: &str,
            _label: &str,
            _content: &str,
        ) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn delete_node_search(&self, _node_id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn search_nodes_fts(
            &self,
            _query: &str,
            _graph_ids: Option<Vec<String>>,
            _limit: i64,
        ) -> crate::error::Result<Vec<crate::models::graph_model::SearchResult>> {
            unimplemented!()
        }
        async fn create_archive_record(
            &self,
            _id: &str,
            _ring_id: &str,
            _node_id: Option<&str>,
            _conversation_id: Option<&str>,
            _message_ids: &str,
            _markdown_path: &str,
            _archived_by: &str,
            _git_commit_sha: Option<&str>,
            _pr_status: Option<&str>,
            _pr_url: Option<&str>,
        ) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn list_archive_records_by_ring(
            &self,
            _ring_id: &str,
        ) -> crate::error::Result<Vec<crate::models::git_model::ArchiveRecord>> {
            unimplemented!()
        }
        async fn get_archive_record(
            &self,
            _id: &str,
        ) -> crate::error::Result<Option<crate::models::git_model::ArchiveRecord>> {
            unimplemented!()
        }
        async fn update_archive_pr_status(
            &self,
            _id: &str,
            _pr_status: &str,
        ) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_member(
            &self,
            new_member: crate::models::member::NewMember,
        ) -> crate::error::Result<crate::models::member::Member> {
            let mut members = self.members.lock().await;
            let token_id = members
                .iter()
                .filter(|m| m.ring_id == new_member.ring_id)
                .map(|m| m.token_id)
                .max()
                .unwrap_or(0)
                + 1;
            let m = Member {
                id: uuid::Uuid::new_v4().to_string(),
                ring_id: new_member.ring_id,
                user_id: new_member.user_id,
                token_id,
                display_name: new_member.display_name,
                role: new_member.role.unwrap_or_else(|| "member".into()),
                joined_at: chrono::Utc::now().to_rfc3339(),
            };
            members.push(m.clone());
            Ok(m)
        }
        async fn get_member(
            &self,
            id: &str,
        ) -> crate::error::Result<Option<crate::models::member::Member>> {
            let members = self.members.lock().await;
            Ok(members.iter().find(|m| m.id == id).cloned())
        }
        async fn list_members_by_ring(
            &self,
            ring_id: &str,
        ) -> crate::error::Result<Vec<crate::models::member::Member>> {
            let members = self.members.lock().await;
            Ok(members
                .iter()
                .filter(|m| m.ring_id == ring_id)
                .cloned()
                .collect())
        }
        async fn get_member_by_user_and_ring(
            &self,
            user_id: &str,
            ring_id: &str,
        ) -> crate::error::Result<Option<crate::models::member::Member>> {
            let members = self.members.lock().await;
            Ok(members
                .iter()
                .find(|m| m.user_id == user_id && m.ring_id == ring_id)
                .cloned())
        }
        async fn update_member_role(&self, id: &str, role: &str) -> crate::error::Result<()> {
            let mut members = self.members.lock().await;
            if let Some(m) = members.iter_mut().find(|m| m.id == id) {
                m.role = role.to_string();
            }
            Ok(())
        }
        async fn delete_member(&self, id: &str) -> crate::error::Result<()> {
            let mut members = self.members.lock().await;
            members.retain(|m| m.id != id);
            Ok(())
        }
        async fn get_next_token_id(&self, _ring_id: &str) -> crate::error::Result<i64> {
            unimplemented!()
        }
        async fn create_notification(
            &self,
            _n: crate::models::notification_model::NewNotification,
        ) -> crate::error::Result<crate::models::notification_model::Notification> {
            unimplemented!()
        }
        async fn list_notifications_by_user(
            &self,
            _user_id: &str,
            _unread_only: bool,
        ) -> crate::error::Result<Vec<crate::models::notification_model::Notification>> {
            unimplemented!()
        }
        async fn mark_notification_read(&self, _id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_session(
            &self,
            _ring_id: &str,
            _title: Option<&str>,
            _scenario: &str,
            _created_by: &str,
            _archive_enabled: bool,
        ) -> crate::error::Result<crate::models::session_model::Session> {
            unimplemented!()
        }
        async fn get_session(&self, _id: &str) -> crate::error::Result<Option<crate::models::session_model::Session>> {
            unimplemented!()
        }
        async fn list_sessions_by_ring(
            &self,
            _ring_id: &str,
            _status: Option<&str>,
        ) -> crate::error::Result<Vec<crate::models::session_model::Session>> {
            unimplemented!()
        }
        async fn update_session_status(&self, _id: &str, _status: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn update_session_archive(&self, _id: &str, _enabled: bool) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn delete_session(&self, _id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_session_member(
            &self,
            _session_id: &str,
            _user_id: &str,
            _role: &str,
        ) -> crate::error::Result<crate::models::session_model::SessionMember> {
            unimplemented!()
        }
        async fn list_session_members(&self, _session_id: &str) -> crate::error::Result<Vec<crate::models::session_model::SessionMember>> {
            unimplemented!()
        }
        async fn leave_session_member(&self, _session_id: &str, _user_id: &str) -> crate::error::Result<()> {
            unimplemented!()
        }
        async fn create_session_message(
            &self,
            _session_id: &str,
            _sender_id: &str,
            _role: &str,
            _content: &str,
            _seq_num: i64,
        ) -> crate::error::Result<crate::models::session_model::SessionMessage> {
            unimplemented!()
        }
        async fn get_session_messages(
            &self,
            _session_id: &str,
            _after_seq: Option<i64>,
            _limit: i64,
        ) -> crate::error::Result<Vec<crate::models::session_model::SessionMessage>> {
            unimplemented!()
        }
    }

    fn make_ring(id: &str, creator_id: &str) -> Ring {
        Ring {
            id: id.to_string(),
            name: "TestRing".into(),
            description: None,
            creator_id: creator_id.to_string(),
            gitlab_repo: "auto_create".into(),
            local_path: ".ring/repos/ring-TestRing".into(),
            next_token_id: 2,
            status: "active".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[tokio::test]
    async fn generate_invite_success() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let svc = MemberService::new(repo);
        let token = svc
            .generate_invite("ring-1", "user-a", "open", "member", 10, None, 86400)
            .await
            .unwrap();
        assert_eq!(token.ring_id, "ring-1");
        assert_eq!(token.inviter_id, "user-a");
    }

    #[tokio::test]
    async fn generate_invite_non_creator_fails() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let svc = MemberService::new(repo);
        let result = svc
            .generate_invite("ring-1", "user-b", "open", "member", 10, None, 86400)
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RingError::Forbidden(msg) => assert!(msg.contains("creator")),
            _ => panic!("expected Forbidden"),
        }
    }

    #[tokio::test]
    async fn join_ring_success() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let svc = MemberService::new(repo.clone());
        let token = svc
            .generate_invite("ring-1", "user-a", "open", "member", 10, None, 86400)
            .await
            .unwrap();

        let member = svc.join_ring(&token.token, "user-b", "Bob").await.unwrap();
        assert_eq!(member.user_id, "user-b");
        assert_eq!(member.ring_id, "ring-1");
        assert_eq!(member.display_name, "Bob");
    }

    #[tokio::test]
    async fn join_ring_expired_token_fails() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let expired_token = InviteToken {
            id: "tok-1".into(),
            ring_id: "ring-1".into(),
            token: "expired-tok".into(),
            token_type: "open".into(),
            role: "member".into(),
            inviter_id: "user-a".into(),
            max_uses: 1,
            use_count: 0,
            max_members: None,
            expires_at: "2020-01-01T00:00:00Z".into(),
            used_at: None,
            revoked_at: None,
            created_at: "2020-01-01T00:00:00Z".into(),
        };
        repo.add_token(expired_token).await;
        let svc = MemberService::new(repo);
        let result = svc.join_ring("expired-tok", "user-b", "Bob").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RingError::Forbidden(msg) => assert!(msg.contains("expired")),
            _ => panic!("expected Forbidden"),
        }
    }

    #[tokio::test]
    async fn join_ring_already_member_fails() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let svc = MemberService::new(repo.clone());
        let token = svc
            .generate_invite("ring-1", "user-a", "open", "member", 10, None, 86400)
            .await
            .unwrap();

        svc.join_ring(&token.token, "user-b", "Bob").await.unwrap();
        let result = svc.join_ring(&token.token, "user-b", "Bob").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RingError::Conflict(msg) => assert!(msg.contains("already a member")),
            _ => panic!("expected Conflict"),
        }
    }

    #[tokio::test]
    async fn update_role_success() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let svc = MemberService::new(repo.clone());
        let token = svc
            .generate_invite("ring-1", "user-a", "open", "member", 10, None, 86400)
            .await
            .unwrap();
        let member = svc.join_ring(&token.token, "user-b", "Bob").await.unwrap();

        svc.update_role("ring-1", &member.id, "admin", "user-a")
            .await
            .unwrap();
        let updated = repo.get_member(&member.id).await.unwrap().unwrap();
        assert_eq!(updated.role, "admin");
    }

    #[tokio::test]
    async fn update_role_non_creator_fails() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let svc = MemberService::new(repo.clone());
        let token = svc
            .generate_invite("ring-1", "user-a", "open", "member", 10, None, 86400)
            .await
            .unwrap();
        let member = svc.join_ring(&token.token, "user-b", "Bob").await.unwrap();

        let result = svc
            .update_role("ring-1", &member.id, "admin", "user-b")
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RingError::Forbidden(msg) => assert!(msg.contains("creator")),
            _ => panic!("expected Forbidden"),
        }
    }

    #[tokio::test]
    async fn remove_member_success() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let svc = MemberService::new(repo.clone());
        let token = svc
            .generate_invite("ring-1", "user-a", "open", "member", 10, None, 86400)
            .await
            .unwrap();
        let member = svc.join_ring(&token.token, "user-b", "Bob").await.unwrap();

        svc.remove_member("ring-1", &member.id, "user-a")
            .await
            .unwrap();
        let found = repo.get_member(&member.id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn remove_member_creator_fails() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let svc = MemberService::new(repo.clone());
        let member = repo
            .create_member(NewMember {
                ring_id: "ring-1".into(),
                user_id: "user-a".into(),
                display_name: "Alice".into(),
                role: Some("creator".into()),
            })
            .await
            .unwrap();

        let result = svc.remove_member("ring-1", &member.id, "user-a").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            RingError::Forbidden(msg) => assert!(msg.contains("cannot remove creator")),
            _ => panic!("expected Forbidden"),
        }
    }

    #[tokio::test]
    async fn list_members_returns_all() {
        let repo = Arc::new(MockMemberRepo::new());
        repo.add_ring(make_ring("ring-1", "user-a")).await;
        let svc = MemberService::new(repo.clone());

        repo.create_member(NewMember {
            ring_id: "ring-1".into(),
            user_id: "user-a".into(),
            display_name: "Alice".into(),
            role: Some("creator".into()),
        })
        .await
        .unwrap();
        repo.create_member(NewMember {
            ring_id: "ring-1".into(),
            user_id: "user-b".into(),
            display_name: "Bob".into(),
            role: None,
        })
        .await
        .unwrap();
        repo.create_member(NewMember {
            ring_id: "ring-1".into(),
            user_id: "user-c".into(),
            display_name: "Carol".into(),
            role: None,
        })
        .await
        .unwrap();

        let members = svc.list_members("ring-1").await.unwrap();
        assert_eq!(members.len(), 3);
    }
}
