use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::{Result, RingError};

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct SessionRow {
    pub id: String,
    pub ring_id: String,
    pub title: String,
    pub description: String,
    pub skill: String,
    pub phase: String,
    pub owner: String,
    pub archivable: bool,
    pub archive_enabled: bool,
    pub summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct SessionParticipantRow {
    pub session_id: String,
    pub token_id: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct SessionMessageRow {
    pub id: String,
    pub session_id: String,
    pub seq_num: i64,
    pub sender: String,
    pub sender_name: String,
    pub content: String,
    pub message_type: String,
    pub created_at: String,
}

#[derive(Debug, FromRow, Serialize)]
pub struct SessionMaterialRow {
    pub id: String,
    pub session_id: String,
    pub item_type: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub highlight: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionInput {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_skill")]
    pub skill: String,
    #[serde(default)]
    pub archivable: bool,
    #[serde(default)]
    pub invitees: Vec<String>,
}

fn default_skill() -> String {
    "discussion".into()
}

#[derive(Debug, Deserialize)]
pub struct InviteParticipantsInput {
    pub token_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ArchiveToggleInput {
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct MaterialHighlightInput {
    pub item_index: usize,
    pub note: String,
}

pub async fn has_active_session(pool: &sqlx::SqlitePool, ring_id: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE ring_id = ?1 AND phase != 'closed'",
    )
    .bind(ring_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn create_session(
    pool: &sqlx::SqlitePool,
    id: &str,
    ring_id: &str,
    owner: &str,
    input: &CreateSessionInput,
) -> Result<SessionRow> {
    let phase = if input.skill == "discussion" {
        "discussion"
    } else {
        "material_prep"
    };

    let session = sqlx::query_as::<_, SessionRow>(
        "INSERT INTO sessions (id, ring_id, title, description, skill, phase, owner, archivable)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         RETURNING *",
    )
    .bind(id)
    .bind(ring_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(&input.skill)
    .bind(phase)
    .bind(owner)
    .bind(input.archivable)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        "INSERT INTO session_participants (session_id, token_id, role) VALUES (?1, ?2, 'owner')",
    )
    .bind(id)
    .bind(owner)
    .execute(pool)
    .await?;

    for invitee in &input.invitees {
        sqlx::query(
            "INSERT OR IGNORE INTO session_participants (session_id, token_id, role) VALUES (?1, ?2, 'participant')",
        )
        .bind(id)
        .bind(invitee)
        .execute(pool)
        .await?;
    }

    Ok(session)
}

pub async fn list_sessions(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    status: Option<&str>,
) -> Result<Vec<SessionRow>> {
    let sessions = match status {
        Some("active") => {
            sqlx::query_as::<_, SessionRow>(
                "SELECT * FROM sessions WHERE ring_id = ?1 AND phase != 'closed' ORDER BY created_at DESC",
            )
            .bind(ring_id)
            .fetch_all(pool)
            .await?
        }
        Some("closed") => {
            sqlx::query_as::<_, SessionRow>(
                "SELECT * FROM sessions WHERE ring_id = ?1 AND phase = 'closed' ORDER BY created_at DESC",
            )
            .bind(ring_id)
            .fetch_all(pool)
            .await?
        }
        _ => {
            sqlx::query_as::<_, SessionRow>(
                "SELECT * FROM sessions WHERE ring_id = ?1 ORDER BY created_at DESC",
            )
            .bind(ring_id)
            .fetch_all(pool)
            .await?
        }
    };
    Ok(sessions)
}

pub async fn get_session(pool: &sqlx::SqlitePool, session_id: &str) -> Result<SessionRow> {
    sqlx::query_as::<_, SessionRow>("SELECT * FROM sessions WHERE id = ?1")
        .bind(session_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound(format!("session {session_id} not found")))
}

pub async fn get_participants(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<Vec<SessionParticipantRow>> {
    let rows = sqlx::query_as::<_, SessionParticipantRow>(
        "SELECT * FROM session_participants WHERE session_id = ?1 ORDER BY joined_at",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn is_participant(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    token_id: &str,
) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM session_participants WHERE session_id = ?1 AND token_id = ?2",
    )
    .bind(session_id)
    .bind(token_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn is_owner(pool: &sqlx::SqlitePool, session_id: &str, token_id: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM session_participants WHERE session_id = ?1 AND token_id = ?2 AND role = 'owner'",
    )
    .bind(session_id)
    .bind(token_id)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn update_phase(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    new_phase: &str,
) -> Result<SessionRow> {
    sqlx::query_as::<_, SessionRow>(
        "UPDATE sessions SET phase = ?1, updated_at = datetime('now') WHERE id = ?2 RETURNING *",
    )
    .bind(new_phase)
    .bind(session_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound(format!("session {session_id} not found")))
}

pub async fn delete_session(pool: &sqlx::SqlitePool, session_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = ?1 AND phase = 'closed'")
        .bind(session_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(RingError::BadRequest(
            "session not found or not closed".into(),
        ));
    }
    Ok(())
}

pub async fn add_participants(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    token_ids: &[String],
) -> Result<Vec<SessionParticipantRow>> {
    for tid in token_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO session_participants (session_id, token_id, role) VALUES (?1, ?2, 'participant')",
        )
        .bind(session_id)
        .bind(tid)
        .execute(pool)
        .await?;
    }
    get_participants(pool, session_id).await
}

pub async fn remove_participant(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    token_id: &str,
) -> Result<()> {
    let result = sqlx::query(
        "DELETE FROM session_participants WHERE session_id = ?1 AND token_id = ?2 AND role != 'owner'",
    )
    .bind(session_id)
    .bind(token_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(RingError::NotFound("participant not found".into()));
    }
    Ok(())
}

pub async fn toggle_archive(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    enabled: bool,
) -> Result<SessionRow> {
    sqlx::query_as::<_, SessionRow>(
        "UPDATE sessions SET archive_enabled = ?1, updated_at = datetime('now') WHERE id = ?2 RETURNING *",
    )
    .bind(enabled)
    .bind(session_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound(format!("session {session_id} not found")))
}

pub async fn get_messages(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    after_seq: i64,
    limit: i64,
) -> Result<Vec<SessionMessageRow>> {
    let rows = sqlx::query_as::<_, SessionMessageRow>(
        "SELECT * FROM session_messages WHERE session_id = ?1 AND seq_num > ?2 ORDER BY seq_num ASC LIMIT ?3",
    )
    .bind(session_id)
    .bind(after_seq)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
