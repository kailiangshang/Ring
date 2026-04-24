# Self Memory Phase 1 — Core Files + Auto-Extraction

## Goal

Add a long-term memory system to Self that automatically extracts facts from conversations, stores them in categorized files, and loads them into the AI's system prompt.

## Phase 1 Scope

- 3 Tier 1 memory files (always loaded into Self's system prompt)
- Post-chat async extraction via LLM
- Memory panel UI (view/edit/delete)
- Auto-compression when files exceed size limit

## File Structure

```
~/.ring/self/memory/
├── user_profile.md      # Facts about the user (name, role, skills, timezone)
├── preferences.md       # Interaction preferences (style, habits, likes)
└── active_goals.md      # Current goals and ongoing tasks
```

Each file is plain markdown bullet points, auto-generated and user-editable.

## Backend

### New Service: `server/src/services/self_memory.rs`

**Functions:**

```rust
pub fn ensure_memory_dir(self_dir: &Path) -> PathBuf
// Returns ~/.ring/self/memory/, creates if not exists

pub fn read_memory_file(self_dir: &Path, name: &str) -> Result<(String, bool)>
// Reads memory/{name}.md, returns (content, exists)

pub fn write_memory_file(self_dir: &Path, name: &str, content: &str) -> Result<()>
// Writes memory/{name}.md

pub fn list_memory_files(self_dir: &Path) -> Result<Vec<MemoryFileInfo>>
// Lists all memory files with name, size, last_modified

pub fn delete_memory_file(self_dir: &Path, name: &str) -> Result<()>
// Deletes a memory file

pub async fn extract_memories(
    llm: &LlmClient,
    self_dir: &Path,
    user_message: &str,
    ai_response: &str,
) -> Result<()>
// Post-chat extraction: LLM analyzes conversation, extracts facts,
// classifies into categories, writes to files

pub async fn compress_memory(
    llm: &LlmClient,
    self_dir: &Path,
    name: &str,
) -> Result<()>
// When file exceeds 500 tokens, LLM rewrites it concisely

pub fn build_memory_context(self_dir: &Path) -> String
// Reads all Tier 1 files, formats for system prompt injection
```

**Extraction prompt** (used in `extract_memories`):

The LLM receives the user message + AI response and outputs a JSON array of facts:
```json
[
  {"fact": "User works as a software engineer", "category": "user_profile"},
  {"fact": "User prefers concise answers", "category": "preferences"}
]
```

Categories map to files: `user_profile` → `user_profile.md`, `preferences` → `preferences.md`, `goals` → `active_goals.md`.

Each extracted fact is appended as a bullet point. If a fact contradicts an existing line (simple substring match), the old line is replaced.

**Compression trigger:** When any Tier 1 file exceeds 2000 chars (~500 tokens), run `compress_memory` which asks the LLM to rewrite the file concisely.

### New Routes: `server/src/routes/self_data.rs` additions

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/self/memory` | List all memory files |
| GET | `/api/self/memory/{name}` | Read a memory file |
| PUT | `/api/self/memory/{name}` | Write/edit a memory file |
| DELETE | `/api/self/memory/{name}` | Delete a memory file |

### Integration into Self Chat

In `server/src/routes/chat.rs` `self_chat` handler, after the SSE stream ends (in the `SseEvent::End` handler), spawn an async task:

```rust
tokio::spawn(async move {
    let _ = crate::services::self_memory::extract_memories(
        &llm, &self_dir, &user_message, &full_content,
    ).await;
    let _ = crate::services::self_memory::check_and_compress(&llm, &self_dir).await;
});
```

### System Prompt Update

In `server/src/services/chat.rs` `build_system_prompt` (when `ring_name` is None = Self mode), add:

```rust
let memory_ctx = crate::services::self_memory::build_memory_context(&self_dir);
// Append to prompt: "\n\n## 长期记忆\n{memory_ctx}"
```

## Frontend

### Memory Tab in SelfFloat

Replace the current `SelfMemory.tsx` (which shows read-only metrics) with a new version that has two sections:

1. **Memory Files** — Cards for each memory file showing:
   - File name (translated: user_profile → "用户画像", preferences → "偏好", active_goals → "当前目标")
   - Preview of first 3 lines
   - Click to expand/edit
   - Delete button

2. **Metrics** — Keep existing metrics dashboard below

### Edit View

When a memory file is clicked, show a textarea with the full content and a SAVE button. Calls `PUT /api/self/memory/{name}`.

## Constraints

- Memory files are private to Self — never shared with Rings or other users
- Extraction is best-effort (failures are logged, not shown to user)
- Compression preserves user-edited content (never auto-delete user-written lines)
- Max 3 Tier 1 files in Phase 1 (user_profile, preferences, active_goals)
- Token budget: ~2000 tokens total for all Tier 1 files in system prompt

## Out of Scope (Future Phases)

- Tier 2 knowledge files with keyword retrieval
- Episodic/weekly summaries
- People/relationship tracking
- Memory search across files
- Conflict resolution beyond simple replacement
