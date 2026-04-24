# File Upload Design

## Goal

Allow users to upload text-based files (PDF, TXT, Markdown, CSV, code files) in Group Ring/Super Ring chat and Session material prep. AI automatically reads extracted content as conversation context. Files are ephemeral — extracted text is injected as messages/materials, original files are deleted.

## Approach

Multipart upload → tmp file → extract text → inject as chat message or session material → delete file. No persistent storage, no new DB tables. Content flows naturally through existing message history and FTS5 search.

## Backend

### Endpoints

| Method | Path | Purpose | Target |
|---|---|---|---|
| POST | `/api/rings/{ring_id}/upload` | Group Ring chat upload | `messages` table |
| POST | `/api/super/upload` | Super Ring chat upload | `messages` table (ring_id="super") |
| POST | `/api/rings/{ring_id}/sessions/{session_id}/upload` | Session material upload | `session_materials` table |

### Upload Flow (Chat)

1. Accept multipart field `file`
2. Validate: allowed extensions only, max 10MB
3. Save to `/tmp/ring-upload-{ulid}.{ext}`, extract text:
   - `.pdf`: use `lopdf` crate to extract text
   - Everything else: `std::fs::read_to_string` (UTF-8)
4. Delete tmp file immediately after extraction
5. Truncate to 50000 chars
6. Insert into `messages` as `role: "system"`, `content: "📎 {filename}\n---\n{extracted_text}"`
7. Index into `search_index` (source_type: "message")
8. Return `MessageRow` JSON

### Upload Flow (Session)

1. Same file validation and text extraction
2. Insert into `session_materials` as `item_type: "document"`, `title: "{filename}"`, `content: extracted_text`, `status: "ready"`
3. Return `SessionMaterialRow` JSON
4. Broadcast via WsHub to all session participants

### File Validation

**Allowed extensions:** `.txt`, `.md`, `.pdf`, `.csv`, `.json`, `.py`, `.js`, `.ts`, `.tsx`, `.rs`, `.go`, `.java`, `.yaml`, `.yml`, `.xml`, `.html`, `.css`, `.toml`, `.sh`, `.sql`, `.log`, `.env`, `.conf`, `.cfg`, `.ini`

**Max size:** 10MB

**PDF handling:** Use `lopdf` or `pdf-extract` crate for text extraction. If extraction fails (e.g. scanned PDF), return error suggesting the user paste the text content directly.

### New Dependencies

- `lopdf` crate for PDF text extraction
- Axum's built-in `Multipart` extractor (already available with `multipart` feature)

### New Files

- `server/src/services/upload.rs` — upload service (file handling, text extraction, message insertion)
- `server/src/routes/upload.rs` — upload route handlers

### Route Registration

In `server/src/routes/mod.rs`, add:
- `.route("/rings/{ring_id}/upload", post(upload::upload_ring_file))`
- `.route("/super/upload", post(upload::upload_super_file))`
- `.route("/rings/{ring_id}/sessions/{session_id}/upload", post(upload::upload_session_file))`

## Frontend

### Upload UI — InputArea.tsx

**📎 Button:** Small icon button to the left of the text input. Clicks a hidden `<input type="file" multiple>` with the allowed extensions as `accept` attribute.

**Drag & drop:** The entire InputArea component is a drop zone. `onDragOver` shows a visual indicator (border highlight), `onDrop` triggers upload.

**Paste:** `onPaste` on the input element checks `clipboardData.files`. If files present, trigger upload.

**Upload flow:**
1. User selects/drops/pastes file(s)
2. Show pending file card(s) with spinner in the chat area
3. Call `api.uploadFile(endpoint, file)` — new function using `FormData`
4. On success: replace pending card with the returned message
5. On error: show error on the card

### API Layer — api.ts

New function:
```typescript
export async function uploadFile(path: string, file: File): Promise<MessageRow> {
  const formData = new FormData()
  formData.append('file', file)
  const token = await getToken()
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers: token ? { 'X-Ring-Token': token } : {},
    body: formData,
  })
  // ... error handling, return JSON
}
```

Note: Do NOT set `Content-Type` header — browser sets it automatically with boundary for multipart.

### File Card Rendering — MessageItem.tsx

Detect file card messages: `role === "system"` AND content starts with `"📎 "`.

Render as a styled card:
- Header row: 📎 icon + filename + file size label
- Body: extracted text content, collapsible (reuse existing >200px collapse logic)
- Style: `border: 1px solid var(--border)`, `border-radius: 6px`, `padding: 8px 12px`, `background: var(--bg-active)`

### Session Material Upload — SessionPanel.tsx

In the Material Prep phase, add an upload button:
- `📎 Upload Document` button in the materials section header
- Same file validation, calls `/sessions/{id}/upload` endpoint
- Uploaded document appears in the materials list as `item_type: "document"` with filename as title
- Content is expandable

## Constraints

- **Ephemeral:** Files are deleted after extraction. No persistent storage.
- **Text only:** No image/binary processing. PDF requires actual text content (not scanned).
- **Size limit:** 10MB per file. Content truncated to 50000 chars.
- **Membership check:** Upload endpoints verify user is a member of the Ring (or session participant).
- **FTS5 integration:** Extracted text is indexed into `search_index` for cross-Ring search.
- **Auto-compact:** File card messages participate in auto-compact like any other message.

## Out of Scope (v1)

- Image upload / OCR
- Word/Excel binary format parsing
- File preview (rendered PDF, syntax-highlighted code)
- Drag-drop reordering of materials
- Multiple file upload progress bar (just sequential upload)
- File editing after upload
