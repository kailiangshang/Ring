use crate::error::Result;
use crate::models::blueprint::BlueprintTemplate;
use crate::models::conversation::{Conversation, Message};
use crate::models::git_model::ArchiveRecord;
use crate::models::graph_model::SearchResult;
use crate::models::invite::InviteToken;
use crate::models::member::{Member, NewMember};
use crate::models::notification_model::{NewNotification, Notification};
use crate::models::ring::{NewRing, Ring};
use crate::models::session_model::{Session, SessionMember, SessionMessage};
use crate::models::user::{NewUser, User};

#[async_trait::async_trait]
#[allow(clippy::too_many_arguments)]
pub trait Repository: Send + Sync {
    async fn create_user(&self, new_user: NewUser) -> Result<User>;
    async fn get_user(&self, id: &str) -> Result<Option<User>>;
    async fn list_all_users(&self) -> Result<Vec<User>>;
    async fn is_setup_completed(&self) -> Result<bool>;
    async fn complete_setup(&self, user_id: &str) -> Result<()>;
    async fn create_ring(&self, new_ring: NewRing) -> Result<Ring>;
    async fn get_ring(&self, id: &str) -> Result<Option<Ring>>;
    async fn list_rings_by_user(&self, user_id: &str) -> Result<Vec<Ring>>;
    async fn update_ring(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Ring>;
    async fn delete_ring(&self, id: &str) -> Result<()>;
    async fn create_invite_token(
        &self,
        ring_id: &str,
        token: &str,
        token_type: &str,
        inviter_id: &str,
    ) -> Result<InviteToken>;
    async fn get_invite_token(&self, token: &str) -> Result<Option<InviteToken>>;
    async fn get_setting(&self, key: &str) -> Result<Option<String>>;
    async fn set_setting(&self, key: &str, value: &str) -> Result<()>;
    async fn count_members_by_ring(&self, ring_id: &str) -> Result<i64>;
    async fn create_conversation(
        &self,
        ring_id: &str,
        title: Option<String>,
        context_mode: &str,
        created_by: &str,
    ) -> Result<Conversation>;
    async fn list_conversations(&self, ring_id: &str) -> Result<Vec<Conversation>>;
    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>>;
    async fn create_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        sender_id: Option<&str>,
    ) -> Result<Message>;
    async fn get_messages(
        &self,
        conversation_id: &str,
        limit: i64,
        before_id: Option<&str>,
    ) -> Result<Vec<Message>>;
    async fn update_ring_status(&self, id: &str, status: &str) -> Result<()>;
    async fn list_blueprint_templates(&self) -> Result<Vec<BlueprintTemplate>>;
    async fn create_blueprint_template(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
        graphs_json: &str,
        is_system: bool,
    ) -> Result<BlueprintTemplate>;
    async fn index_node_search(
        &self,
        node_id: &str,
        graph_id: &str,
        label: &str,
        content: &str,
    ) -> Result<()>;
    async fn delete_node_search(&self, node_id: &str) -> Result<()>;
    async fn search_nodes_fts(
        &self,
        query: &str,
        graph_ids: Option<Vec<String>>,
        limit: i64,
    ) -> Result<Vec<SearchResult>>;
    async fn create_archive_record(
        &self,
        id: &str,
        ring_id: &str,
        node_id: Option<&str>,
        conversation_id: Option<&str>,
        message_ids: &str,
        markdown_path: &str,
        archived_by: &str,
        git_commit_sha: Option<&str>,
        pr_status: Option<&str>,
        pr_url: Option<&str>,
    ) -> Result<()>;
    async fn list_archive_records_by_ring(&self, ring_id: &str) -> Result<Vec<ArchiveRecord>>;
    async fn get_archive_record(&self, id: &str) -> Result<Option<ArchiveRecord>>;
    async fn update_archive_pr_status(&self, id: &str, pr_status: &str) -> Result<()>;
    async fn create_member(&self, new_member: NewMember) -> Result<Member>;
    async fn get_member(&self, id: &str) -> Result<Option<Member>>;
    async fn list_members_by_ring(&self, ring_id: &str) -> Result<Vec<Member>>;
    async fn get_member_by_user_and_ring(
        &self,
        user_id: &str,
        ring_id: &str,
    ) -> Result<Option<Member>>;
    async fn update_member_role(&self, id: &str, role: &str) -> Result<()>;
    async fn delete_member(&self, id: &str) -> Result<()>;
    async fn get_next_token_id(&self, ring_id: &str) -> Result<i64>;
    async fn create_notification(&self, n: NewNotification) -> Result<Notification>;
    async fn list_notifications_by_user(
        &self,
        user_id: &str,
        unread_only: bool,
    ) -> Result<Vec<Notification>>;
    async fn mark_notification_read(&self, id: &str) -> Result<()>;
    async fn create_session(
        &self,
        ring_id: &str,
        title: Option<&str>,
        scenario: &str,
        created_by: &str,
        archive_enabled: bool,
    ) -> Result<Session>;
    async fn get_session(&self, id: &str) -> Result<Option<Session>>;
    async fn list_sessions_by_ring(
        &self,
        ring_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<Session>>;
    async fn update_session_status(&self, id: &str, status: &str) -> Result<()>;
    async fn update_session_archive(&self, id: &str, enabled: bool) -> Result<()>;
    async fn delete_session(&self, id: &str) -> Result<()>;
    async fn create_session_member(
        &self,
        session_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<SessionMember>;
    async fn list_session_members(&self, session_id: &str) -> Result<Vec<SessionMember>>;
    async fn leave_session_member(&self, session_id: &str, user_id: &str) -> Result<()>;
    async fn create_session_message(
        &self,
        session_id: &str,
        sender_id: &str,
        role: &str,
        content: &str,
        seq_num: i64,
    ) -> Result<SessionMessage>;
    async fn get_session_messages(
        &self,
        session_id: &str,
        after_seq: Option<i64>,
        limit: i64,
    ) -> Result<Vec<SessionMessage>>;
}
