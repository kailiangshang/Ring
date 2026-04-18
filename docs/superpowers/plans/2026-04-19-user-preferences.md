# User Preferences Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add user preferences storage (`~/.ring/hub/user_preferences.md`) with Super Ring tool access + CLI `%prefs` commands.

**Architecture:** File-based Markdown preferences, following the existing `system_prompt.md` pattern. Two new Super Ring tools (`query_user_preferences` / `update_user_preferences`) + two new API endpoints (`GET/PUT /api/super/preferences`) + frontend `%prefs` / `%prefs set` CLI commands.

**Tech Stack:** Rust + Axum (backend), TypeScript + Zustand (frontend), Markdown file storage

---

### Task 1: Backend — Constants + Helper Functions

**Files:**
- Modify: `server/src/services/super_chat.rs`

- [ ] **Step 1: Add DEFAULT_PREFERENCES constant and helper functions**

Add after `DEFAULT_SUPER_SYSTEM_PROMPT` constant (line 14) in `server/src/services/super_chat.rs`:

```rust
const DEFAULT_PREFERENCES: &str = "## 语言\n- default: zh-CN\n\n## LLM\n- default_provider: openai\n\n## 输出格式\n- style: concise\n\n## 默认模式\n- mode: normal";
```

Add these two functions after `update_system_prompt` (after line 40):

```rust
pub fn get_user_preferences(hub_dir: &Path) -> String {
    let prefs_file = hub_dir.join("user_preferences.md");
    match std::fs::read_to_string(&prefs_file) {
        Ok(content) if !content.trim().is_empty() => content,
        _ => DEFAULT_PREFERENCES.to_string(),
    }
}

pub fn get_user_preferences_info(hub_dir: &Path) -> (String, bool) {
    let prefs_file = hub_dir.join("user_preferences.md");
    match std::fs::read_to_string(&prefs_file) {
        Ok(ref content) if !content.trim().is_empty() => (content.clone(), true),
        _ => (DEFAULT_PREFERENCES.to_string(), false),
    }
}

pub fn update_user_preferences(hub_dir: &Path, content: &str) -> Result<()> {
    let prefs_file = hub_dir.join("user_preferences.md");
    if content.trim().is_empty() {
        let _ = std::fs::remove_file(&prefs_file);
    } else {
        std::fs::write(&prefs_file, content)?;
    }
    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/services/super_chat.rs
git commit -m "Add user preferences constants and helper functions"
```

---

### Task 2: Backend — Add 2 New Tools to Tool Framework

**Files:**
- Modify: `server/src/services/super_chat.rs`

- [ ] **Step 1: Add query_user_preferences tool to get_super_tools()**

In `get_super_tools()` function (line 47), add a new `ChatCompletionTool` to the `vec![]` after the `query_ring_detail` tool:

```rust
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: "query_user_preferences".to_string(),
                description: Some(
                    "读取用户的全局偏好设置，包括语言、默认 LLM、输出格式、默认模式等。".to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {},
                        "required": []
                    }),
                ),
                strict: None,
            },
        },
        ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: FunctionObject {
                name: "update_user_preferences".to_string(),
                description: Some(
                    "更新用户的全局偏好设置。接收完整的 Markdown 内容覆盖写入户好文件。修改前应先用 query_user_preferences 读取当前内容。".to_string(),
                ),
                parameters: Some(
                    serde_json::json!({
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "完整的偏好设置 Markdown 内容"
                            }
                        },
                        "required": ["content"]
                    }),
                ),
                strict: None,
            },
        },
```

- [ ] **Step 2: Add Deserialize struct for update args**

Add after the existing `QueryRingDetailArgs` struct (after line 45):

```rust
#[derive(Debug, Deserialize)]
struct UpdatePreferencesArgs {
    content: String,
}
```

- [ ] **Step 3: Add tool routing in execute_tool()**

In `execute_tool()` (line 140), add two new match arms before the `_` wildcard:

