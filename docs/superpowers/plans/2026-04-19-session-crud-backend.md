# Plan 5a: Session CRUD Backend

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add session CRUD backend — DB tables, models, services, routes — so sessions can be created, listed, closed, reopened, deleted, and participants invited/removed.

**Architecture:** Follows existing handler→service→model pattern. 4 new tables in migration 005. Session model handles all queries. Session service enforces business rules (single active session per Ring, owner-only operations, participant must be Ring member). Routes are thin handlers.

**Tech Stack:** Rust + Axum + SQLite (sqlx) + ulid. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-04-19-session-lifecycle-design.md` §2, §3, §5, §9.

---

## File Structure

```
server/
├── migrations/
│   └── 005_sessions.sql              # NEW: 4 tables + ALTER TABLE members
├── src/
│   ├── models/
│   │   ├── mod.rs                    # MODIFY: add pub mod session
│   │   └── session.rs                # NEW: all session model types + queries
│   ├── services/
│   │   ├── mod.rs                    # MODIFY: add pub mod session
│   │   └── session.rs                # NEW: business logic
│   └── routes/
│       ├── mod.rs                    # MODIFY: add mod session + routes
│       └── session.rs                # NEW: CRUD handlers
└── tests/
    └── integration.rs                # MODIFY: add session tests
```

---

### Task 1: Database Migration — Session Tables

**Files:**
- Create: `server/migrations/005_sessions.sql`

- [ ] **Step 1: Create migration file**

`server/migrations/005_sessions.sql`:

```sql
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    skill TEXT NOT NULL DEFAULT 'discussion',
    phase TEXT NOT NULL DEFAULT 'material_prep',
    owner TEXT NOT NULL,
    archivable INTEGER NOT NULL DEFAULT 0,
    archive_enabled INTEGER NOT NULL DEFAULT 0,
    summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_sessions_ring ON sessions(ring_id);
CREATE INDEX IF NOT EXISTS idx_sessions_phase ON sessions(phase);

CREATE TABLE IF NOT EXISTS session_participants (
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    token_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'participant',
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (session_id, token_id)
);

CREATE TABLE IF NOT EXISTS session_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    seq_num INTEGER NOT NULL,
    sender TEXT NOT NULL,
    sender_name TEXT NOT NULL,
    content TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'user',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_session_messages_seq ON session_messages(session_id, seq_num);
CREATE INDEX IF NOT EXISTS idx_session_messages_session ON session_messages(session_id, created_at);

