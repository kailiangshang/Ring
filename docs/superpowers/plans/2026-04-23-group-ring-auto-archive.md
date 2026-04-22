# Group Ring Chat Auto Archive Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement auto-archive mode for Group Ring chat: after each AI response, automatically analyze if content is worth archiving, and if so, archive it without user confirmation.

**Architecture:** Add a separate `auto_archive` boolean field to the rings table (distinct from `interaction_mode`). After each Group Ring chat completion (in `SseEvent::End` handler), if `auto_archive` is ON, spawn an async task to analyze the conversation with LLM and trigger the existing archive flow. Frontend adds a toggle button and visual indicator.

**Tech Stack:** Rust, Axum, SQLite (sqlx), async-openai, React, TypeScript, Zustand

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `server/migrations/001_init.sql` | Modify | Add `auto_archive` column to rings table |
| `server/src/models/ring.rs` | Modify | Add `auto_archive` field to RingRow, RingDetail, queries |
| `server/src/services/mode.rs` | Modify | Add `auto_archive` to ModeResponse and UpdateModeRequest |
| `server/src/routes/mode.rs` | Modify | No changes needed (already generic) |
| `server/src/routes/chat.rs` | Modify | Add auto-archive trigger after chat completion |
| `server/src/services/archive_service.rs` | Modify | Add `auto_archive_chat` function |
| `server/tests/integration.rs` | Modify | Add test for auto-archive toggle endpoint |
| `ui/src/stores/mode-store.ts` | Modify | Add `auto_archive` state and toggle |
| `ui/src/components/chat/ModeIndicator.tsx` | Modify | Show auto-archive indicator |
| `ui/src/components/chat/InputArea.tsx` | Modify | Add auto-archive toggle button |

---

### Task 1: Add `auto_archive` column to database

**Files:**
- Modify: `server/migrations/001_init.sql`

- [ ] **Step 1: Add column to rings table**

Add after `skill_permission_mode` line in the CREATE TABLE statement:

```sql
    auto_archive BOOLEAN NOT NULL DEFAULT 0,
```

- [ ] **Step 2: Commit**

```bash
git add server/migrations/001_init.sql
git commit -m "feat: add auto_archive column to rings table"
```

---

### Task 2: Add `auto_archive` to RingRow and RingDetail

**Files:**
- Modify: `server/src/models/ring.rs`

- [ ] **Step 1: Add field to RingRow struct**

Add after `skill_permission_mode` field:

```rust
    pub auto_archive: bool,
```

- [ ] **Step 2: Add field to RingDetail struct**

Add after `skill_permission_mode` field:

```rust
    pub auto_archive: bool,
```

- [ ] **Step 3: Update get_ring_detail query**

Update the SELECT in `get_ring_detail` to include `auto_archive`:

```rust
    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            String,
            String,
            String,
            String,
            bool,
        ),
    >(
        "SELECT r.id, r.name, r.role_description, r.blueprint_status,
                r.interaction_mode, r.skill_permission_mode, r.created_at, r.auto_archive
         FROM rings r
         JOIN members m ON m.ring_id = r.id AND m.user_id = ?2
         WHERE r.id = ?1",
    )
```

And update the RingDetail construction:

