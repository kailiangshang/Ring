# P0 Gap Fixes Design

Date: 2026-04-12

## Context

`known-gaps.md` lists 5 P0 gaps. Code review reveals GAP-02 (search index) and GAP-04 (markdown docs) are already fully resolved. Three remaining items need fixes.

## Fix A: Edge CRUD Persistence (GAP-01 residual)

### Problem

`handlers/graph.rs:102-122` — `create_edge` and `delete_edge` call `state.graph_store` directly, bypassing `GraphService`. Edge mutations never trigger `persist_graph()`, so edge changes are lost on restart.

### Solution

Add `create_edge` and `delete_edge` methods to `GraphService` that delegate to the store and then call `persist_graph()`. Update handlers to route through `make_graph_service()`.

### Files

| File | Change |
|------|--------|
| `ring-server/src/services/graph_service.rs` | Add `create_edge(graph_id, NewEdge) -> Result<EdgeData>` and `delete_edge(graph_id, edge_id) -> Result<()>` |
| `ring-server/src/handlers/graph.rs` | Replace direct `state.graph_store` calls with service methods |

### Design Details

`GraphService::create_edge`:
1. Validate source and target nodes exist via `store.get_node`
2. Call `store.create_edge`
3. Call `self.persist_graph(graph_id)`

`GraphService::delete_edge`:
1. Call `store.delete_edge`
2. Call `self.persist_graph(graph_id)`

Handler `create_edge` and `delete_edge` functions use `make_graph_service(&state)` like all node handlers already do. Import `NewEdge` type from `crate::graph::types` in the handler moves to the service layer.

## Fix B: Frontend active_tools Passthrough (GAP-03 residual)

### Problem

Frontend Toolbar toggle state (`tools: ToolStatus[]`) is purely local. The `send_message` API call only sends `{ message }` — no tool selection reaches the backend. Backend currently uses all registered tools unconditionally.

### Solution

Wire `active_tools` from UI through to backend:
1. Backend request structs accept `active_tools: Option<Vec<String>>`
2. `AiService` methods accept optional tool filter
3. `ToolDispatcher` gains a filtered definitions method
4. Frontend passes active tool names through the call chain

### Files

| Layer | File | Change |
|-------|------|--------|
| Backend | `handlers/conversation.rs` | Add `active_tools: Option<Vec<String>>` to `SendMessageRequest` |
| Backend | `handlers/ai.rs` | Add `active_tools: Option<Vec<String>>` to `SuperRingRequest` |
| Backend | `services/ai_service.rs` | `super_ring_chat`/`group_ring_chat` accept `active_tools: Option<Vec<String>>`, pass to `chat_with_tools` |
| Backend | `services/tool_engine/dispatcher.rs` | Add `definitions_filtered(names: Option<&[String]>) -> Vec<ToolDefinition>` |
| Frontend | `pages/RingSpace/ChatView.tsx` | Extract active tool names from state, pass to `send_message` |
| Frontend | `stores/chatStore.ts` | Add `active_tools` param to `send_message` store method |
| Frontend | `api/client.ts` | `send_message` accepts `active_tools`, includes in JSON body |

### Design Details

**Backend — ToolDispatcher.filtered_definitions:**
```rust
pub fn definitions_filtered(&self, names: Option<&[String]>) -> Vec<ToolDefinition> {
    let all = self.definitions();
    match names {
        Some(filter) if !filter.is_empty() => {
            all.into_iter().filter(|t| filter.contains(&t.name)).collect()
        }
        _ => all,
    }
}
```

When `active_tools` is `None` or empty, all tools are used (backward compatible).

**Backend — AiService flow change:**
```rust
// Before:
let tools = self.tool_dispatcher.definitions();
// After:
let tools = self.tool_dispatcher.definitions_filtered(active_tools.as_deref());
```

**Frontend — ChatView handle_send:**
```ts
const active_tool_names = tools.filter(t => t.active).map(t => t.name)
send_message(ringId, content, active_tool_names)
```

**Frontend — API client:**
```ts
body: JSON.stringify({ message: content, active_tools })
```

## Fix C: Naming Consistency (GAP-05)

### Problem

Mixed usage of "Super Ring" / "Group Ring" vs target naming "Ring Super" / "Ring Group" across prompts, components, routes, CSS, and mock handlers.

### Solution

Systematic rename:
- Prompt text: "Super Ring" → "Ring Super", "Group Ring" → "Ring Group"
- Components: `SuperRingChat` → `RingSuperChat`
- Store: `useSuperRingStore` → `useRingSuperStore`, file `superRingStore.ts` → `ringSuperStore.ts`
- Route: `/super-ring` → `/ring-super`
- CSS classes: `.super-ring-*` → `.ring-super-*`
- API: `super_ring_chat` → `ring_super_chat`, endpoint `/super-ring/chat` → `/ring-super/chat`
- Mock handlers: URL paths updated
- Backend routes: `/super-ring` → `/ring-super`

### Files

| File | Change |
|------|--------|
| `ring-server/src/services/context_loader.rs` | Prompt text: "Super Ring" → "Ring Super", "Group Ring" → "Ring Group" |
| `ring-server/src/routes.rs` | Route path `/super-ring` → `/ring-super` |
| `ring-frontend/src/pages/RingHub/SuperRingChat.tsx` | Rename file → `RingSuperChat.tsx`, component → `RingSuperChat`, CSS classes |
| `ring-frontend/src/pages/RingHub/SuperRingChat.test.tsx` | Rename file → `RingSuperChat.test.tsx`, update references |
| `ring-frontend/src/pages/RingHub/RingHub.css` | `.super-ring-*` → `.ring-super-*` |
| `ring-frontend/src/stores/superRingStore.ts` | Rename file → `ringSuperStore.ts`, `useSuperRingStore` → `useRingSuperStore` |
| `ring-frontend/src/components/layout/HubNavBar.tsx` | NavLink path and text |
| `ring-frontend/src/api/client.ts` | Function name, URL path |
| `ring-frontend/src/App.tsx` | Import, Route path |
| `ring-frontend/src/mocks/handlers.ts` | Mock URLs |
| Tests referencing old names | Update |

### Backend route change

The handler function `super_ring_chat` stays the same name internally (it's a handler, not user-facing). Only the URL path changes: `POST /super-ring/chat` → `POST /ring-super/chat`.

## Execution Order

1. **Fix A first** — smallest, no dependencies, quick to verify
2. **Fix B second** — touches both backend and frontend API layer
3. **Fix C last** — purely cosmetic, most files but lowest risk

Each fix includes updating relevant tests.

## Testing

- Fix A: Run `cargo test` — existing `graph_service` tests should pass; add edge persistence test
- Fix B: Run `cargo test` for backend + `cd ring-frontend && npm test` for frontend
- Fix C: Run `cd ring-frontend && npm test` — rename updates test references; verify no "super-ring" or "Super Ring" remains via grep
