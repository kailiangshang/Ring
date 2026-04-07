use crate::error::{Result, RingError};
use crate::models::git_model::ArchiveRecord;

use crate::db::sqlite::SqliteRepository;

impl SqliteRepository {
    #[allow(clippy::too_many_arguments)]
    pub async fn create_archive_record_inner(
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
        sqlx::query(
            "INSERT INTO archive_records (id, ring_id, node_id, conversation_id, message_ids, markdown_path, archived_by, git_commit_sha, pr_status, pr_url) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(ring_id)
        .bind(node_id)
        .bind(conversation_id)
        .bind(message_ids)
        .bind(markdown_path)
        .bind(archived_by)
        .bind(git_commit_sha)
        .bind(pr_status)
        .bind(pr_url)
        .execute(self.pool())
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }

    pub async fn list_archive_records_by_ring_inner(
        &self,
        ring_id: &str,
    ) -> Result<Vec<ArchiveRecord>> {
        let rows = sqlx::query_as::<_, ArchiveRecord>(
            "SELECT id, ring_id, node_id, conversation_id, message_ids, markdown_path, archived_by, git_commit_sha, pr_status, pr_url, created_at FROM archive_records WHERE ring_id = ? ORDER BY created_at DESC",
        )
        .bind(ring_id)
        .fetch_all(self.pool())
        .await
        .map_err(RingError::Database)?;
        Ok(rows)
    }

    pub async fn get_archive_record_inner(&self, id: &str) -> Result<Option<ArchiveRecord>> {
        let row = sqlx::query_as::<_, ArchiveRecord>(
            "SELECT id, ring_id, node_id, conversation_id, message_ids, markdown_path, archived_by, git_commit_sha, pr_status, pr_url, created_at FROM archive_records WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(RingError::Database)?;
        Ok(row)
    }

    pub async fn update_archive_pr_status_inner(&self, id: &str, pr_status: &str) -> Result<()> {
        sqlx::query("UPDATE archive_records SET pr_status = ? WHERE id = ?")
            .bind(pr_status)
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }
}
