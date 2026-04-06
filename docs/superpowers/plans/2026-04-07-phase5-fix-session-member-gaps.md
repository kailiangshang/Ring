# Phase 5 Fix: Session Service, Member Handlers, and WebSocket Hub

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fill in the missing backend pieces from Phase 5 — session models/service/handler/routes, member handlers/routes, notification handlers/routes, and WebSocket hub — so the frontend can actually call these APIs.

**Architecture:** Follow the established pattern: models → DB trait/impl → service → handler → route. All business logic in services, handlers only parse params → call service → return response. WebSocket hub uses `tokio::sync::broadcast` channels stored in `AppState`.

**Tech Stack:** Rust + Axum 0.8 (with `ws` feature already enabled) + SQLite (sqlx) + tokio broadcast channels

---

## File Structure

| Action | File | Responsibility |
|--------|------|----------------|
| Create | `src/models/session_model.rs` | Session + SessionMember + SessionMessage + request/response types |
| Create | `src/services/session_service.rs` | Session CRUD, invite, close, leave, delete, archive toggle |
| Create | `src/services/notification_service.rs` | Notification listing + mark read |
| Create | `src/services/ws_hub.rs` | WebSocket hub with broadcast channels |
| Create | `src/handlers/session.rs` | Session HTTP handlers (7 endpoints) |
| Create | `src/handlers/member.rs` | Member HTTP handlers (5 endpoints) |
| Create | `src/handlers/notification.rs` | Notification HTTP handlers (2 endpoints) |
| Create | `src/handlers/ws.rs` | WebSocket upgrade handler |
| Modify | `src/models/mod.rs` | Add `pub mod session_model` |
| Modify | `src/services/mod.rs` | Add session/notification/ws_hub modules + re-exports |
| Modify | `src/handlers/mod.rs` | Add session/member/notification/ws modules |
| Modify | `src/db/traits.rs` | Add session DB methods (10 methods) |
| Modify | `src/db/sqlite.rs` | Implement session DB methods + FromRow structs |
| Modify | `src/state.rs` | Add `ws_hub: Arc<WsHub>` field |
| Modify | `src/routes.rs` | Add member/session/notification/ws route groups |
| Modify | `src/main.rs` | Create WsHub and pass to AppState |

---

## Task 1: Session Models

**Files:**
- Create: `ring-server/src/models/session_model.rs`
- Modify: `ring-server/src/models/mod.rs`

