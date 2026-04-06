# Phase 5 Implementation Plan — Collaboration & Permissions (TDD)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现多人协作基础：邀请机制、成员管理、权限校验中间件、三模式切换、WebSocket 实时通信、Session 创建与管理。

**Architecture:** MemberService 编排成员 CRUD 和邀请流程（生成 token、验证、加入）。PermissionService 封装角色权限检查（handler 层调用）。SessionService 管理 Session 生命周期。WebSocket hub 基于 tokio broadcast channel 实现消息广播。三模式（chat/archive/auto）作为 conversation 的 mode 字段切换。

**Tech Stack:** axum WebSocket, tokio broadcast, serde_json, uuid

**Environment Note:** `cargo test` 无法运行（OpenSSL x86_64/arm64 链接器问题），用 `cargo clippy -- -D warnings` 验证编译。前端用 `npm test` + `npm run build` 验证。

**Reference docs:**
- `docs/technical/api-design.md` — Section 8 (成员 API), Section 9 (Session API), Section 14 (WebSocket)
- `docs/technical/data-model.md` — members, invite_tokens, sessions, session_members, session_messages, notifications 表
- `docs/product/permissions.md` — 权限矩阵、角色定义、三模式说明
- `docs/technical/sse-protocol.md` — WebSocket 消息类型

---

## File Structure

```
ring-server/src/
├── services/
│   ├── member_service.rs       # 成员管理 + 邀请流程
│   ├── permission_service.rs   # 权限校验（角色检查）
│   ├── session_service.rs      # Session CRUD + 归档开关
│   └── ws_service.rs           # WebSocket hub（broadcast channel）
├── handlers/
│   ├── member.rs               # 成员 API endpoints
│   ├── session.rs              # Session API endpoints
│   └── ws.rs                   # WebSocket upgrade handler
├── models/
│   ├── session_model.rs        # Session + SessionMember + SessionMessage models
│   └── notification_model.rs   # Notification model
└── (routes.rs updated with member + session + ws routes)

ring-frontend/src/
├── components/session/
│   └── SessionView.tsx         # Session 页面
├── components/member/
│   └── MemberList.tsx          # 成员列表
├── stores/
│   ├── memberStore.ts          # 成员状态管理
│   └── sessionStore.ts         # Session 状态管理
└── App.tsx                     # Add session routes
```

---

## Module 1: Member Service + Permission Service + Models

**Files:**
- Create: `ring-server/src/services/member_service.rs`
- Create: `ring-server/src/services/permission_service.rs`
- Create: `ring-server/src/models/notification_model.rs`
- Modify: `ring-server/src/services/mod.rs`
- Modify: `ring-server/src/models/mod.rs`
- Modify: `ring-server/src/db/traits.rs` — add member/notification query methods
- Modify: `ring-server/src/db/sqlite.rs` — implement member/notification queries
- Modify: `ring-server/src/services/ai_service.rs` — update MockRepo

### Step 1: Create notification model

Create `ring-server/src/models/notification_model.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub ring_id: String,
    pub user_id: String,
    pub r#type: String,
    pub title: String,
    pub body: Option<String>,
    pub related_id: Option<String>,
    pub is_read: bool,
    pub created_at: String,
}
```

Add `pub mod notification_model;` to `models/mod.rs`.

### Step 2: Add DB methods for members and notifications

Add to `db/traits.rs`:

```rust
use crate::models::member::{Member, NewMember};
use crate::models::notification_model::Notification;

async fn create_member(&self, new_member: NewMember) -> Result<Member>;
async fn get_member(&self, id: &str) -> Result<Option<Member>>;
async fn list_members_by_ring(&self, ring_id: &str) -> Result<Vec<Member>>;
async fn get_member_by_user_and_ring(&self, user_id: &str, ring_id: &str) -> Result<Option<Member>>;
async fn update_member_role(&self, id: &str, role: &str) -> Result<()>;
async fn delete_member(&self, id: &str) -> Result<()>;
async fn get_next_token_id(&self, ring_id: &str) -> Result<i64>;
async fn create_notification(&self, n: NewNotification) -> Result<Notification>;
async fn list_notifications_by_user(&self, user_id: &str, unread_only: bool) -> Result<Vec<Notification>>;
async fn mark_notification_read(&self, id: &str) -> Result<()>;
```

