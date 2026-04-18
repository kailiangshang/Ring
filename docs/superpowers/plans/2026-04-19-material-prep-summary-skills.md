# Material Prep + Summary + Skills Implementation Plan (5d)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add three new backend endpoints (`POST start`, `POST summarize`, `GET material-prep`, `POST material-prep/highlights`), a skill loading service with built-in skill definitions, and frontend views for material prep phase and summarize streaming.

**Architecture:** Backend-first: skill.rs loads built-in skill system prompts → session routes add start/summarize/material-prep endpoints → session service adds phase transition logic → frontend SessionPanel gets material prep view + summarize view. The summarize endpoint uses the same SSE streaming pattern as chat.

**Tech Stack:** Rust + Axum SSE (same as chat.rs), async-openai for LLM calls, React SSE consumption (reuse existing `sse.ts` pattern).

---

## File Structure

```
server/src/
├── services/
│   └── skill.rs                # CREATE — built-in skill system prompts + loader
├── routes/
│   └── session.rs              # MODIFY — add start, summarize, material-prep endpoints
├── services/
│   ├── mod.rs                  # MODIFY — add pub mod skill
│   └── session.rs              # MODIFY — add start_session, summarize_session, get_materials, highlight_material
├── models/
│   └── session.rs              # MODIFY — add material queries (get_materials, create_material, update_highlight)

ui/src/
├── components/panels/
│   └── SessionPanel.tsx        # MODIFY — add MaterialPrepView + SummarizeView
├── stores/
│   └── session-store.ts        # MODIFY — add startSession, summarizeSession, fetchMaterials, highlightMaterial
├── types/
│   └── session.ts              # MODIFY — add SessionMaterial type
```

---

### Task 1: Create skill service with built-in definitions

**Files:**
- Create: `server/src/services/skill.rs`
- Modify: `server/src/services/mod.rs`

- [ ] **Step 1: Create skill.rs with built-in skill system prompts**

Each skill has a `system_prompt` for the Session Ring AI. The `discussion` skill is a no-op (no material prep, no summary). The other 5 skills define how the AI should collect materials and format summaries.

```rust
pub struct SkillDef {
    pub name: &'static str,
    pub material_prompt: &'static str,
    pub summary_prompt: &'static str,
}

const SKILLS: &[SkillDef] = &[
    SkillDef {
        name: "decision",
        material_prompt: "You are assisting a decision-making session. Based on the session title and description, identify and collect relevant documents, data points, and graph nodes. For each material, create a concise summary. List pros, cons, risks, and options related to the decision topic.",
        summary_prompt: "Summarize this decision-making session. Include: 1) The key decision made, 2) Main arguments for and against, 3) Action items with owners, 4) Follow-up dates. Format as structured markdown.",
    },
    SkillDef {
        name: "research",
        material_prompt: "You are assisting a research session. Based on the session title and description, collect relevant resources, references, and existing knowledge from the graph. Identify gaps in knowledge and suggest areas to investigate.",
        summary_prompt: "Write a research report summarizing this session. Include: 1) Research question, 2) Key findings, 3) Data sources, 4) Conclusions, 5) Recommendations for further research. Format as structured markdown.",
    },
    SkillDef {
        name: "review",
        material_prompt: "You are assisting a review session. Based on the session title and description, collect the review targets (documents, code, designs). Identify review criteria and checklists relevant to the review type.",
        summary_prompt: "Summarize this review session. Include: 1) Items reviewed, 2) Key findings (issues and positive aspects), 3) Improvement suggestions with priority levels, 4) Agreed actions. Format as structured markdown.",
    },
    SkillDef {
        name: "retrospective",
        material_prompt: "You are assisting a retrospective session. Based on the session title and description, collect project timeline data, metrics, and previous retrospective outcomes from the graph. Identify key events and milestones.",
        summary_prompt: "Summarize this retrospective. Include: 1) What went well, 2) What could be improved, 3) Lessons learned, 4) Action items for next cycle. Format as structured markdown.",
    },
    SkillDef {
        name: "knowledge_sharing",
        material_prompt: "You are assisting a knowledge sharing session. Based on the session title and description, collect relevant materials, prior discussions, and graph nodes related to the topic. Organize materials into a logical flow for presentation.",
        summary_prompt: "Create organized notes from this knowledge sharing session. Include: 1) Key topics covered, 2) Important takeaways, 3) References and resources mentioned, 4) Open questions. Format as structured markdown.",
    },
];

pub fn get_skill(name: &str) -> Option<&'static SkillDef> {
    SKILLS.iter().find(|s| s.name == name)
}

pub fn build_material_system_prompt(skill_name: &str, session_title: &str, session_description: &str) -> Option<String> {
    let skill = get_skill(skill_name)?;
    Some(format!(
        "{}\n\nSession: {}\nDescription: {}\n\nAnalyze the topic and provide a structured list of materials that should be prepared for this session. For each material, specify: title, type (document/graph_node/ai_generated), and a brief description of what it should contain.",
        skill.material_prompt,
        session_title,
        if session_description.is_empty() { "N/A" } else { session_description },
    ))
}

pub fn build_summary_system_prompt(skill_name: &str) -> Option<String> {
    let skill = get_skill(skill_name)?;
    Some(skill.summary_prompt.to_string())
}
```

