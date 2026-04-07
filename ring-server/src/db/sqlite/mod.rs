mod archive_repo;
mod blueprint_repo;
mod conversation_repo;
mod member_repo;
mod notification_repo;
mod ring_repo;
mod search_repo;
mod session_repo;
mod settings_repo;
#[cfg(test)]
mod tests;
mod user_repo;

use crate::db::traits::Repository;
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
use sqlx::SqlitePool;

pub struct SqliteRepository {
    pool: SqlitePool,
    jieba: std::sync::Mutex<Option<jieba_rs::Jieba>>,
}

impl SqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        SqliteRepository {
            pool,
            jieba: std::sync::Mutex::new(None),
        }
    }

    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub(crate) fn get_jieba(&self) -> jieba_rs::Jieba {
        let mut guard = self.jieba.lock().unwrap();
        if guard.is_none() {
            *guard = Some(jieba_rs::Jieba::new());
        }
        guard.clone().unwrap()
    }
}

#[async_trait::async_trait]
#[allow(clippy::too_many_arguments)]
impl Repository for SqliteRepository {
    async fn create_user(&self, new_user: NewUser) -> Result<User> {
        self.create_user_inner(new_user).await
    }

    async fn get_user(&self, id: &str) -> Result<Option<User>> {
        self.get_user_inner(id).await
    }

    async fn list_all_users(&self) -> Result<Vec<User>> {
        self.list_all_users_inner().await
    }

    async fn is_setup_completed(&self) -> Result<bool> {
        self.is_setup_completed_inner().await
    }

    async fn complete_setup(&self, user_id: &str) -> Result<()> {
        self.complete_setup_inner(user_id).await
    }

    async fn create_ring(&self, new_ring: NewRing) -> Result<Ring> {
        self.create_ring_inner(new_ring).await
    }

    async fn get_ring(&self, id: &str) -> Result<Option<Ring>> {
        self.get_ring_inner(id).await
    }

    async fn list_rings_by_user(&self, user_id: &str) -> Result<Vec<Ring>> {
        self.list_rings_by_user_inner(user_id).await
    }

    async fn update_ring(
        &self,
        id: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Ring> {
        self.update_ring_inner(id, name, description).await
    }

    async fn delete_ring(&self, id: &str) -> Result<()> {
        self.delete_ring_inner(id).await
    }

    async fn update_ring_status(&self, id: &str, status: &str) -> Result<()> {
        self.update_ring_status_inner(id, status).await
    }

    async fn create_member(&self, new_member: NewMember) -> Result<Member> {
        self.create_member_inner(new_member).await
    }

    async fn get_member(&self, id: &str) -> Result<Option<Member>> {
        self.get_member_inner(id).await
    }

    async fn list_members_by_ring(&self, ring_id: &str) -> Result<Vec<Member>> {
        self.list_members_by_ring_inner(ring_id).await
    }

    async fn get_member_by_user_and_ring(
        &self,
        user_id: &str,
        ring_id: &str,
    ) -> Result<Option<Member>> {
        self.get_member_by_user_and_ring_inner(user_id, ring_id)
            .await
    }

    async fn update_member_role(&self, id: &str, role: &str) -> Result<()> {
        self.update_member_role_inner(id, role).await
    }

    async fn delete_member(&self, id: &str) -> Result<()> {
        self.delete_member_inner(id).await
    }

    async fn get_next_token_id(&self, ring_id: &str) -> Result<i64> {
        self.get_next_token_id_inner(ring_id).await
    }

    async fn create_invite_token(
        &self,
        ring_id: &str,
        token: &str,
        token_type: &str,
        role: &str,
        inviter_id: &str,
    ) -> Result<InviteToken> {
        self.create_invite_token_inner(ring_id, token, token_type, role, inviter_id)
            .await
    }

    async fn get_invite_token(&self, token: &str) -> Result<Option<InviteToken>> {
        self.get_invite_token_inner(token).await
    }

    async fn count_members_by_ring(&self, ring_id: &str) -> Result<i64> {
        self.count_members_by_ring_inner(ring_id).await
    }

    async fn create_conversation(
        &self,
        ring_id: &str,
        title: Option<String>,
        context_mode: &str,
        created_by: &str,
    ) -> Result<Conversation> {
        self.create_conversation_inner(ring_id, title, context_mode, created_by)
            .await
    }

    async fn list_conversations(&self, ring_id: &str) -> Result<Vec<Conversation>> {
        self.list_conversations_inner(ring_id).await
    }

    async fn get_conversation(&self, id: &str) -> Result<Option<Conversation>> {
        self.get_conversation_inner(id).await
    }

    async fn create_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        sender_id: Option<&str>,
    ) -> Result<Message> {
        self.create_message_inner(conversation_id, role, content, sender_id)
            .await
    }