- [ ] **Step 1: Create `session_model.rs`**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub ring_id: String,
    pub title: Option<String>,
    pub scenario: String,
    pub created_by: String,
    pub archive_enabled: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMember {
    pub id: String,
    pub session_id: String,
    pub user_id: String,
    pub role: String,
    pub status: String,
    pub joined_at: String,
    pub left_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub session_id: String,
    pub sender_id: String,
    pub role: String,
    pub content: String,
    pub seq_num: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    pub scenario: String,
    pub archive_enabled: Option<bool>,
    pub invite_member_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetailResponse {
    pub id: String,
    pub ring_id: String,
    pub title: Option<String>,
    pub scenario: String,
    pub created_by: String,
    pub archive_enabled: bool,
    pub status: String,
    pub members: Vec<SessionMemberBrief>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMemberBrief {
    pub user_id: String,
    pub role: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionListItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListItem {
    pub id: String,
    pub title: Option<String>,
    pub created_by: String,
    pub member_count: i64,
    pub archive_enabled: bool,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteSessionRequest {
    pub member_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveToggleRequest {
    pub archive_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessagesResponse {
    pub messages: Vec<SessionMessage>,
}
```

- [ ] **Step 2: Register module in `mod.rs`**

Add to `ring-server/src/models/mod.rs`:

```rust
pub mod session_model;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: compiles successfully (model only, no usage yet)

---

## Task 2: Session DB Trait Methods

**Files:**
- Modify: `ring-server/src/db/traits.rs`

- [ ] **Step 1: Add session methods to Repository trait**

Append these methods to the `Repository` trait in `ring-server/src/db/traits.rs` (before the closing `}`):

```rust
    async fn create_session(
        &self,
        ring_id: &str,
        title: Option<&str>,
        scenario: &str,
        created_by: &str,
        archive_enabled: bool,
    ) -> Result<Session>;
    async fn get_session(&self, id: &str) -> Result<Option<Session>>;
    async fn list_sessions_by_ring(
        &self,
        ring_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<Session>>;
    async fn update_session_status(&self, id: &str, status: &str) -> Result<()>;
    async fn update_session_archive(&self, id: &str, enabled: bool) -> Result<()>;
    async fn delete_session(&self, id: &str) -> Result<()>;
    async fn create_session_member(
        &self,
        session_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<SessionMember>;
    async fn list_session_members(&self, session_id: &str) -> Result<Vec<SessionMember>>;
    async fn leave_session_member(&self, session_id: &str, user_id: &str) -> Result<()>;
    async fn create_session_message(
        &self,
        session_id: &str,
        sender_id: &str,
        role: &str,
        content: &str,
        seq_num: i64,
    ) -> Result<SessionMessage>;
    async fn get_session_messages(
        &self,
        session_id: &str,
        after_seq: Option<i64>,
        limit: i64,
    ) -> Result<Vec<SessionMessage>>;
```

Also add the imports at the top:

```rust
use crate::models::session_model::{Session, SessionMember, SessionMessage};
```

- [ ] **Step 2: Verify compilation fails**

Run: `cargo check`
Expected: compilation errors because `SqliteRepository` doesn't implement the new methods yet. This confirms the trait is correctly extended.

---

## Task 3: Session DB SQLite Implementation

**Files:**
- Modify: `ring-server/src/db/sqlite.rs`

- [ ] **Step 1: Add FromRow structs for session tables**

Add these structs after the existing `NotificationRow` struct (around line 1072):

```rust
#[derive(sqlx::FromRow)]
struct SessionRow {
    id: String,
    ring_id: String,
    title: Option<String>,
    scenario: String,
    created_by: String,
    archive_enabled: bool,
    status: String,
    created_at: String,
    updated_at: String,
}

#[derive(sqlx::FromRow)]
struct SessionMemberRow {
    id: String,
    session_id: String,
    user_id: String,
    role: String,
    status: String,
    joined_at: String,
    left_at: Option<String>,
}

#[derive(sqlx::FromRow)]
struct SessionMessageRow {
    id: String,
    session_id: String,
    sender_id: String,
    role: String,
    content: String,
    seq_num: i64,
    created_at: String,
}
```

- [ ] **Step 2: Implement session Repository methods on SqliteRepository**

Add these implementations inside the `impl Repository for SqliteRepository` block (before the closing `}` at line 908):

```rust
    async fn create_session(
        &self,
        ring_id: &str,
        title: Option<&str>,
        scenario: &str,
        created_by: &str,
        archive_enabled: bool,
    ) -> Result<Session> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO sessions (id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, 'active', ?, ?)",
        )
        .bind(&id)
        .bind(ring_id)
        .bind(title)
        .bind(scenario)
        .bind(created_by)
        .bind(archive_enabled)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(Session {
            id,
            ring_id: ring_id.to_string(),
            title: title.map(|s| s.to_string()),
            scenario: scenario.to_string(),
            created_by: created_by.to_string(),
            archive_enabled,
            status: "active".to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        let row = sqlx::query_as::<_, SessionRow>(
            "SELECT id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at FROM sessions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(row.map(|r| Session {
            id: r.id,
            ring_id: r.ring_id,
            title: r.title,
            scenario: r.scenario,
            created_by: r.created_by,
            archive_enabled: r.archive_enabled,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn list_sessions_by_ring(
        &self,
        ring_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<Session>> {
        let rows = if let Some(s) = status {
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at FROM sessions WHERE ring_id = ? AND status = ? ORDER BY created_at DESC",
            )
            .bind(ring_id)
            .bind(s)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, SessionRow>(
                "SELECT id, ring_id, title, scenario, created_by, archive_enabled, status, created_at, updated_at FROM sessions WHERE ring_id = ? ORDER BY created_at DESC",
            )
            .bind(ring_id)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        };

        Ok(rows
            .into_iter()
            .map(|r| Session {
                id: r.id,
                ring_id: r.ring_id,
                title: r.title,
                scenario: r.scenario,
                created_by: r.created_by,
                archive_enabled: r.archive_enabled,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    async fn update_session_status(&self, id: &str, status: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE sessions SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn update_session_archive(&self, id: &str, enabled: bool) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query("UPDATE sessions SET archive_enabled = ?, updated_at = ? WHERE id = ?")
            .bind(enabled)
            .bind(&now)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn delete_session(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM session_messages WHERE session_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        sqlx::query("DELETE FROM session_members WHERE session_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(RingError::Database)?;
        Ok(())
    }

    async fn create_session_member(
        &self,
        session_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<SessionMember> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO session_members (id, session_id, user_id, role, status, joined_at) VALUES (?, ?, ?, ?, 'active', ?)",
        )
        .bind(&id)
        .bind(session_id)
        .bind(user_id)
        .bind(role)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(SessionMember {
            id,
            session_id: session_id.to_string(),
            user_id: user_id.to_string(),
            role: role.to_string(),
            status: "active".to_string(),
            joined_at: now,
            left_at: None,
        })
    }

    async fn list_session_members(&self, session_id: &str) -> Result<Vec<SessionMember>> {
        let rows = sqlx::query_as::<_, SessionMemberRow>(
            "SELECT id, session_id, user_id, role, status, joined_at, left_at FROM session_members WHERE session_id = ? AND status = 'active'",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(rows
            .into_iter()
            .map(|r| SessionMember {
                id: r.id,
                session_id: r.session_id,
                user_id: r.user_id,
                role: r.role,
                status: r.status,
                joined_at: r.joined_at,
                left_at: r.left_at,
            })
            .collect())
    }

    async fn leave_session_member(&self, session_id: &str, user_id: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE session_members SET status = 'left', left_at = ? WHERE session_id = ? AND user_id = ?",
        )
        .bind(&now)
        .bind(session_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;
        Ok(())
    }

    async fn create_session_message(
        &self,
        session_id: &str,
        sender_id: &str,
        role: &str,
        content: &str,
        seq_num: i64,
    ) -> Result<SessionMessage> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO session_messages (id, session_id, sender_id, role, content, seq_num, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(session_id)
        .bind(sender_id)
        .bind(role)
        .bind(content)
        .bind(seq_num)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(RingError::Database)?;

        Ok(SessionMessage {
            id,
            session_id: session_id.to_string(),
            sender_id: sender_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            seq_num,
            created_at: now,
        })
    }

    async fn get_session_messages(
        &self,
        session_id: &str,
        after_seq: Option<i64>,
        limit: i64,
    ) -> Result<Vec<SessionMessage>> {
        let rows = if let Some(seq) = after_seq {
            sqlx::query_as::<_, SessionMessageRow>(
                "SELECT id, session_id, sender_id, role, content, seq_num, created_at FROM session_messages WHERE session_id = ? AND seq_num > ? ORDER BY seq_num ASC LIMIT ?",
            )
            .bind(session_id)
            .bind(seq)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        } else {
            sqlx::query_as::<_, SessionMessageRow>(
                "SELECT id, session_id, sender_id, role, content, seq_num, created_at FROM session_messages WHERE session_id = ? ORDER BY seq_num ASC LIMIT ?",
            )
            .bind(session_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(RingError::Database)?
        };

        Ok(rows
            .into_iter()
            .map(|r| SessionMessage {
                id: r.id,
                session_id: r.session_id,
                sender_id: r.sender_id,
                role: r.role,
                content: r.content,
                seq_num: r.seq_num,
                created_at: r.created_at,
            })
            .collect())
    }
```

Also add the import at the top:

```rust
use crate::models::session_model::{Session, SessionMember, SessionMessage};
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: compiles. The new trait methods are now implemented.

---

## Task 4: Notification Service

**Files:**
- Create: `ring-server/src/services/notification_service.rs`
- Modify: `ring-server/src/services/mod.rs`

- [ ] **Step 1: Create `notification_service.rs`**

```rust
use std::sync::Arc;

use crate::db::traits::Repository;
use crate::error::Result;
use crate::models::notification_model::Notification;

pub struct NotificationService {
    db: Arc<dyn Repository>,
}

impl NotificationService {
    pub fn new(db: Arc<dyn Repository>) -> Self {
        NotificationService { db }
    }

    pub async fn list_for_user(
        &self,
        user_id: &str,
        unread_only: bool,
    ) -> Result<Vec<Notification>> {
        self.db.list_notifications_by_user(user_id, unread_only).await
    }

    pub async fn mark_read(&self, notification_id: &str) -> Result<()> {
        self.db.mark_notification_read(notification_id).await
    }
}
```

- [ ] **Step 2: Register in `services/mod.rs`**

Add to `ring-server/src/services/mod.rs`:

```rust
pub mod notification_service;
```

And add the re-export:

```rust
pub use notification_service::NotificationService;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

---

## Task 5: WebSocket Hub

**Files:**
- Create: `ring-server/src/services/ws_hub.rs`
- Modify: `ring-server/src/services/mod.rs`

- [ ] **Step 1: Create `ws_hub.rs`**

```rust
use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize)]
pub struct WsMessage {
    pub msg_type: String,
    pub payload: serde_json::Value,
}

pub struct WsHub {
    channels: RwLock<HashMap<String, broadcast::Sender<WsMessage>>>,
}

impl WsHub {
    pub fn new() -> Self {
        WsHub {
            channels: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_or_create_channel(&self, ring_id: &str) -> broadcast::Receiver<WsMessage> {
        let mut channels = self.channels.write().await;
        let tx = channels
            .entry(ring_id.to_string())
            .or_insert_with(|| broadcast::channel(256).0);
        tx.subscribe()
    }

    pub async fn broadcast(&self, ring_id: &str, msg: WsMessage) {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(ring_id) {
            let _ = tx.send(msg);
        }
    }
}
```

- [ ] **Step 2: Register in `services/mod.rs`**

Add to `ring-server/src/services/mod.rs`:

```rust
pub mod ws_hub;
```

And add the re-export:

```rust
pub use ws_hub::WsHub;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

---

## Task 6: Update AppState

**Files:**
- Modify: `ring-server/src/state.rs`

- [ ] **Step 1: Add ws_hub field to AppState**

```rust
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::Config;
use crate::db::traits::Repository;
use crate::graph::petgraph_store::PetgraphStore;
use crate::services::llm_provider::LlmProvider;
use crate::services::ws_hub::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Repository>,
    pub graph_store: Arc<RwLock<PetgraphStore>>,
    pub config: Arc<Config>,
    pub llm_provider: Arc<dyn LlmProvider>,
    pub ws_hub: Arc<WsHub>,
}
```

- [ ] **Step 2: Update `main.rs` to create WsHub**

In `ring-server/src/main.rs`, after the `let config = Arc::new(config);` line, add:

```rust
    let ws_hub = Arc::new(WsHub::new());
```

And update the `AppState` construction:

```rust
    let state = AppState {
        db,
        graph_store,
        config,
        llm_provider,
        ws_hub,
    };
```

Add the import at top:

```rust
use ring_server::services::ws_hub::WsHub;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

---

## Task 7: Session Service

**Files:**
- Create: `ring-server/src/services/session_service.rs`
- Modify: `ring-server/src/services/mod.rs`

- [ ] **Step 1: Create `session_service.rs`**

```rust
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

        let active = self.db.list_sessions_by_ring(ring_id, Some("active")).await?;
        if !active.is_empty() {
            return Err(RingError::Conflict("an active session already exists for this ring".into()));
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
            return Err(RingError::NotFound(format!("session not in ring {}", ring_id)));
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
            return Err(RingError::NotFound(format!("session not in ring {}", ring_id)));
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
            if existing.iter().any(|m| m.user_id == *mid && m.status == "active") {
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
            return Err(RingError::NotFound(format!("session not in ring {}", ring_id)));
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
            return Err(RingError::NotFound(format!("session not in ring {}", ring_id)));
        }
        if session.created_by == user_id {
            return Err(RingError::Validation("owner cannot leave, use close instead".into()));
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
            return Err(RingError::NotFound(format!("session not in ring {}", ring_id)));
        }
        if session.created_by != caller_id {
            return Err(RingError::Forbidden("only session owner can toggle archive".into()));
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
            return Err(RingError::NotFound(format!("session not in ring {}", ring_id)));
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
            return Err(RingError::NotFound(format!("session not in ring {}", ring_id)));
        }
        let messages = self
            .db
            .get_session_messages(session_id, after_seq, limit)
            .await?;
        Ok(SessionMessagesResponse { messages })
    }
}
```

- [ ] **Step 2: Register in `services/mod.rs`**

Add:

```rust
pub mod session_service;
```

And re-export:

```rust
pub use session_service::SessionService;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

---

## Task 8: Member Handlers

**Files:**
- Create: `ring-server/src/handlers/member.rs`
- Modify: `ring-server/src/handlers/mod.rs`

- [ ] **Step 1: Create `member.rs` handler**

The handlers follow the existing pattern (see `archive.rs`). Hardcoded `user_id` is consistent with Phase 4's approach.

```rust
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::error::RingError;
use crate::models::invite::InviteToken;
use crate::models::member::Member;
use crate::services::MemberService;
use crate::state::AppState;

pub async fn list_members(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> Result<Json<serde_json::Value>, RingError> {
    let service = MemberService::new(state.db.clone());
    let members = service.list_members(&ring_id).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}

#[derive(Deserialize)]
pub struct GenerateInviteRequest {
    pub token_type: String,
    pub role: String,
    pub max_uses: i64,
    pub max_members: Option<i64>,
}

pub async fn generate_invite(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Json(req): Json<GenerateInviteRequest>,
) -> Result<Json<InviteToken>, RingError> {
    let service = MemberService::new(state.db.clone());
    let token = service
        .generate_invite(
            &ring_id,
            "user-1",
            &req.token_type,
            &req.role,
            req.max_uses,
            req.max_members,
            86400,
        )
        .await?;
    Ok(Json(token))
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: String,
}

pub async fn update_role(
    State(state): State<AppState>,
    Path((ring_id, member_id)): Path<(String, String)>,
    Json(req): Json<UpdateRoleRequest>,
) -> Result<StatusCode, RingError> {
    let service = MemberService::new(state.db.clone());
    service
        .update_role(&ring_id, &member_id, &req.role, "user-1")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_member(
    State(state): State<AppState>,
    Path((ring_id, member_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = MemberService::new(state.db.clone());
    service.remove_member(&ring_id, &member_id, "user-1").await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct JoinRequest {
    pub display_name: String,
}

pub async fn join_ring(
    State(state): State<AppState>,
    Query(params): Query<JoinQueryParams>,
    Json(req): Json<JoinRequest>,
) -> Result<Json<Member>, RingError> {
    let service = MemberService::new(state.db.clone());
    let member = service.join_ring(&params.token, "user-1", &req.display_name).await?;
    Ok(Json(member))
}

#[derive(Deserialize)]
pub struct JoinQueryParams {
    pub token: String,
}
```

- [ ] **Step 2: Add to `handlers/mod.rs`**

```rust
pub mod member;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

---

## Task 9: Session Handlers

**Files:**
- Create: `ring-server/src/handlers/session.rs`
- Modify: `ring-server/src/handlers/mod.rs`

- [ ] **Step 1: Create `session.rs` handler**

```rust
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::error::RingError;
use crate::models::session_model::*;
use crate::services::session_service::SessionService;
use crate::state::AppState;

pub async fn create_session(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<(StatusCode, Json<SessionDetailResponse>), RingError> {
    let service = SessionService::new(state.db.clone());
    let session = service.create_session(&ring_id, &req, "user-1").await?;
    Ok((StatusCode::CREATED, Json(session)))
}

#[derive(Deserialize)]
pub struct ListSessionsQuery {
    pub status: Option<String>,
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<Json<SessionListResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let resp = service
        .list_sessions(&ring_id, query.status.as_deref())
        .await?;
    Ok(Json(resp))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<SessionDetailResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let detail = service.get_session_detail(&ring_id, &session_id).await?;
    Ok(Json(detail))
}

pub async fn close_session(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service.close_session(&ring_id, &session_id, "user-1").await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn leave_session(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service.leave_session(&ring_id, &session_id, "user-1").await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn toggle_archive(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(req): Json<ArchiveToggleRequest>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service
        .toggle_archive(&ring_id, &session_id, req.archive_enabled, "user-1")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn invite_member(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(req): Json<InviteSessionRequest>,
) -> Result<Json<serde_json::Value>, RingError> {
    let service = SessionService::new(state.db.clone());
    let invited = service
        .invite_member(&ring_id, &session_id, &req.member_ids, "user-1")
        .await?;
    Ok(Json(serde_json::json!({ "invited": invited })))
}

pub async fn delete_session(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<StatusCode, RingError> {
    let service = SessionService::new(state.db.clone());
    service.delete_session(&ring_id, &session_id, "user-1").await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct MessagesQuery {
    pub after_seq: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_messages(
    State(state): State<AppState>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<SessionMessagesResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let resp = service
        .get_messages(&ring_id, &session_id, query.after_seq, query.limit.unwrap_or(50))
        .await?;
    Ok(Json(resp))
}
```

- [ ] **Step 2: Add to `handlers/mod.rs`**

```rust
pub mod session;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

---

## Task 10: Notification Handlers

**Files:**
- Create: `ring-server/src/handlers/notification.rs`
- Modify: `ring-server/src/handlers/mod.rs`

- [ ] **Step 1: Create `notification.rs` handler**

```rust
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::error::RingError;
use crate::services::notification_service::NotificationService;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListNotificationsQuery {
    pub unread_only: Option<bool>,
}

pub async fn list_notifications(
    State(state): State<AppState>,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<serde_json::Value>, RingError> {
    let service = NotificationService::new(state.db.clone());
    let notifications = service
        .list_for_user("user-1", query.unread_only.unwrap_or(false))
        .await?;
    Ok(Json(serde_json::json!({ "notifications": notifications })))
}

pub async fn mark_read(
    State(state): State<AppState>,
    Path(notification_id): Path<String>,
) -> Result<StatusCode, RingError> {
    let service = NotificationService::new(state.db.clone());
    service.mark_read(&notification_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

- [ ] **Step 2: Add to `handlers/mod.rs`**

```rust
pub mod notification;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

---

## Task 11: WebSocket Handler

**Files:**
- Create: `ring-server/src/handlers/ws.rs`
- Modify: `ring-server/src/handlers/mod.rs`

- [ ] **Step 1: Create `ws.rs` handler**

```rust
use axum::extract::{Path, State, WebSocketUpgrade, ws::{Message, WebSocket}};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};

use crate::services::ws_hub::WsMessage;
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, ring_id))
}

async fn handle_socket(socket: WebSocket, state: AppState, ring_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_hub.get_or_create_channel(&ring_id).await;

    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if msg.is_err() {
                break;
            }
        }
    });

    let hub = state.ws_hub.clone();
    let ring_id_clone = ring_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(ws_msg) = rx.recv().await {
            let json = match serde_json::to_string(&ws_msg) {
                Ok(j) => j,
                Err(_) => continue,
            };
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = (&mut recv_task) => send_task.abort(),
        _ = (&mut send_task) => recv_task.abort(),
    }
}
```

- [ ] **Step 2: Add to `handlers/mod.rs`**

```rust
pub mod ws;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

---

## Task 12: Register All Routes

**Files:**
- Modify: `ring-server/src/routes.rs`

- [ ] **Step 1: Update `routes.rs`**

Replace the entire `routes.rs` with:

```rust
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::handlers::ai;
use crate::handlers::archive;
use crate::handlers::blueprint;
use crate::handlers::conversation;
use crate::handlers::git;
use crate::handlers::graph;
use crate::handlers::install;
use crate::handlers::member;
use crate::handlers::notification;
use crate::handlers::ring;
use crate::handlers::search;
use crate::handlers::session;
use crate::handlers::setup;
use crate::handlers::ws;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let setup_routes = Router::new()
        .route("/status", get(setup::get_status))
        .route("/username", post(setup::set_username))
        .route("/llm", post(setup::set_llm))
        .route("/gitlab", post(setup::set_gitlab))
        .route("/complete", post(setup::complete));

    let ring_routes = Router::new()
        .route("/", get(ring::list_rings).post(ring::create_ring))
        .route(
            "/{ringId}",
            get(ring::get_ring)
                .put(ring::update_ring)
                .delete(ring::delete_ring),
        );

    let member_routes = Router::new()
        .route("/", get(member::list_members))
        .route("/invites", post(member::generate_invite))
        .route("/{memberId}/role", put(member::update_role))
        .route("/{memberId}", delete(member::remove_member));

    let session_routes = Router::new()
        .route("/", post(session::create_session).get(session::list_sessions))
        .route(
            "/{sessionId}",
            get(session::get_session).delete(session::delete_session),
        )
        .route("/{sessionId}/close", post(session::close_session))
        .route("/{sessionId}/leave", post(session::leave_session))
        .route(
            "/{sessionId}/archive-toggle",
            put(session::toggle_archive),
        )
        .route("/{sessionId}/invite", post(session::invite_member))
        .route(
            "/{sessionId}/messages",
            get(session::get_messages),
        );

    let conversation_routes = Router::new()
        .route("/", get(conversation::list).post(conversation::create))
        .route("/{convId}", get(conversation::get))
        .route(
            "/{convId}/messages",
            get(conversation::get_messages).post(conversation::send_message),
        );

    let blueprint_routes = Router::new()
        .route("/templates", get(blueprint::list_templates))
        .route("/chat", post(blueprint::blueprint_chat))
        .route("/preview", post(blueprint::preview_blueprint))
        .route("/confirm", post(blueprint::confirm_blueprint));

    let graph_routes = Router::new()
        .route("/", get(graph::list_graphs))
        .route("/{graphId}", get(graph::get_graph))
        .route("/{graphId}/nodes", post(graph::create_node))
        .route(
            "/{graphId}/nodes/{nodeId}",
            get(graph::get_node)
                .put(graph::update_node)
                .delete(graph::delete_node),
        )
        .route(
            "/{graphId}/nodes/{nodeId}/content",
            get(graph::get_node_content),
        )
        .route("/{graphId}/edges", post(graph::create_edge))
        .route("/{graphId}/edges/{edgeId}", delete(graph::delete_edge));

    let search_routes = Router::new().route("/", post(search::search_nodes));

    let archive_routes = Router::new()
        .route("/", post(archive::archive))
        .route("/queue", get(archive::get_queue))
        .route("/{archiveId}/confirm", post(archive::confirm_archive));

    let git_routes = Router::new()
        .route("/prs", get(git::list_prs))
        .route("/prs/{prId}/diff", get(git::get_pr_diff))
        .route("/prs/{prId}/merge", post(git::merge_pr))
        .route("/prs/{prId}/reject", post(git::reject_pr))
        .route("/commits", get(git::get_commit_log));

    let notification_routes = Router::new()
        .route("/", get(notification::list_notifications))
        .route("/{notificationId}", post(notification::mark_read));

    Router::new()
        .nest("/api/v1/setup", setup_routes)
        .nest("/api/v1/rings", ring_routes)
        .nest("/api/v1/rings/{ringId}/members", member_routes)
        .nest("/api/v1/rings/{ringId}/sessions", session_routes)
        .nest("/api/v1/rings/{ringId}/conversations", conversation_routes)
        .nest("/api/v1/rings/{ringId}/blueprint", blueprint_routes)
        .nest("/api/v1/rings/{ringId}/graphs", graph_routes)
        .nest("/api/v1/rings/{ringId}/search", search_routes)
        .nest("/api/v1/rings/{ringId}/archive", archive_routes)
        .nest("/api/v1/rings/{ringId}/git", git_routes)
        .nest("/api/v1/notifications", notification_routes)
        .route("/api/v1/super-ring/chat", post(ai::super_ring_chat))
        .route("/api/v1/ws/{ringId}", get(ws::ws_handler))
        .route("/join", get(install::join_page))
        .with_state(state)
}
```

Note: The `join` route is kept as a separate route (not nested) since it's at a different path level. The frontend calls `POST /rings/join?token=xxx` which we handle via a separate join route.

Also add a route for joining:
```rust
        .route("/api/v1/rings/join", post(member::join_ring))
```

This goes BEFORE the `/{ringId}` route in the ring_routes to avoid matching "join" as a ringId. Update ring_routes:

```rust
    let ring_routes = Router::new()
        .route("/join", post(member::join_ring))
        .route("/", get(ring::list_rings).post(ring::create_ring))
        .route(
            "/{ringId}",
            get(ring::get_ring)
                .put(ring::update_ring)
                .delete(ring::delete_ring),
        );
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`

---

## Task 13: Clippy + Final Verification

- [ ] **Step 1: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: no warnings

- [ ] **Step 2: Run cargo fmt**

Run: `cargo fmt`
Expected: files formatted

- [ ] **Step 3: Run clippy again after fmt**

Run: `cargo clippy -- -D warnings`
Expected: no warnings

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "fix(phase5): add session service, member/notification handlers, WebSocket hub, and routes"
```
