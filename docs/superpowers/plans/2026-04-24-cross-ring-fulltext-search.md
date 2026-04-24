# Cross-Ring Full-Text Search Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable Super Ring to automatically search across all user Rings and inject relevant context into the system prompt, producing answers with inline citations that link to source material.

**Architecture:** SQLite FTS5 virtual table (`search_index`) indexes all searchable text. On Super Ring chat, backend runs FTS5 query before LLM call, injects results as `<cross_ring_context>` into system prompt. LLM synthesizes answer with `[RingName > Title]` citations. Frontend renders citations as clickable links.

**Tech Stack:** SQLite FTS5 (unicode61 tokenizer), Rust (sqlx), React/TypeScript, existing Zustand stores

---

### Task 1: FTS5 Migration

**Files:**
- Create: `server/migrations/013_search_index.sql`

- [ ] **Step 1: Write migration file**

```sql
CREATE VIRTUAL TABLE search_index USING fts5(
    source_type,
    source_id,
    ring_id,
    ring_name,
    title,
    content,
    metadata,
    content='',
    tokenize='unicode61'
);

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'message', m.id, m.ring_id, COALESCE(r.name, ''), m.sender_name, m.content,
    json_object('role', m.role)
FROM messages m
LEFT JOIN rings r ON m.ring_id = r.id
WHERE m.ring_id IS NOT NULL AND m.ring_id != 'super';

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'session_message', sm.id, s.ring_id, r.name, sm.sender_name, sm.content,
    json_object('session_id', sm.session_id, 'message_type', sm.message_type)
FROM session_messages sm
JOIN sessions s ON sm.session_id = s.id
JOIN rings r ON s.ring_id = r.id;

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'graph_node', gn.id, gn.ring_id, r.name, gn.label,
    gn.content || ' ' || gn.tags,
    json_object('node_type', gn.node_type, 'graph_id', gn.graph_id)
FROM graph_nodes gn
JOIN rings r ON gn.ring_id = r.id;

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'session', s.id, s.ring_id, r.name, s.title,
    COALESCE(s.description, '') || ' ' || COALESCE(s.summary, ''),
    json_object('skill', s.skill, 'phase', s.phase)
FROM sessions s
JOIN rings r ON s.ring_id = r.id;

INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
SELECT 'group_doc', gd.ring_id || ':' || gd.doc_name, gd.ring_id, r.name, gd.doc_name, gd.content,
    '{}'
FROM group_docs gd
JOIN rings r ON gd.ring_id = r.id;
```

- [ ] **Step 2: Run tests to verify migration passes**

Run: `cd server && cargo test`
Expected: All existing tests pass. Migration applies cleanly.

- [ ] **Step 3: Commit**

```bash
git add server/migrations/013_search_index.sql
git commit -m "feat(search): add FTS5 search_index migration"
```

---

### Task 2: Search Service

**Files:**
- Create: `server/src/services/search.rs`
- Modify: `server/src/services/mod.rs` — add `pub mod search;`

- [ ] **Step 1: Write search service**

