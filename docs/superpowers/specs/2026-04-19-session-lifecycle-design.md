# Session Lifecycle Design

Date: 2026-04-19

## 1. Overview

Session is a multi-person real-time discussion space within a Ring. A Session Ring (independent AI instance) is activated during sessions, behavior determined by the loaded Skill. Sessions follow a 5-phase lifecycle: create → material prep → discussion → summarize → close.

**Implementation split into 4 sub-plans:**
- **Plan 5a**: Session CRUD backend + DB tables
- **Plan 5b**: WebSocket real-time chat
- **Plan 5c**: Frontend SessionPanel complete UI
- **Plan 5d**: Material prep + Summary + Skill loading

---

## 2. Data Model

### 2.1 `sessions` table

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    skill TEXT NOT NULL DEFAULT 'discussion',
    phase TEXT NOT NULL DEFAULT 'discussing',
    owner TEXT NOT NULL,
    archivable INTEGER NOT NULL DEFAULT 0,
    summary TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

| Column | Description |
|--------|-------------|
| `id` | ULID |
| `ring_id` | Parent Ring |
| `title` | Session title |
| `description` | Optional description |
| `skill` | One of: `decision`, `research`, `review`, `retrospective`, `knowledge_sharing`, `discussion` |
| `phase` | `material_prep`, `discussing`, `summarizing`, `closed` |
| `owner` | token_id of the session creator |
| `archivable` | 0 or 1, archive toggle (owner controls) |
| `summary` | AI-generated summary text (nullable, set after summarize phase) |

**Business rules:**
- Only one active session per Ring at a time (enforced in service layer)
- `phase` transitions follow state machine (see §5)
- `discussion` skill sessions skip `material_prep` and `summarizing` phases

### 2.2 `session_participants` table

```sql
CREATE TABLE session_participants (
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    token_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'participant',
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (session_id, token_id)
);
```

| Column | Description |
|--------|-------------|
| `role` | `owner` or `participant` |

### 2.3 `session_messages` table

```sql
CREATE TABLE session_messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    seq_num INTEGER NOT NULL,
    sender TEXT NOT NULL,
    sender_name TEXT NOT NULL,
    content TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'user',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX idx_session_messages_seq ON session_messages(session_id, seq_num);
CREATE INDEX idx_session_messages_session ON session_messages(session_id, created_at);
```

| Column | Description |
|--------|-------------|
| `seq_num` | Monotonically increasing per session, used for catch-up |
| `sender` | token_id of sender |
| `sender_name` | Display name at time of sending (denormalized) |
| `message_type` | `user`, `system`, `ai_delta`, `ai_end` |

### 2.4 `session_materials` table

```sql
CREATE TABLE session_materials (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    item_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'collecting',
    highlight TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_session_materials_session ON session_materials(session_id);
```

| Column | Description |
|--------|-------------|
| `item_type` | `document`, `graph_node`, `ai_generated` |
| `status` | `collecting`, `analyzing`, `ready` |
| `highlight` | Optional note from owner marking important items |

---

## 3. Backend API

### 3.1 CRUD Endpoints

