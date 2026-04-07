# Future Optimizations

## 1. Logging Layer Management

- Implement structured, layered logging via `tracing`
- Per-module log levels (e.g. `handlers=info, services=debug, db=warn`)
- Log rotation and persistence

## 2. Strict Naming Hierarchy

| Level | Name | Description |
|-------|------|-------------|
| Overall | Ring Hub | The entire platform |
| Top | Ring Super | Cross-ring meta AI |
| Mid | Ring Group | A group knowledge space (currently "Ring") |
| Low | Ring Session | Collaborative session within a Ring Group |

- All UI text, class names, component names, route labels, and variable names must follow this hierarchy
- Current inconsistencies to fix:
  - "Group Ring" → "Ring Group"
  - Code references: `ring_hub`, `ring_group`, `ring_session`
  - Component naming: `RingHub` → `RingGroupList` or similar

## 3. UI Layout & Visual Polish

- Optimize overall layout for visual consistency and user experience
- Logo placeholder: navbar brand area needs proper logo display (image + text), currently text-only
- Consistent spacing, typography, and color usage across all pages
- Dark mode support: most components use hardcoded inline colors instead of CSS custom properties
- Responsive design for different screen sizes
- Loading states and empty states with proper illustrations

## 4. Streaming Output & Tool Call Display

- Improve streaming output: show tokens incrementally with smooth rendering, support markdown streaming parse
- Tool call visualization: display tool invocation details (name, parameters, status) in real-time during execution
- Tool result display: show tool output with expandable/collapsible detail panels
- Reference implementation: similar to modern AI chat UIs (like Claude/ChatGPT web interface)
  - Streaming text with cursor indicator
  - Thinking/reasoning blocks with toggle
  - Tool calls shown as structured cards with status badges (running/success/error)
  - Code blocks with syntax highlighting during streaming

## 5. Tool Collection Management

- Tool registry split into two categories:
  - **System tools** (non-editable): built-in tools like Search, TextClean, WebScrape, PrivacyFilter, MarkdownGen
  - **User tools** (editable): tools created/edited by users at Ring Group level
- Allow users to upload custom tools (define name, description, parameters, execution logic)
- Tool CRUD UI in Ring Group settings: list, create, edit, delete user tools
- Tool versioning and validation before activation

## 6. Ring Super Context Injection (On-demand Query)

- Current limitation: Ring Super can only see Ring names/IDs, cannot access actual Ring content
- Design principle: **parameterized on-demand query**, never dump all data at once
- Query types to support:
  - `ring_summary`: node count, edge count, root nodes, last update time for a specific Ring
  - `ring_nodes`: list nodes with filters (type, date range, keyword)
  - `ring_recent`: recent conversations and archived documents (with pagination)
  - `ring_search`: FTS search across a Ring's content
  - `ring_graph_snapshot`: lightweight graph structure (nodes + edges) for a specific Ring
- Implementation:
  - LLM triggers tool calls to fetch specific Ring data on demand
  - Results injected into conversation context as needed
  - Pagination for large result sets
  - Cache with TTL to avoid repeated queries
- Avoid loading all Ring data into system prompt — too large, wasteful, and slow

## 7. Embedding & Vector Retrieval

- Settings page needs embedding model configuration (provider, model, dimensions, API key)
- Embedding pipeline: chunk documents → generate embeddings → store in vector DB
- Retrieval flow: user query → embed query → similarity search → inject top-k results into LLM context
- Avoid stuffing full documents into prompts — use embedding-based retrieval for long content
- Support local embedding models (Ollama) and cloud models (OpenAI text-embedding-3)
- Vector storage: SQLite vector extension or dedicated vector DB (qdrant/milvus)

## 8. Multi-conversation Management & Monitoring

- Current issue: single chatStore, switching conversations may block or lose state
- Separate store per conversation (Map<convId, ChatState>) to avoid cross-contamination
- Conversation list sidebar in Group Ring Chat for switching between conversations
- Request queue with cancellation: abort pending stream when switching conversations

