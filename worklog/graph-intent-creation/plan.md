# Graph Intent Creation Plan

## Approach

Add a dedicated graph intent branch in group chat, keep it separate from archive intent, and return a structured preview that the frontend can confirm before actual graph writes.

## Phases

1. Add worklog and inspect current graph/archive/chat flow.
2. Backend:
   - split archive and graph intent detection
   - add graph extraction preview branch
   - support explicit `graph_id` for node and edge creation
3. Frontend:
   - parse graph action marker
   - auto-open confirmation modal
   - write extracted nodes and edges into the selected graph
4. Verification:
   - targeted backend tests
   - targeted frontend tests if lightweight, otherwise manual verification notes

## Milestones

- M1: intent routing fixed
- M2: preview response returned for graph intent
- M3: confirm-to-create flow works in UI
- M4: tests and verification complete

## Risks

- Prompted extraction may return malformed JSON.
- Duplicate concept labels can create redundant nodes if not deduped.
- Frontend auto-popup must avoid reopening repeatedly on rerender.

## Validation

- Explicit graph phrases trigger preview, not archive.
- Confirm writes nodes and edges.
- Cancel performs no writes.
- Existing `/save` archive path still works.