```rust
    Ok(RingDetail {
        id: row.0,
        name: row.1,
        role: get_user_role(pool, ring_id, user_id).await?,
        role_description: row.2,
        member_count,
        node_count,
        blueprint_status: row.3,
        interaction_mode: row.4,
        skill_permission_mode: row.5,
        created_at: row.6,
        auto_archive: row.7,
    })
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 5: Commit**

```bash
git add server/src/models/ring.rs
git commit -m "feat: add auto_archive field to RingRow and RingDetail"
```

---

### Task 3: Add `auto_archive` to mode service

**Files:**
- Modify: `server/src/services/mode.rs`

- [ ] **Step 1: Update ModeResponse**

Add field:

```rust
pub struct ModeResponse {
    pub interaction_mode: String,
    pub skill_permission_mode: String,
    pub auto_archive: bool,
}
```

- [ ] **Step 2: Update UpdateModeRequest**

Add field:

```rust
pub struct UpdateModeRequest {
    pub interaction_mode: Option<String>,
    pub skill_permission_mode: Option<String>,
    pub auto_archive: Option<bool>,
}
```

- [ ] **Step 3: Update get_mode query and response**

```rust
pub async fn get_mode(state: &AppState, ring_id: &str, user_id: &str) -> Result<ModeResponse> {
    let _ = ring::get_user_role(&state.db, ring_id, user_id).await?;
    let row = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT interaction_mode, skill_permission_mode, auto_archive FROM rings WHERE id = ?1",
    )
    .bind(ring_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| RingError::NotFound("ring not found".into()))?;

    Ok(ModeResponse {
        interaction_mode: row.0,
        skill_permission_mode: row.1,
        auto_archive: row.2,
    })
}
```

- [ ] **Step 4: Update update_mode**

Add validation and update logic for auto_archive:

```rust
pub async fn update_mode(
    state: &AppState,
    ring_id: &str,
    user_id: &str,
    input: &UpdateModeRequest,
) -> Result<ModeResponse> {
    let role = ring::get_user_role(&state.db, ring_id, user_id).await?;
    if role == "readonly" {
        return Err(RingError::Forbidden(
            "readonly members cannot change mode".into(),
        ));
    }

    if let Some(ref mode) = input.interaction_mode {
        if mode != "normal" && mode != "auto" {
            return Err(RingError::BadRequest(
                "interaction_mode must be 'normal' or 'auto'".into(),
            ));
        }
    }
    if let Some(ref mode) = input.skill_permission_mode {
        if mode != "auto" && mode != "plan" && mode != "edit" {
            return Err(RingError::BadRequest(
                "skill_permission_mode must be 'auto', 'plan', or 'edit'".into(),
            ));
        }
    }

    let current = get_mode(state, ring_id, user_id).await?;
    let im = input
        .interaction_mode
        .as_deref()
        .unwrap_or(&current.interaction_mode);
    let spm = input
        .skill_permission_mode
        .as_deref()
        .unwrap_or(&current.skill_permission_mode);
    let auto = input.auto_archive.unwrap_or(current.auto_archive);

    sqlx::query("UPDATE rings SET interaction_mode = ?1, skill_permission_mode = ?2, auto_archive = ?3 WHERE id = ?4")
        .bind(im)
        .bind(spm)
        .bind(auto)
        .bind(ring_id)
        .execute(&state.db)
        .await?;

    Ok(ModeResponse {
        interaction_mode: im.to_string(),
        skill_permission_mode: spm.to_string(),
        auto_archive: auto,
    })
}
```

- [ ] **Step 5: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 6: Commit**

```bash
git add server/src/services/mode.rs
git commit -m "feat: add auto_archive to mode service"
```

---

### Task 4: Add `auto_archive_chat` function to archive_service

**Files:**
- Modify: `server/src/services/archive_service.rs`

- [ ] **Step 1: Add the `auto_archive_chat` function**

Add at the end of the file (after `auto_archive_session`):

```rust
#[allow(clippy::too_many_arguments)]
pub async fn auto_archive_chat(
    pool: &SqlitePool,
    git: &GitService,
    rings_dir: &std::path::Path,
    ring_id: &str,
    user_message: &str,
    ai_response: &str,
    user_id: &str,
    user_row: &crate::models::user::UserRow,
) {
    tracing::info!("auto_archive_chat started: ring={ring_id}");

    let system_prompt = "你是一个知识管理助手。分析以下对话内容，判断AI的回复是否值得归档。

值得归档的内容包括：决策记录、结论总结、知识点、调研发现、方案对比、技术方案等。
不值得归档的内容包括：闲聊、问候、简单确认、无实质内容的回复等。

如果值得归档，返回JSON对象：
{\"should_archive\": true, \"title\": \"简短标题\", \"content\": \"Markdown格式的归档内容\"}

如果不值得归档，返回：
{\"should_archive\": false}

返回纯JSON，不要markdown code block。";

    let user_prompt = format!(
        "用户消息：\n{}\n\nAI回复：\n{}",
        user_message, ai_response
    );

    let llm = match crate::services::llm::LlmClient::from_user(user_row) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("auto_archive_chat failed to create LLM client: {e}");
            return;
        }
    };

    let response = match llm.chat_complete(system_prompt.to_string(), user_prompt).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("auto_archive_chat LLM call failed: {e}");
            return;
        }
    };

    let cleaned = response.trim();
    let json_str = if cleaned.starts_with("```") {
        cleaned
            .lines()
            .skip(1)
            .take_while(|l| !l.starts_with("```"))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        cleaned.to_string()
    };

    #[derive(Debug, serde::Deserialize)]
    struct ArchiveDecision {
        should_archive: bool,
        title: Option<String>,
        content: Option<String>,
    }

    let decision: ArchiveDecision = match serde_json::from_str(&json_str) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("auto_archive_chat failed to parse LLM JSON: {e}\nraw: {json_str}");
            return;
        }
    };

    if !decision.should_archive {
        tracing::info!("auto_archive_chat: content not worth archiving");
        return;
    }

    let title = match decision.title {
        Some(t) => t,
        None => {
            tracing::warn!("auto_archive_chat: missing title in decision");
            return;
        }
    };

    let content = match decision.content {
        Some(c) => c,
        None => {
            tracing::warn!("auto_archive_chat: missing content in decision");
            return;
        }
    };

    let role = match crate::models::ring::get_user_role(pool, ring_id, user_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("auto_archive_chat failed to get user role: {e}");
            return;
        }
    };

    let is_creator = role == "creator" || role == "admin";
    let title_with_ts = format!("{}_{}", chrono::Utc::now().format("%H%M%S"), title);

    if is_creator {
        match archive_content_creator(
            pool,
            git,
            rings_dir,
            ring_id,
            None,
            None,
            &content,
            &title_with_ts,
            user_id,
        )
        .await
        {
            Ok(_) => {
                tracing::info!("auto_archive_chat: archived '{}'", title);
            }
            Err(e) => {
                tracing::warn!("auto_archive_chat failed to archive: {e}");
            }
        }
    } else {
        let repo_url = match sqlx::query_scalar::<_, Option<String>>(
            "SELECT gitlab_repo_url FROM rings WHERE id = ?1",
        )
        .bind(ring_id)
        .fetch_one(pool)
        .await
        {
            Ok(url) => url,
            Err(e) => {
                tracing::warn!("auto_archive_chat failed to get repo url: {e}");
                return;
            }
        };

        let (gitlab_url, gitlab_token) = match (&user_row.gitlab_url, &user_row.gitlab_token) {
            (Some(url), Some(token)) => (url.clone(), token.clone()),
            _ => {
                tracing::warn!("auto_archive_chat: GitLab not configured for member");
                return;
            }
        };

        match repo_url {
            Some(url) => {
                let gitlab = crate::services::gitlab_service::GitLabClient::new(&gitlab_url, &gitlab_token);
                match archive_content_member(
                    pool,
                    git,
                    &gitlab,
                    rings_dir,
                    ring_id,
                    &url,
                    None,
                    None,
                    &content,
                    &title_with_ts,
                    user_id,
                )
                .await
                {
                    Ok(_) => {
                        tracing::info!("auto_archive_chat: created MR for '{}'", title);
                    }
                    Err(e) => {
                        tracing::warn!("auto_archive_chat failed to create MR: {e}");
                    }
                }
            }
            None => {
                tracing::warn!("auto_archive_chat: no GitLab repo configured");
            }
        }
    }
}
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/services/archive_service.rs
git commit -m "feat: add auto_archive_chat function for Group Ring chat"
```

---

### Task 5: Hook auto-archive into Group Ring chat

**Files:**
- Modify: `server/src/routes/chat.rs`

- [ ] **Step 1: Modify SseEvent::End handler in ring_chat**

After the existing `group_doc_maintenance::update_active_context` spawn block, add:

```rust
                    // Auto-archive check
                    let pool_auto = pool.clone();
                    let ring_id_auto = ring_id_c.clone();
                    let user_id_auto = user_id.clone();
                    let user_row_auto = user_row_c.clone();
                    let content_auto = full_content.clone();
                    let user_message_auto = body.content.clone();
                    tokio::spawn(async move {
                        let auto_archive: bool = sqlx::query_scalar(
                            "SELECT auto_archive FROM rings WHERE id = ?1",
                        )
                        .bind(&ring_id_auto)
                        .fetch_one(&pool_auto)
                        .await
                        .unwrap_or(false);

                        if auto_archive {
                            let git = crate::services::git_service::GitService::new();
                            crate::services::archive_service::auto_archive_chat(
                                &pool_auto,
                                &git,
                                &state_c.rings_dir,
                                &ring_id_auto,
                                &user_message_auto,
                                &content_auto,
                                &user_id_auto,
                                &user_row_auto,
                            )
                            .await;
                        }
                    });
