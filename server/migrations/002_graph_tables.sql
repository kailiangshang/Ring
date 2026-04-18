CREATE TABLE IF NOT EXISTS graph_nodes (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'concept',
    content TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS graph_edges (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    source_id TEXT NOT NULL REFERENCES graph_nodes(id) ON DELETE CASCADE,
    target_id TEXT NOT NULL REFERENCES graph_nodes(id) ON DELETE CASCADE,
    label TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_graph_nodes_ring ON graph_nodes(ring_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_ring ON graph_edges(ring_id);