- [ ] **Step 2: Register module in mod.rs**

Add `pub mod skill;` to `server/src/services/mod.rs`.

- [ ] **Step 3: Verify compiles**

Run: `cd server && cargo check`

- [ ] **Step 4: Commit**

```bash
git add server/src/services/skill.rs server/src/services/mod.rs
git commit -m "feat(server): skill service with built-in definitions"
```

---

### Task 2: Add material model queries

**Files:**
- Modify: `server/src/models/session.rs`

- [ ] **Step 1: Add material queries to session.rs**

Add these queries at the end of the file:

```rust
pub async fn get_materials(
    pool: &sqlx::SqlitePool,
    session_id: &str,
) -> Result<Vec<SessionMaterialRow>> {
    let rows = sqlx::query_as::<_, SessionMaterialRow>(
        "SELECT * FROM session_materials WHERE session_id = ?1 ORDER BY created_at",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn create_material(
    pool: &sqlx::SqlitePool,
    id: &str,
    session_id: &str,
    item_type: &str,
    title: &str,
    content: &str,
) -> Result<SessionMaterialRow> {
    sqlx::query_as::<_, SessionMaterialRow>(
        "INSERT INTO session_materials (id, session_id, item_type, title, content, status)
         VALUES (?1, ?2, ?3, ?4, ?5, 'collecting')
         RETURNING *",
    )
    .bind(id)
    .bind(session_id)
    .bind(item_type)
    .bind(title)
    .bind(content)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn update_material_status(
    pool: &sqlx::SqlitePool,
    material_id: &str,
    status: &str,
) -> Result<SessionMaterialRow> {
    sqlx::query_as::<_, SessionMaterialRow>(
        "UPDATE session_materials SET status = ?1 WHERE id = ?2 RETURNING *",
    )
    .bind(status)
    .bind(material_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound(format!("material {material_id} not found")))
}

pub async fn update_material_highlight(
    pool: &sqlx::SqlitePool,
    material_id: &str,
    highlight: &str,
) -> Result<SessionMaterialRow> {
    sqlx::query_as::<_, SessionMaterialRow>(
        "UPDATE session_materials SET highlight = ?1 WHERE id = ?2 RETURNING *",
    )
    .bind(highlight)
    .bind(material_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound(format!("material {material_id} not found")))
}

pub async fn set_summary(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    summary: &str,
) -> Result<SessionRow> {
    sqlx::query_as::<_, SessionRow>(
        "UPDATE sessions SET summary = ?1, updated_at = datetime('now') WHERE id = ?2 RETURNING *",
    )
    .bind(summary)
    .bind(session_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| RingError::NotFound(format!("session {session_id} not found")))
}
```