Implement in `sqlite.rs` following existing patterns (sqlx::query, FromRow structs).

Note: The Member model already has `token_id: i64` field but NewMember doesn't. The DB implementation should auto-assign token_id using `get_next_token_id` which does `SELECT MAX(token_id) FROM members WHERE ring_id = ?` and returns max+1 (or 2 if no members, since creator is #1).

### Step 3: Create PermissionService

Create `ring-server/src/services/permission_service.rs`:

```rust
use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use std::sync::Arc;

pub struct PermissionService {
    db: Arc<dyn Repository>,
}

impl PermissionService {
    pub fn new(db: Arc<dyn Repository>) -> Self {
        PermissionService { db }
    }

    pub async fn check_ring_access(&self, ring_id: &str, user_id: &str) -> Result<()> {
        let member = self.db.get_member_by_user_and_ring(user_id, ring_id).await?;
        if member.is_none() {
            return Err(RingError::Forbidden("not a member of this ring".into()));
        }
        Ok(())
    }

    pub async fn check_creator_or_admin(&self, ring_id: &str, user_id: &str) -> Result<()> {
        let member = self.db.get_member_by_user_and_ring(user_id, ring_id).await?;
        match member {
            Some(m) if m.role == "creator" || m.role == "admin" => Ok(()),
            Some(_) => Err(RingError::Forbidden("creator or admin required".into())),
            None => Err(RingError::Forbidden("not a member of this ring".into())),
        }
    }

    pub async fn check_creator(&self, ring_id: &str, user_id: &str) -> Result<()> {
        let member = self.db.get_member_by_user_and_ring(user_id, ring_id).await?;
        match member {
            Some(m) if m.role == "creator" => Ok(()),
            Some(_) => Err(RingError::Forbidden("creator required".into())),
            None => Err(RingError::Forbidden("not a member of this ring".into())),
        }
    }

    pub async fn get_member_role(&self, ring_id: &str, user_id: &str) -> Result<Option<String>> {
        let member = self.db.get_member_by_user_and_ring(user_id, ring_id).await?;
        Ok(member.map(|m| m.role))
    }

    pub async fn is_creator(&self, ring_id: &str, user_id: &str) -> Result<bool> {
        Ok(self.get_member_role(ring_id, user_id).await? == Some("creator".into()))
    }
}
```

### Step 4: Create MemberService

Create `ring-server/src/services/member_service.rs`:

```rust
use std::sync::Arc;
use crate::db::traits::Repository;
use crate::error::{Result, RingError};
use crate::models::invite::InviteToken;
use crate::models::member::{Member, NewMember};
use crate::models::notification_model::Notification;
use uuid::Uuid;
use chrono::Utc;

pub struct MemberService {
    db: Arc<dyn Repository>,
}

impl MemberService {
    pub fn new(db: Arc<dyn Repository>) -> Self {
        MemberService { db }
    }

    pub async fn generate_invite(
        &self,
        ring_id: &str,
        inviter_id: &str,
        token_type: &str,
        role: &str,
        max_uses: i64,
        max_members: Option<i64>,
        expires_in_seconds: i64,
    ) -> Result<InviteToken> {
        let ring = self.db.get_ring(ring_id).await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;
        if ring.creator_id != inviter_id {
            return Err(RingError::Forbidden("only creator can invite".into()));
        }
        if let Some(mm) = max_members {
            let count = self.db.count_members_by_ring(ring_id).await?;
            if count >= mm {
                return Err(RingError::Conflict("ring member limit reached".into()));
            }
        }
        let token_bytes = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::seconds(expires_in_seconds);
        self.db.create_invite_token(
            ring_id,
            &token_bytes,
            token_type,
            inviter_id,
        ).await
    }

    pub async fn join_ring(
        &self,
        token_str: &str,
        user_id: &str,
        display_name: &str,
    ) -> Result<Member> {
        let token = self.db.get_invite_token(token_str).await?
            .ok_or_else(|| RingError::NotFound("invalid invite token".into()))?;

        if token.revoked_at.is_some() {
            return Err(RingError::Forbidden("token has been revoked".into()));
        }
        let now = Utc::now().to_rfc3339();
        if token.expires_at < now {
            return Err(RingError::Forbidden("token has expired".into()));
        }
        if token.max_uses > 0 && token.use_count >= token.max_uses {
            return Err(RingError::Forbidden("token usage limit reached".into()));
        }

        let existing = self.db.get_member_by_user_and_ring(user_id, &token.ring_id).await?;
        if existing.is_some() {
            return Err(RingError::Conflict("already a member".into()));
        }

        let token_id = self.db.get_next_token_id(&token.ring_id).await?;

        let new_member = NewMember {
            ring_id: token.ring_id.clone(),
            user_id: user_id.to_string(),
            display_name: display_name.to_string(),
            role: Some(token.role.clone()),
        };
        let member = self.db.create_member(new_member).await?;
        Ok(member)
    }

    pub async fn list_members(&self, ring_id: &str) -> Result<Vec<Member>> {
        self.db.list_members_by_ring(ring_id).await
    }

    pub async fn update_role(&self, ring_id: &str, member_id: &str, new_role: &str, caller_id: &str) -> Result<()> {
        let ring = self.db.get_ring(ring_id).await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;
        if ring.creator_id != caller_id {
            return Err(RingError::Forbidden("only creator can change roles".into()));
        }
        let member = self.db.get_member(member_id).await?
            .ok_or_else(|| RingError::NotFound(format!("member {}", member_id)))?;
        if member.ring_id != ring_id {
            return Err(RingError::NotFound(format!("member {} not in ring", member_id)))?;
        }
        if member.role == "creator" {
            return Err(RingError::Forbidden("cannot change creator role".into()));
        }
        self.db.update_member_role(member_id, new_role).await
    }

    pub async fn remove_member(&self, ring_id: &str, member_id: &str, caller_id: &str) -> Result<()> {
        let ring = self.db.get_ring(ring_id).await?
            .ok_or_else(|| RingError::NotFound(format!("ring {}", ring_id)))?;
        if ring.creator_id != caller_id {
            return Err(RingError::Forbidden("only creator can remove members".into()));
        }
        let member = self.db.get_member(member_id).await?
            .ok_or_else(|| RingError::NotFound(format!("member {}", member_id)))?;
        if member.role == "creator" {
            return Err(RingError::Forbidden("cannot remove creator".into()));
        }
        self.db.delete_member(member_id).await
    }
}
```

### Step 5: Update module declarations

- Add `pub mod member_service;`, `pub mod permission_service;` to `services/mod.rs`
- Add `pub use member_service::MemberService;`, `pub use permission_service::PermissionService;` to `services/mod.rs`

### Step 6: Write tests for MemberService

In `member_service.rs`, add `#[cfg(test)] mod tests` with:

- `generate_invite_creates_token` — create a ring, generate invite, verify token returned
- `join_ring_with_valid_token` — join with valid token, verify member created with correct role
- `join_ring_with_expired_token_fails` — create expired token (set expires_at in past), verify error
- `join_ring_already_member_fails` — join twice, second should fail with Conflict
- `non_creator_cannot_invite` — try to invite as non-creator, verify Forbidden
- `remove_member_succeeds` — creator removes a member, verify removed
- `cannot_remove_creator` — try to remove creator, verify Forbidden

Use the same MockRepo pattern from archive_service.rs.

### Step 7: Run clippy

```bash
cd ring-server && cargo clippy -- -D warnings
```

### Step 8: Commit

```bash
git add ring-server/
git commit -m "feat(phase5): add member service, permission service, and notification model"
```

---

## Module 2: Session Service + WebSocket Hub

**Files:**
- Create: `ring-server/src/services/session_service.rs`
- Create: `ring-server/src/services/ws_service.rs`
- Create: `ring-server/src/models/session_model.rs`
- Modify: `ring-server/src/services/mod.rs`
- Modify: `ring-server/src/models/mod.rs`
- Modify: `ring-server/src/db/traits.rs` — add session query methods
- Modify: `ring-server/src/db/sqlite.rs` — implement session queries
- Modify: `ring-server/src/services/ai_service.rs` — update MockRepo