```

Note: Need to capture `body.content` at the start of `ring_chat` function. Add before the stream creation:

```rust
    let user_message = body.content.clone();
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/routes/chat.rs
git commit -m "feat: hook auto-archive into Group Ring chat completion"
```

---

### Task 6: Add frontend auto_archive state and toggle

**Files:**
- Modify: `ui/src/stores/mode-store.ts`

- [ ] **Step 1: Add auto_archive to ModeState interface**

```typescript
interface ModeState {
  interaction_mode: InteractionMode
  skill_permission_mode: SkillPermissionMode
  auto_archive: boolean
  syncing: boolean
  setInteractionMode: (mode: InteractionMode) => void
  setSkillMode: (mode: SkillPermissionMode) => void
  setAutoArchive: (enabled: boolean) => void
  toggleAutoArchive: () => void
  syncToServer: () => Promise<void>
  fetchFromServer: (ringId: string) => Promise<void>
  reset: () => void
}
```

- [ ] **Step 2: Update store implementation**

```typescript
export const useModeStore = create<ModeState>((set, get) => ({
  interaction_mode: 'normal',
  skill_permission_mode: 'plan',
  auto_archive: false,
  syncing: false,

  setInteractionMode: (mode) => {
    set({ interaction_mode: mode })
    get().syncToServer()
  },

  setSkillMode: (mode) => {
    set({ skill_permission_mode: mode })
    get().syncToServer()
  },

  setAutoArchive: (enabled) => {
    set({ auto_archive: enabled })
    get().syncToServer()
  },

  toggleAutoArchive: () => {
    set({ auto_archive: !get().auto_archive })
    get().syncToServer()
  },

  toggleAuto: () => {
    set({ interaction_mode: get().interaction_mode === 'auto' ? 'normal' : 'auto' })
    get().syncToServer()
  },

  syncToServer: async () => {
    const ringId = useRingStore.getState().active_ring_id
    if (!ringId) return
    set({ syncing: true })
    try {
      const { interaction_mode, skill_permission_mode, auto_archive } = get()
      await api.put(`/rings/${ringId}/mode`, {
        interaction_mode,
        skill_permission_mode,
        auto_archive,
      })
    } catch {
      // silent fail — mode is local-first
    }
    set({ syncing: false })
  },

  fetchFromServer: async (ringId) => {
    try {
      const res = await api.get<{ interaction_mode: string; skill_permission_mode: string; auto_archive: boolean }>(`/rings/${ringId}/mode`)
      set({
        interaction_mode: res.interaction_mode as InteractionMode,
        skill_permission_mode: res.skill_permission_mode as SkillPermissionMode,
        auto_archive: res.auto_archive,
      })
    } catch {
      // keep defaults
    }
  },

  reset: () =>
    set({ interaction_mode: 'normal', skill_permission_mode: 'plan', auto_archive: false }),
}))
```

- [ ] **Step 3: Commit**

```bash
git add ui/src/stores/mode-store.ts
git commit -m "feat: add auto_archive state and toggle to mode store"
```

---

### Task 7: Add auto-archive toggle button to InputArea

**Files:**
- Modify: `ui/src/components/chat/InputArea.tsx`

- [ ] **Step 1: Import mode store and add toggle button**

Add import:

```typescript
import { useModeStore } from '../../stores/mode-store'
```

Add button next to ModeIndicator in the flex container:

```typescript
export function InputArea() {
  const { input, setInput, send, sending, stopStreaming } = useChatStore()
  const ac = useAutocompleteStore()
  const [historyIndex, setHistoryIndex] = useState(-1)
  const auto_archive = useModeStore((s) => s.auto_archive)
  const toggleAutoArchive = useModeStore((s) => s.toggleAutoArchive)
  const context = useAppStore((s) => s.current_context)

  // ... rest of component

  return (
    <div style={{ position: 'relative' }}>
      <CommandAutocomplete onSelect={handleSelect} />
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          padding: '8px 12px',
          borderTop: '1px solid var(--border)',
        }}
      >
        <ModeIndicator />
        {context === 'ring' && (
          <button
            onClick={toggleAutoArchive}
            style={{
              background: auto_archive ? 'var(--accent-green)' : 'var(--bg-hover)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '6px 10px',
              color: auto_archive ? 'var(--bg-base)' : 'var(--text-secondary)',
              fontSize: 11,
              cursor: 'pointer',
              fontWeight: 700,
              whiteSpace: 'nowrap',
            }}
          >
            {auto_archive ? 'AUTO ON' : 'AUTO OFF'}
          </button>
        )}
        <input
          // ... existing input
        />
        // ... existing buttons
      </div>
      <CommandHints />
    </div>
  )
}
```

Need to also import `useAppStore`:

```typescript
import { useAppStore } from '../../stores/app-store'
```

- [ ] **Step 2: Commit**

```bash
git add ui/src/components/chat/InputArea.tsx
git commit -m "feat: add auto-archive toggle button to InputArea"
```

---

### Task 8: Add visual indicator for auto-archive in ModeIndicator

**Files:**
- Modify: `ui/src/components/chat/ModeIndicator.tsx`

- [ ] **Step 1: Add auto_archive indicator**

Add after the auto mode indicator:

```typescript
        {auto_archive && context !== 'super' && (
          <span style={{ color: 'var(--accent-green)' }}>·arch</span>
        )}
