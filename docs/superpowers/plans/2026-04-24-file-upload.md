# File Upload Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow users to upload text-based files in Group Ring/Super Ring chat and Session material prep, with AI automatically reading extracted content as conversation context.

**Architecture:** Multipart upload → tmp file → extract text → inject as chat message (system role with 📎 prefix) or session material (item_type "document") → delete file. No persistent storage, no new DB tables.

**Tech Stack:** Axum multipart extractor, lopdf (PDF text extraction), React FormData, existing Zustand stores

---

### Task 1: Add Dependencies

**Files:**
- Modify: `server/Cargo.toml`

- [ ] **Step 1: Add multipart feature to axum and lopdf crate**

In `server/Cargo.toml`, change line 8 from:
```toml
axum = { version = "0.8", features = ["ws"] }
```
to:
```toml
axum = { version = "0.8", features = ["ws", "multipart"] }
```

Add at end of `[dependencies]`:
```toml
lopdf = "0.34"
```

- [ ] **Step 2: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add server/Cargo.toml
git commit -m "feat(upload): add multipart and lopdf dependencies"
```

---

### Task 2: Upload Service

**Files:**
- Create: `server/src/services/upload.rs`
- Modify: `server/src/services/mod.rs` — add `pub mod upload;`

- [ ] **Step 1: Create the upload service**

Create `server/src/services/upload.rs`:

```rust
use std::path::Path;

use axum::body::Bytes;
use lopdf::Document;

use crate::error::{Result, RingError};
use crate::models::message::{self, MessageRow};
use crate::models::session::{self, SessionMaterialRow};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
const MAX_CONTENT_CHARS: usize = 50000;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "txt", "md", "csv", "json", "py", "js", "ts", "tsx", "rs", "go", "java",
    "yaml", "yml", "xml", "html", "css", "toml", "sh", "sql", "log", "env",
    "conf", "cfg", "ini", "pdf",
];

pub fn validate_file(filename: &str, size: usize) -> Result<()> {
    if size > MAX_FILE_SIZE {
        return Err(RingError::BadRequest(format!(
            "file too large: {} bytes (max {})",
            size, MAX_FILE_SIZE
        )));
    }

    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(RingError::BadRequest(format!(
            "unsupported file type: .{ext}"
        )));
    }

    Ok(())
}

pub fn extract_text(filename: &str, data: &[u8]) -> Result<String> {
    let ext = filename
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    let text = if ext == "pdf" {
        extract_pdf_text(data)?
    } else {
        String::from_utf8_lossy(data).into_owned()
    };

    let truncated: String = text.chars().take(MAX_CONTENT_CHARS).collect();
    Ok(truncated)
}

fn extract_pdf_text(data: &[u8]) -> Result<String> {
    let doc = Document::load_mem(data)
        .map_err(|e| RingError::BadRequest(format!("failed to parse PDF: {e}")))?;

    let mut text = String::new();
    let pages = doc.get_pages();

    for (_, page_id) in &pages {
        if let Ok(page_text) = doc.extract_text(page_id) {
            text.push_str(&page_text);
            text.push('\n');
        }
    }

    if text.trim().is_empty() {
        return Err(RingError::BadRequest(
            "PDF contains no extractable text (possibly scanned image)".into(),
        ));
    }

    Ok(text)
}

pub async fn upload_to_chat(
    db: &sqlx::SqlitePool,
    ring_id: Option<&str>,
    user_id: &str,
    sender_name: &str,
    filename: &str,
    data: &[u8],
) -> Result<MessageRow> {
    validate_file(filename, data.len())?;
    let content = extract_text(filename, data)?;

    let msg_id = ulid::Ulid::new().to_string();
    let file_content = format!("📎 {filename}\n---\n{content}");

    let msg = message::insert_message(
        db,
        &message::NewMessage {
            id: &msg_id,
            ring_id,
            user_id,
            role: "system",
            sender_name,
            content: &file_content,
            node_refs: &[],
            tag_refs: &[],
            token_usage: None,
        },
    )
    .await?;

    if let Some(rid) = ring_id {
        let ring_name = crate::services::search::get_ring_name(db, rid)
            .await
            .unwrap_or_default();
        let _ = crate::services::search::upsert_search_index(
            db,
            "message",
            &msg_id,
            rid,
            &ring_name,
            &format!("📎 {filename}"),
            &content,
            &serde_json::json!({"role": "system", "filename": filename}).to_string(),
        )
        .await;
    }

    Ok(msg)
}

