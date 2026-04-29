use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::services::cross_ring_cache::CrossRingCache;
use crate::services::encryption::CredentialEncryption;
use crate::ws_hub::WsHub;

pub type DwellBuffer = Arc<Mutex<HashMap<String, HashMap<String, u64>>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub ws_hub: WsHub,
    pub rings_dir: PathBuf,
    pub hub_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub encryption: CredentialEncryption,
    pub dwell_buffer: DwellBuffer,
    pub cross_ring_cache: CrossRingCache,
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
            dwell_buffer: Arc::new(Mutex::new(HashMap::new())),
            cross_ring_cache: crate::services::cross_ring_cache::new_cache(),
        }
    }

    pub async fn get_user_decrypted(
        &self,
        token_id: &str,
    ) -> crate::error::Result<crate::models::user::UserRow> {
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