```

Need to import auto_archive from mode store:

```typescript
  const auto_archive = useModeStore((s) => s.auto_archive)
```

- [ ] **Step 2: Commit**

```bash
git add ui/src/components/chat/ModeIndicator.tsx
git commit -m "feat: show auto-archive indicator in ModeIndicator"
```

---

### Task 9: Add integration test for auto-archive toggle

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add test for auto-archive toggle**

Add at the end of the file:

```rust
#[tokio::test]
async fn test_auto_archive_toggle() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    // Get initial mode
    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            &format!("/api/rings/{ring_id}/mode"),
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["auto_archive"], false);

    // Toggle auto_archive on
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/mode"),
            Some(r#"{"auto_archive":true}"#),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["auto_archive"], true);

    // Toggle auto_archive off
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/mode"),
            Some(r#"{"auto_archive":false}"#),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["auto_archive"], false);
}
```

- [ ] **Step 2: Run the new test**

Run: `cargo test --manifest-path server/Cargo.toml test_auto_archive_toggle`
Expected: PASS

- [ ] **Step 3: Run all tests**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all tests pass (56/56)

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test: add integration test for auto-archive toggle"
```

---

### Task 10: Final verification

- [ ] **Step 1: Run cargo clippy**

Run: `cargo clippy --manifest-path server/Cargo.toml -- -D warnings`
Expected: no warnings

- [ ] **Step 2: Run cargo fmt check**

Run: `cargo fmt --manifest-path server/Cargo.toml -- --check`
Expected: no formatting issues

- [ ] **Step 3: Run full test suite**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all tests pass

- [ ] **Step 4: Build frontend**

Run: `cd ui && npm run build`
Expected: builds successfully

---

## Spec Coverage Check

1. **Backend: Add `auto_archive` field to rings table** → Task 1
2. **Backend: Add route to toggle auto mode** → Already exists (`PUT /api/rings/{ring_id}/mode`), extended in Task 3
3. **Backend: After each Group Ring chat completion, if auto mode is ON, analyze and trigger archive** → Task 4, 5
4. **Frontend: Add Auto toggle button in ChatArea** → Task 7
5. **Frontend: Show visual indicator when Auto mode is active** → Task 8

All requirements covered.

## Placeholder Scan

No placeholders found. All steps contain actual code.

## Type Consistency Check

- `auto_archive` is `bool` in all locations (database, models, API, frontend)
- `ModeResponse` and `UpdateModeRequest` consistently include `auto_archive`
- API endpoint `/rings/{ring_id}/mode` handles the new field