pub async fn upload_to_session(
    db: &sqlx::SqlitePool,
    ring_id: &str,
    session_id: &str,
    filename: &str,
    data: &[u8],
) -> Result<SessionMaterialRow> {
    validate_file(filename, data.len())?;
    let content = extract_text(filename, data)?;

    let material_id = ulid::Ulid::new().to_string();
    let material = session::create_material(
        db,
        &material_id,
        session_id,
        "document",
        filename,
        &content,
        "ready",
    )
    .await?;

    let _ = crate::services::search::upsert_search_index(
        db,
        "session_message",
        &material_id,
        ring_id,
        "",
        filename,
        &content,
        &serde_json::json!({"session_id": session_id, "item_type": "document"}).to_string(),
    )
    .await;

    Ok(material)
}
```

- [ ] **Step 2: Add module to mod.rs**

In `server/src/services/mod.rs`, add at end:

```rust
pub mod upload;
```

- [ ] **Step 3: Check that `session::create_material` exists in the model**

Read `server/src/models/session.rs` and verify there is a `create_material` function. If not, you need to add it. Search for `create_material` or `insert_material`. If it doesn't exist, add this function to `server/src/models/session.rs`:

```rust
pub async fn create_material(
    pool: &sqlx::SqlitePool,
    id: &str,
    session_id: &str,
    item_type: &str,
    title: &str,
    content: &str,
    status: &str,
) -> Result<SessionMaterialRow> {
    sqlx::query_as::<_, SessionMaterialRow>(
        "INSERT INTO session_materials (id, session_id, item_type, title, content, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         RETURNING *",
    )
    .bind(id)
    .bind(session_id)
    .bind(item_type)
    .bind(title)
    .bind(content)
    .bind(status)
    .fetch_one(pool)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))
}
```

- [ ] **Step 4: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles without errors.

- [ ] **Step 5: Commit**

```bash
git add server/src/services/upload.rs server/src/services/mod.rs server/src/models/session.rs
git commit -m "feat(upload): add upload service with file validation and text extraction"
```

---

### Task 3: Upload Routes

**Files:**
- Create: `server/src/routes/upload.rs`
- Modify: `server/src/routes/mod.rs` — add module + 3 routes

- [ ] **Step 1: Create upload route handlers**

Create `server/src/routes/upload.rs`:

```rust
use axum::extract::{Multipart, Path, State};
use axum::Json;

use crate::error::Result;
use crate::extractors::AuthUser;
use crate::models::ring;
use crate::state::AppState;