```rust
use sqlx::SqlitePool;

use crate::error::{Result, RingError};

#[derive(Debug, sqlx::FromRow)]
pub struct SearchRow {
    pub source_type: String,
    pub source_id: String,
    pub ring_id: String,
    pub ring_name: String,
    pub title: String,
    pub content: String,
    pub metadata: String,
    pub rank: f64,
}

pub async fn search_cross_ring(
    db: &SqlitePool,
    ring_ids: &[String],
    query: &str,
    limit: i64,
) -> Result<Vec<SearchRow>> {
    if ring_ids.is_empty() || query.trim().is_empty() {
        return Ok(vec![]);
    }

    let fts_query = sanitize_fts_query(query);
    if fts_query.is_empty() {
        return Ok(vec![]);
    }

    let placeholders: Vec<&str> = ring_ids.iter().map(|_| "?").collect();
    let sql = format!(
        "SELECT source_type, source_id, ring_id, ring_name, title, content, metadata, rank \
         FROM search_index \
         WHERE search_index MATCH ?1 \
         AND ring_id IN ({}) \
         ORDER BY bm25(search_index) \
         LIMIT ?2",
        placeholders.join(",")
    );

    let mut q = sqlx::query_as::<_, SearchRow>(&sql).bind(&fts_query).bind(limit);
    for id in ring_ids {
        q = q.bind(id);
    }

    q.fetch_all(db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))
}

pub async fn upsert_search_index(
    db: &SqlitePool,
    source_type: &str,
    source_id: &str,
    ring_id: &str,
    ring_name: &str,
    title: &str,
    content: &str,
    metadata: &str,
) -> Result<()> {
    delete_search_index(db, source_type, source_id).await.ok();

    sqlx::query(
        "INSERT INTO search_index (source_type, source_id, ring_id, ring_name, title, content, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(source_type)
    .bind(source_id)
    .bind(ring_id)
    .bind(ring_name)
    .bind(title)
    .bind(content)
    .bind(metadata)
    .execute(db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn delete_search_index(
    db: &SqlitePool,
    source_type: &str,
    source_id: &str,
) -> Result<()> {
    sqlx::query(
        "DELETE FROM search_index WHERE source_type = ?1 AND source_id = ?2",
    )
    .bind(source_type)
    .bind(source_id)
    .execute(db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}

pub async fn delete_search_index_by_ring(
    db: &SqlitePool,
    ring_id: &str,
) -> Result<()> {
    sqlx::query("DELETE FROM search_index WHERE ring_id = ?1")
        .bind(ring_id)
        .execute(db)
        .await
        .map_err(|e| RingError::Internal(e.to_string()))?;

    Ok(())
}

fn sanitize_fts_query(input: &str) -> String {
    let chars_to_strip = ['*', '"', '(', ')', ':', '^', '{', '}', '[', ']'];
    let cleaned: String = input
        .chars()
        .map(|c| if chars_to_strip.contains(&c) { ' ' } else { c })
        .collect();
    let terms: Vec<&str> = cleaned
        .split_whitespace()
        .filter(|t| t.len() >= 2)
        .take(10)
        .collect();
    if terms.is_empty() {
        return String::new();
    }
    terms
        .iter()
        .map(|t| format!("{t}*"))
        .collect::<Vec<_>>()
        .join(" OR ")
}

pub async fn get_user_ring_ids(db: &SqlitePool, user_id: &str) -> Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT r.id FROM rings r JOIN members m ON r.id = m.ring_id WHERE m.user_id = ?1",
    )
    .bind(user_id)
    .fetch_all(db)
    .await
    .map_err(|e| RingError::Internal(e.to_string()))?;
    Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub async fn get_ring_name(db: &SqlitePool, ring_id: &str) -> Result<String> {
    let name: Option<String> =
        sqlx::query_scalar("SELECT name FROM rings WHERE id = ?1")
            .bind(ring_id)
            .fetch_optional(db)
            .await
            .map_err(|e| RingError::Internal(e.to_string()))?;
    Ok(name.unwrap_or_default())
}

pub fn format_search_context(results: &[SearchRow]) -> String {
    if results.is_empty() {
        return String::new();
    }
    let mut ctx = String::from("<cross_ring_context>\n以下是从用户的所有 Ring 中检索到的相关内容：\n\n");
    for r in results {
        let truncated: String = r.content.chars().take(500).collect();
        let ellipsis = if r.content.len() > 500 { "..." } else { "" };
        ctx.push_str(&format!(
            "[Ring: {} > {}]\ntype: {}, id: {}\n{}{}\n\n",
            r.ring_name, r.title, r.source_type, r.source_id, truncated, ellipsis
        ));
    }
    ctx.push_str("</cross_ring_context>");
    ctx
}
```

- [ ] **Step 2: Register module in mod.rs**

