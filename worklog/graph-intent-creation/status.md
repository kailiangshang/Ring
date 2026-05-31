# Graph Intent Creation Status

## Current Status

Completed.

## Completed

- Identified the current disconnect between graph intent, archive routing, and actual graph writes.
- Confirmed that `knowledge_extract` currently returns suggestions only.
- Confirmed that actual graph writes only happen through graph APIs or archive-with-node-suggestion flow.
- Initialized worklog for this task.
- Added explicit `detect_graph_intent` routing ahead of archive intent in ring chat.
- Added backend graph preview responses that return `knowledge_extraction` plus `graph_action=confirm_create_graph`.
- Added explicit `graph_id` support to graph node/edge creation models and services.
- Updated frontend graph store to create extracted nodes/edges in the active graph and dedupe labels/edges.
- Added frontend confirmation modal flow in chat messages before graph creation.
- Added targeted backend and frontend regression tests.
- Verified frontend build and targeted tests.
- Fixed backend graph-preview parsing to accept `<knowledge_extraction>...</knowledge_extraction>` payloads instead of only raw JSON.
- Normalized graph preview payloads so frontend receives `concepts[{label,node_type,tags}]` and `relations[{from,to,relation}]`.
- Fixed frontend post-create refresh so it stays on the graph that was actually written.
- Ran end-to-end verification against a live local instance:
  - generated graph preview from chat history,
  - confirmed creation in the UI,
  - verified nodes and edges persisted in SQLite and were visible in the Graph panel.
- Verified that the real ring `微服务架构ring` had `graph_nodes=0` and `graph_edges=0` despite misleading chat replies claiming success.
- Expanded graph intent detection to cover short follow-up prompts such as `图谱呢` and other explicit Chinese graph-generation phrases.
- Added route-level intent logging during debugging to confirm whether chat requests are entering the graph-intent branch.
- Tightened graph preview context selection to use substantive recent user discussion instead of polluted recent assistant graph chatter.
- Revalidated live backend behavior for the real ring:
  - `图谱呢` now logs `graph_intent=true`
  - chat returns a `graph_action=confirm_create_graph` preview payload instead of fake success copy.
- Added a backend fallback relation synthesizer so graph previews do not degrade into node-only graphs when the model omits `relations`.
- Strengthened the knowledge extraction prompt to require non-empty relations for multi-concept graphs and to force relation endpoints to reuse concept labels exactly.
- Added route-level tests covering relation backfill for empty or partially disconnected extraction results.

## In Progress

- None.

## Blockers

- None.

## Latest Decisions

- Graph generation only triggers on explicit graph intent.
- Confirmation is required before writing graph data.
- Confirmed graph target is the currently selected graph, with fallback to default graph.
- Kept archive intent behavior intact for archive phrases; graph phrases are intercepted earlier by the new graph-intent branch.

## Next Steps

- Consider tightening `detect_archive_intent` keywords so graph phrases are removed from that list instead of relying on route ordering.
- Consider improving the preview text and extraction failure states in Chinese to match the rest of the product copy.
- Add a route-level chat integration test once LLM/tool execution is mockable in test harness.
- Remove or downgrade temporary route-level intent debug logging after this fix is considered stable.

## Verification

- `npm test -- graph-store`
- `npm run build`
- `cargo test --test integration test_graph_service_respects_explicit_graph_id` with `CARGO_TARGET_DIR=target-test`
- `cargo test detect_graph_intent_keywords --lib -- --nocapture`
- `cargo test normalize_extraction --lib -- --nocapture`
- Live backend check: `GET /api/health` returned OK on `http://127.0.0.1:7420`
- Live E2E result on ring `01KSY7BX666GX81NWFXAJH7KEM`:
  - UI confirmation message: `Graph updated: 10 node(s), 9 edge(s) created in main.`
  - SQLite counts: `graph_nodes=10`, `graph_edges=9`
  - Graph API returned persisted nodes/edges for the ring
- Live real-ring debug result on `01KSXZF0RDJ6KV5RBPRJK8WWFP`:
  - SQLite before fix confirmation: `graph_nodes=0`, `graph_edges=0`
  - POST chat with `图谱呢` now returns a graph preview payload containing `graph_action` and `knowledge_extraction`
  - Intent log confirms `graph_intent=true`, `archive_intent=false`
- Live relation fallback verification on `01KSXYFEAF1VS56EVV358GPXAG`:
  - a fresh graph preview for `图谱呢` returned non-empty relations after fallback
  - direct response inspection showed `relations=31` in the preview payload

## Remaining Risks

- `detect_archive_intent` still contains graph-related keywords in `server/src/services/chat.rs`; current behavior is correct because `ring_chat` checks `detect_graph_intent` first, but the two detectors are still semantically overlapping.
- The new chat preview path still depends on `execute_knowledge_extract`; if the model returns content with no recoverable concepts, the user gets a clear preview failure message and no graph is created.
- Some model-generated labels/relations are still mojibake in this environment for non-ASCII text, so English concept names are currently safer for deterministic graph verification.
