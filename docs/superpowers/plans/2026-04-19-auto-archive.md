# Auto Archive Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** When a session is closed in auto mode with archive enabled, automatically extract knowledge units from the conversation and create Markdown archive files via Git.

**Architecture:** `close_session` spawns an async background task. The task reads all messages, calls LLM (non-streaming) to extract archive units as JSON, then iterates over each unit calling the existing `archive_content_creator`. No frontend changes, no migration.

**Tech Stack:** Rust, Axum, async-openai (non-streaming), serde_json, tokio::spawn, tracing

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `server/src/services/llm.rs` | Modify | Add `chat_complete` method for non-streaming LLM calls |
| `server/src/models/session.rs` | Modify | Add `get_all_messages_ordered` query |
| `server/src/services/archive_service.rs` | Modify | Add `auto_archive_session` function |
| `server/src/services/session.rs` | Modify | Hook auto-archive into `close_session` |

---

### Task 1: Add `get_all_messages_ordered` query

**Files:**
- Modify: `server/src/models/session.rs:321` (after `get_messages` function)

- [ ] **Step 1: Add the query function**

Add after the `get_messages` function (line 321):

```rust
pub async fn get_all_messages_ordered(
    pool: &sqlx::SqlitePool,
    session_id: &str,
    limit: i64,
) -> Result<Vec<SessionMessageRow>> {
    let mut rows = sqlx::query_as::<_, SessionMessageRow>(
        "SELECT * FROM session_messages WHERE session_id = ?1 ORDER BY seq_num DESC LIMIT ?2",
    )
    .bind(session_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    rows.reverse();
    Ok(rows)
}
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/models/session.rs
git commit -m "feat: add get_all_messages_ordered query for auto archive"
```

---

### Task 2: Add `chat_complete` to LlmClient

**Files:**
- Modify: `server/src/services/llm.rs:157` (before closing `}` of `impl LlmClient`)

- [ ] **Step 1: Add the `chat_complete` method**

Add inside `impl LlmClient` block, after the `chat_stream` method (before the closing `}` of the impl block at line 157):

```rust
    pub async fn chat_complete(
        self,
        system_prompt: String,
        user_message: String,
    ) -> crate::error::Result<String> {
        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(system_prompt),
                    name: None,
                },
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(user_message),
                    name: None,
                },
            ),
        ];

        let request = CreateChatCompletionRequest {
            messages,
            model: self.model,
            ..Default::default()
        };

        let response = self.client.chat().create(request).await?;
        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        Ok(content)
    }
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/services/llm.rs
git commit -m "feat: add chat_complete method for non-streaming LLM calls"
```

---

### Task 3: Add `auto_archive_session` to archive_service

**Files:**
- Modify: `server/src/services/archive_service.rs` (add after `ArchiveStep` impl block, line 236)

- [ ] **Step 1: Add the `ArchiveUnit` struct and `auto_archive_session` function**

Add at the end of the file:

```rust
#[derive(Debug, serde::Deserialize)]
pub struct ArchiveUnit {
    pub title: String,
    pub content: String,
}

pub async fn auto_archive_session(
    pool: &SqlitePool,
    git: &GitService,
    rings_dir: &std::path::Path,
    ring_id: &str,
    session_id: &str,
    session_title: &str,
    session_skill: &str,
    creator_user: &crate::models::user::UserRow,
) {
    tracing::info!("auto_archive started: session={session_id}, ring={ring_id}");

    let messages = match crate::models::session::get_all_messages_ordered(pool, session_id, 100).await {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("auto_archive failed to load messages: {e}");
            return;
        }
    };

    if messages.is_empty() {
        tracing::info!("auto_archive: no messages in session {session_id}, skipping");
        return;
    }

    let messages_text = messages
        .iter()
        .map(|m| format!("[{}]: {}", m.sender_name, m.content))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = "你是一个知识管理助手。分析以下讨论记录，提取值得长期保存的知识单元。\n\n每个单元包含：\n- title: 简短标题（用于文件名，不超过 30 字，不含特殊字符）\n- content: Markdown 格式的完整归档内容\n\n归档单元可以是：决策记录、结论总结、知识点、调研发现、方案对比等。\n只提取有实质内容的单元。如果讨论内容没有值得归档的，返回空数组。\n\n返回纯 JSON 数组，不要 markdown code block：\n[{\"title\": \"...\", \"content\": \"...\"}]";

    let user_message = format!(
        "Session 标题: {session_title}\nSkill: {session_skill}\n\n讨论记录：\n{messages_text}"
    );

    let llm = match crate::services::llm::LlmClient::from_user(creator_user) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("auto_archive failed to create LLM client: {e}");
            return;
        }
    };

    let response = match llm.chat_complete(system_prompt, user_message).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("auto_archive LLM call failed: {e}");
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

    let units: Vec<ArchiveUnit> = match serde_json::from_str(&json_str) {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!("auto_archive failed to parse LLM JSON: {e}\nraw: {json_str}");
            return;
        }
    };

    tracing::info!("auto_archive extracted {} units", units.len());

    if units.is_empty() {
        return;
    }

    let mut success_count = 0u32;
    for unit in &units {
        let title_with_ts = format!("{}_{}", chrono::Utc::now().format("%H%M%S"), unit.title);
        match archive_content_creator(
            pool,
            git,
            rings_dir,
            ring_id,
            Some(session_id),
            None,
            &unit.content,
            &title_with_ts,
            &creator_user.token_id,
        )
        .await
        {
            Ok(_) => success_count += 1,
            Err(e) => {
                tracing::warn!("auto_archive unit failed: title={}, error={}", unit.title, e);
            }
        }
    }

    tracing::info!(
        "auto_archive completed: session={session_id}, {success_count}/{} files created",
        units.len()
    );
}
```