CREATE TABLE IF NOT EXISTS session_materials (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'collecting',
    highlight TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_session_materials_session ON session_materials(session_id);

ALTER TABLE members ADD COLUMN session_grant INTEGER NOT NULL DEFAULT 0;
```

- [ ] **Step 2: Verify build**

Run: `cd server && cargo build 2>&1`
Expected: Compiles (migration runs at startup).

- [ ] **Step 3: Commit**

```bash
git add server/migrations/005_sessions.sql
git commit -m "feat(server): add sessions, session_participants, session_messages, session_materials tables"
```

---

### Task 2: Session Model — Types and Queries

**Files:**
- Create: `server/src/models/session.rs`
- Modify: `server/src/models/mod.rs`

- [ ] **Step 1: Create session model**

`server/src/models/session.rs`:

```rust
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

pub async fn has_active_session(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
) -> Result<bool> {
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

    let row = sqlx::query_as::<_, SessionRow>(
        "INSERT INTO sessions (id, ring_id, title, description, skill, phase, owner, archivable, archive_enabled)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)
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

    Ok(row)
}

pub async fn list_sessions(
    pool: &sqlx::SqlitePool,
    ring_id: &str,
    status: Option<&str>,
) -> Result<Vec<SessionRow>> {
    let rows = match status {
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
    Ok(rows)
}

pub async fn get_session(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<SessionRow> {
    sqlx::query_as::<_, SessionRow>("SELECT * FROM sessions WHERE id = ?1")
        .bind(session_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| RingError::NotFound("session not found".into()))
}

pub async fn get_participants(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<Vec<SessionParticipantRow>> {
    sqlx::query_as::<_, SessionParticipantRow>(
        "SELECT * FROM session_participants WHERE session_id = ?1 ORDER BY joined_at",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
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

pub async fn is_owner(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    token_id: &str,
) -> Result<bool> {
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
    let row = sqlx::query_as::<_, SessionRow>(
        "UPDATE sessions SET phase = ?1, updated_at = datetime('now') WHERE id = ?2 RETURNING *",
    )
    .bind(new_phase)
    .bind(session_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound("session not found".into()))?;
    Ok(row)
}

pub async fn delete_session(pool: &sqlx::SqlitePool, session_id: &str) -> Result<()> {
    let result = sqlx::query("DELETE FROM sessions WHERE id = ?1 AND phase = 'closed'")
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;
    if result.rows_affected() == 0 {
        return Err(RingError::BadRequest(
            "can only delete closed sessions".into(),
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
    let result =
        sqlx::query("DELETE FROM session_participants WHERE session_id = ?1 AND token_id = ?2 AND role != 'owner'")
            .bind(session_id)
            .bind(token_id)
            .execute(pool)
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;
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
    let row = sqlx::query_as::<_, SessionRow>(
        "UPDATE sessions SET archive_enabled = ?1, updated_at = datetime('now') WHERE id = ?2 RETURNING *",
    )
    .bind(enabled)
    .bind(session_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound("session not found".into()))?;
    Ok(row)
}

pub async fn get_messages(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    after_seq: i64,
    limit: i64,
) -> Result<Vec<SessionMessageRow>> {
    sqlx::query_as::<_, SessionMessageRow>(
        "SELECT * FROM session_messages WHERE session_id = ?1 AND seq_num > ?2 ORDER BY seq_num ASC LIMIT ?3",
    )
    .bind(session_id)
    .bind(after_seq)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}
```

- [ ] **Step 2: Register module**

In `server/src/models/mod.rs`, add `pub mod session;` after the existing mod declarations.

- [ ] **Step 3: Verify build**

Run: `cd server && cargo check 2>&1`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add server/src/models/session.rs server/src/models/mod.rs
git commit -m "feat(server): session model with CRUD queries"
```

---

### Task 3: Session Service — Business Logic

**Files:**
- Create: `server/src/services/session.rs`
- Modify: `server/src/services/mod.rs`

- [ ] **Step 1: Create session service**

`server/src/services/session.rs`:

```rust
use crate::error::{Result, RingError};
use crate::models::ring;
use crate::models::session;
use crate::state::AppState;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    #[serde(flatten)]
    pub session: session::SessionRow,
    pub participants: Vec<session::SessionParticipantRow>,
}

pub async fn create_session(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    input: &session::CreateSessionInput,
) -> Result<SessionResponse> {
    let role = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if role != "creator" && role != "admin" {
        let has_grant: bool =
            sqlx::query_scalar::<_, bool>("SELECT session_grant FROM members WHERE ring_id = ?1 AND user_id = ?2 AND session_grant = 1")
                .bind(ring_id)
                .bind(user_id)
                .fetch_optional(&state.db)
                .await?
                .unwrap_or(false);
        if !has_grant {
            return Err(RingError::Forbidden(
                "no permission to create sessions".into(),
            ));
        }
    }

    if session::has_active_session(&state.db, ring_id).await? {
        return Err(RingError::Conflict(
            "ring already has an active session".into(),
        ));
    }

    let valid_skills = [
        "decision",
        "research",
        "review",
        "retrospective",
        "knowledge_sharing",
        "discussion",
    ];
    if !valid_skills.contains(&input.skill.as_str()) {
        return Err(RingError::BadRequest(format!(
            "invalid skill: {}",
            input.skill
        )));
    }

    for invitee in &input.invitees {
        let is_member: bool = sqlx::query_scalar::<_, bool>(
            "SELECT COUNT(*) > 0 FROM members WHERE ring_id = ?1 AND user_id = ?2",
        )
        .bind(ring_id)
        .bind(invitee)
        .fetch_one(&state.db)
        .await?;
        if !is_member {
            return Err(RingError::BadRequest(format!(
                "invitee {invitee} is not a ring member"
            )));
        }
    }

    let id = ulid::Ulid::new().to_string();
    let row = session::create_session(&state.db, &id, ring_id, user_id, input).await?;
    let participants = session::get_participants(&state.db, &id).await?;

    Ok(SessionResponse {
        session: row,
        participants,
    })
}

pub async fn list_sessions(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    status: Option<&str>,
) -> Result<Vec<session::SessionRow>> {
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
    let sess = session::get_session(&state.db, session_id).await?;
    if sess.ring_id != ring_id {
        return Err(RingError::NotFound("session not found".into()));
    }
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse {
        session: sess,
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
    let sess = session::get_session(&state.db, session_id).await?;
    if sess.ring_id != ring_id {
        return Err(RingError::NotFound("session not found".into()));
    }
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("only session owner can close".into()));
    }
    if sess.phase == "closed" {
        return Err(RingError::BadRequest("session already closed".into()));
    }
    let row = session::update_phase(&state.db, session_id, "closed").await?;
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse {
        session: row,
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
    let sess = session::get_session(&state.db, session_id).await?;
    if sess.ring_id != ring_id {
        return Err(RingError::NotFound("session not found".into()));
    }
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("only session owner can reopen".into()));
    }
    if sess.phase != "closed" {
        return Err(RingError::BadRequest("only closed sessions can be reopened".into()));
    }
    if session::has_active_session(&state.db, ring_id).await? {
        return Err(RingError::Conflict(
            "ring already has an active session".into(),
        ));
    }
    let target_phase = if sess.skill == "discussion" {
        "discussion"
    } else {
        "discussion"
    };
    let row = session::update_phase(&state.db, session_id, target_phase).await?;
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse {
        session: row,
        participants,
    })
}

pub async fn delete_session(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
) -> Result<serde_json::Value> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    let sess = session::get_session(&state.db, session_id).await?;
    if sess.ring_id != ring_id {
        return Err(RingError::NotFound("session not found".into()));
    }
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("only session owner can delete".into()));
    }
    session::delete_session(&state.db, session_id).await?;
    Ok(serde_json::json!({"status": "deleted"}))
}

pub async fn invite_participants(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
    input: &session::InviteParticipantsInput,
) -> Result<Vec<session::SessionParticipantRow>> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    let sess = session::get_session(&state.db, session_id).await?;
    if sess.ring_id != ring_id {
        return Err(RingError::NotFound("session not found".into()));
    }
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("only session owner can invite".into()));
    }
    for tid in &input.token_ids {
        let is_member: bool = sqlx::query_scalar::<_, bool>(
            "SELECT COUNT(*) > 0 FROM members WHERE ring_id = ?1 AND user_id = ?2",
        )
        .bind(ring_id)
        .bind(tid)
        .fetch_one(&state.db)
        .await?;
        if !is_member {
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
) -> Result<serde_json::Value> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    let sess = session::get_session(&state.db, session_id).await?;
    if sess.ring_id != ring_id {
        return Err(RingError::NotFound("session not found".into()));
    }
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden(
            "only session owner can remove participants".into(),
        ));
    }
    session::remove_participant(&state.db, session_id, target_id).await?;
    Ok(serde_json::json!({"status": "removed"}))
}

pub async fn toggle_archive(
    state: &AppState,
    ring_id: &str,
    session_id: &str,
    user_id: &str,
    enabled: bool,
) -> Result<SessionResponse> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    let sess = session::get_session(&state.db, session_id).await?;
    if sess.ring_id != ring_id {
        return Err(RingError::NotFound("session not found".into()));
    }
    if !session::is_owner(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden(
            "only session owner can toggle archive".into(),
        ));
    }
    if !sess.archivable {
        return Err(RingError::BadRequest(
            "session does not have archive capability".into(),
        ));
    }
    let row = session::toggle_archive(&state.db, session_id, enabled).await?;
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse {
        session: row,
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
    let sess = session::get_session(&state.db, session_id).await?;
    if sess.ring_id != ring_id {
        return Err(RingError::NotFound("session not found".into()));
    }
    if !session::is_participant(&state.db, session_id, user_id).await? {
        return Err(RingError::Forbidden("not a session participant".into()));
    }
    session::get_messages(&state.db, session_id, after_seq, limit).await
}
```

- [ ] **Step 2: Register module**

In `server/src/services/mod.rs`, add `pub mod session;` after the existing mod declarations.

- [ ] **Step 3: Verify build**

Run: `cd server && cargo check 2>&1`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add server/src/services/session.rs server/src/services/mod.rs
git commit -m "feat(server): session service with business logic"
```

---

### Task 4: Session Routes — CRUD Handlers

**Files:**
- Create: `server/src/routes/session.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Create session routes**

`server/src/routes/session.rs`:

```rust
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::error::Result;
use crate::extractors::auth::AuthUser;
use crate::models::session::{ArchiveToggleInput, CreateSessionInput, InviteParticipantsInput};
use crate::services;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MessagesQuery {
    pub after_seq: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn create_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Json(body): Json<CreateSessionInput>,
) -> Result<Json<services::session::SessionResponse>> {
    let resp = services::session::create_session(&state, &ring_id, &user.token_id, &body).await?;
    Ok(Json(resp))
}

pub async fn list_sessions(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    Query(query): Query<ListSessionsQuery>,
) -> Result<Json<serde_json::Value>> {
    let sessions = services::session::list_sessions(
        &state,
        &ring_id,
        &user.token_id,
        query.status.as_deref(),
    )
    .await?;
    Ok(Json(serde_json::json!({"sessions": sessions})))
}

pub async fn get_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<services::session::SessionResponse>> {
    let resp =
        services::session::get_session_detail(&state, &ring_id, &session_id, &user.token_id)
            .await?;
    Ok(Json(resp))
}

pub async fn close_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<services::session::SessionResponse>> {
    let resp = services::session::close_session(&state, &ring_id, &session_id, &user.token_id)
        .await?;
    Ok(Json(resp))
}

pub async fn reopen_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<services::session::SessionResponse>> {
    let resp = services::session::reopen_session(&state, &ring_id, &session_id, &user.token_id)
        .await?;
    Ok(Json(resp))
}

pub async fn delete_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>> {
    let result =
        services::session::delete_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(result))
}

pub async fn invite_participants(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(body): Json<InviteParticipantsInput>,
) -> Result<Json<serde_json::Value>> {
    let participants = services::session::invite_participants(
        &state,
        &ring_id,
        &session_id,
        &user.token_id,
        &body,
    )
    .await?;
    Ok(Json(serde_json::json!({"participants": participants})))
}

pub async fn remove_participant(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id, target_id)): Path<(String, String, String)>,
) -> Result<Json<serde_json::Value>> {
    let result = services::session::remove_participant(
        &state,
        &ring_id,
        &session_id,
        &target_id,
        &user.token_id,
    )
    .await?;
    Ok(Json(result))
}

pub async fn archive_toggle(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(body): Json<ArchiveToggleInput>,
) -> Result<Json<services::session::SessionResponse>> {
    let resp = services::session::toggle_archive(
        &state,
        &ring_id,
        &session_id,
        &user.token_id,
        body.enabled,
    )
    .await?;
    Ok(Json(resp))
}

pub async fn get_messages(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<serde_json::Value>> {
    let messages = services::session::get_messages(
        &state,
        &ring_id,
        &session_id,
        &user.token_id,
        query.after_seq.unwrap_or(0),
        query.limit.unwrap_or(50),
    )
    .await?;
    Ok(Json(serde_json::json!({"messages": messages})))
}
```

- [ ] **Step 2: Register routes**

In `server/src/routes/mod.rs`:
- Add `mod session;` after the existing mod declarations (after `mod setup;`)
- Add these routes inside the `let api = Router::new()` block, before `.with_state(state)`:

```rust
        .route(
            "/rings/{ring_id}/sessions",
            get(session::list_sessions).post(session::create_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}",
            get(session::get_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/close",
            post(session::close_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/reopen",
            post(session::reopen_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}",
            delete(session::delete_session),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/participants",
            post(session::invite_participants),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/participants/{target_id}",
            delete(session::remove_participant),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/archive-toggle",
            put(session::archive_toggle),
        )
        .route(
            "/rings/{ring_id}/sessions/{session_id}/messages",
            get(session::get_messages),
        )
```

**IMPORTANT:** The DELETE route for sessions (`delete(session::delete_session)`) conflicts with the GET route at the same path (`/rings/{ring_id}/sessions/{session_id}`). They must be combined into one `.route()` call:

```rust
        .route(
            "/rings/{ring_id}/sessions/{session_id}",
            get(session::get_session).delete(session::delete_session),
        )
```

So the final route registration replaces the separate GET and DELETE entries with the combined one.

- [ ] **Step 3: Verify build**

Run: `cd server && cargo check 2>&1`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/session.rs server/src/routes/mod.rs
git commit -m "feat(server): session CRUD routes — create, list, get, close, reopen, delete, participants, archive toggle"
```

---

### Task 5: Integration Tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add session tests**

Append to `server/tests/integration.rs`:

```rust
#[tokio::test]
async fn test_session_crud() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let create_body = r#"{"title":"Test Session","description":"A test","skill":"discussion","archivable":true,"invitees":[]}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(create_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let json = read_body(resp).await;
    let session_id = json["id"].as_str().unwrap();
    assert_eq!(json["skill"], "discussion");
    assert_eq!(json["phase"], "discussion");
    assert_eq!(json["owner"], token);

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/sessions"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["sessions"].as_array().unwrap().len(), 1);

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/sessions/{session_id}"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["title"], "Test Session");
    assert_eq!(json["participants"].as_array().unwrap().len(), 1);

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/close"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["phase"], "closed");

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/reopen"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["phase"], "discussion");

    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/close"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let resp = app
        .clone()
        .oneshot(make_request(
            "DELETE",
            &format!("/api/rings/{ring_id}/sessions/{session_id}"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_session_single_active() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let create_body = r#"{"title":"Session 1","skill":"discussion"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(create_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let create_body2 = r#"{"title":"Session 2","skill":"discussion"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(create_body2),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_session_archive_toggle() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let create_body = r#"{"title":"Test","skill":"discussion","archivable":true}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(create_body),
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    let session_id = json["id"].as_str().unwrap();

    let toggle_body = r#"{"enabled":true}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/archive-toggle"),
            Some(toggle_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["archive_enabled"], true);
}
```

- [ ] **Step 2: Run tests**

Run: `cd server && cargo test 2>&1`
Expected: All tests pass (6 existing + 3 new = 9).

- [ ] **Step 3: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test(server): session CRUD integration tests"
```

---

## Self-Review

### 1. Spec Coverage

| Spec Requirement | Covered | Task |
|------------------|---------|------|
| 4 DB tables + ALTER members | Yes | Task 1 |
| Session CRUD model queries | Yes | Task 2 |
| Single active session per Ring | Yes | Task 3 (create_session check) |
| Owner-only operations | Yes | Task 3 (is_owner checks) |
| Participant must be Ring member | Yes | Task 3 (is_member check on invite) |
| discussion skill → phase=discussion | Yes | Task 2 (create_session phase logic) |
| session_grant permission | Yes | Task 3 (create_session role check) |
| All 11 API endpoints | Yes | Task 4 |
| Phase transitions: close, reopen, delete | Yes | Task 3 |
| archive_toggle checks archivable | Yes | Task 3 |
| Integration tests | Yes | Task 5 |

### 2. Placeholder Scan

No TBD/TODO found. All steps contain complete code.

### 3. Type Consistency

- `CreateSessionInput` defined in model → used in route → used in service ✓
- `SessionResponse` defined in service → returned by all session routes ✓
- `SessionRow.phase` is `String` — matches DB `TEXT` and frontend `SessionPhase` union type ✓
- `SessionRow.archivable` is `bool` — matches `INTEGER NOT NULL DEFAULT 0` in SQLite (sqlx handles) ✓
- Route DELETE at same path as GET — combined into one `.route()` call ✓