```rust
        "query_user_preferences" => execute_query_user_preferences(hub_dir),
        "update_user_preferences" => {
            let args: UpdatePreferencesArgs = serde_json::from_str(arguments)
                .map_err(|e| RingError::BadRequest(format!("invalid tool arguments: {e}")))?;
            execute_update_user_preferences(hub_dir, &args.content)
        }
```

- [ ] **Step 4: Add tool implementation functions**

Add before `execute_query_rings` (before line 158):

```rust
fn execute_query_user_preferences(hub_dir: &Path) -> Result<String> {
    Ok(get_user_preferences(hub_dir))
}

fn execute_update_user_preferences(hub_dir: &Path, content: &str) -> Result<String> {
    update_user_preferences(hub_dir, content)?;
    Ok("偏好设置已更新。".to_string())
}
```

- [ ] **Step 5: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles without errors

- [ ] **Step 6: Commit**

```bash
git add server/src/services/super_chat.rs
git commit -m "Add query_user_preferences and update_user_preferences tools"
```

---

### Task 3: Backend — Inject Preferences into System Prompt

**Files:**
- Modify: `server/src/services/super_chat.rs`

- [ ] **Step 1: Modify start_super_chat to append preferences**

In `start_super_chat()` function, after the line that builds system_prompt (line 286):

```rust
    let system_prompt = format!("{base_prompt}\n\n{ring_summary}");
```

Change to:

```rust
    let prefs = get_user_preferences(&state.hub_dir);
    let system_prompt = format!("{base_prompt}\n\n{ring_summary}\n\n## 用户偏好\n{prefs}");
```

- [ ] **Step 2: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles without errors

- [ ] **Step 3: Commit**

```bash
git add server/src/services/super_chat.rs
git commit -m "Inject user preferences into Super Ring system prompt"
```

---

### Task 4: Backend — API Endpoints

**Files:**
- Modify: `server/src/routes/super_chat.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Add request/response types**

In `server/src/routes/super_chat.rs`, add after `SystemPromptResponse` struct (after line 49):

```rust
#[derive(Debug, serde::Serialize)]
pub struct PreferencesResponse {
    pub content: String,
    pub is_custom: bool,
}