Note: `title_with_ts` prefixes the timestamp `%H%M%S` to `sanitize_filename`'s date prefix, making the final filename `YYYY-MM-DD_HHMMSS_<title>.md` to avoid collisions when multiple units are archived in the same day.

- [ ] **Step 2: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/services/archive_service.rs
git commit -m "feat: add auto_archive_session with LLM extraction and batch commit"
```

---

### Task 4: Hook auto-archive into `close_session`

**Files:**
- Modify: `server/src/services/session.rs:124-144` (the `close_session` function)

- [ ] **Step 1: Modify `close_session` to spawn auto-archive task**

Replace the entire `close_session` function (lines 124-144) with:

```rust
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

    let interaction_mode: String = sqlx::query_scalar(
        "SELECT interaction_mode FROM rings WHERE id = ?1",
    )
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
```

- [ ] **Step 2: Run cargo check**

Run: `cargo check --manifest-path server/Cargo.toml`
Expected: compiles without errors

- [ ] **Step 3: Run existing tests**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all existing tests pass (12/12)

- [ ] **Step 4: Commit**

```bash
git add server/src/services/session.rs
git commit -m "feat: hook auto archive into close_session with async spawn"
```

---

### Task 5: Add integration test for auto-archive trigger

**Files:**
- Modify: `server/tests/integration.rs` (add after existing tests)

- [ ] **Step 1: Add test for close-session in auto mode with archive_enabled**

Add at the end of `integration.rs`:

```rust
#[tokio::test]
async fn test_close_session_triggers_auto_archive_check() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/mode"),
            Some(r#"{"interaction_mode":"auto"}"#),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let session_body =
        r#"{"title":"Auto Archive Test","skill":"discussion","archivable":true}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            &format!("/api/rings/{ring_id}/sessions"),
            Some(session_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let json = read_body(resp).await;
    let session_id = json["id"].as_str().unwrap();

    let archive_body = r#"{"enabled":true}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            &format!("/api/rings/{ring_id}/sessions/{session_id}/archive-toggle"),
            Some(archive_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

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
}
```

This test verifies the close-session path works when `interaction_mode=auto` and `archive_enabled=true`. The auto-archive spawns a task that will fail gracefully (no real LLM API key), confirming the spawn doesn't block or crash the response.

- [ ] **Step 2: Run the new test**

Run: `cargo test --manifest-path server/Cargo.toml test_close_session_triggers_auto_archive_check`
Expected: PASS

- [ ] **Step 3: Run all tests**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all tests pass (13/13)

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test: add integration test for auto archive trigger on session close"
```

---

### Task 6: Final verification

- [ ] **Step 1: Run cargo clippy**

Run: `cargo clippy --manifest-path server/Cargo.toml -- -D warnings`
Expected: no warnings

- [ ] **Step 2: Run cargo fmt check**

Run: `cargo fmt --manifest-path server/Cargo.toml -- --check`
Expected: no formatting issues

- [ ] **Step 3: Run full test suite**

Run: `cargo test --manifest-path server/Cargo.toml`
Expected: all tests pass