In `server/src/services/mod.rs`, add at end:

```rust
pub mod search;
```

- [ ] **Step 3: Run tests**

Run: `cd server && cargo test`
Expected: Compiles, all tests pass.

- [ ] **Step 4: Commit**

```bash
git add server/src/services/search.rs server/src/services/mod.rs
git commit -m "feat(search): add search service with FTS5 query, upsert, delete"
```

---

### Task 3: Index Maintenance — Messages

**Files:**
- Modify: `server/src/services/chat.rs:192-210` — after user message insert
- Modify: `server/src/services/chat.rs:113-130` — after AI message insert (auto_compact)

- [ ] **Step 1: Add search index upsert after user message insert in chat.rs**

In `server/src/services/chat.rs`, after the `message::insert_message` call at line 209 (the closing `});`), add:

```rust
    if let Some(ring_id) = params.ring_id {
        let ring_name = crate::services::search::get_ring_name(&state.db, ring_id).await.unwrap_or_default();
        let _ = crate::services::search::upsert_search_index(
            &state.db, "message", &user_msg_id, ring_id, &ring_name,
            &user.display_name, params.content,
            &serde_json::json!({"role": "user"}).to_string(),
        ).await;
    }
```

This goes after the closing `}` of the `if !params.ephemeral { ... }` block (after line 210).

- [ ] **Step 2: Add search index upsert for AI messages in ring chat**

In `server/src/routes/chat.rs`, after the AI message `insert_message` call at line 155-168 (inside the `SseEvent::End` handler for ring chat), add:

```rust
let ring_name_search = crate::services::search::get_ring_name(&pool, &ring_id_c).await.unwrap_or_default();
let _ = crate::services::search::upsert_search_index(
    &pool, "message", &message_id, &ring_id_c, &ring_name_search,
    "GROUP RING", &full_content,
    &serde_json::json!({"role": "group_ring"}).to_string(),
).await;
```

Note: Self chat messages (line 302-315) have `ring_id: None` — skip indexing those. Super Ring messages are indexed in Task 9 separately.

- [ ] **Step 3: Add search index upsert for auto_compact system messages**

In `server/src/services/chat.rs`, after the compact summary `insert_message` call at line 106-120, add:

```rust
if let Some(ring_id) = ring_id {
    let ring_name = crate::services::search::get_ring_name(&state.db, ring_id).await.unwrap_or_default();
    let _ = crate::services::search::upsert_search_index(
        &state.db, "message", &summary_id, ring_id, &ring_name,
        "SYSTEM", &format!("[历史摘要] {}", summary),
        &serde_json::json!({"role": "system"}).to_string(),
    ).await;
}
```

- [ ] **Step 3: Run tests**

Run: `cd server && cargo test`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add server/src/services/chat.rs server/src/routes/chat.rs
git commit -m "feat(search): index messages on write"
```

---

### Task 4: Index Maintenance — Graph Nodes

**Files:**
- Modify: `server/src/services/graph.rs:33-41` — create_node
- Modify: `server/src/services/graph.rs:43-49` — update_node
- Modify: `server/src/services/graph.rs:51-53` — delete_node

- [ ] **Step 1: Add index upsert after create_node in graph service**

In `server/src/services/graph.rs`, the `create_node` function (line 33-41) returns `Result<GraphNodeRow>`. After the model `create_node` call succeeds, add:

```rust
let ring_name = crate::services::search::get_ring_name(&state.db, &ring_id).await.unwrap_or_default();
let content = format!("{} {}", &node.content, &node.tags);
let metadata = serde_json::json!({"node_type": &node.node_type, "graph_id": &node.graph_id}).to_string();
let _ = crate::services::search::upsert_search_index(
    &state.db, "graph_node", &node.id, &ring_id, &ring_name,
    &node.label, &content, &metadata,
).await;
```

Where `node` is the returned `GraphNodeRow` from `models::graph::create_node`.

- [ ] **Step 2: Add index upsert after update_node in graph service**

In `server/src/services/graph.rs`, the `update_node` function (line 43-49). After the model call, add the same pattern:

```rust
let ring_name = crate::services::search::get_ring_name(&state.db, &node.ring_id).await.unwrap_or_default();
let content = format!("{} {}", &node.content, &node.tags);
let metadata = serde_json::json!({"node_type": &node.node_type, "graph_id": &node.graph_id}).to_string();
let _ = crate::services::search::upsert_search_index(
    &state.db, "graph_node", &node.id, &node.ring_id, &ring_name,
    &node.label, &content, &metadata,
).await;
```

- [ ] **Step 3: Add index delete after delete_node in graph service**

In `server/src/services/graph.rs`, the `delete_node` function (line 51-53). Before the model `delete_node` call, add:

```rust
let _ = crate::services::search::delete_search_index(&state.db, "graph_node", node_id).await;
```

- [ ] **Step 4: Run tests**

Run: `cd server && cargo test`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add server/src/services/graph.rs
git commit -m "feat(search): index graph nodes on create/update/delete"
```