### Phase 1: In-chat Tool Call Display (near-term)
- Tool call cards within assistant replies (already have ToolCallBubble/ToolResultBubble)
- Collapsible details: click to expand params and return values
- Status animation: running → success/error with elapsed time
- Progressive rendering: thinking → tool_call → tool_result → text in order

### Phase 2: Independent Monitor Panel (mid-term)
- Dedicated /monitor page or Monitor tab within Ring Group
- Request timeline: TTFT (time to first token), total latency per message
- Token usage: per conversation / per day consumption stats
- Tool call log: searchable history of all tool invocations
- LLM error rate: 429/500 error trends
- Connection health indicator (backend reachable/unreachable)

## 9. AI Reply Interactive Selection

- Support selectable options in AI responses (e.g. "Which approach do you prefer?" with clickable cards)
- LLM returns structured choice blocks, frontend renders as selectable buttons/cards
- Use cases: blueprint template selection, action confirmation, multi-option recommendations
- Visual design: highlight on hover, selected state animation, consistent with overall UI style

## 10. Data Management & Cleanup

- File preview: support previewing stored files (markdown, images, documents) before deleting
- Manual cleanup: per-file delete with confirmation, show file size and last access time
- Clear Auto: one-click cleanup of stale data with configurable retention policy
  - Old conversation history (beyond N days)
  - Log files beyond retention period
  - Orphaned archive records
  - Unused graph snapshots
- Storage dashboard: show disk usage breakdown by category (conversations, archives, graphs, logs)
- Safety: preview what will be deleted before executing, support undo within grace period

## 11. Session Archive → Knowledge Graph Pipeline

- Current state: `archive_enabled` is just a boolean toggle, no actual archive logic
- Full pipeline to implement:
  1. Session conversation → generate `.ring/` markdown files (structured, with metadata)
  2. Markdown files → LLM-assisted knowledge extraction (nodes, edges, categories)
  3. Extracted knowledge → write into petgraph via graph_service
- Trigger modes: manual archive button, auto-archive on session close, scheduled batch
- LLM extraction prompt: analyze conversation, identify key concepts, relationships, and hierarchy
- Deduplication: merge extracted nodes with existing graph, avoid duplicates
- Archive history: track which sessions have been archived, support re-extraction

## 12. Code Block Copy Button

- Add one-click copy button to all code blocks in chat bubbles (ReactMarkdown rendered)
- Copy to clipboard with visual feedback (icon change: copy → check)
- Apply to all chat views: Group Ring Chat, Ring Super, Blueprint, Session Chat

## 13. Blueprint Confirm Flow UX

- Current state: blueprint confirm is just a button, no structured review step
- Before confirm, show structured preview:
  - Mermaid graph rendering of the blueprint
  - List of all node types with their Markdown document templates
  - Estimated total nodes and relationships
  - Warning if any node types have no document template
- Require explicit user confirmation (checkbox or "I confirm" input) before creating graph
- Support "edit this part" to modify individual dimensions without restarting

## 14. Ring Local File System — Node Markdown Documents

- Current state: `~/.ring/` only has `data/ring.db` and `repos/ring-{name}/graph.json`, no markdown documents
- Target directory structure:
  ```
  ~/.ring/repos/ring-{name}/
  ├── graph.json           # 图谱持久化（已实现）
  ├── nodes/
  │   ├── {node_id}.md     # 每个节点的完整内容文档
  │   └── ...
  └── archive/
      └── {session_id}.md  # session 归档文档
  ```
- Node md document structure:
  - YAML frontmatter: node_id, type, labels, created_at, updated_at
  - Body: full content, code examples, references, `[[wiki-links]]` to other nodes
- Write triggers: blueprint confirm creates initial docs, archive extracts from session, user manual edit
- Read triggers: graph node detail view, search result detail, LLM context injection
- `markdown_path` field in NodeData currently always null — must be populated on node creation
- This is the core gap closing the knowledge management loop: conversation → document → graph node