pub async fn upload_ring_file(
    State(state): State<AppState>,
    user: AuthUser,
    Path(ring_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>> {
    let _role = ring::get_user_role(&state.db, &ring_id, &user.token_id).await?;
    let user_row = state.get_user_decrypted(&user.token_id).await?;

    let (filename, data) = extract_file(&mut multipart).await?;

    let msg = crate::services::upload::upload_to_chat(
        &state.db,
        Some(&ring_id),
        &user.token_id,
        &user_row.display_name,
        &filename,
        &data,
    )
    .await?;

    Ok(Json(serde_json::to_value(msg).unwrap()))
}

pub async fn upload_super_file(
    State(state): State<AppState>,
    user: AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>> {
    let user_row = state.get_user_decrypted(&user.token_id).await?;

    let (filename, data) = extract_file(&mut multipart).await?;

    let msg = crate::services::upload::upload_to_chat(
        &state.db,
        Some("super"),
        &user.token_id,
        &user_row.display_name,
        &filename,
        &data,
    )
    .await?;

    Ok(Json(serde_json::to_value(msg).unwrap()))
}

pub async fn upload_session_file(
    State(state): State<AppState>,
    user: AuthUser,
    Path((ring_id, session_id)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>> {
    crate::models::session::is_participant(&state.db, &session_id, &user.token_id).await?;

    let (filename, data) = extract_file(&mut multipart).await?;

    let material = crate::services::upload::upload_to_session(
        &state.db,
        &ring_id,
        &session_id,
        &filename,
        &data,
    )
    .await?;

    let broadcast = serde_json::json!({
        "type": "session_material_added",
        "session_id": session_id,
        "material": material,
    });
    state.ws_hub.broadcast_to_session(&session_id, &broadcast.to_string());

    Ok(Json(serde_json::to_value(material).unwrap()))
}

async fn extract_file(multipart: &mut Multipart) -> Result<(String, Vec<u8>)> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| crate::error::RingError::BadRequest(format!("failed to read upload: {e}")))?
        .ok_or_else(|| crate::error::RingError::BadRequest("no file provided".into()))?;

    let filename = field
        .file_name()
        .unwrap_or("unnamed.txt")
        .to_string();

    let data = field
        .bytes()
        .await
        .map_err(|e| crate::error::RingError::BadRequest(format!("failed to read file data: {e}")))?;

    Ok((filename, data.to_vec()))
}
```

- [ ] **Step 2: Register module and routes in mod.rs**

In `server/src/routes/mod.rs`, add to the module declarations (around line 28):

```rust
mod upload;
```

Add routes to the api router. After the existing ring chat route (line 83 `.route("/rings/{ring_id}/chat", post(chat::ring_chat))`), add:

```rust
        .route("/rings/{ring_id}/upload", post(upload::upload_ring_file))
```

After the existing super chat route (line 214 `.route("/super/chat", post(super_chat::super_chat_handler))`), add:

```rust
        .route("/super/upload", post(upload::upload_super_file))
```

After the existing material-prep route (around line 179 `.route("/rings/{ring_id}/sessions/{session_id}/material-prep", ...)`), add:

```rust
        .route(
            "/rings/{ring_id}/sessions/{session_id}/material-prep/upload",
            post(upload::upload_session_file),
        )
```

- [ ] **Step 3: Verify compilation**

Run: `cd server && cargo check`
Expected: Compiles without errors.

- [ ] **Step 4: Run tests**

Run: `cd server && cargo test`
Expected: All existing tests pass.

- [ ] **Step 5: Commit**

```bash
git add server/src/routes/upload.rs server/src/routes/mod.rs
git commit -m "feat(upload): add upload routes for ring chat, super chat, and session"
```

---

### Task 4: Frontend — API Upload Function

**Files:**
- Modify: `ui/src/services/api.ts` — add `uploadFile` function
- Modify: `ui/src/types/chat.ts` — no changes needed (file card is just a system message)

- [ ] **Step 1: Add uploadFile function to api.ts**

In `ui/src/services/api.ts`, after the `exportFile` function (around line 226), add:

```typescript
export async function uploadFile(path: string, file: File): Promise<any> {
  const token = await getToken()
  const formData = new FormData()
  formData.append('file', file)
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers: token ? { 'X-Ring-Token': token } : {},
    body: formData,
  })
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new ApiError(
      res.status,
      body?.error?.code ?? 'unknown',
      body?.error?.message ?? res.statusText,
    )
  }
  return res.json()
}
```

- [ ] **Step 2: Verify build**

Run: `cd ui && npm run build`
Expected: Clean build.

- [ ] **Step 3: Commit**

```bash
git add ui/src/services/api.ts
git commit -m "feat(upload): add uploadFile API function with multipart support"
```

---

### Task 5: Frontend — Upload UI in InputArea

**Files:**
- Modify: `ui/src/components/chat/InputArea.tsx`

- [ ] **Step 1: Add imports and upload state**

At the top of `ui/src/components/chat/InputArea.tsx`, add to imports (after line 6):

```typescript
import { uploadFile } from '../../services/api'
import type { ChatMessage } from '../../types/chat'
```

Inside the `InputArea` function, after `const inputRef = useRef<HTMLInputElement>(null)` (line 17), add:

```typescript
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [uploading, setUploading] = useState(false)
```

Add after the `setShowArchiveBanner` state (line 16):

```typescript
  const addMessage = useChatStore((s) => s.addMessage)
