# Graph Intent Creation Spec

## Background

Ring currently recognizes archive intent inside group chat, but graph-related phrases such as "save to graph" or "挂载到图谱" do not create graph content. The assistant may reply as if graph creation succeeded, while the backend only writes archive files or returns extraction suggestions.

## Goal

- Recognize explicit graph-generation intent in group chat.
- Generate a graph extraction preview from recent discussion context.
- Ask the user to confirm before writing graph data.
- On confirmation, create graph nodes and edges in the currently selected graph, or fall back to the default graph.

## Scope

- Backend chat intent routing.
- Graph extraction preview response shape.
- Graph create-node/create-edge input support for explicit `graph_id`.
- Frontend confirmation modal and graph creation flow.
- Focused tests for detection and graph creation behavior.

## Non-goals

- No fully automatic graph generation for ordinary chat without explicit intent.
- No changes to archive semantics beyond removing graph phrases from archive intent detection.
- No redesign of graph UI beyond the new confirmation flow.

## Constraints

- Use current Ring chat SSE response mechanism.
- Keep graph creation explicit and confirmable.
- Do not break existing manual graph editing or archive flow.

## Acceptance Criteria

- Saying "挂到图谱/生成图谱/提取到图谱" no longer triggers quick archive.
- The assistant returns a preview plus a confirmation prompt.
- Confirming creates graph nodes and edges in the selected graph.
- Cancelling creates nothing.
- Archive intent such as `/save` still works.

## Related Files

- `server/src/routes/chat.rs`
- `server/src/services/chat.rs`
- `server/src/services/workflow.rs`
- `server/src/services/graph.rs`
- `server/src/models/graph.rs`
- `ui/src/components/chat/MessageItem.tsx`
- `ui/src/stores/graph-store.ts`