---

### Task 5: Index Maintenance — Sessions

**Files:**
- Modify: `server/src/services/session.rs:41-122` — create_session
- Modify: `server/src/routes/ws.rs:139-158` — session_message insert

- [ ] **Step 1: Add index upsert after create_session**

In `server/src/services/session.rs`, the `create_session` function. After the successful `session::create_session` model call (which returns `SessionRow`), add:

```rust
let ring_name = crate::services::search::get_ring_name(&state.db, &ring_id).await.unwrap_or_default();
let content = format!("{} {}", &sess.description, "");
let metadata = serde_json::json!({"skill": &sess.skill, "phase": &sess.phase}).to_string();
let _ = crate::services::search::upsert_search_index(
    &state.db, "session", &sess.id, &ring_id, &ring_name,
    &sess.title, &content, &metadata,
).await;
```

Where `sess` is the returned `SessionRow`.

- [ ] **Step 2: Add index upsert after session_message insert in ws.rs**

In `server/src/routes/ws.rs`, after the successful `session::insert_message` call (line 145-158), add:

```rust
let ring_id = &sess.ring_id;
let ring_name = crate::services::search::get_ring_name(db, ring_id).await.unwrap_or_default();
let metadata = serde_json::json!({"session_id": session_id, "message_type": "user"}).to_string();
let _ = crate::services::search::upsert_search_index(
    db, "session_message", &id, ring_id, &ring_name,
    &sender_name, content, &metadata,
).await;
```

Note: `sess` is already available from the `session::get_session` call at line 122.

- [ ] **Step 3: Add index upsert after session phase change and summary**

In `server/src/services/session.rs`, find where `session::update_phase` and `session::set_summary` are called. After each, upsert the session index entry. The key locations:

1. After `session::update_phase` calls — update the search index for the session with new phase
2. After `session::set_summary` calls — update the search index with the summary in the content

For each, after the model call succeeds, add:

```rust
let ring_name = crate::services::search::get_ring_name(&state.db, &updated_session.ring_id).await.unwrap_or_default();
let content = format!("{} {}", &updated_session.description, updated_session.summary.as_deref().unwrap_or(""));
let metadata = serde_json::json!({"skill": &updated_session.skill, "phase": &updated_session.phase}).to_string();
let _ = crate::services::search::upsert_search_index(
    &state.db, "session", &updated_session.id, &updated_session.ring_id, &ring_name,
    &updated_session.title, &content, &metadata,
).await;
```

- [ ] **Step 4: Add index delete on session delete**

In `server/src/services/session.rs`, the `delete_session` function. After the model `delete_session` call, add:

```rust
let _ = crate::services::search::delete_search_index(&state.db, "session", session_id).await;
```

- [ ] **Step 5: Run tests**