```

Check if `addMessage` exists in the chat store. If not, you need to add it. Read `ui/src/stores/chat-store.ts` and check for an `addMessage` action. If it doesn't exist, add to the store interface and implementation:

```typescript
addMessage: (msg: ChatMessage) => void
```

Implementation:
```typescript
addMessage: (msg) => set((state) => ({ messages: [...state.messages, msg] })),
```

- [ ] **Step 2: Add upload handler function**

After the existing handler functions (around line 130), add:

```typescript
  const handleFileUpload = async (files: FileList | null) => {
    if (!files || files.length === 0) return
    setUploading(true)

    const currentContext = useAppStore.getState().current_context
    const activeRingId = useAppStore.getState().active_ring_id

    for (let i = 0; i < files.length; i++) {
      const file = files[i]
      try {
        let endpoint: string
        if (currentContext === 'session') {
          const sessionId = useAppStore.getState().active_session_id
          if (!activeRingId || !sessionId) continue
          endpoint = `/rings/${activeRingId}/sessions/${sessionId}/material-prep/upload`
        } else if (currentContext === 'super') {
          endpoint = '/super/upload'
        } else if (activeRingId) {
          endpoint = `/rings/${activeRingId}/upload`
        } else {
          continue
        }

        const result = await uploadFile(endpoint, file)
        if (currentContext !== 'session' && addMessage) {
          addMessage(result as ChatMessage)
        }
      } catch (e: any) {
        console.error('upload failed:', e.message)
      }
    }
    setUploading(false)
  }

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    handleFileUpload(e.dataTransfer.files)
  }

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }

  const handlePaste = (e: React.ClipboardEvent) => {
    if (e.clipboardData.files.length > 0) {
      e.preventDefault()
      handleFileUpload(e.clipboardData.files)
    }
  }
```

- [ ] **Step 3: Add hidden file input and upload button to JSX**

Before the `<input ref={inputRef}` element (before line 204), add a hidden file input and the 📎 button:

After `<ModeIndicator />` (line 203), add:

```tsx
        <input
          ref={fileInputRef}
          type="file"
          multiple
          accept=".txt,.md,.csv,.json,.py,.js,.ts,.tsx,.rs,.go,.java,.yaml,.yml,.xml,.html,.css,.toml,.sh,.sql,.log,.pdf"
          style={{ display: 'none' }}
          onChange={(e) => handleFileUpload(e.target.files)}
        />
        <button
          onClick={() => fileInputRef.current?.click()}
          disabled={uploading}
          style={{
            background: 'none',
            border: 'none',
            color: uploading ? 'var(--text-dim)' : 'var(--text-secondary)',
            cursor: uploading ? 'default' : 'pointer',
            fontSize: 16,
            padding: '4px 4px',
            lineHeight: 1,
          }}
          title="Upload file"
        >
          {uploading ? '⏳' : '📎'}
        </button>
```

- [ ] **Step 4: Add drag-and-drop and paste handlers to the wrapper div**

The outer wrapper div of InputArea needs `onDrop`, `onDragOver`, and the `<input>` needs `onPaste`.

Find the outermost `<div>` returned by InputArea (the wrapper). Add:

```tsx
onDrop={handleDrop}
onDragOver={handleDragOver}
```

Add `onPaste={handlePaste}` to the text `<input>` element (alongside the existing `onKeyDown` and `onChange`).

- [ ] **Step 5: Verify build**

Run: `cd ui && npm run build`
Expected: Clean build.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/chat/InputArea.tsx ui/src/stores/chat-store.ts
git commit -m "feat(upload): add file upload UI with button, drag-drop, and paste"
```

---

### Task 6: Frontend — File Card Rendering

**Files:**
- Modify: `ui/src/components/chat/MessageItem.tsx`

- [ ] **Step 1: Add file card detection and rendering**

Read `ui/src/components/chat/MessageItem.tsx`. Before the main content `<div ref={contentRef}>` block, add file card detection:

