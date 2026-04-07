use std::sync::Arc;

use crate::db::traits::Repository;
use crate::error::{Result, RingError};

pub struct PermissionService {
    db: Arc<dyn Repository>,
}

impl PermissionService {
    pub fn new(db: Arc<dyn Repository>) -> Self {
        PermissionService { db }
    }

    pub async fn check_ring_access(&self, ring_id: &str, user_id: &str) -> Result<()> {
        let ring = self.db.get_ring(ring_id).await?;
        if let Some(r) = &ring {
            if r.creator_id == user_id {
                return Ok(());
            }
        }
        let member = self
            .db
            .get_member_by_user_and_ring(user_id, ring_id)
            .await?;
        if member.is_none() {
            return Err(RingError::Forbidden("not a member of this ring".into()));
        }
        Ok(())
    }

    pub async fn check_creator_or_admin(&self, ring_id: &str, user_id: &str) -> Result<()> {
        let ring = self.db.get_ring(ring_id).await?;
        if let Some(r) = &ring {
            if r.creator_id == user_id {
                return Ok(());
            }
        }
        let member = self
            .db
            .get_member_by_user_and_ring(user_id, ring_id)
            .await?;
        match member {
            Some(m) if m.role == "admin" => Ok(()),
            Some(_) => Err(RingError::Forbidden("creator or admin required".into())),
            None => Err(RingError::Forbidden("not a member of this ring".into())),
        }
    }

    pub async fn check_creator(&self, ring_id: &str, user_id: &str) -> Result<()> {
        let ring = self.db.get_ring(ring_id).await?;
        if let Some(r) = &ring {
            if r.creator_id == user_id {
                return Ok(());
            }
        }
        let member = self
            .db
            .get_member_by_user_and_ring(user_id, ring_id)
            .await?;
        match member {
            Some(m) if m.role == "creator" => Ok(()),
            Some(_) => Err(RingError::Forbidden("creator required".into())),
            None => Err(RingError::Forbidden("not a member of this ring".into())),
        }
    }

    pub async fn get_member_role(&self, ring_id: &str, user_id: &str) -> Result<Option<String>> {
        let member = self
            .db
            .get_member_by_user_and_ring(user_id, ring_id)
            .await?;
        Ok(member.map(|m| m.role))
    }

    pub async fn is_creator(&self, ring_id: &str, user_id: &str) -> Result<bool> {
        Ok(self.get_member_role(ring_id, user_id).await? == Some("creator".into()))
    }

    pub async fn is_member(&self, ring_id: &str, user_id: &str) -> Result<bool> {
        Ok(self
            .db
            .get_member_by_user_and_ring(user_id, ring_id)
            .await?
            .is_some())
    }
}