    async fn get_messages(
        &self,
        conversation_id: &str,
        limit: i64,
        before_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        self.get_messages_inner(conversation_id, limit, before_id)
            .await
    }

    async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        self.get_setting_inner(key).await
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.set_setting_inner(key, value).await
    }

    async fn list_blueprint_templates(&self) -> Result<Vec<BlueprintTemplate>> {
        self.list_blueprint_templates_inner().await
    }

    async fn create_blueprint_template(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
        graphs_json: &str,
        is_system: bool,
    ) -> Result<BlueprintTemplate> {
        self.create_blueprint_template_inner(id, name, description, graphs_json, is_system)
            .await
    }

    async fn index_node_search(
        &self,
        node_id: &str,
        graph_id: &str,
        label: &str,
        content: &str,
    ) -> Result<()> {
        self.index_node_search_inner(node_id, graph_id, label, content)
            .await
    }

    async fn delete_node_search(&self, node_id: &str) -> Result<()> {
        self.delete_node_search_inner(node_id).await
    }

    async fn search_nodes_fts(
        &self,
        query: &str,
        graph_ids: Option<Vec<String>>,
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        self.search_nodes_fts_inner(query, graph_ids, limit).await
    }

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
    ) -> Result<()> {
        self.create_archive_record_inner(
            id,
            ring_id,
            node_id,
            conversation_id,
            message_ids,
            markdown_path,
            archived_by,
            git_commit_sha,
            pr_status,
            pr_url,
        )
        .await
    }

    async fn list_archive_records_by_ring(&self, ring_id: &str) -> Result<Vec<ArchiveRecord>> {
        self.list_archive_records_by_ring_inner(ring_id).await
    }

    async fn get_archive_record(&self, id: &str) -> Result<Option<ArchiveRecord>> {
        self.get_archive_record_inner(id).await
    }

    async fn update_archive_pr_status(&self, id: &str, pr_status: &str) -> Result<()> {
        self.update_archive_pr_status_inner(id, pr_status).await
    }

    async fn create_notification(&self, n: NewNotification) -> Result<Notification> {
        self.create_notification_inner(n).await
    }

    async fn list_notifications_by_user(
        &self,
        user_id: &str,
        unread_only: bool,
    ) -> Result<Vec<Notification>> {
        self.list_notifications_by_user_inner(user_id, unread_only)
            .await
    }

    async fn mark_notification_read(&self, id: &str) -> Result<()> {
        self.mark_notification_read_inner(id).await
    }

    async fn create_session(
        &self,
        ring_id: &str,
        title: Option<&str>,
        scenario: &str,
        created_by: &str,
        archive_enabled: bool,
    ) -> Result<Session> {
        self.create_session_inner(ring_id, title, scenario, created_by, archive_enabled)
            .await
    }

    async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        self.get_session_inner(id).await
    }

    async fn list_sessions_by_ring(
        &self,
        ring_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<Session>> {
        self.list_sessions_by_ring_inner(ring_id, status).await
    }

    async fn update_session_status(&self, id: &str, status: &str) -> Result<()> {
        self.update_session_status_inner(id, status).await
    }

    async fn update_session_archive(&self, id: &str, enabled: bool) -> Result<()> {
        self.update_session_archive_inner(id, enabled).await
    }

    async fn delete_session(&self, id: &str) -> Result<()> {
        self.delete_session_inner(id).await
    }

    async fn create_session_member(
        &self,
        session_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<SessionMember> {
        self.create_session_member_inner(session_id, user_id, role)
            .await
    }

    async fn list_session_members(&self, session_id: &str) -> Result<Vec<SessionMember>> {
        self.list_session_members_inner(session_id).await
    }

    async fn leave_session_member(&self, session_id: &str, user_id: &str) -> Result<()> {
        self.leave_session_member_inner(session_id, user_id).await
    }

    async fn create_session_message(
        &self,
        session_id: &str,
        sender_id: &str,
        role: &str,
        content: &str,
        seq_num: i64,
    ) -> Result<SessionMessage> {
        self.create_session_message_inner(session_id, sender_id, role, content, seq_num)
            .await
    }

    async fn get_session_messages(
        &self,
        session_id: &str,
        after_seq: Option<i64>,
        limit: i64,
    ) -> Result<Vec<SessionMessage>> {
        self.get_session_messages_inner(session_id, after_seq, limit)
            .await
    }
}
