# Cross-Ring Full-Text Search Design

## Goal

Enable Super Ring to answer cross-Ring knowledge questions by searching all text content the user has access to, then synthesizing an answer with inline citations that link to source material.

## Approach

SQLite FTS5 for retrieval + LLM synthesis. No external dependencies, no separate search UI — embedded in Super Ring chat.

## Data Layer

### FTS5 Virtual Table: `search_index`

```sql
CREATE VIRTUAL TABLE search_index USING fts5(
    source_type,
    source_id,
    ring_id,
    ring_name,
    title,
    content,
    metadata,
    content=''              -- not external content mode
);
```

| Column        | Description                                                       |
|---------------|-------------------------------------------------------------------|
| `source_type` | `message`, `session_message`, `graph_node`, `session`, `group_doc`, `archive_file` |
| `source_id`   | PK of the source row                                              |
| `ring_id`     | Ring scope for membership filtering                               |
| `ring_name`   | Denormalized for citation display                                 |
| `title`       | Node label / session title / doc name / archive filename          |
| `content`     | Full searchable text                                              |
| `metadata`    | JSON: `{message_role, session_id, node_type, ...}`               |

**Tokenizer:** `unicode61` with `tokenchars "_"`. Handles CJK characters natively (each character is a token) and works well for mixed Chinese/English text.

### Searchable Content Sources

| source_type       | Origin table           | title field              | content field                      |
|-------------------|------------------------|--------------------------|------------------------------------|
| `message`         | `messages`             | `sender_name`            | `content`                          |
| `session_message` | `session_messages`     | `sender_name`            | `content`                          |
| `graph_node`      | `graph_nodes`          | `label`                  | `content` + tags joined           |
| `session`         | `sessions`             | `title`                  | `description` + `summary`         |
| `group_doc`       | `group_docs`           | `doc_name`               | `content`                          |
| `archive_file`    | filesystem `archives/` | filename                 | file contents (read at write time) |

### Migration

New migration `014_create_search_index.sql`:

1. Create the FTS5 virtual table
2. Populate from existing `messages`, `session_messages`, `graph_nodes`, `sessions`, `group_docs`
3. For archive files: iterate `~/.ring/rings/<ring_id>/archives/*.md`, read content, insert

## Backend

### New Files

- `server/src/services/search.rs` — search service
- No new route file — search is called internally by `super_chat.rs`

### Search Service API

```rust
pub struct SearchResult {
    pub source_type: String,
    pub source_id: String,
    pub ring_id: String,
    pub ring_name: String,
    pub title: String,
    pub snippet: String,    // truncated to 500 chars
    pub rank: f64,
}

pub async fn search_cross_ring(
    db: &SqlitePool,
    user_id: &str,
    query: &str,
    limit: i64,
) -> Result<Vec<SearchResult>> {
    // 1. Get all ring_ids the user is a member of
    // 2. SELECT from search_index WHERE ring_id IN (...) AND MATCH query
    // 3. ORDER BY bm25(search_index) LIMIT
    // 4. Truncate content to 500 chars
}
```

### Index Maintenance Helper

```rust
pub async fn upsert_search_index(
    db: &SqlitePool,
    source_type: &str,
    source_id: &str,
    ring_id: &str,
    ring_name: &str,
    title: &str,
    content: &str,
    metadata: &str,
) -> Result<()>

pub async fn delete_search_index(
    db: &SqlitePool,
    source_type: &str,
    source_id: &str,
) -> Result<()>
```

### Integration into Super Ring Chat

In `server/src/services/super_chat.rs`, before the LLM call:

1. **Heuristic trigger:** If user message length >= 5 chars and doesn't start with `/`, run search
2. **Extract keywords:** Sanitize the user message for FTS5 (escape special chars `* " ( ) : ^`, split into space-separated terms joined with `OR`)
3. **Run `search_cross_ring()`** with limit=20
4. **Format results** into a `<cross_ring_context>` block in the system prompt:
   ```
   <cross_ring_context>
   [Ring: Backend Team > Node: API Design]
   The API follows REST conventions with...

   [Ring: Frontend Team > Session: Sprint Review Apr 20]
   We decided to use WebSocket for real-time...
   </cross_ring_context>
   ```
5. **LLM system prompt** includes instruction: "When using cross_ring_context, cite sources inline as [RingName > Title]. These are clickable links for the user."

### Call Sites for Index Maintenance

| Operation                        | Service file              | Index action         |
|----------------------------------|---------------------------|----------------------|
| Send Ring/Self/Super message     | `chat.rs`                 | upsert (source_type=message) |
| Create/update/delete graph node  | `graph.rs` (model layer)  | upsert / delete      |
| Create/update session            | `session.rs`              | upsert               |
| Send session message             | `session.rs` (WS handler) | upsert               |
| Update group doc                 | `group_docs.rs`           | upsert               |
| Write archive file               | `archive_service.rs`      | upsert               |
| Delete Ring                      | `ring.rs`                 | delete WHERE ring_id |

## Frontend

### Citation Rendering

In `ui/src/components/chat/MessageItem.tsx`:

- Detect citation pattern `[RingName > SourceTitle]` via regex
- Render as clickable `<a>` elements with subtle link styling
- On click, dispatch navigation action via stores

### Navigation Mapping

| source_type       | Click action                                          |
|-------------------|-------------------------------------------------------|
| `graph_node`      | Switch to Ring → open Graph panel → highlight node    |
| `message`         | Switch to Ring → scroll to message in chat            |
| `session_message` | Switch to Ring → open Session panel → highlight msg   |
| `session`         | Switch to Ring → open Session panel                   |
| `group_doc`       | Switch to Ring → show tooltip (no doc panel yet)      |
| `archive_file`    | Switch to Ring → open Archive panel                   |

### Implementation

- Regex: `/\[([^\]]+ > [^\]]+)\]/g` — matches `[Ring > Title]`
- Replace with `<a onClick={navigate}>[Ring > Title]</a>`
- Navigation uses existing store actions: `ringStore.selectRing()`, `panelStore.openPanel()`, etc.
- For graph node navigation: `graphStore.selectNode(source_id)` + `panelStore.openPanel('graph')`

### No New Components

All changes are in existing components. No new stores needed — uses existing `ring-store`, `panel-store`, `graph-store`.

## Constraints

- **Membership check:** Search only returns results from Rings the user is a member of
- **Performance:** Max 20 results per query, each truncated to 500 chars. Skip search for messages < 5 chars or starting with `/`
- **No background indexing:** All indexing is synchronous on write
- **Chinese support:** `unicode61` tokenizer handles CJK characters as individual tokens, which works well for Chinese full-text search
- **Archive content:** Indexed at write time into FTS5. If archive files are modified outside of Ring, they won't be re-indexed (acceptable for v1)

## Out of Scope (v1)

- Semantic / embedding-based search
- Dedicated search panel or `/search` command
- Re-indexing on external file changes
- Search within Self chat (Self is private, not cross-Ring)
- Highlighting search terms in results UI
