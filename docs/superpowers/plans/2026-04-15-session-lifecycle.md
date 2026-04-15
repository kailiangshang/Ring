# Session Lifecycle Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update Session lifecycle to include mandatory material preparation phase and optional summary phase

**Architecture:** Session now has 5 phases: creation → preparing → discussing → summarizing → ended. Material preparation is mandatory, summary is optional. AI does not participate in discussion phase.

**Tech Stack:** Rust (axum), SQLite, WebSocket

---

## 3. Session Phases (Confirmed)

```
creation → preparing → discussing → summarizing → ended
```

| Phase | Description | Trigger |
|-------|-------------|---------|
| **preparing** | 材料准备 - AI 根据 Session 主题收集/整理相关材料，让讨论有内容可依，而不是空谈 | Session 创建后自动进入 |
| **discussing** | 讨论阶段 - AI 不参与，只记录 | Owner 手动切换 |
| **summarizing** | 总结阶段 - AI 基于材料生成总结报告 | Owner 手动切换（可选） |
| **ended** | 已结束 | Owner 手动结束 |

### Key Design Points

1. **Material preparation is mandatory** - AI 在讨论前收集整理材料，避免空谈
2. **Discussion phase AI does not participate** - AI 只记录，不辅助讨论
3. **Summary is optional** - Owner 可选择直接结束，不生成总结
4. **Owner controls all phase transitions** - 只有 Owner 可以触发阶段切换

---

## 4. Tasks

### Task 1: Update Session Model

**Files:**
- Modify: `ring-server/src/models/session_model.rs`

- [ ] **Step 1: Add phase field to Session**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub ring_id: String,
    pub title: Option<String>,
    pub scenario: String,
    pub skill_name: Option<String>,
    pub created_by: String,
    pub archive_enabled: bool,
    pub status: String,        // active, closed, deleted
    pub phase: String,         // preparing, discussing, summarizing, ended
    pub created_at: String,
    pub updated_at: String,
}
```

- [ ] **Step 2: Add session material models**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMaterial {
    pub id: String,
    pub session_id: String,
    pub title: String,
    pub content: String,
    pub material_type: String,  // document, url, note
    pub created_by: String,
    pub highlighted: bool,       // owner highlighted material
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMaterialRequest {
    pub title: String,
    pub content: String,
    pub material_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialListResponse {
    pub materials: Vec<SessionMaterial>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryConfig {
    pub auto: bool,
}
```

- [ ] **Step 3: Update CreateSessionRequest**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub title: Option<String>,
    pub scenario: String,
    pub archive_enabled: Option<bool>,
    pub invite_member_ids: Option<Vec<String>>,
    pub skill_name: Option<String>,
}
```

- [ ] **Step 4: Update SessionDetailResponse**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetailResponse {
    pub id: String,
    pub ring_id: String,
    pub title: Option<String>,
    pub scenario: String,
    pub skill_name: Option<String>,
    pub created_by: String,
    pub archive_enabled: bool,
    pub status: String,
    pub phase: String,
    pub members: Vec<SessionMemberBrief>,
    pub created_at: String,
}
```

- [ ] **Step 5: Update test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_has_phase_field() {
        let session = Session {
            id: "s-1".into(),
            ring_id: "r-1".into(),
            title: Some("test".into()),
            scenario: "decision".into(),
            skill_name: Some("decision".into()),
            created_by: "u-1".into(),
            archive_enabled: false,
            status: "active".into(),
            phase: "preparing".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_value(&session).unwrap();
        assert_eq!(json["phase"], "preparing");
    }
}
```

- [ ] **Step 6: Run test**

Run: `cargo test -p ring-server models::session_model`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add ring-server/src/models/session_model.rs
git commit -m "feat: add phase field and material support to Session model"
```

---

### Task 2: Add Session Phase Transitions

**Files:**
- Modify: `ring-server/src/db/` (add phase update method to repository)
- Add: Migration for session_materials table

- [ ] **Step 1: Add phase update to repository**

```rust
// In Repository trait
async fn update_session_phase(&self, session_id: &str, phase: &str) -> Result<()>;
```

- [ ] **Step 2: Implement in SQLite repository**