### Step 1: Create session models

Create `ring-server/src/models/session_model.rs`:

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
pub struct SessionResponse {
    pub id: String,
    pub ring_id: String,
    pub title: Option<String>,
    pub scenario: String,
    pub created_by: String,
    pub archive_enabled: bool,
    pub status: String,
    pub members: Vec<SessionMemberResponse>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMemberResponse {
    pub user_id: String,
    pub role: String,
    pub status: String,
}
```

Add `pub mod session_model;` to `models/mod.rs`.

### Step 2: Add DB methods for sessions

Add to `db/traits.rs`:

```rust
use crate::models::session_model::{Session, SessionMember, SessionMessage};

async fn create_session(&self, ring_id: &str, title: Option<&str>, scenario: &str, created_by: &str, archive_enabled: bool) -> Result<Session>;
async fn get_session(&self, id: &str) -> Result<Option<Session>>;
async fn list_sessions_by_ring(&self, ring_id: &str, status: Option<&str>) -> Result<Vec<Session>>;
async fn update_session_status(&self, id: &str, status: &str) -> Result<()>;
async fn update_session_archive(&self, id: &str, enabled: bool) -> Result<()>;
async fn get_active_session_for_ring(&self, ring_id: &str) -> Result<Option<Session>>;
async fn create_session_member(&self, session_id: &str, user_id: &str, role: &str) -> Result<SessionMember>;
async fn list_session_members(&self, session_id: &str) -> Result<Vec<SessionMember>>;
async fn update_session_member_status(&self, session_id: &str, user_id: &str, status: &str) -> Result<()>;
async fn create_session_message(&self, session_id: &str, sender_id: &str, role: &str, content: &str) -> Result<SessionMessage>;
async fn get_session_messages(&self, session_id: &str, after_seq: Option<i64>, limit: i64) -> Result<Vec<SessionMessage>>;
```

Implement in `sqlite.rs`. For `create_session_message`, use `SELECT COALESCE(MAX(seq_num), 0) + 1 FROM session_messages WHERE session_id = ?` to auto-increment seq_num.

### Step 3: Create SessionService

Create `ring-server/src/services/session_service.rs`:

Key methods:
- `create_session(ring_id, user_id, req) -> SessionResponse` — check no active session exists, create session + add creator as owner
- `get_session(ring_id, session_id) -> SessionResponse`
- `list_sessions(ring_id, status) -> Vec<Session>`
- `close_session(ring_id, session_id, user_id)` — check owner, update status
- `leave_session(ring_id, session_id, user_id)` — update member status
- `toggle_archive(ring_id, session_id, user_id, enabled)` — check owner
- `invite_to_session(ring_id, session_id, user_id, member_ids)` — check owner, verify ring membership
- `send_session_message(ring_id, session_id, user_id, content) -> SessionMessage`

### Step 4: Create WebSocket hub

Create `ring-server/src/services/ws_service.rs`:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde_json::Value;

pub type WsSender = tokio::sync::mpsc::UnboundedSender<Value>;

pub struct WsHub {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Value>>>>,
}

impl WsHub {
    pub fn new() -> Self {
        WsHub {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_channel(&self, ring_id: &str) -> broadcast::Sender<Value> {
        let mut channels = self.channels.write().await;
        channels.entry(ring_id.to_string())
            .or_insert_with(|| broadcast::channel(100).0)
            .clone()
    }

    pub async fn broadcast(&self, ring_id: &str, msg: Value) {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(ring_id) {
            let _ = tx.send(msg);
        }
    }

    pub async fn subscribe(&self, ring_id: &str) -> broadcast::Receiver<Value> {
        let channels = self.channels.read().await;
        if let Some(tx) = channels.get(ring_id) {
            tx.subscribe()
        } else {
            drop(channels);
            self.create_channel(ring_id).await;
            let channels = self.channels.read().await;
            channels.get(ring_id).unwrap().subscribe()
        }
    }
}
```

### Step 5: Update module declarations and MockRepo

- Add `pub mod session_service;`, `pub mod ws_service;` to `services/mod.rs`
- Add `pub use session_service::SessionService;` to `services/mod.rs`
- Update MockRepo in `ai_service.rs` with all new DB methods (unimplemented!)

### Step 6: Write tests for SessionService

- `create_session_success` — create session, verify response with owner as member
- `create_session_conflict_if_active` — two active sessions, second should 409
- `close_session_by_owner` — owner closes, verify status changed
- `close_session_by_non_owner_fails` — non-owner tries, verify Forbidden
- `toggle_archive_by_owner` — owner toggles, verify changed
- `invite_to_session_checks_ring_membership` — invite non-ring-member, verify error

### Step 7: Run clippy

```bash
cd ring-server && cargo clippy -- -D warnings
```

### Step 8: Commit

```bash
git add ring-server/
git commit -m "feat(phase5): add session service, WebSocket hub, and session models"
```

---

## Module 3: Member + Session Handlers + Routes

**Files:**
- Create: `ring-server/src/handlers/member.rs`
- Create: `ring-server/src/handlers/session.rs`
- Create: `ring-server/src/handlers/ws.rs`
- Modify: `ring-server/src/handlers/mod.rs`
- Modify: `ring-server/src/routes.rs`
- Modify: `ring-server/src/state.rs` — add WsHub

### Step 1: Add WsHub to AppState

In `state.rs`:

```rust
use crate::services::ws_service::WsHub;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<dyn Repository>,
    pub graph_store: Arc<RwLock<PetgraphStore>>,
    pub config: Arc<Config>,
    pub llm_provider: Arc<dyn LlmProvider>,
    pub ws_hub: Arc<WsHub>,
}
```

Update `main.rs` to create WsHub and pass it to AppState.

### Step 2: Create member handler

Create `ring-server/src/handlers/member.rs` with these endpoints:

```
POST /api/v1/rings/{ringId}/invites          → generate_invite
POST /api/v1/rings/join?token={token}         → join_ring
GET  /api/v1/rings/{ringId}/members           → list_members
PUT  /api/v1/rings/{ringId}/members/{memberId}/role → update_role
DELETE /api/v1/rings/{ringId}/members/{memberId}    → remove_member
```

Handler pattern: extract params → construct MemberService → call service → return response.

### Step 3: Create session handler

Create `ring-server/src/handlers/session.rs` with:

```
POST   /api/v1/rings/{ringId}/sessions                     → create_session
GET    /api/v1/rings/{ringId}/sessions                     → list_sessions
GET    /api/v1/rings/{ringId}/sessions/{sessionId}         → get_session
POST   /api/v1/rings/{ringId}/sessions/{sessionId}/invite   → invite_to_session
POST   /api/v1/rings/{ringId}/sessions/{sessionId}/leave    → leave_session
POST   /api/v1/rings/{ringId}/sessions/{sessionId}/close    → close_session
PUT    /api/v1/rings/{ringId}/sessions/{sessionId}/archive-toggle → toggle_archive
DELETE /api/v1/rings/{ringId}/sessions/{sessionId}         → delete_session
```

### Step 4: Create WebSocket handler

Create `ring-server/src/handlers/ws.rs`:

```rust
use axum::extract::{Path, State, WebSocketUpgrade, ws::{Message, WebSocket}};
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(ring_id): Path<String>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, ring_id))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, ring_id: String) {
    let mut rx = state.ws_hub.subscribe(&ring_id).await;
    let (mut sender, mut receiver) = socket.split();
    
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        // handle incoming messages (mode_change, etc.)
    }
}
```

### Step 5: Update routes and handlers/mod.rs

Add to `handlers/mod.rs`:
```rust
pub mod member;
pub mod session;
pub mod ws;
```

Add route groups to `routes.rs`:
```rust
let member_routes = Router::new()
    .route("/invites", post(member::generate_invite))
    .route("/members", get(member::list_members))
    .route("/members/{memberId}/role", put(member::update_role))
    .route("/members/{memberId}", delete(member::remove_member));

let session_routes = Router::new()
    .route("/", post(session::create_session).get(session::list_sessions))
    .route("/{sessionId}", get(session::get_session).delete(session::delete_session))
    .route("/{sessionId}/invite", post(session::invite_to_session))
    .route("/{sessionId}/leave", post(session::leave_session))
    .route("/{sessionId}/close", post(session::close_session))
    .route("/{sessionId}/archive-toggle", put(session::toggle_archive));

// Top-level join route (no ringId in path)
// POST /api/v1/rings/join?token={token}

.nest("/api/v1/rings/{ringId}", member_routes)
.nest("/api/v1/rings/{ringId}/sessions", session_routes)
.route("/api/v1/ws/{ringId}", get(ws::ws_handler))
```

Also add a standalone route for join: `.route("/api/v1/rings/join", post(member::join_ring))`

### Step 6: Run clippy

```bash
cd ring-server && cargo clippy -- -D warnings
```

### Step 7: Commit

```bash
git add ring-server/
git commit -m "feat(phase5): add member, session, and WebSocket handlers with routes"
```

---

## Module 4: Frontend — Member List + Session View + Stores

**Files:**
- Create: `ring-frontend/src/components/member/MemberList.tsx`
- Create: `ring-frontend/src/components/session/SessionView.tsx`
- Create: `ring-frontend/src/stores/memberStore.ts`
- Create: `ring-frontend/src/stores/sessionStore.ts`
- Modify: `ring-frontend/src/api/client.ts` — add member + session API functions
- Modify: `ring-frontend/src/types/index.ts` — add member + session types
- Modify: `ring-frontend/src/App.tsx` — add session route

### Step 1: Add types

Add to `types/index.ts`:

```typescript
export interface Member {
  id: string
  ring_id: string
  user_id: string
  token_id: number
  display_name: string
  role: string
  joined_at: string
}

export interface Session {
  id: string
  ring_id: string
  title: string | null
  scenario: string
  created_by: string
  archive_enabled: boolean
  status: string
  members: SessionMemberResponse[]
  created_at: string
}

export interface SessionMemberResponse {
  user_id: string
  role: string
  status: string
}

export interface CreateSessionRequest {
  title?: string
  scenario: string
  archive_enabled?: boolean
  invite_member_ids?: string[]
}

export interface InviteRequest {
  token_type: string
  role: string
  max_uses: number
  max_members?: number
}
```

### Step 2: Add API functions

Add to `client.ts`:

```typescript
export async function list_members(ring_id: string): Promise<Member[]> {
  const data = await request<{ members: Member[] }>(`/rings/${ring_id}/members`)
  return data.members
}

export async function generate_invite(ring_id: string, req: InviteRequest): Promise<InviteToken> {
  return request<InviteToken>(`/rings/${ring_id}/invites`, {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export async function update_member_role(ring_id: string, member_id: string, role: string): Promise<void> {
  return request<void>(`/rings/${ring_id}/members/${member_id}/role`, {
    method: 'PUT',
    body: JSON.stringify({ role }),
  })
}

export async function remove_member(ring_id: string, member_id: string): Promise<void> {
  return request<void>(`/rings/${ring_id}/members/${member_id}`, { method: 'DELETE' })
}

export async function join_ring(token: string, display_name: string): Promise<Member> {
  return request<Member>(`/rings/join?token=${token}`, {
    method: 'POST',
    body: JSON.stringify({ display_name }),
  })
}

export async function create_session(ring_id: string, req: CreateSessionRequest): Promise<Session> {
  return request<Session>(`/rings/${ring_id}/sessions`, {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export async function list_sessions(ring_id: string, status?: string): Promise<Session[]> {
  const data = await request<{ sessions: Session[] }>(`/rings/${ring_id}/sessions${status ? `?status=${status}` : ''}`)
  return data.sessions
}

export async function get_session(ring_id: string, session_id: string): Promise<Session> {
  return request<Session>(`/rings/${ring_id}/sessions/${session_id}`)
}

export async function close_session(ring_id: string, session_id: string): Promise<void> {
  return request<void>(`/rings/${ring_id}/sessions/${session_id}/close`, { method: 'POST' })
}

export async function leave_session(ring_id: string, session_id: string): Promise<void> {
  return request<void>(`/rings/${ring_id}/sessions/${session_id}/leave`, { method: 'POST' })
}

export async function toggle_session_archive(ring_id: string, session_id: string, enabled: boolean): Promise<void> {
  return request<void>(`/rings/${ring_id}/sessions/${session_id}/archive-toggle`, {
    method: 'PUT',
    body: JSON.stringify({ archive_enabled: enabled }),
  })
}

export async function invite_to_session(ring_id: string, session_id: string, member_ids: string[]): Promise<void> {
  return request<void>(`/rings/${ring_id}/sessions/${session_id}/invite`, {
    method: 'POST',
    body: JSON.stringify({ member_ids }),
  })
}

export async function delete_session(ring_id: string, session_id: string): Promise<void> {
  return request<void>(`/rings/${ring_id}/sessions/${session_id}`, { method: 'DELETE' })
}
```

### Step 3: Create stores

`stores/memberStore.ts` — Zustand store with:
- State: members[], invite_token, loading, error
- Actions: load_members, generate_invite, update_role, remove_member, join_ring

`stores/sessionStore.ts` — Zustand store with:
- State: sessions[], current_session, loading, error
- Actions: load_sessions, create_session, close_session, leave_session, toggle_archive, invite_to_session, delete_session

### Step 4: Create components

`MemberList.tsx` — Table showing members with role badges, invite button, role change dropdown, remove button

`SessionView.tsx` — Session list + create form + session detail with members, archive toggle, close/leave buttons

### Step 5: Add route to App.tsx

```tsx
<Route path="/ring/:ringId/sessions" element={<SetupGuard><SessionView /></SetupGuard>} />
```

### Step 6: Write tests for MemberList

Create `ring-frontend/src/components/member/__tests__/MemberList.test.tsx`:
- `renders member list with roles` — mock members, verify rendered
- `renders empty state` — no members, verify empty message
- `shows invite button for creator` — verify invite button visible

### Step 7: Run tests + build

```bash
cd ring-frontend && npm test && npm run build
```

### Step 8: Commit

```bash
git add ring-frontend/
git commit -m "feat(phase5): add member list, session view, and collaboration frontend"
```

---

## Module 5: Three-Mode Switch + Integration Verification

**Files:**
- Modify: `ring-server/src/handlers/conversation.rs` — add mode parameter support
- Modify: `ring-frontend/src/pages/RingSpace/ChatView.tsx` — add mode toggle UI
- Modify: `ring-frontend/src/stores/chatStore.ts` — add mode state

### Step 1: Backend mode support

The conversation `mode` field already exists in the DB schema (`chat`/`archive`/`auto`). Add a PATCH endpoint for switching mode:

```
PATCH /api/v1/rings/{ringId}/conversations/{convId}/mode
```

Request body: `{"mode": "chat"}` or `{"mode": "archive"}` or `{"mode": "auto"}`

Add to `db/traits.rs`: `async fn update_conversation_mode(&self, id: &str, mode: &str) -> Result<()>;`

### Step 2: Frontend mode toggle

Add a mode toggle bar in ChatView with three buttons: Chat / Archive / Auto. Clicking switches the mode via API.

### Step 3: Run all verification

```bash
cd ring-server && cargo clippy -- -D warnings && cargo fmt --check
cd ring-frontend && npm test && npm run build
```

### Step 4: Commit

```bash
git add .
git commit -m "feat(phase5): add three-mode switch for conversations"
```

---

## Module 6: Final Integration Verification

- [ ] **Step 1:** `cd ring-server && cargo clippy -- -D warnings`
- [ ] **Step 2:** `cd ring-server && cargo fmt --check`
- [ ] **Step 3:** `cd ring-frontend && npm test`
- [ ] **Step 4:** `cd ring-frontend && npm run build`
- [ ] **Step 5:** `git commit --allow-empty -m "milestone: Phase 5 complete — collaboration & permissions"`
- [ ] **Step 6:** `git checkout main && git merge feat/phase5-collaboration-permissions --no-ff`
