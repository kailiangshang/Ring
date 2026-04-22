use std::collections::HashSet;

use serde::Serialize;

use crate::error::{Result, RingError};
use crate::models::ring;
use crate::models::session::{
    self, CreateSessionInput, InviteParticipantsInput, SessionParticipantRow, SessionRow,
};
use crate::models::user;
use crate::services::llm::{LlmClient, SseEvent};
use crate::services::privacy_filter::{apply_filters, PrivacyFilters};
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
    let sess = session::create_session(&state.db, &id, ring_id, user_id, &input).await?;
    let participants = session::get_participants(&state.db, &id).await?;

    let participant_ids: HashSet<String> =
        participants.iter().map(|p| p.token_id.clone()).collect();
    state
        .ws_hub
        .register_session(id.clone(), user_id.to_string(), participant_ids);

    if input.skill != "discussion" {
        let state_c = state.clone();
        let session_id = id.clone();
        let ring_id_c = ring_id.to_string();
        let skill = input.skill.clone();
        let title = input.title.clone();
        let description = input.description.clone();
        let user_id_c = user_id.to_string();
        tokio::spawn(async move {
            if let Ok(user_row) = state_c.get_user_decrypted(&user_id_c).await {
                let _ = crate::services::material_prep::generate_materials(
                    &state_c, &session_id, &ring_id_c, &skill, &title, &description, &user_row,
                ).await;
            }
        });
    }

    Ok(SessionResponse {
        session: sess,
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

    let interaction_mode: String =
        sqlx::query_scalar("SELECT interaction_mode FROM rings WHERE id = ?1")
            .bind(ring_id)
            .fetch_one(&state.db)
            .await
            .unwrap_or_else(|_| "normal".to_string());

    if interaction_mode == "auto" && session.archive_enabled {
        let pool = state.db.clone();
        let rings_dir = state.rings_dir.clone();
        let ring_id = ring_id.to_string();
        let session_id = session_id.to_string();
        let session_title = session.title.clone();
        let session_skill = session.skill.clone();
        let creator_id = session.owner.clone();

        tokio::spawn(async move {
            let creator_user = match crate::models::user::get_user(&pool, &creator_id).await {
                Ok(u) => u,
                Err(e) => {
                    tracing::warn!("auto_archive: failed to get creator user: {e}");
                    return;
                }
            };

            let git = crate::services::git_service::GitService::new();
            crate::services::archive_service::auto_archive_session(
                &pool,
                &git,
                &rings_dir,
                &ring_id,
                &session_id,
                &session_title,
                &session_skill,
                &creator_user,
            )
            .await;
        });
    }

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
    let sess = session::update_phase(&state.db, session_id, "discussion").await?;
    let participants = session::get_participants(&state.db, session_id).await?;

    let participant_ids: HashSet<String> =
        participants.iter().map(|p| p.token_id.clone()).collect();
    state
        .ws_hub
        .register_session(session_id.to_string(), sess.owner.clone(), participant_ids);

    Ok(SessionResponse {
        session: sess,
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
    session::delete_session(&state.db, session_id).await?;
    state.ws_hub.remove_session(session_id);
    Ok(())
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
    let result = session::add_participants(&state.db, session_id, &input.token_ids).await?;
    for tid in &input.token_ids {
        state
            .ws_hub
            .add_session_participant(session_id, tid.clone());
    }
    Ok(result)
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
    session::remove_participant(&state.db, session_id, target_id).await?;
    state
        .ws_hub
        .remove_session_participant(session_id, target_id);
    Ok(())
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

pub async fn start_session(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
) -> Result<SessionResponse> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("only owner can start session".into()));
    }
    let sess = session::get_session(&state.db, session_id).await?;
    if sess.phase != "material_prep" {
        return Err(RingError::BadRequest(
            "session is not in material_prep phase".into(),
        ));
    }
    let sess = session::update_phase(&state.db, session_id, "discussion").await?;
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse {
        session: sess,
        participants,
    })
}

pub async fn get_materials_service(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
) -> Result<Vec<session::SessionMaterialRow>> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_participant(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("not a session participant".into()));
    }
    session::get_materials(&state.db, session_id).await
}

pub async fn highlight_material(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
    material_id: &str,
    note: &str,
) -> Result<session::SessionMaterialRow> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden(
            "only owner can highlight materials".into(),
        ));
    }
    session::update_material_highlight(&state.db, material_id, note).await
}

pub struct SummarizeContext {
    pub session_id: String,
    pub skill: String,
    pub messages_text: String,
}

pub fn start_summarize_stream(
    _state: &AppState,
    user_row: &user::UserRow,
    ctx: SummarizeContext,
) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
    let system_prompt = crate::services::skill::build_summary_system_prompt(&ctx.skill)
        .unwrap_or_else(|| "Summarize the following discussion.".to_string());

    let filters = user_row
        .privacy_filters
        .as_deref()
        .map(PrivacyFilters::from_json)
        .unwrap_or_default();
    let filtered_messages = apply_filters(&ctx.messages_text, &filters);

    let user_message = format!(
        "Here is the discussion transcript:\n\n{}\n\nPlease generate the summary.",
        filtered_messages
    );

    let llm = LlmClient::from_user(user_row)?;
    let rx = llm.chat_stream(
        system_prompt,
        vec![],
        user_message,
        "session_ring".to_string(),
    );
    Ok(rx)
}