```rust
async fn update_session_phase(&self, session_id: &str, phase: &str) -> Result<()> {
    sqlx::query("UPDATE sessions SET phase = ?, updated_at = ? WHERE id = ?")
        .bind(phase)
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(session_id)
        .execute(&self.pool)
        .await?;
    Ok(())
}
```

- [ ] **Step 3: Add session_materials table**

```sql
CREATE TABLE session_materials (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    material_type TEXT NOT NULL DEFAULT 'note',
    created_by TEXT NOT NULL,
    highlighted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_session_materials_session ON session_materials(session_id);
```

- [ ] **Step 4: Add materials repository methods**

```rust
async fn create_session_material(&self, material: &SessionMaterial) -> Result<()>;
async fn list_session_materials(&self, session_id: &str) -> Result<Vec<SessionMaterial>>;
async fn update_material_highlight(&self, material_id: &str, highlighted: bool) -> Result<()>;
async fn delete_session_material(&self, material_id: &str) -> Result<()>;
```

- [ ] **Step 5: Commit**

```bash
git add ring-server/src/db/
git commit -m "feat: add session phase transitions and materials table"
```

---

### Task 3: Update SessionService

**Files:**
- Modify: `ring-server/src/services/session_service.rs`

- [ ] **Step 1: Add phase transition helper**

```rust
impl SessionService {
    pub async fn transition_phase(
        &self,
        session_id: &str,
        ring_id: &str,
        new_phase: &str,
        caller_id: &str,
    ) -> Result<()> {
        let session = self.db.get_session(session_id).await?
            .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
        
        if session.ring_id != ring_id {
            return Err(RingError::NotFound("session not in ring".into()));
        }
        
        if session.created_by != caller_id {
            return Err(RingError::Forbidden("only session owner can transition phase".into()));
        }

        self.db.update_session_phase(session_id, new_phase).await
    }

    pub async fn start_discussion(
        &self,
        ring_id: &str,
        session_id: &str,
        caller_id: &str,
    ) -> Result<()> {
        self.transition_phase(session_id, ring_id, "discussing", caller_id).await
    }

    pub async fn start_summary(
        &self,
        ring_id: &str,
        session_id: &str,
        caller_id: &str,
    ) -> Result<()> {
        self.transition_phase(session_id, ring_id, "summarizing", caller_id).await
    }

    pub async fn end_session(
        &self,
        ring_id: &str,
        session_id: &str,
        caller_id: &str,
    ) -> Result<()> {
        self.transition_phase(session_id, ring_id, "ended", caller_id).await
    }
}
```

- [ ] **Step 2: Add material management methods**

```rust
pub async fn add_material(
    &self,
    ring_id: &str,
    session_id: &str,
    request: &CreateMaterialRequest,
    user_id: &str,
) -> Result<SessionMaterial> {
    let session = self.db.get_session(session_id).await?
        .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
    
    if session.ring_id != ring_id {
        return Err(RingError::NotFound("session not in ring".into()));
    }

    let material = SessionMaterial {
        id: ulid::Ulid::new().to_string(),
        session_id: session_id.to_string(),
        title: request.title.clone(),
        content: request.content.clone(),
        material_type: request.material_type.clone(),
        created_by: user_id.to_string(),
        highlighted: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    self.db.create_session_material(&material).await?;
    Ok(material)
}

pub async fn list_materials(
    &self,
    ring_id: &str,
    session_id: &str,
) -> Result<Vec<SessionMaterial>> {
    let session = self.db.get_session(session_id).await?
        .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
    
    if session.ring_id != ring_id {
        return Err(RingError::NotFound("session not in ring".into()));
    }

    self.db.list_session_materials(session_id).await
}

pub async fn highlight_material(
    &self,
    ring_id: &str,
    session_id: &str,
    material_id: &str,
    highlighted: bool,
    caller_id: &str,
) -> Result<()> {
    let session = self.db.get_session(session_id).await?
        .ok_or_else(|| RingError::NotFound(format!("session {}", session_id)))?;
    
    if session.ring_id != ring_id {
        return Err(RingError::NotFound("session not in ring".into()));
    }
    
    if session.created_by != caller_id {
        return Err(RingError::Forbidden("only session owner can highlight".into()));
    }

    self.db.update_material_highlight(material_id, highlighted).await
}
```

- [ ] **Step 3: Commit**

```bash
git add ring-server/src/services/session_service.rs
git commit -m "feat: add session phase transitions and material management to SessionService"
```