#[derive(Debug, Deserialize)]
pub struct PreferencesRequest {
    pub content: String,
}
```

- [ ] **Step 2: Add handler functions**

Add at the end of `server/src/routes/super_chat.rs`:

```rust
pub async fn get_preferences(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<PreferencesResponse>> {
    let (content, is_custom) = super_chat::get_user_preferences_info(&state.hub_dir);
    Ok(Json(PreferencesResponse { content, is_custom }))
}

pub async fn update_preferences(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(body): Json<PreferencesRequest>,
) -> Result<Json<PreferencesResponse>> {
    super_chat::update_user_preferences(&state.hub_dir, &body.content)?;
    let (content, is_custom) = super_chat::get_user_preferences_info(&state.hub_dir);
    Ok(Json(PreferencesResponse { content, is_custom }))
}
```

- [ ] **Step 3: Register routes in mod.rs**

In `server/src/routes/mod.rs`, add after the `/super/system-prompt` route (after line 149):

```rust
        .route(
            "/super/preferences",
            get(super_chat::get_preferences).put(super_chat::update_preferences),
        )
```

- [ ] **Step 4: Verify compilation**

Run: `cd server && cargo check`
Expected: compiles without errors

- [ ] **Step 5: Commit**

```bash
git add server/src/routes/super_chat.rs server/src/routes/mod.rs
git commit -m "Add GET/PUT /api/super/preferences endpoints"
```

---

### Task 5: Backend — Integration Tests

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add test for default preferences**

Add at the end of `server/tests/integration.rs`:

```rust
#[tokio::test]
async fn test_super_preferences_default() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/super/preferences",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["is_custom"], false);
    assert!(json["content"].as_str().unwrap().contains("zh-CN"));
    assert!(json["content"].as_str().unwrap().contains("openai"));
}
```

- [ ] **Step 2: Add test for update preferences**

```rust
#[tokio::test]
async fn test_super_preferences_update() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let update_body = r#"{"content":"## 语言\n- default: en\n\n## LLM\n- default_provider: ollama"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            "/api/super/preferences",
            Some(update_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["is_custom"], true);
    assert!(json["content"].as_str().unwrap().contains("default: en"));

    let resp = app
        .clone()
        .oneshot(make_request(
            "GET",
            "/api/super/preferences",
            None,
            Some(&token),
        ))
        .await
        .unwrap();
    let json = read_body(resp).await;
    assert!(json["content"].as_str().unwrap().contains("default_provider: ollama"));
}
```

- [ ] **Step 3: Add test for reset preferences (empty content)**

```rust
#[tokio::test]
async fn test_super_preferences_reset() {
    let state = setup_app().await;
    let app = build_router(state);

    let token = do_setup(&app).await;

    let update_body = r#"{"content":"## 语言\n- default: en"}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            "/api/super/preferences",
            Some(update_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let reset_body = r#"{"content":""}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "PUT",
            "/api/super/preferences",
            Some(reset_body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = read_body(resp).await;
    assert_eq!(json["is_custom"], false);
}
```

- [ ] **Step 4: Run all tests**

Run: `cd server && cargo test`
Expected: all tests pass (21/21)

- [ ] **Step 5: Run clippy + fmt**

Run: `cd server && cargo fmt && cargo clippy -- -D warnings`
Expected: no errors

- [ ] **Step 6: Commit**

```bash
git add server/tests/integration.rs
git commit -m "Add integration tests for user preferences endpoints"
```

---

### Task 6: Frontend — API Functions

**Files:**
- Modify: `ui/src/services/api.ts`

- [ ] **Step 1: Add preferences API functions**

Add at the end of `ui/src/services/api.ts` (before the closing of the file, after `triggerArchiveSSE`):

```typescript
export async function getPreferences(): Promise<{ content: string; is_custom: boolean }> {
  return api.get('/super/preferences')
}

export async function updatePreferences(content: string): Promise<{ content: string; is_custom: boolean }> {
  return api.put('/super/preferences', { content })
}
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/services/api.ts
git commit -m "Add getPreferences and updatePreferences API functions"
```

---

### Task 7: Frontend — %prefs CLI Command

**Files:**
- Modify: `ui/src/services/command-parser.ts`
- Modify: `ui/src/stores/chat-store.ts`

- [ ] **Step 1: Add prefs command type to parser**

In `ui/src/services/command-parser.ts`, update `ParsedCommand` type to add a `prefs` variant:

```typescript
export type ParsedCommand =
  | { type: 'address'; target: string; rest: string }
  | { type: 'reference'; name: string }
  | { type: 'action'; action: string; args: string }
  | { type: 'meta'; key: string; value: string }
  | { type: 'prefs'; subcommand: 'show' | 'set'; key?: string; value?: string }
```

In the `parseCommand` function, update the `%` token handling block (lines 49-55). Replace the existing `%` block with:

```typescript
    if (token.startsWith('%')) {
      hasCommand = true
      const body = token.slice(1).toLowerCase()
      if (body === 'prefs') {
        const subcommand = tokens[i + 1]?.toLowerCase()
        if (subcommand === 'set' && tokens[i + 2] && tokens[i + 3]) {
          commands.push({ type: 'prefs', subcommand: 'set', key: tokens[i + 2].toLowerCase(), value: tokens.slice(i + 3).join(' ') })
        } else {
          commands.push({ type: 'prefs', subcommand: 'show' })
        }
        break
      }
      const nextToken = tokens[i + 1]
      commands.push({ type: 'meta', key: body, value: nextToken ?? '' })
      break
    }
```

- [ ] **Step 2: Handle %prefs in chat-store send()**

In `ui/src/stores/chat-store.ts`, add the import for preferences API:

```typescript
import { getPreferences, updatePreferences } from '../services/api'
```

In the `send()` function's parsed command handling, add a new case inside the `for (const cmd of parsed)` loop, after the existing `case 'meta'` block (after line 127):

```typescript
          case 'prefs': {
            if (cmd.subcommand === 'set' && cmd.key && cmd.value) {
              handlePrefsSet(cmd.key, cmd.value, addMessage)
            } else {
              handlePrefsShow(addMessage)
            }
            break
          }
```

Add these helper functions at the top of the file (after imports, before `interface ChatState`):

```typescript
const PREFS_KEY_MAP: Record<string, { section: string; key: string }> = {
  language: { section: '语言', key: 'default' },
  provider: { section: 'LLM', key: 'default_provider' },
  style: { section: '输出格式', key: 'style' },
  mode: { section: '默认模式', key: 'mode' },
}

async function handlePrefsShow(addMessage: (msg: import('../types/chat').ChatMessage) => void) {
  try {
    const { content, is_custom } = await getPreferences()
    const label = is_custom ? '当前偏好设置（自定义）：' : '当前偏好设置（默认）：'
    addMessage({
      id: `sys-prefs-${Date.now()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `${label}\n\`\`\`\n${content}\n\`\`\``,
      created_at: new Date().toISOString(),
    })
  } catch {
    addMessage({
      id: `sys-prefs-err-${Date.now()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: 'Failed to load preferences.',
      created_at: new Date().toISOString(),
    })
  }
}

async function handlePrefsSet(key: string, value: string, addMessage: (msg: import('../types/chat').ChatMessage) => void) {
  const mapping = PREFS_KEY_MAP[key]
  if (!mapping) {
    addMessage({
      id: `sys-prefs-err-${Date.now()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `Unknown preference key "${key}". Supported keys: ${Object.keys(PREFS_KEY_MAP).join(', ')}. For other changes, ask Super Ring.`,
      created_at: new Date().toISOString(),
    })
    return
  }

  try {
    const { content } = await getPreferences()
    const lines = content.split('\n')
    let inSection = false
    let found = false
    const updated = lines.map(line => {
      if (line.trim() === `## ${mapping.section}`) {
        inSection = true
        return line
      }
      if (inSection && line.trim().startsWith(`- ${mapping.key}:`)) {
        found = true
        return `- ${mapping.key}: ${value}`
      }
      if (line.startsWith('## ') && inSection) {
        inSection = false
      }
      return line
    }).join('\n')

    if (!found) {
      addMessage({
        id: `sys-prefs-err-${Date.now()}`,
        role: 'system',
        sender_name: 'SYSTEM',
        content: `Could not find preference "${key}" in current settings. Please use Super Ring to modify.`,
        created_at: new Date().toISOString(),
      })
      return
    }

    await updatePreferences(updated)
    addMessage({
      id: `sys-prefs-${Date.now()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `Preference updated: ${key} = ${value}`,
      created_at: new Date().toISOString(),
    })
  } catch {
    addMessage({
      id: `sys-prefs-err-${Date.now()}`,
      role: 'system',
      sender_name: 'SYSTEM',
      content: `Failed to update preference "${key}".`,
      created_at: new Date().toISOString(),
    })
  }
}
```

- [ ] **Step 3: Update command-parser test**

Add test cases for the `%prefs` command in `ui/src/test/services/command-parser.test.ts` if it exists.

- [ ] **Step 4: Verify build**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add ui/src/services/command-parser.ts ui/src/stores/chat-store.ts
git commit -m "Add %prefs and %prefs set CLI commands for user preferences"
```

---

### Task 8: Final Verification

- [ ] **Step 1: Run all backend tests**

Run: `cd server && cargo test`
Expected: all tests pass

- [ ] **Step 2: Run clippy + fmt**

Run: `cd server && cargo fmt --check && cargo clippy -- -D warnings`
Expected: no errors

- [ ] **Step 3: Run frontend build**

Run: `cd ui && npm run build`
Expected: build succeeds

- [ ] **Step 4: Final commit (if any formatting fixes needed)**

```bash
git add -A
git commit -m "Final formatting and lint fixes for user preferences feature"
```
