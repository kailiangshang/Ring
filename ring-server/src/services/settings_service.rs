use std::sync::Arc;

use crate::db::traits::Repository;
use crate::error::Result;

pub struct SettingsService {
    repo: Arc<dyn Repository>,
}

impl SettingsService {
    pub fn new(repo: Arc<dyn Repository>) -> Self {
        SettingsService { repo }
    }

    pub async fn get_all_settings(&self) -> Result<serde_json::Value> {
        let mut settings = serde_json::Map::new();
        for key in &[
            "llm_provider",
            "llm_model",
            "llm_api_key",
            "llm_base_url",
            "privacy_enabled",
        ] {
            if let Some(val) = self.repo.get_setting(key).await? {
                settings.insert(key.to_string(), serde_json::Value::String(val));
            }
        }
        Ok(serde_json::Value::Object(settings))
    }

    pub async fn update_settings(&self, settings: serde_json::Value) -> Result<()> {
        if let Some(obj) = settings.as_object() {
            for (key, value) in obj {
                let val_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                self.repo.set_setting(key, &val_str).await?;
            }
        }
        Ok(())
    }
}
