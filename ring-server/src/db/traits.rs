use crate::error::Result;
use crate::models::blueprint::BlueprintTemplate;
use crate::models::conversation::{Conversation, Message};
use crate::models::invite::InviteToken;
use crate::models::ring::{NewRing, Ring};
use crate::models::user::{NewUser, User};

#[async_trait::async_trait]
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
    async fn list_blueprint_templates(&self) -> Result<Vec<BlueprintTemplate>>;
    async fn create_blueprint_template(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
        graphs_json: &str,
        is_system: bool,
    ) -> Result<BlueprintTemplate>;
}