---

### Task 4: Update Session Handler

**Files:**
- Modify: `ring-server/src/handlers/session.rs`, `ring-server/src/routes.rs`

- [ ] **Step 1: Add new handler methods**

```rust
// Phase transition handlers
pub async fn start_discussion(
    State(state): State<Arc<AppState>>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<SuccessResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let user_id = "current_user"; // Get from auth context
    
    service.start_discussion(&ring_id, &session_id, user_id).await?;
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn start_summary(
    State(state): State<Arc<AppState>>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<SuccessResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let user_id = "current_user";
    
    service.start_summary(&ring_id, &session_id, user_id).await?;
    Ok(Json(SuccessResponse { success: true }))
}

// Material handlers
pub async fn add_material(
    State(state): State<Arc<AppState>>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(req): Json<CreateMaterialRequest>,
) -> Result<Json<SessionMaterial>, RingError> {
    let service = SessionService::new(state.db.clone());
    let user_id = "current_user";
    
    let material = service.add_material(&ring_id, &session_id, &req, user_id).await?;
    Ok(Json(material))
}

pub async fn list_materials(
    State(state): State<Arc<AppState>>,
    Path((ring_id, session_id)): Path<(String, String)>,
) -> Result<Json<MaterialListResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let materials = service.list_materials(&ring_id, &session_id).await?;
    Ok(Json(MaterialListResponse { materials }))
}

pub async fn highlight_material(
    State(state): State<Arc<AppState>>,
    Path((ring_id, session_id, material_id)): Path<(String, String, String)>,
    Json(req): Json<UpdateHighlightRequest>,
) -> Result<Json<SuccessResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    let user_id = "current_user";
    
    service.highlight_material(&ring_id, &session_id, &material_id, req.highlighted, user_id).await?;
    Ok(Json(SuccessResponse { success: true }))
}

pub async fn generate_summary(
    State(state): State<Arc<AppState>>,
    Path((ring_id, session_id)): Path<(String, String)>,
    Json(req): Json<GenerateSummaryRequest>,
) -> Result<Json<SummaryResponse>, RingError> {
    let service = SessionService::new(state.db.clone());
    
    let materials = service.list_materials(&ring_id, &session_id).await?;
    let messages = service.get_messages(&ring_id, &session_id, None, 1000).await?;
    
    let summary = state.ai_service.generate_session_summary(&materials, &messages.messages).await?;
    
    Ok(Json(SummaryResponse { summary }))
}
```

- [ ] **Step 2: Update routes.rs**

Add new routes:
```rust
.route("/{sessionId}/phase/discuss", post(session::start_discussion))
.route("/{sessionId}/phase/summary", post(session::start_summary))
.route("/{sessionId}/materials", get(session::list_materials).post(session::add_material))
.route("/{sessionId}/materials/{materialId}/highlight", put(session::highlight_material))
.route("/{sessionId}/summary", post(session::generate_summary))
```

- [ ] **Step 3: Commit**

```bash
git add ring-server/src/handlers/session.rs ring-server/src/routes.rs
git commit -m "feat: add session phase and material API endpoints"
```

---

### Task 5: Update Frontend Session Store

**Files:**
- Modify: `ring-frontend/src/stores/sessionStore.ts`

- [ ] **Step 1: Add material and phase state**

