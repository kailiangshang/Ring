use crate::error::Result;
use crate::models::invite::InviteToken;
use crate::models::ring::{NewRing, Ring};
use crate::models::user::{NewUser, User};

#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    async fn create_user(&self, new_user: NewUser) -> Result<User>;
    async fn get_user(&self, id: &str) -> Result<Option<User>>;
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
}
