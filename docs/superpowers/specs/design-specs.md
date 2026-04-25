# Ring Design Specs

All implemented features. Ordered by completion date.

---

## 1. Cross-Ring Full-Text Search (2026-04-24)

**Goal:** Super Ring searches all user-accessible text content via SQLite FTS5, synthesizes answers with citations.

- FTS5 virtual table `search_index` indexes messages, session_messages, graph_nodes, sessions, group_docs, archive_files
- `search_cross_ring()` filters by user's ring membership, `bm25` ranking, 20 results max
- `upsert_search_index()` / `delete_search_index()` called on every write path
- Super Chat injects `<cross_ring_context>` block into system prompt before LLM call
- Frontend renders `[Ring > Title]` citations as clickable navigation links

---

## 2. File Upload (2026-04-24)

**Goal:** Upload text files in Group Ring/Super Ring chat and Session material prep. Files are ephemeral — extracted text injected as messages, originals deleted.

- Endpoints: `POST /rings/{id}/upload`, `POST /super/upload`, `POST /rings/{id}/sessions/{sid}/upload`
- Multipart upload → tmp file → text extraction (lopdf for PDF, read_to_string for rest) → delete tmp → insert as system message
- Max 10MB, allowed extensions whitelist
- Content truncated to 50000 chars, indexed into FTS5
- Frontend: file button + drag-drop + paste in InputArea, file card rendering in MessageItem

---

## 3. Self Memory Phase 1 (2026-04-25)

**Goal:** Long-term memory for Self AI. Auto-extracts facts from conversations into categorized files, loads into system prompt.

- 3 memory files: `user_profile.md`, `preferences.md`, `active_goals.md` in `~/.ring/self/memory/`
- `extract_memories()`: post-chat LLM extraction → JSON facts → append to files
- `check_and_compress()`: when file > 2000 chars, LLM rewrites concisely
- `build_memory_context()`: reads all files → injects into Self system prompt
- Routes: CRUD on `/api/self/memory/{name}`
- Frontend: Memory panel in SelfFloat with view/edit/delete

---

## 4. Self Metrics (2026-04-25)

**Goal:** Track user behavior (dwell time + tool usage), display in frontend, inject summary into Self system prompt.

- `dwell_time.json`: cumulative + daily per-view seconds
- `tool_usage.json`: per-tool counts + last_used timestamps
- Heartbeat: frontend sends every 30s, backend batches in DwellBuffer (Mutex<HashMap>), flushes every 60s
- Tool usage instrumented at 9 route points (search, graph_edit, archive, upload, export, blueprint, session_create, session_summarize, memory_extract)
- `metrics_context()` in prompts.rs generates behavior summary for Self prompt
- Frontend: expanded metrics display in SelfMemory.tsx

---

## 5. Deep Blueprint Builder (2026-04-25)

**Goal:** AI-guided conversational blueprint design for Group Ring knowledge graphs, with live D3 preview.

- Two modes: quick path (template selection) and deep path (AI chat)
- Deep path: `POST /rings/{id}/blueprint/chat` → SSE stream with blueprint system prompt
- AI outputs `<blueprint>` JSON blocks in chat messages, frontend parses for live D3 preview
- Context: sliding window (15 msgs) + `current_blueprint` injection (no auto_compact)
- Confirm: creates graphs + nodes + edges from JSON
- Frontend: BlueprintPanel with mode selector, chat area, D3 MiniGraphPreview

---

## 6. Preset Workflow Tools (2026-04-25)

**Goal:** Two AI-driven tools (file_parse, knowledge_extract) for Group Ring via tool_calls.

- `chat_stream_with_tools()` on LlmClient: non-streaming first call → detect tool_calls → execute → streaming second call
- Tools: `file_parse` (file → structured extraction), `knowledge_extract` (text → concept extraction)
- `get_group_ring_tools()` + `execute_group_tool()` in chat.rs
- ring_chat handler uses tool-calling path instead of start_chat_stream
- Frontend: `<file_analysis>` / `<knowledge_extraction>` XML blocks → ExtractionCard with "添加到图谱" button
- `createNodesFromExtraction()` in graph-store creates nodes + edges from extraction data

---

## 7. Cross Ring Cache (2026-04-25)

**Goal:** Speed up Super Ring by caching ring_summary and ring_detail in memory.

- `CrossRingCache` = `Arc<Mutex<HashMap<String, (String, Instant)>>>` in AppState
- 3 key types: `summary:{user_id}`, `detail:{ring_id}`, `graph:{ring_id}`
- 5-min TTL, active invalidation on archive/graph/ring/member changes
- `get_summary()` and `get_detail()` with cache-aside pattern
- Fire-and-forget invalidation via `tokio::spawn` to not block write path