- [ ] **Step 2: Verify compiles**

Run: `cd server && cargo check`

- [ ] **Step 3: Commit**

```bash
git add server/src/models/session.rs
git commit -m "feat(server): material and summary model queries"
```

---

### Task 3: Add session service functions for start, summarize, materials

**Files:**
- Modify: `server/src/services/session.rs`

- [ ] **Step 1: Add start_session, get_materials_service, highlight_material, summarize_session**

Add these functions to the end of `session.rs`:

```rust
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
        return Err(RingError::BadRequest("session is not in material_prep phase".into()));
    }
    let session = session::update_phase(&state.db, session_id, "discussion").await?;
    let participants = session::get_participants(&state.db, session_id).await?;
    Ok(SessionResponse { session, participants })
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
        return Err(RingError::Forbidden("only owner can highlight materials".into()));
    }
    session::update_material_highlight(&state.db, material_id, note).await
}
```

For summarize, we need to use the LLM client. We'll create a service function that returns an `mpsc::Receiver<SseEvent>`:

```rust
use crate::services::llm::{LlmClient, SseEvent};
use crate::models::user;

pub struct SummarizeContext {
    pub session_id: String,
    pub ring_id: String,
    pub skill: String,
    pub messages_text: String,
}

pub fn start_summarize_stream(
    state: &AppState,
    user: &user::UserRow,
    ctx: SummarizeContext,
) -> Result<tokio::sync::mpsc::Receiver<SseEvent>> {
    let system_prompt = crate::services::skill::build_summary_system_prompt(&ctx.skill)
        .unwrap_or_else(|| "Summarize the following discussion.".to_string());

    let user_message = format!(
        "Here is the discussion transcript:\n\n{}\n\nPlease generate the summary.",
        ctx.messages_text
    );

    let llm = LlmClient::from_user(user)?;
    let rx = llm.chat_stream(
        system_prompt,
        vec![],
        user_message,
        "session_ring".to_string(),
    );
    Ok(rx)
}
```

- [ ] **Step 2: Verify compiles**

Run: `cd server && cargo check`

Fix any import issues. Need to ensure `use crate::models::session;` and `use crate::services::llm::{LlmClient, SseEvent};` and `use crate::models::user;` are imported at the top.

- [ ] **Step 3: Commit**

```bash
git add server/src/services/session.rs
git commit -m "feat(server): start, summarize, material service functions"
```

---

### Task 4: Add session route handlers for start, summarize, material-prep

