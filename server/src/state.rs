use std::path::PathBuf;

use sqlx::SqlitePool;

use crate::services::encryption::CredentialEncryption;
use crate::ws_hub::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub ws_hub: WsHub,
    pub rings_dir: PathBuf,
    pub hub_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub encryption: CredentialEncryption,
}

impl AppState {
    pub fn new(db: SqlitePool, rings_dir: PathBuf, hub_dir: PathBuf, skills_dir: PathBuf) -> Self {
        let data_dir = hub_dir.parent().unwrap_or(&hub_dir).to_path_buf();
        let encryption = CredentialEncryption::new(&data_dir);
        Self {
            db,
            ws_hub: WsHub::new(),
            rings_dir,
            hub_dir,
            skills_dir,
            encryption,
        }
    }

    pub async fn get_user_decrypted(&self, token_id: &str) -> crate::error::Result<crate::models::user::UserRow> {
        let mut user = crate::models::user::get_user(&self.db, token_id).await?;

        if let Some(ref encrypted) = user.llm_api_key {
            if let Some(decrypted) = self.encryption.decrypt(encrypted) {
                user.llm_api_key = Some(decrypted);
            }
        }

        if let Some(ref encrypted) = user.gitlab_token {
            if let Some(decrypted) = self.encryption.decrypt(encrypted) {
                user.gitlab_token = Some(decrypted);
            }
        }

        Ok(user)
    }
}