All endpoints follow existing pattern: route → service → model.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/rings/{ring_id}/sessions` | Ring member (creator/admin, or member with grant) | Create session |
| GET | `/api/rings/{ring_id}/sessions?status=active` | Ring member | List sessions |
| GET | `/api/rings/{ring_id}/sessions/{session_id}` | Ring member | Get session detail |
| POST | `/api/rings/{ring_id}/sessions/{session_id}/close` | Session owner | Close session |
| POST | `/api/rings/{ring_id}/sessions/{session_id}/reopen` | Session owner | Reopen closed session |
| DELETE | `/api/rings/{ring_id}/sessions/{session_id}` | Session owner | Delete session permanently |
| POST | `/api/rings/{ring_id}/sessions/{session_id}/participants` | Session owner | Invite Ring members |
| DELETE | `/api/rings/{ring_id}/sessions/{session_id}/participants/{token_id}` | Session owner | Remove participant |
| PUT | `/api/rings/{ring_id}/sessions/{session_id}/archive-toggle` | Session owner | Toggle archive capability |
| POST | `/api/rings/{ring_id}/sessions/{session_id}/summarize` | Session owner | Trigger AI summary (SSE stream) |
| GET | `/api/rings/{ring_id}/sessions/{session_id}/messages?after_seq=0&limit=50` | Session participant | Get message history |
| GET | `/api/rings/{ring_id}/sessions/{session_id}/material-prep` | Session participant | Get material prep progress |
| POST | `/api/rings/{ring_id}/sessions/{session_id}/material-prep/highlights` | Session owner | Mark material highlight |

### 3.2 Request/Response Shapes

**Create Session:**
```json
// POST /api/rings/{ring_id}/sessions
// Request:
{
  "title": "竞品 A 深度讨论",
  "description": "讨论竞品 A 的最新功能更新",
  "skill": "decision",
  "archivable": true,
  "invitees": ["user-002", "user-003"]
}

// Response 201:
{
  "id": "01JTYSESS",
  "title": "竞品 A 深度讨论",
  "skill": "decision",
  "phase": "material_prep",
  "owner": "user-001",
  "participants": [
    {"token_id": "user-001", "display_name": "Kai", "role": "owner"},
    {"token_id": "user-002", "display_name": "Alice", "role": "participant"}
  ],
  "archivable": true,
  "created_at": "2026-04-19T08:00:00Z"
}
```

**Phase transition rules:**
- `discussion` skill → phase starts at `discussing` (skip material_prep)
- Other skills → phase starts at `material_prep`
- Owner manually transitions from `material_prep` → `discussing`
- Summarize is triggered by owner → phase becomes `summarizing` → `closed` when done

### 3.3 File Structure

```
server/
├── migrations/
│   └── 005_sessions.sql
├── src/
│   ├── models/
│   │   └── session.rs          # SessionRow, SessionParticipant, SessionMessage, SessionMaterial + queries
│   ├── services/
│   │   └── session.rs          # Business logic: create, phase transitions, participants, archive toggle
│   └── routes/
│       ├── session.rs          # CRUD handlers
│       ├── ws.rs               # WebSocket handler (Plan 5b)
│       └── mod.rs              # Route registration
└── Cargo.toml                  # No new deps for Plan 5a; tokio-tungstenite not needed (axum WS built-in)
```

---

## 4. WebSocket Hub

### 4.1 Architecture

```
AppState {
    db: SqlitePool,
    ws_hub: WsHub,
}

WsHub {
    sessions: DashMap<session_id, SessionChannel>,
    connections: DashMap<token_id, Sender>,
}

SessionChannel {
    participants: HashSet<token_id>,
    owner: token_id,
}
```

Using `dashmap` for concurrent access. `Sender` is `tokio::sync::mpsc::UnboundedSender<String>`.

### 4.2 Connection Lifecycle

1. Client connects: `WS /api/ws?token=<user_token_id>`
2. Server validates token, registers connection in `WsHub::connections`
3. Client sends/receives JSON messages
4. On disconnect: remove from `connections`, check if any session owner went offline → broadcast `session_paused`

### 4.3 Message Types

**Client → Server:**
```json
{
  "type": "session_message",
  "session_id": "01JTYSESS",
  "content": "我觉得竞品 A 的定价策略有变化"
}
```

**Server → Client (broadcast):**
```json
{
  "type": "session_message",
  "session_id": "01JTYSESS",
  "seq_num": 44,
  "sender": "user-002",
  "sender_name": "Alice",
  "content": "我觉得竞品 A 的定价策略有变化",
  "created_at": "2026-04-19T08:05:01Z"
}
```

**Server → Client (catch-up):**
```json
{
  "type": "session_catchup",
  "session_id": "01JTYSESS",
  "messages": [...]
}
```

**Server → Client (session paused/resumed):**
```json
{ "type": "session_paused", "session_id": "01JTYSESS", "reason": "owner_offline" }
{ "type": "session_resumed", "session_id": "01JTYSESS" }
```

**Server → Client (notifications):**
```json
{
  "type": "notification",
  "notification": {
    "id": "01JTYNOTIF",
    "category": "session_invite",
    "title": "Session 邀请",
    "body": "Kai 邀请你参加「竞品 A 深度讨论」",
    "ring_id": "01JTYRING",
    "created_at": "2026-04-19T08:00:00Z"
  }
}
```

**Server → Client (member kicked):**
```json
{ "type": "session_member_kicked", "session_id": "01JTYSESS" }
```

### 4.4 Owner Offline Detection

When a WebSocket connection drops:
1. Remove from `connections`
2. Check if disconnected user is owner of any active session
3. If yes → broadcast `session_paused` to all participants
4. When owner reconnects → broadcast `session_resumed`

### 4.5 Catch-up Mechanism

When a participant reconnects:
1. Client sends last received `seq_num`
2. Server queries `session_messages WHERE seq_num > last_seq`
3. Server sends `session_catchup` with all missed messages

---

## 5. Phase State Machine

```
                        ┌── skill=discussion ──┐
                        │                      ▼
