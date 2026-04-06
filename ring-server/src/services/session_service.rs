use std::sync::Arc;

use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::models::session_model::*;
use crate::services::permission_service::PermissionService;

pub struct SessionService {
    db: Arc<dyn Repository>,
    permission: PermissionService,
}

impl SessionService {
    pub fn new(db: Arc<dyn Repository>) -> Self {
        let permission = PermissionService::new(db.clone());
        SessionService { db, permission }
    }

    pub async fn create_session(
        &self,
        ring_id: &str,
        req: &CreateSessionRequest,
        user_id: &str,
    ) -> Result<SessionDetailResponse> {
        self.permission.check_ring_access(ring_id, user_id).await?;

        let active = self
            .db
            .list_sessions_by_ring(ring_id, Some("active"))
            .await?;
        if !active.is_empty() {
            return Err(RingError::Conflict(
                "an active session already exists for this ring".into(),
            ));
        }

        let session = self
            .db
            .create_session(
                ring_id,
                req.title.as_deref(),
                &req.scenario,
                user_id,
                req.archive_enabled.unwrap_or(false),
            )
            .await?;

        let owner = self
            .db
            .create_session_member(&session.id, user_id, "owner")
            .await?;

        let mut members = vec![SessionMemberBrief {
            user_id: owner.user_id,
            role: owner.role,
            status: owner.status,
        }];

        if let Some(ref invite_ids) = req.invite_member_ids {
            for mid in invite_ids {
                if mid == user_id {
                    continue;
                }
                let sm = self
                    .db
                    .create_session_member(&session.id, mid, "participant")
                    .await?;
                members.push(SessionMemberBrief {
                    user_id: sm.user_id,
                    role: sm.role,
                    status: sm.status,
                });
            }
        }

        Ok(SessionDetailResponse {
            id: session.id,
            ring_id: session.ring_id,
            title: session.title,
            scenario: session.scenario,
            created_by: session.created_by,
            archive_enabled: session.archive_enabled,
            status: session.status,
            members,
            created_at: session.created_at,
        })
    }

    pub async fn list_sessions(
        &self,
        ring_id: &str,
        status: Option<&str>,
    ) -> Result<SessionListResponse> {
        let sessions = self.db.list_sessions_by_ring(ring_id, status).await?;
        let mut items = Vec::new();
        for s in &sessions {
            let members = self.db.list_session_members(&s.id).await?;
            items.push(SessionListItem {
                id: s.id.clone(),
                title: s.title.clone(),
                created_by: s.created_by.clone(),
                member_count: members.len() as i64,
                archive_enabled: s.archive_enabled,
                status: s.status.clone(),
                created_at: s.created_at.clone(),
            });
        }
        Ok(SessionListResponse { sessions: items })
    }

    pub async fn get_session_detail(
        &self,
        ring_id: &str,
        session_id: &str,
    ) -> Result<SessionDetailResponse> {
        let session = self
            .db
            .get_session(session_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
        if session.ring_id != ring_id {
            return Err(RingError::NotFound(format!(
                "session not in ring {}",
                ring_id
            )));
        }
        let members = self.db.list_session_members(session_id).await?;
        let briefs: Vec<SessionMemberBrief> = members
            .into_iter()
            .map(|m| SessionMemberBrief {
                user_id: m.user_id,
                role: m.role,
                status: m.status,
            })
            .collect();

        Ok(SessionDetailResponse {
            id: session.id,
            ring_id: session.ring_id,
            title: session.title,
            scenario: session.scenario,
            created_by: session.created_by,
            archive_enabled: session.archive_enabled,
            status: session.status,
            members: briefs,
            created_at: session.created_at,
        })
    }

    pub async fn invite_member(
        &self,
        ring_id: &str,
        session_id: &str,
        member_ids: &[String],
        caller_id: &str,
    ) -> Result<Vec<SessionMemberBrief>> {
        let session = self
            .db
            .get_session(session_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
        if session.ring_id != ring_id {
            return Err(RingError::NotFound(format!(
                "session not in ring {}",
                ring_id
            )));
        }
        if session.created_by != caller_id {
            return Err(RingError::Forbidden("only session owner can invite".into()));
        }
        if session.status != "active" {
            return Err(RingError::Validation("session is not active".into()));
        }

        let mut result = Vec::new();
        for mid in member_ids {
            self.permission.check_ring_access(ring_id, mid).await?;
            let existing = self.db.list_session_members(session_id).await?;
            if existing
                .iter()
                .any(|m| m.user_id == *mid && m.status == "active")
            {
                continue;
            }
            let sm = self
                .db
                .create_session_member(session_id, mid, "participant")
                .await?;
            result.push(SessionMemberBrief {
                user_id: sm.user_id,
                role: sm.role,
                status: sm.status,
            });
        }
        Ok(result)
    }

    pub async fn close_session(
        &self,
        ring_id: &str,
        session_id: &str,
        caller_id: &str,
    ) -> Result<()> {
        let session = self
            .db
            .get_session(session_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
        if session.ring_id != ring_id {
            return Err(RingError::NotFound(format!(
                "session not in ring {}",
                ring_id
            )));
        }
        if session.created_by != caller_id {
            return Err(RingError::Forbidden("only session owner can close".into()));
        }
        self.db.update_session_status(session_id, "closed").await
    }

    pub async fn leave_session(
        &self,
        ring_id: &str,
        session_id: &str,
        user_id: &str,
    ) -> Result<()> {
        let session = self
            .db
            .get_session(session_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
        if session.ring_id != ring_id {
            return Err(RingError::NotFound(format!(
                "session not in ring {}",
                ring_id
            )));
        }
        if session.created_by == user_id {
            return Err(RingError::Validation(
                "owner cannot leave, use close instead".into(),
            ));
        }
        self.db.leave_session_member(session_id, user_id).await
    }

    pub async fn toggle_archive(
        &self,
        ring_id: &str,
        session_id: &str,
        enabled: bool,
        caller_id: &str,
    ) -> Result<()> {
        let session = self
            .db
            .get_session(session_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
        if session.ring_id != ring_id {
            return Err(RingError::NotFound(format!(
                "session not in ring {}",
                ring_id
            )));
        }
        if session.created_by != caller_id {
            return Err(RingError::Forbidden(
                "only session owner can toggle archive".into(),
            ));
        }
        self.db.update_session_archive(session_id, enabled).await
    }

    pub async fn delete_session(
        &self,
        ring_id: &str,
        session_id: &str,
        caller_id: &str,
    ) -> Result<()> {
        let session = self
            .db
            .get_session(session_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
        if session.ring_id != ring_id {
            return Err(RingError::NotFound(format!(
                "session not in ring {}",
                ring_id
            )));
        }
        if session.created_by != caller_id {
            return Err(RingError::Forbidden("only session owner can delete".into()));
        }
        self.db.delete_session(session_id).await
    }

    pub async fn get_messages(
        &self,
        ring_id: &str,
        session_id: &str,
        after_seq: Option<i64>,
        limit: i64,
    ) -> Result<SessionMessagesResponse> {
        let session = self
            .db
            .get_session(session_id)
            .await?
            .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
        if session.ring_id != ring_id {
            return Err(RingError::NotFound(format!(
                "session not in ring {}",
                ring_id
            )));
        }
        let messages = self
            .db
            .get_session_messages(session_id, after_seq, limit)
            .await?;
        Ok(SessionMessagesResponse { messages })
    }
}