Run: `cd server && cargo test`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add server/src/services/session.rs server/src/routes/ws.rs
git commit -m "feat(search): index sessions and session messages on write"
```

---

### Task 6: Index Maintenance — Group Docs

**Files:**
- Modify: `server/src/routes/group_docs.rs:84-92` — after UPSERT SQL

- [ ] **Step 1: Add index upsert after group doc update**

In `server/src/routes/group_docs.rs`, after the `sqlx::query(...)` UPSERT at line 84-92 (after `.execute(&state.db).await?;`), add:

```rust
let ring_name = crate::services::search::get_ring_name(&state.db, &ring_id).await.unwrap_or_default();
let source_id = format!("{}:{}", &ring_id, &doc_name);
let _ = crate::services::search::upsert_search_index(
    &state.db, "group_doc", &source_id, &ring_id, &ring_name,
    &doc_name, &body.content, "{}",
).await;
```

- [ ] **Step 2: Run tests**

Run: `cd server && cargo test`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add server/src/routes/group_docs.rs
git commit -m "feat(search): index group docs on update"
```

---

### Task 7: Index Maintenance — Archives

**Files:**
- Modify: `server/src/services/archive_service.rs:225-232` — after archive record insert

- [ ] **Step 1: Add index upsert after archive file write**

In `server/src/services/archive_service.rs`, the `archive_content_creator` function. After the `archive::insert_record` call (line 226-229) and before the status update (line 232), add:

```rust
let ring_name = crate::services::search::get_ring_name(pool, ring_id).await.unwrap_or_default();
let source_id = format!("archive:{}", &record_id);
let _ = crate::services::search::upsert_search_index(
    pool, "archive_file", &source_id, ring_id, &ring_name,
    &file_name, content, "{}",
).await;
```

- [ ] **Step 2: Run tests**