[create] ──→ material_prep ────────→ discussing ────→ closed
                 │                       │               ▲
                 └──(owner: start)───────┘               │
                                         │               │
                                  [owner: summarize]     │
                                         │               │
                                         ▼               │
                                    summarizing ──────────┘
                                         │        (reopen)
                                    [AI complete]
                                         │
                                         ▼
                                      closed
```

**Phase rules:**
- `discussion` skill: skip `material_prep`, go directly to `discussing`
- `material_prep` → `discussing`: owner triggers "start discussion"
- `discussing` → `summarizing`: owner triggers summarize (optional, skill-dependent)
- `summarizing` → `closed`: AI completes summary automatically
- `discussing` → `closed`: owner closes directly (if no summarize needed)
- `closed` → `discussing`: owner reopens
- Delete: only from `closed` state, permanent

---

## 6. Skill System

### 6.1 Skill Loading

When a session is created with a non-`discussion` skill:
1. Load skill definition from `~/.ring/skills/{skill}/SKILL.md` (or built-in)
2. Inject into Session Ring system prompt
3. Skill determines material prep behavior and summary format

### 6.2 Built-in Skills

5 pre-installed skills, stored as embedded strings in the binary (not files):

| Skill | Material Prep | Summary Output |
|-------|--------------|----------------|
| `decision` | Collects relevant documents + graph nodes | Decision conclusions + action items |
| `research` | Collects + generates research resources | Research report |
| `review` | Collects review targets | Review opinions + improvement suggestions |
| `retrospective` | Collects project timeline + metrics | Lessons learned + improvement plan |
| `knowledge_sharing` | Collects sharing materials | Organized notes |
| `discussion` | Skipped | Skipped |

### 6.3 Material Prep Flow

1. Session created with skill → phase = `material_prep`
2. Session Ring loads skill system prompt
3. AI collects materials based on session title/description:
   - Searches graph nodes for relevant content
   - Generates AI analysis of the topic
   - Creates `session_materials` entries
4. Participants can view progress via `GET material-prep`
5. Owner can highlight items via `POST material-prep/highlights`
6. Owner triggers "start discussion" → phase = `discussing`

### 6.4 Summary Flow

1. Owner triggers summarize → phase = `summarizing`
2. Session Ring generates summary via SSE stream (same pattern as chat)
3. Summary text stored in `sessions.summary`
4. Phase transitions to `closed`
5. If archivable, owner can trigger archive (uses Ring standard archive flow)

---

## 7. Frontend Architecture

### 7.1 New/Modified Files

```
ui/src/
├── stores/
│   ├── session-store.ts        # Session CRUD + WS message state
│   └── ws-store.ts             # WebSocket connection management
├── components/
│   └── panels/
│       └── SessionPanel.tsx    # Complete rewrite
└── services/
    └── ws-client.ts            # WebSocket client wrapper
