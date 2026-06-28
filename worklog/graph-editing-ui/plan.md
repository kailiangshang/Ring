# Graph Editing UI Plan

## Approach

Add missing backend edge-update capability first, then wire explicit edge selection/editing into the frontend graph store and graph panel. Keep the interaction model simple: select an edge to edit it, select a node to start relation creation, then choose a target node and confirm.

## Phases

1. Task setup and current graph interaction audit.
2. Backend:
   - add edge update input/model/service/route
   - cover edge update/delete with integration tests
3. Frontend store and types:
   - add selected edge state
   - add edge update action
   - extend create edge to support labels
4. Frontend UI:
   - edge click selection and highlight in canvas
   - visible edge edit/delete panel
   - visible relation creation flow from selected node
5. Verification:
   - targeted frontend test updates
   - backend integration test
   - local build and smoke check

## Milestones

- M1: backend edge update supported
- M2: graph store supports edge selection and mutation
- M3: graph canvas and panel expose visible edge editing
- M4: visible node-to-node relation creation works
- M5: tests and build pass

## Risks

- D3 edge click targets may be too small without an explicit hit area.
- Graph panel state can become confusing if node selection and edge selection overlap.
- Existing float graph view uses `GraphCanvas` too, so prop changes must stay compatible.

## Validation

- Clicking an edge updates selected-edge UI state.
- Editing relation type persists to backend and refreshes the graph.
- Deleting an edge removes it from graph API response and canvas.
- Starting a relation from one node and choosing another creates an edge without using `Shift`.
- Existing node creation and graph generation flows still work.