Run: `cd server && cargo test`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add server/src/services/archive_service.rs
git commit -m "feat(search): index archive files on write"
```

---

### Task 8: Add Search Prompt Section

**Files:**
- Modify: `server/src/prompts.rs` — add `pub mod search` block

- [ ] **Step 1: Add search prompt module**

At the end of `server/src/prompts.rs`, add:

```rust
pub mod search {
    pub fn cross_ring_context_instruction() -> String {
        "## 跨 Ring 知识检索\n\n\
         系统已根据用户的问题自动搜索了所有 Ring 中的相关内容，结果在 <cross_ring_context> 标签中。\n\n\
         引用规则：\n\
         - 使用 [Ring名 > 标题] 格式引用来源\n\
         - 引用必须是方括号格式，例如：[后端团队 > API 设计]\n\
         - 每个 Ring名 和标题之间用 > 分隔\n\
         - 在回答中自然地嵌入引用，不要单独列出\n\
         - 如果检索结果与用户问题无关，忽略它们\n\
         - 基于检索结果回答，但用自己的语言组织".to_string()
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd server && cargo test`
Expected: Compiles and passes.

- [ ] **Step 3: Commit**

```bash
git add server/src/prompts.rs
git commit -m "feat(search): add citation instruction prompt for LLM"
```

---

### Task 9: Integrate Search into Super Chat

**Files:**
- Modify: `server/src/services/super_chat.rs:490-493` — inject search context into system prompt

- [ ] **Step 1: Add search call before LLM invocation**

In `server/src/services/super_chat.rs`, in `stream_super_chat_inner`, after line 493 (where `system_prompt` is built) and before line 495 (where history is loaded), add the search logic:

```rust
let search_ctx = if content.len() >= 5 && !content.starts_with('/') {
    let ring_ids = crate::services::search::get_user_ring_ids(&state.db, &user.token_id).await.unwrap_or_default();
    if !ring_ids.is_empty() {
        let results = crate::services::search::search_cross_ring(&state.db, &ring_ids, &content, 20).await.unwrap_or_default();
        let ctx = crate::services::search::format_search_context(&results);
        if !ctx.is_empty() {
            format!("\n\n{}\n\n{}", crate::prompts::search::cross_ring_context_instruction(), ctx)
        } else {
            String::new()
        }
    } else {
        String::new()
    }
} else {
    String::new()
};
```

Then modify line 493 to include the search context:

Change:
```rust
let system_prompt = format!("{base_prompt}\n\n{ring_summary}\n\n## 用户偏好\n{prefs}");
```
To:
```rust
let system_prompt = format!("{base_prompt}\n\n{ring_summary}\n\n## 用户偏好\n{prefs}{search_ctx}");
```

- [ ] **Step 2: Add index upsert for Super Ring AI messages**

In `server/src/services/super_chat.rs`, after the AI message insert at line 695-709 (the `message::insert_message` for `role: "super_ring"`), add:

```rust
let _ = crate::services::search::upsert_search_index(
    &state.db, "message", &ai_msg_id, "super", "",
    "SUPER RING", &full_content,
    &serde_json::json!({"role": "super_ring"}).to_string(),
).await;
```

Note: Super Ring messages are indexed with `ring_id: "super"` but since "super" is not in the user's ring list, they won't appear in cross-ring search results (by design — Super chat is not a Ring).

- [ ] **Step 2: Run tests**

Run: `cd server && cargo test`
Expected: All tests pass.

- [ ] **Step 3: Commit**

```bash
git add server/src/services/super_chat.rs
git commit -m "feat(search): integrate FTS5 search into super chat flow"
```

---

### Task 10: Frontend — Citation Rendering

**Files:**
- Modify: `ui/src/components/chat/MessageItem.tsx` — add citation detection and click handler
- Modify: `ui/src/components/chat/MessageItem.tsx` — update ReactMarkdown components

- [ ] **Step 1: Add imports and citation click handler**

At the top of `ui/src/components/chat/MessageItem.tsx`, after the existing imports (line 5), add:

```typescript
import { useRingStore } from '../../stores/ring-store'
import { usePanelStore } from '../../stores/panel-store'
import { useAppStore } from '../../stores/app-store'
import { useGraphStore } from '../../stores/graph-store'
```

Inside the `MessageItem` component function (after line 71), add:

```typescript
  const selectRing = useRingStore((s) => s.selectRing)
  const panelOpen = usePanelStore((s) => s.open)
  const setActiveRing = useAppStore((s) => s.setActiveRing)
  const selectNode = useGraphStore((s) => s.selectNode)

  const rings = useRingStore((s) => s.rings)

  const handleCitationClick = (ringName: string) => {
    const ring = rings.find((r) => r.name === ringName)
    if (!ring) return
    selectRing(ring.id)
    setActiveRing(ring.id)
  }
```

- [ ] **Step 2: Move mdComponents inside component and add citation rendering**

The `mdComponents` object is currently a module-level constant (line 19). Move it inside the `MessageItem` function body so it can access `handleCitationClick` and `rings`. Convert it from a `const` to a `useMemo` or just define it inline.

Replace the `p` component in `mdComponents` with citation-aware rendering:

```typescript
  p(props: any) {
    const text = Array.isArray(props.children)
      ? props.children.join('')
      : String(props.children ?? '')
    const citationRegex = /\[([^\]]+ > [^\]]+)\]/g
    const parts: Array<{ text: string; citation?: { ringName: string; title: string; match: string } }> = []
    let lastIndex = 0
    let match: RegExpExecArray | null

    while ((match = citationRegex.exec(text)) !== null) {
      if (match.index > lastIndex) {
        parts.push({ text: text.slice(lastIndex, match.index) })
      }
      const [full, ref] = match
      const sep = ref.indexOf(' > ')
      const ringName = ref.slice(0, sep).trim()
      const title = ref.slice(sep + 3).trim()
      parts.push({ text: '', citation: { ringName, title, match: full } })
      lastIndex = match.index + full.length
    }
    if (lastIndex < text.length) {
      parts.push({ text: text.slice(lastIndex) })
    }

