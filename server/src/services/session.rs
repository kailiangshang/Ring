use serde::Serialize;

use crate::error::{Result, RingError};
use crate::models::ring;
use crate::models::session::{
    self, CreateSessionInput, InviteParticipantsInput, SessionParticipantRow, SessionRow,
};
use crate::state::AppState;

const VALID_SKILLS: &[&str] = &[
    "decision",
    "research",
    "review",
    "retrospective",
    "knowledge_sharing",
    "discussion",
];

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    #[serde(flatten)]
    pub session: SessionRow,
    pub participants: Vec<SessionParticipantRow>,
}

async fn is_member(pool: &sqlx::SqlitePool, ring_id: &str, user_id: &str) -> Result<bool> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM members WHERE ring_id = ?1 AND user_id = ?2")
            .bind(ring_id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;
    Ok(count > 0)
}

pub async fn create_session(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    input: CreateSessionInput,
) -> Result<SessionResponse> {
    let role = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if role != "creator" && role != "admin" {
        let has_grant: bool = sqlx::query_scalar::<_, bool>(
            "SELECT session_grant FROM members WHERE ring_id = ?1 AND user_id = ?2",
        )
        .bind(ring_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(false);
        if !has_grant {
            return Err(RingError::Forbidden(
                "not authorized to create sessions".into(),
            ));
        }
    }

    if session::has_active_session(&state.db, ring_id).await? {
        return Err(RingError::Conflict("active session already exists".into()));
    }

    if !VALID_SKILLS.contains(&input.skill.as_str()) {
        return Err(RingError::BadRequest(format!(
            "invalid skill: {}",
            input.skill
        )));
    }

    for invitee in &input.invitees {
        let member = is_member(&state.db, ring_id, invitee).await?;
        if !member {
            return Err(RingError::BadRequest(format!(
                "invitee {invitee} is not a ring member"
            )));
        }
    }

    let id = ulid::Ulid::new().to_string();
    let session = session::create_session(&state.db, &id, ring_id, user_id, &input).await?;
    let participants = session::get_participants(&state.db, &id).await?;

    Ok(SessionResponse {
        session,
        participants,
    })
}

pub async fn list_sessions(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    status: Option<&str>,
) -> Result<Vec<SessionRow>> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    session::list_sessions(&state.db, ring_id, status).await
}

pub async fn get_session_detail(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
) -> Result<SessionResponse> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    let session = session::get_session(&state.db, session_id).await?;
    if session.ring_id != ring_id {
        return Err(RingError::NotFound("session not found".into()));
    }
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse {
        session,
        participants,
    })
}

pub async fn close_session(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
) -> Result<SessionResponse> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("only owner can close session".into()));
    }
    let session = session::get_session(&state.db, session_id).await?;
    if session.phase == "closed" {
        return Err(RingError::BadRequest("session already closed".into()));
    }
    let session = session::update_phase(&state.db, session_id, "closed").await?;
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse {
        session,
        participants,
    })
}

pub async fn reopen_session(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
) -> Result<SessionResponse> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("only owner can reopen session".into()));
    }
    let session = session::get_session(&state.db, session_id).await?;
    if session.phase != "closed" {
        return Err(RingError::BadRequest("session is not closed".into()));
    }
    if session::has_active_session(&state.db, ring_id).await? {
        return Err(RingError::Conflict("active session already exists".into()));
    }
    let session = session::update_phase(&state.db, session_id, "discussion").await?;
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse {
        session,
        participants,
    })
}

pub async fn delete_session(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
) -> Result<()> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("only owner can delete session".into()));
    }
    session::delete_session(&state.db, session_id).await
}

pub async fn invite_participants(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
    input: InviteParticipantsInput,
) -> Result<Vec<SessionParticipantRow>> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden(
            "only owner can invite participants".into(),
        ));
    }
    for tid in &input.token_ids {
        let member = is_member(&state.db, ring_id, tid).await?;
        if !member {
            return Err(RingError::BadRequest(format!(
                "invitee {tid} is not a ring member"
            )));
        }
    }
    session::add_participants(&state.db, session_id, &input.token_ids).await
}

pub async fn remove_participant(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    target_id: &str,
    user_id: &str,
) -> Result<()> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden(
            "only owner can remove participants".into(),
        ));
    }
    session::remove_participant(&state.db, session_id, target_id).await
}

pub async fn toggle_archive(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
    enabled: bool,
) -> Result<SessionResponse> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("only owner can toggle archive".into()));
    }
    let session = session::get_session(&state.db, session_id).await?;
    if !session.archivable {
        return Err(RingError::BadRequest("session is not archivable".into()));
    }
    let session = session::toggle_archive(&state.db, session_id, enabled).await?;
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse {
        session,
        participants,
    })
}

pub async fn get_messages(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
    after_seq: i64,
    limit: i64,
) -> Result<Vec<session::SessionMessageRow>> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_participant(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("not a session participant".into()));
    }
    session::get_messages(&state.db, session_id, after_seq, limit).await
}