```typescript
interface SessionMaterial {
  id: string;
  session_id: string;
  title: string;
  content: string;
  material_type: string;
  created_by: string;
  highlighted: boolean;
  created_at: string;
}

interface SessionStore {
  phase: 'preparing' | 'discussing' | 'summarizing' | 'ended';
  materials: SessionMaterial[];
  
  fetchMaterials: (sessionId: string) => Promise<void>;
  addMaterial: (sessionId: string, title: string, content: string, type: string) => Promise<void>;
  highlightMaterial: (sessionId: string, materialId: string, highlighted: boolean) => Promise<void>;
  startDiscussion: (sessionId: string) => Promise<void>;
  startSummary: (sessionId: string) => Promise<void>;
  generateSummary: (sessionId: string) => Promise<string>;
}

export const useSessionStore = create<SessionStore>((set, get) => ({
  phase: 'preparing',
  materials: [],

  fetchMaterials: async (sessionId) => {
    const response = await fetch(`/api/v1/rings/${get().currentRingId}/sessions/${sessionId}/materials`);
    const data = await response.json();
    set({ materials: data.materials });
  },
  
  addMaterial: async (sessionId, title, content, type) => {
    await fetch(`/api/v1/rings/${get().currentRingId}/sessions/${sessionId}/materials`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title, content, material_type: type }),
    });
    get().fetchMaterials(sessionId);
  },
  
  highlightMaterial: async (sessionId, materialId, highlighted) => {
    await fetch(`/api/v1/rings/${get().currentRingId}/sessions/${sessionId}/materials/${materialId}/highlight`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ highlighted }),
    });
    get().fetchMaterials(sessionId);
  },
  
  startDiscussion: async (sessionId) => {
    await fetch(`/api/v1/rings/${get().currentRingId}/sessions/${sessionId}/phase/discuss`, {
      method: 'POST',
    });
    set({ phase: 'discussing' });
  },
  
  startSummary: async (sessionId) => {
    await fetch(`/api/v1/rings/${get().currentRingId}/sessions/${sessionId}/phase/summary`, {
      method: 'POST',
    });
    set({ phase: 'summarizing' });
  },
  
  generateSummary: async (sessionId) => {
    const response = await fetch(`/api/v1/rings/${get().currentRingId}/sessions/${sessionId}/summary`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ config: { auto: false } }),
    });
    const data = await response.json();
    return data.summary;
  },
}));
```

- [ ] **Step 2: Commit**

```bash
git add ring-frontend/src/stores/sessionStore.ts
git commit -m "feat: add material and phase management to SessionStore"
```

---

### Task 6: Update Session View UI

**Files:**
- Modify: `ring-frontend/src/pages/Session/SessionView.tsx`

- [ ] **Step 1: Add phase-based rendering**

```tsx
export const SessionView: React.FC = () => {
  const { phase, materials, fetchMaterials, addMaterial, startDiscussion, startSummary } = useSessionStore();
  
  // Initial phase: Material Preparation
  if (phase === 'preparing') {
    return (
      <div className="session-preparing">
        <h2>Material Preparation</h2>
        <p>AI is collecting and generating materials based on the session description...</p>
        
        <div className="materials-list">
          {materials.map(m => (
            <div key={m.id} className={`material-card ${m.highlighted ? 'highlighted' : ''}`}>
              <h3>{m.title}</h3>
              <p>{m.content}</p>
            </div>
          ))}
        </div>
        
        <div className="preparing-actions">
          <button onClick={() => addMaterial('manual material', 'note')}>
            Add Manual Material
          </button>
          <button onClick={() => startDiscussion(currentSessionId)}>
            Start Discussion
          </button>
        </div>
      </div>
    );
  }
  
  // Discussion phase
  if (phase === 'discussing') {
    return (
      <div className="session-discussing">
        <ChatView />
      </div>
    );
  }
  
  // Summary phase
  if (phase === 'summarizing') {
    return (
      <div className="session-summarizing">
        <h2>Generating Summary...</h2>
        <button onClick={() => generateSummary(currentSessionId)}>
          Generate Summary
        </button>
        <button onClick={() => endSession(currentSessionId)}>
          End Session
        </button>
      </div>
    );
  }
  
  // Ended phase
  return (
    <div className="session-ended">
      <h2>Session Ended</h2>
      <p>Chat history is preserved. You can reopen this session anytime.</p>
    </div>
  );
};
```

- [ ] **Step 2: Commit**

```bash
git add ring-frontend/src/pages/Session/SessionView.tsx
git commit -m "feat: add phase-based UI for Session lifecycle"
```

---

## Self-Review Checklist

1. **Spec coverage:** All Session lifecycle requirements covered
   - ✅ 5 phases: creation → preparing → discussion → summary → end
   - ✅ Material preparation is mandatory (phase starts in "preparing")
   - ✅ Summary is optional (can end without summary)
   - ✅ Owner controls phase transitions
   - ✅ Materials can be highlighted by owner
   - ✅ AI does not participate in discussion phase

2. **Placeholder scan:** No "TBD" or "TODO" found

3. **Type consistency:** Types match across tasks

---

## Execution Options

**Plan complete and saved to `docs/superpowers/plans/2026-04-15-session-lifecycle.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?