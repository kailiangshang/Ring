# Graph Editing UI Status

## Current Status

Implemented and verified locally.

## Completed

- Identified that graph relationships were only partially editable and that the old UI relied on a hidden multi-select gesture.
- Added backend edge update support:
  - `PUT /api/rings/{ring_id}/graph/edges/{edge_id}`
  - model/service/route wiring
  - graph snapshot persistence after edge updates
- Added backend integration coverage for edge update behavior.
- Extended the graph store with:
  - `selected_edge_id`
  - explicit edge selection
  - edge update action
  - labeled edge creation support
- Extended the graph canvas with:
  - clickable edge hit areas
  - selected-edge highlighting
- Extended the graph panel with:
  - visible `Link` action from a selected node
  - visible relation-creation flow after choosing a source node
  - visible relation editor for selected edges
  - edge delete confirmation
- Refined relation creation UX so users can:
  - explicitly switch whether they are editing the source or target node
  - clear either endpoint independently
  - swap source and target
  - see source/target node highlights directly in the graph canvas
- Rebuilt and restarted the backend on the main local target.
- Verified:
  - `cargo test --test integration test_graph_edge_update_route` using isolated `CARGO_TARGET_DIR=target-test`
  - `npm test -- --run src/test/stores/graph-store.test.ts --pool=threads`
  - `npm run build`

## In Progress

- None.

## Blockers

- None.

## Latest Decisions

- Keep the graph panel as the main editing surface.
- Support direct edge selection in canvas instead of adding a separate edge list first.
- Make relation creation visible from node selection rather than relying only on `Shift + click`.
- Preserve existing graph-generation behavior while layering manual editing on top.

## Next Steps

- Watch for UX polish gaps after real-user interaction, especially around discoverability and copy.
- Consider adding drag-to-connect or inline edge labels if we want a richer graph editor later.

## Final Result and Residual Risk

- Result: graph relationships are now visibly editable in the UI. Users can select an edge, update relation type/label, delete it, or start a visible node-to-node relation flow from the panel.
- Residual risk:
  - Some older text labels in `GraphPanel.tsx` still contain mojibake from pre-existing encoding issues outside the newly added edge-editing flow.
  - The UI now has a clearer manual relation flow, but more advanced editing patterns such as drag-to-connect are still out of scope.