```

### 7.2 SessionPanel Layout

```
┌─────────────────────────────────────┐
│ Session: 竞品 A 深度讨论            │
│ Skill: decision · Phase: discussion │
│ Owner: Kai · Participants: 3       │
├─────────────────────────────────────┤
│ ┌─────────────────────────────────┐ │
│ │                                 │ │
│ │  [Chat messages area]           │ │
│ │  - seq 44: Alice: 我觉得...     │ │
│ │  - seq 45: Bob: 同意，另外...   │ │
│ │                                 │ │
│ └─────────────────────────────────┘ │
│ ┌──────────────────┐ [Send]        │
│ │ Message input... │               │
│ └──────────────────┘               │
├─────────────────────────────────────┤
│ [Close Session] [Summarize] [+]    │
└─────────────────────────────────────┘
```

### 7.3 Session Creation Flow

1. User types `!session` or clicks Session tab → panel opens
2. If no active session: show "Create Session" form
   - Title (required)
   - Description (optional)
   - Skill selector (6 options, default: discussion)
   - Archive toggle
   - Participant selector (multi-select from Ring members)
3. Submit → POST create → WS notifies invited members
4. Panel switches to session chat view

### 7.4 Sidebar Integration

- Existing session indicator dot in sidebar
- When active session exists, show session title + participant count
- Click to toggle SessionPanel

---

## 8. Implementation Sub-Plans

### Plan 5a: Session CRUD Backend

**Scope:** DB migrations + models + services + routes for session CRUD. No WebSocket.

**Files:**
- `server/migrations/005_sessions.sql`
- `server/src/models/session.rs`
- `server/src/services/session.rs`
- `server/src/routes/session.rs`
- Modify: `server/src/models/mod.rs`, `server/src/services/mod.rs`, `server/src/routes/mod.rs`

**Verification:** `cargo check`, `cargo test`, curl E2E (create, list, get, close, reopen, delete, participants, archive toggle).

### Plan 5b: WebSocket Real-time Chat

**Scope:** WsHub, WebSocket endpoint, message relay, owner offline detection, catch-up.

**New dependency:** `dashmap` for concurrent HashMap.

**Files:**
- `server/src/ws_hub.rs` (WsHub struct + broadcast logic)
- `server/src/routes/ws.rs` (WebSocket handler)
- Modify: `server/src/state.rs` (add WsHub to AppState)
- Modify: `server/src/routes/mod.rs` (add WS route)

**Verification:** `cargo check`, manual E2E with `websocat` or `wscat` to test message relay.

### Plan 5c: Frontend SessionPanel

**Scope:** Session store, WS client, complete SessionPanel rewrite, sidebar integration.

**Files:**
- `ui/src/stores/session-store.ts`
- `ui/src/stores/ws-store.ts`
- `ui/src/services/ws-client.ts`
- Rewrite: `ui/src/components/panels/SessionPanel.tsx`
- Modify: sidebar component (session indicator)

**Verification:** `npx tsc --noEmit`, `npm run build`, manual UI test.

### Plan 5d: Material Prep + Summary + Skills

**Scope:** Skill loading, material prep phase, AI summarize via SSE, material highlights.

**Files:**
- `server/src/services/skill.rs` (skill loading + system prompt builder)
- `server/src/routes/session.rs` (add material-prep endpoints, summarize endpoint)
- `ui/src/components/panels/SessionPanel.tsx` (add material prep view, summarize view)

**Verification:** `cargo check`, `npx tsc --noEmit`, E2E: create skill session → verify material prep phase → trigger summarize → verify summary stored.

---

## 9. Constraints

- **Single active session per Ring** — enforced at service layer, not DB constraint
- **Messages stored on creator backend SQLite** — no cross-backend sync needed (single-user binary)
- **Owner offline = session paused** — all participants blocked until owner reconnects
- **No temporary ownership transfer** — messages always on owner's backend
- **Session participants must be Ring members** — verified on invite
- **`discussion` skill skips material_prep and summarizing** — simplified lifecycle