Inside the component function, before the return statement, add:

```typescript
  const isFileCard = message.role === 'system' && message.content.startsWith('📎 ')
  const fileCardMatch = isFileCard ? message.content.match(/^📎 (.+)\n---\n([\s\S]*)$/) : null
  const fileCardFilename = fileCardMatch ? fileCardMatch[1] : ''
  const fileCardContent = fileCardMatch ? fileCardMatch[2] : ''
```

- [ ] **Step 2: Add file card JSX**

In the JSX, before the `<div ref={contentRef}>` block that renders the message content, add a conditional file card:

```tsx
        {isFileCard && fileCardMatch && (
          <div style={{
            border: '1px solid var(--border)',
            borderRadius: 6,
            padding: '8px 12px',
            background: 'var(--bg-active)',
            marginBottom: 8,
            fontSize: 13,
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
              <span style={{ fontSize: 14 }}>📎</span>
              <span style={{ fontWeight: 700, color: 'var(--accent-ice)', fontSize: 12 }}>
                {fileCardFilename}
              </span>
            </div>
            <div
              ref={isFileCard ? contentRef : undefined}
              style={{
                color: 'var(--text-secondary)',
                fontSize: 12,
                lineHeight: 1.5,
                maxHeight: collapsed ? 200 : undefined,
                overflow: collapsed ? 'hidden' : 'visible',
                position: 'relative',
              }}
            >
              <pre style={{ margin: 0, whiteSpace: 'pre-wrap', wordBreak: 'break-word', fontFamily: 'inherit' }}>
                {fileCardContent.slice(0, 500)}{fileCardContent.length > 500 ? '...' : ''}
              </pre>
              {collapsed && (
                <div
                  style={{
                    position: 'absolute',
                    bottom: 0,
                    left: 0,
                    right: 0,
                    height: 40,
                    background: 'linear-gradient(transparent, var(--bg-active))',
                    display: 'flex',
                    alignItems: 'flex-end',
                    justifyContent: 'center',
                    cursor: 'pointer',
                  }}
                  onClick={() => setCollapsed(false)}
                >
                  <span style={{ fontSize: 10, color: 'var(--accent-cyan)', fontWeight: 700, paddingBottom: 4 }}>
                    EXPAND
                  </span>
                </div>
              )}
            </div>
          </div>
        )}
```

Note: When `isFileCard` is true, set `ref={contentRef}` on the file card content div instead of the markdown div, so the collapse detection works on the file card content.

- [ ] **Step 3: Skip normal markdown rendering for file cards**

Wrap the existing `<div ref={contentRef}>` block (the ReactMarkdown rendering) in a condition:

Change:
```tsx
        <div
          ref={contentRef}
          style={{...}}
        >
          <ReactMarkdown ...>
```

To:
```tsx
        {!isFileCard && (
        <div
          ref={contentRef}
          style={{...}}
        >
          <ReactMarkdown ...>
            {message.content}
          </ReactMarkdown>
          {isStreaming && (...)}
          {collapsed && overflowing && (...)}
        </div>
        )}
```

Keep the collapse button (`{!collapsed && overflowing && isAi && ...}`) outside the conditional so it still shows.

- [ ] **Step 4: Verify build**

Run: `cd ui && npm run build`
Expected: Clean build.

- [ ] **Step 5: Commit**

```bash
git add ui/src/components/chat/MessageItem.tsx
git commit -m "feat(upload): render file cards for uploaded documents in chat"
```

---

### Task 7: Frontend — Session Material Upload

**Files:**
- Modify: `ui/src/components/panels/SessionPanel.tsx`

- [ ] **Step 1: Add upload button to MaterialPrepView**

Read `ui/src/components/panels/SessionPanel.tsx`. Find the `MaterialPrepView` component (around line 162).

Add an import at the top of the file:

```typescript
import { uploadFile } from '../../services/api'
```

Inside `MaterialPrepView`, add state and handler:

