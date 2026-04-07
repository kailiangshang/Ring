use crate::error::{Result, RingError};

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    pub async fn get_setting_inner(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(row.map(|(v,)| v))
    }

    pub async fn set_setting_inner(&self, key: &str, value: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }
}