**Files:**
- Modify: `server/src/routes/session.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Add route handlers to session.rs**

Add imports at top:

```rust
use async_stream::stream;
use axum::response::sse::{Event, KeepAlive, Sse};
use std::convert::Infallible;
use crate::services::llm::SseEvent;
use crate::models::user;
```

Add these handlers:

```rust
pub async fn start_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let result = session::start_session(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn summarize_session(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>>> {
    let user_row = user::get_user(&state.db, &user.token_id).await?;
    let _ = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    if !session_model::is_owner(&state.db, &session_id, &user.token_id).await? {
        return Err(RingError::Forbidden("only owner can summarize".into()));
    }

    let sess = session_model::get_session(&state.db, &session_id).await?;
    if sess.phase != "discussion" {
        return Err(RingError::BadRequest("session is not in discussion phase".into()));
    }

    session_model::update_phase(&state.db, &session_id, "summary").await?;

    let messages = session_model::get_messages(&state.db, &session_id, 0, 10000).await?;
    let messages_text = messages
        .iter()
        .map(|m| format!("[{}] {}: {}", m.created_at, m.sender_name, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let ctx = session::SummarizeContext {
        session_id: session_id.clone(),
        ring_id: ring_id.clone(),
        skill: sess.skill.clone(),
        messages_text,
    };

    let mut rx = session::start_summarize_stream(&state, &user_row, ctx)?;

    let pool = state.db.clone();
    let sid = session_id.clone();

    let s = stream! {
        let mut full_content = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                SseEvent::Start { message_id, role } => {
                    let data = serde_json::json!({"message_id": message_id, "role": role});
                    yield Ok(Event::default().event("message_start").data(data.to_string()));
                }
                SseEvent::Delta { content } => {
                    full_content.push_str(&content);
                    let data = serde_json::json!({ "content": content });
                    yield Ok(Event::default().event("delta").data(data.to_string()));
                }
                SseEvent::End { message_id, full_content: fc } => {
                    full_content = fc;
                    let data = serde_json::json!({
                        "message_id": message_id,
                        "usage": { "prompt_tokens": 0, "completion_tokens": 0 }
                    });
                    yield Ok(Event::default().event("message_end").data(data.to_string()));

                    let _ = session_model::set_summary(&pool, &sid, &full_content).await;
                    let _ = session_model::update_phase(&pool, &sid, "closed").await;
                }
                SseEvent::Error(msg) => {
                    let _ = session_model::update_phase(&pool, &sid, "discussion").await;
                    let data = serde_json::json!({ "error": msg });
                    yield Ok(Event::default().event("error").data(data.to_string()));
                }
            }
        }
    };

    Ok(Sse::new(s).keep_alive(KeepAlive::default()))
}

pub async fn get_material_prep(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    let materials = session::get_materials_service(&state, &ring_id, &session_id, &user.token_id).await?;
    Ok(Json(serde_json::json!({ "materials": materials })))
}

#[derive(Debug, Deserialize)]
pub struct HighlightInput {
    pub material_id: String,
    pub note: String,
}

pub async fn highlight_material(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(body): Json<HighlightInput>,
) -> Result<Json<Value>> {
    let result = session::highlight_material(
        &state,
        &ring_id,
        &session_id,
        &user.token_id,
        &body.material_id,
        &body.note,
    )
    .await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}
```

Important: need to rename `use crate::models::session` to `use crate::models::session as session_model` to avoid name clash with `use crate::services::session`.

- [ ] **Step 2: Register routes in mod.rs**

Add these routes inside the `api` Router chain, after the existing session routes:

```rust
.route(
    "/rings/{ring_id}/sessions/{session_id}/start",
    post(session::start_session),
)
.route(
    "/rings/{ring_id}/sessions/{session_id}/summarize",
    post(session::summarize_session),
)
.route(
    "/rings/{ring_id}/sessions/{session_id}/material-prep",
    get(session::get_material_prep),
)
.route(
    "/rings/{ring_id}/sessions/{session_id}/material-prep/highlights",
    post(session::highlight_material),
)
```

- [ ] **Step 3: Verify compiles**

Run: `cd server && cargo clippy`

Fix any issues (import renames, type mismatches, etc.)

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/session.rs server/src/routes/mod.rs
git commit -m "feat(server): start, summarize SSE, material-prep endpoints"
```

---

### Task 5: Update session types and store for material prep + summarize

**Files:**
- Modify: `ui/src/types/session.ts`
- Modify: `ui/src/stores/session-store.ts`

- [ ] **Step 1: Add SessionMaterial type to session.ts**

Add to end of `session.ts`:

```typescript
export interface SessionMaterial {
  id: string
  session_id: string
  item_type: 'document' | 'graph_node' | 'ai_generated'
  title: string
  content: string
  status: 'collecting' | 'analyzing' | 'ready'
  highlight: string | null
  created_at: string
}
```

- [ ] **Step 2: Add startSession, fetchMaterials, highlightMaterial, summarizeSession to session-store.ts**

Add these to the `SessionState` interface and implementation:

```typescript
// In interface:
materials: SessionMaterial[]
startSession: (ring_id: string, session_id: string) => Promise<void>
fetchMaterials: (ring_id: string, session_id: string) => Promise<void>
highlightMaterial: (ring_id: string, session_id: string, material_id: string, note: string) => Promise<void>

// In initial state:
materials: [],

// In implementation:
startSession: async (ring_id, session_id) => {
  try {
    const res = await api.post<FlatSessionResponse>(`/rings/${ring_id}/sessions/${session_id}/start`, {})
    set({ active_session: toSession(res) })
  } catch {
    // keep state
  }
},

fetchMaterials: async (ring_id, session_id) => {
  try {
    const res = await api.get<{ materials: SessionMaterial[] }>(`/rings/${ring_id}/sessions/${session_id}/material-prep`)
    set({ materials: res.materials ?? [] })
  } catch {
    // keep existing
  }
},

highlightMaterial: async (ring_id, session_id, material_id, note) => {
  try {
    await api.post(`/rings/${ring_id}/sessions/${session_id}/material-prep/highlights`, { material_id, note })
    set((s) => ({
      materials: s.materials.map((m) =>
        m.id === material_id ? { ...m, highlight: note } : m
      ),
    }))
  } catch {
    // keep state
  }
},
```

Import `SessionMaterial` from types.

- [ ] **Step 3: Verify compiles**

Run: `cd ui && npx tsc --noEmit`

- [ ] **Step 4: Commit**

```bash
git add ui/src/types/session.ts ui/src/stores/session-store.ts
git commit -m "feat(ui): material prep types and store functions"
```

---

### Task 6: Add MaterialPrepView and SummarizeView to SessionPanel

**Files:**
- Modify: `ui/src/components/panels/SessionPanel.tsx`

- [ ] **Step 1: Add MaterialPrepView component**

This shows when `session.phase === 'material_prep'`. Displays materials list, allows owner to highlight items and start the session.

```tsx
function MaterialPrepView() {
  const session = useSessionStore((s) => s.active_session)
  const materials = useSessionStore((s) => s.materials)
  const fetchMaterials = useSessionStore((s) => s.fetchMaterials)
  const highlightMaterial = useSessionStore((s) => s.highlightMaterial)
  const startSession = useSessionStore((s) => s.startSession)
  const active_ring_id = useRingStore((s) => s.active_ring_id)

  useEffect(() => {
    if (session && active_ring_id) {
      fetchMaterials(active_ring_id, session.id)
    }
  }, [session, active_ring_id, fetchMaterials])

  if (!session) return null

  const handleStart = async () => {
    if (!active_ring_id) return
    await startSession(active_ring_id, session.id)
  }

  const handleHighlight = async (material_id: string) => {
    if (!active_ring_id) return
    const note = prompt('Highlight note:')
    if (note) {
      await highlightMaterial(active_ring_id, session.id, material_id, note)
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)', marginBottom: 4 }}>
        <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 4 }}>
          {session.title}
        </div>
        <div style={{ fontSize: 10, color: 'var(--text-dim)', display: 'flex', gap: 8 }}>
          <span>Skill: {session.skill}</span>
          <span style={{ color: 'var(--accent-cyan)' }}>Phase: material_prep</span>
        </div>
      </div>

      <ScrollContainer>
        {materials.length === 0 ? (
          <div style={{ padding: '16px 0', color: 'var(--text-dim)', fontSize: 11, textAlign: 'center' }}>
            No materials yet
          </div>
        ) : (
          materials.map((mat) => (
            <div
              key={mat.id}
              style={{
                padding: '8px 0',
                borderBottom: '1px solid var(--border)',
              }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ fontSize: 11, fontWeight: 700, color: 'var(--text-primary)' }}>
                  {mat.title}
                </span>
                <div style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                  <span
                    style={{
                      fontSize: 9,
                      padding: '1px 6px',
                      borderRadius: 2,
                      background: mat.status === 'ready' ? 'var(--accent-green)' : mat.status === 'analyzing' ? 'var(--accent-amber)' : 'var(--bg-hover)',
                      color: mat.status === 'ready' ? 'var(--bg-base)' : 'var(--text-dim)',
                    }}
                  >
                    {mat.status}
                  </span>
                  <span
                    style={{
                      fontSize: 9,
                      padding: '1px 4px',
                      borderRadius: 2,
                      background: 'var(--bg-hover)',
                      color: 'var(--text-dim)',
                    }}
                  >
                    {mat.item_type}
                  </span>
                  <button
                    onClick={() => handleHighlight(mat.id)}
                    style={{
                      background: 'none',
                      border: 'none',
                      color: mat.highlight ? 'var(--accent-cyan)' : 'var(--text-dim)',
                      cursor: 'pointer',
                      fontSize: 10,
                      padding: '0 2px',
                    }}
                  >
                    ★
                  </button>
                </div>
              </div>
              <div style={{ fontSize: 10, color: 'var(--text-secondary)', marginTop: 2, lineHeight: 1.4 }}>
                {mat.content}
              </div>
              {mat.highlight && (
                <div style={{ fontSize: 10, color: 'var(--accent-cyan)', marginTop: 4, fontStyle: 'italic' }}>
                  ★ {mat.highlight}
                </div>
              )}
            </div>
          ))
        )}
      </ScrollContainer>

      <div style={{ borderTop: '1px solid var(--border)', paddingTop: 8 }}>
        <button
          onClick={handleStart}
          style={{
            background: 'var(--accent-cyan)',
            color: 'var(--bg-base)',
            border: 'none',
            borderRadius: 4,
            padding: '8px 16px',
            fontSize: 12,
            fontWeight: 700,
            cursor: 'pointer',
            width: '100%',
            letterSpacing: '0.05em',
          }}
        >
          START DISCUSSION
        </button>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Add SummarizeView component**

This shows when `session.phase === 'summary'`. Displays streaming AI summary with SSE.

```tsx
function SummarizeView() {
  const session = useSessionStore((s) => s.active_session)
  const [summary, setSummary] = useState('')
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!session) return

    const token = localStorage.getItem('ring_token')
    const ring_id = useRingStore.getState().active_ring_id
    if (!ring_id) return

    const url = `/api/rings/${ring_id}/sessions/${session.id}/summarize`

    fetch(url, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(token ? { 'X-Ring-Token': token } : {}),
      },
    })
      .then(async (res) => {
        if (!res.ok) {
          const err = await res.json().catch(() => ({}))
          setError(err?.error?.message ?? 'Summarize failed')
          return
        }
        const reader = res.body?.getReader()
        if (!reader) { setError('No response body'); return }

        const decoder = new TextDecoder()
        let buffer = ''

        while (true) {
          const { done, value } = await reader.read()
          if (done) break
          buffer += decoder.decode(value, { stream: true })
          const lines = buffer.split('\n')
          buffer = lines.pop() ?? ''

          let currentEvent = ''
          for (const line of lines) {
            if (line.startsWith('event: ')) {
              currentEvent = line.slice(7).trim()
            } else if (line.startsWith('data: ')) {
              const data = line.slice(6)
              try {
                const parsed = JSON.parse(data)
                if (currentEvent === 'delta' && parsed.content) {
                  setSummary((prev) => prev + parsed.content)
                }
                if (currentEvent === 'error') {
                  setError(parsed.error ?? 'Unknown error')
                }
                if (currentEvent === 'message_end') {
                  useSessionStore.getState().fetchActiveSession(ring_id)
                }
              } catch {
                // skip
              }
              currentEvent = ''
            }
          }
        }
      })
      .catch((e) => setError(e.message))
  }, [session])

  if (!session) return null

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div style={{ padding: '8px 0', borderBottom: '1px solid var(--border)', marginBottom: 4 }}>
        <div style={{ fontSize: 12, fontWeight: 700, color: 'var(--accent-ice)', marginBottom: 4 }}>
          {session.title}
        </div>
        <div style={{ fontSize: 10, color: 'var(--accent-amber)' }}>
          Generating summary...
        </div>
      </div>

      <ScrollContainer>
        {error ? (
          <div style={{ padding: 16, color: 'var(--accent-amber)', fontSize: 12 }}>
            Error: {error}
          </div>
        ) : (
          <div style={{ color: 'var(--text-primary)', fontSize: 11, whiteSpace: 'pre-wrap', lineHeight: 1.6 }}>
            {summary}
            {!summary && (
              <span style={{ color: 'var(--text-dim)' }}>Waiting for AI response...</span>
            )}
            {summary && !error && (
              <span style={{
                display: 'inline-block',
                width: 6,
                height: 14,
                background: 'var(--accent-cyan)',
                marginLeft: 2,
                verticalAlign: 'middle',
                animation: 'blink 1s step-end infinite',
              }} />
            )}
          </div>
        )}
      </ScrollContainer>
    </div>
  )
}
```

- [ ] **Step 3: Update SessionPanel main component to route by phase**

In the main `SessionPanel` function, replace the simple `if (active_session) return <SessionChat />` with:

```tsx
if (active_session) {
  if (active_session.phase === 'material_prep') return <MaterialPrepView />
  if (active_session.phase === 'summary') return <SummarizeView />
  return <SessionChat />
}
```

Also add a "Summarize" button to `SessionChat` when `session.skill !== 'discussion'` and `is_discussion`:

```tsx
{is_discussion && session.skill !== 'discussion' && (
  <button
    onClick={async () => {
      if (!active_ring_id) return
      const res = await api.post<FlatSessionResponse>(`/rings/${active_ring_id}/sessions/${session.id}/summarize`, {})
      set((s) => ({ active_session: toSession(res) }))
    }}
    style={{
      background: 'var(--bg-hover)',
      border: '1px solid var(--border)',
      borderRadius: 3,
      padding: '3px 8px',
      fontSize: 10,
      color: 'var(--accent-cyan)',
      cursor: 'pointer',
    }}
  >
    Summarize
  </button>
)}
```

Wait — the summarize endpoint is SSE, not JSON. The button just needs to trigger a phase change. Instead, we should update the session store to have a `startSummarize` that sets the phase to 'summary' (which will cause the UI to switch to SummarizeView which handles the SSE call). But actually the phase transition happens server-side when POST /summarize is called. So the SummarizeView should handle this directly.

Let me simplify: the "Summarize" button in SessionChat should change `active_session.phase` to `'summary'` locally (triggers SummarizeView), and SummarizeView will call the actual SSE endpoint. We don't need to POST first — the SSE endpoint handles the phase transition.

Actually re-reading the design: `POST .../summarize` is the endpoint itself (SSE stream). So clicking "Summarize" should navigate to SummarizeView which calls the SSE POST. The SummarizeView is self-contained.

So the button should just set phase locally:

```tsx
{is_discussion && session.skill !== 'discussion' && (
  <button
    onClick={() => {
      useSessionStore.setState((s) => ({
        active_session: s.active_session ? { ...s.active_session, phase: 'summary' as const } : null,
      }))
    }}
    style={{...}}
  >
    Summarize
  </button>
)}
```

- [ ] **Step 4: Verify compiles**

Run: `cd ui && npx tsc --noEmit`

- [ ] **Step 5: Commit**

```bash
git add ui/src/components/panels/SessionPanel.tsx
git commit -m "feat(ui): material prep and summarize views in SessionPanel"
```

---

### Task 7: Final verification

- [ ] **Step 1: Full backend check**

Run: `cd server && cargo clippy && cargo test`

- [ ] **Step 2: Full frontend check**

Run: `cd ui && npx tsc --noEmit && npm run build`

- [ ] **Step 3: Fix any issues and final commit**

---

## Notes

- Skill loading from filesystem (`~/.ring/skills/`) is deferred — this plan only uses built-in embedded definitions. Filesystem loading can be added later.
- Material prep AI auto-generation (where AI creates session_materials entries) is deferred to a follow-up. For now, materials are created manually or the list starts empty and the "START DISCUSSION" button is available immediately.
- The SummarizeView calls the SSE endpoint directly (not through session store) to handle streaming in the component.
- The `prompt()` call for highlight is a placeholder — can be replaced with a proper inline input later.