```typescript
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [uploading, setUploading] = useState(false)

  const handleUpload = async (files: FileList | null) => {
    if (!files || files.length === 0) return
    setUploading(true)
    for (let i = 0; i < files.length; i++) {
      try {
        await uploadFile(
          `/rings/${ring_id}/sessions/${session.id}/material-prep/upload`,
          files[i],
        )
        fetchMaterials(ring_id, session.id)
      } catch (e: any) {
        console.error('upload failed:', e.message)
      }
    }
    setUploading(false)
  }
```

Add the hidden file input and upload button in the materials section header (find where the materials list starts):

```tsx
        <input
          ref={fileInputRef}
          type="file"
          multiple
          accept=".txt,.md,.csv,.json,.py,.js,.ts,.tsx,.rs,.go,.java,.yaml,.yml,.xml,.html,.css,.toml,.sh,.sql,.log,.pdf"
          style={{ display: 'none' }}
          onChange={(e) => handleUpload(e.target.files)}
        />
        <button
          onClick={() => fileInputRef.current?.click()}
          disabled={uploading}
          style={{
            background: 'var(--bg-active)',
            border: '1px solid var(--border)',
            borderRadius: 4,
            color: 'var(--text-primary)',
            fontSize: 12,
            padding: '6px 12px',
            cursor: uploading ? 'default' : 'pointer',
          }}
        >
          {uploading ? 'Uploading...' : '📎 Upload Document'}
        </button>
```

Place this button in the materials section header area, near the title/heading.

- [ ] **Step 2: Handle WebSocket material_added event**

In `ui/src/stores/session-store.ts`, find the `handleWsMessage` function. Add a case for `session_material_added`:

```typescript
      case 'session_material_added':
        if (data.session_id === get().active_session?.id) {
          const material = data.material
          set((state) => ({
            materials: [...state.materials, material],
          }))
        }
        break
```

- [ ] **Step 3: Verify build**

Run: `cd ui && npm run build`
Expected: Clean build.

- [ ] **Step 4: Commit**

```bash
git add ui/src/components/panels/SessionPanel.tsx ui/src/stores/session-store.ts
git commit -m "feat(upload): add file upload to session material prep"
```

---

### Task 8: Integration Test

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add upload integration test**

Add at end of `server/tests/integration.rs`:

```rust
#[tokio::test]
async fn test_file_upload_to_chat() {
    let app = setup_unique_app().await;
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    let content = "Hello, this is a test file content.";
    let filename = "test.txt";

    crate::services::upload::validate_file(filename, content.len()).unwrap();

    let msg = crate::services::upload::upload_to_chat(
        &app.db,
        Some(&ring_id),
        &token,
        "TestUser",
        filename,
        content.as_bytes(),
    )
    .await
    .unwrap();

    assert_eq!(msg.role, "system");
    assert!(msg.content.starts_with("📎 test.txt\n---\n"));
    assert!(msg.content.contains("Hello, this is a test file content."));

    let results = crate::services::search::search_cross_ring(
        &app.db,
        &[ring_id],
        "test file content",
        10,
    )
    .await
    .unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_file_upload_rejects_large_file() {
    let result = crate::services::upload::validate_file("big.txt", 11 * 1024 * 1024);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_file_upload_rejects_bad_extension() {
    let result = crate::services::upload::validate_file("evil.exe", 100);
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests**

Run: `cd server && cargo test`
Expected: All tests pass, including the 3 new upload tests.

- [ ] **Step 3: Run frontend build**

Run: `cd ui && npm run build`
Expected: Clean build.

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test(upload): add file upload integration tests"
```

---

### Task 9: Update STATUS.md

**Files:**
- Modify: `docs/STATUS.md`

- [ ] **Step 1: Update STATUS.md**

Move "文件上传" from the PRD missing section to the completed section. Add:

```
- **文件上传** — 支持在 Group Ring/Super Ring 聊天和 Session 材料准备中上传文本文件（PDF/TXT/MD/CSV/代码），自动提取内容注入对话上下文，📎 按钮上传 + 拖拽 + 粘贴，文件卡片渲染
```

- [ ] **Step 2: Commit**

```bash
git add docs/STATUS.md
git commit -m "docs: update STATUS.md with file upload completion"
```
