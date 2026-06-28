# Graph Editing UI Spec

## Background

The current graph UI can display nodes and edges, and it can create edges through a hidden two-node multi-select gesture. However, users cannot visibly select an existing edge, edit its relation, or delete it from the UI. This makes the graph feel generated but not truly editable.

## Goal

- Make graph relationships visibly editable in the UI.
- Allow users to select an existing edge and change or delete it.
- Replace the hidden edge-creation gesture with a visible relation-creation flow.

## Scope

- Backend graph edge update API.
- Frontend graph store support for selecting, creating, updating, and deleting edges.
- Graph canvas interaction for edge selection and highlighting.
- Graph panel controls for visible edge editing and visible edge creation flow.
- Focused backend and frontend tests for edge editing behavior.

## Non-goals

- No redesign of the graph layout engine.
- No collaborative multi-user live graph editing semantics.
- No bulk graph refactor or node hierarchy redesign.

## Constraints

- Keep existing graph node editing behavior working.
- Keep current graph generation flow compatible with manual editing.
- Reuse the current graph panel rather than introducing a separate editor screen.

## Acceptance Criteria

- Users can click an edge in the graph canvas and see it selected.
- Selected edges can be updated from the graph panel.
- Selected edges can be deleted from the graph panel.
- Users can start creating a relation from a selected node using a visible UI affordance.
- Creating a relation no longer depends on hidden `Shift + click` knowledge.

## Related Files

- `server/src/models/graph.rs`
- `server/src/services/graph.rs`
- `server/src/routes/graph.rs`
- `server/src/routes/mod.rs`
- `server/tests/integration.rs`
- `ui/src/types/graph.ts`
- `ui/src/stores/graph-store.ts`
- `ui/src/components/panels/GraphPanel.tsx`
- `ui/src/components/panels/GraphCanvas.tsx`
- `ui/src/test/stores/graph-store.test.ts`