    return (
      <p style={{ margin: '0 0 8px' }}>
        {parts.map((part, i) =>
          part.citation ? (
            <a
              key={i}
              href="#"
              onClick={(e) => {
                e.preventDefault()
                handleCitationClick(part.citation!.ringName)
              }}
              style={{
                color: 'var(--accent-teal)',
                textDecoration: 'none',
                cursor: 'pointer',
                fontWeight: 600,
                borderBottom: '1px dashed var(--accent-teal)',
              }}
              title={`Go to Ring: ${part.citation.ringName}`}
            >
              {part.citation.match}
            </a>
          ) : (
            <span key={i}>{part.text}</span>
          )
        )}
      </p>
    )
  },

- [ ] **Step 3: Run build**

Run: `cd ui && npm run build`
Expected: Clean build with no errors.

- [ ] **Step 4: Commit**

```bash
git add ui/src/components/chat/MessageItem.tsx
git commit -m "feat(search): render cross-ring citations as clickable links"
```

---

### Task 11: Integration Test

**Files:**
- Modify: `server/tests/integration.rs` — add search integration test

- [ ] **Step 1: Write integration test**

Add a test function at the end of `server/tests/integration.rs`:

```rust
#[sqlx::test(migrations = "./migrations")]
async fn test_search_index_auto_populates(pool: SqlitePool) {
    let app = setup_app_with_pool(pool).await;
    let token = do_setup(&app).await;
    let ring_id = create_ring(&app, &token).await;

    sqlx::query(
        "INSERT INTO graph_nodes (id, graph_id, ring_id, label, node_type, content, tags, metadata, created_at, updated_at)
         VALUES ('node-1', 'graph-1', ?1, 'API设计', 'topic', 'REST API with JWT auth', '[]', '{}', datetime('now'), datetime('now'))"
    )
    .bind(&ring_id)
    .execute(&app.db)
    .await
    .unwrap();

    crate::services::search::upsert_search_index(
        &app.db, "graph_node", "node-1", &ring_id, "Test Ring",
        "API设计", "REST API with JWT auth []",
        "{}",
    ).await.unwrap();

    let ring_ids = crate::services::search::get_user_ring_ids(&app.db, &token).await.unwrap();
    assert!(!ring_ids.is_empty());

    let results = crate::services::search::search_cross_ring(&app.db, &ring_ids, "API JWT", 10).await.unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].source_type, "graph_node");
    assert_eq!(results[0].title, "API设计");
}

async fn setup_app_with_pool(pool: SqlitePool) -> AppState {
    let rings_dir = std::env::temp_dir().join(format!("ring-test-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let hub_dir = rings_dir.join("hub");
    let skills_dir = rings_dir.join("skills");
    std::fs::create_dir_all(&rings_dir).unwrap();
    std::fs::create_dir_all(&hub_dir).unwrap();
    std::fs::create_dir_all(&skills_dir).unwrap();
    AppState::new(pool, rings_dir, hub_dir, skills_dir)
}
```

- [ ] **Step 2: Run the test**

Run: `cd server && cargo test test_search_index_auto_populates`
Expected: PASS

- [ ] **Step 3: Run full test suite**

Run: `cd server && cargo test`
Expected: All tests pass.

- [ ] **Step 4: Run frontend build**

Run: `cd ui && npm run build`
Expected: Clean build.

- [ ] **Step 5: Commit**

```bash
git add server/tests/integration.rs
git commit -m "test(search): add FTS5 search integration test"
```

---

### Task 12: Update STATUS.md

**Files:**
- Modify: `docs/STATUS.md`

- [ ] **Step 1: Update STATUS.md**

Move "全文搜索（跨 Ring/节点/消息）" from the incomplete section to the completed section. Add a brief entry describing what was implemented.

- [ ] **Step 2: Commit**

```bash
git add docs/STATUS.md
git commit -m "docs: update STATUS.md with fulltext search completion"
```
