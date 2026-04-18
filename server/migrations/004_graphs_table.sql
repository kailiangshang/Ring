CREATE TABLE IF NOT EXISTS graphs (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT 'main',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_graphs_ring ON graphs(ring_id);

ALTER TABLE graph_nodes ADD COLUMN graph_id TEXT REFERENCES graphs(id) ON DELETE CASCADE;
ALTER TABLE graph_nodes ADD COLUMN parent_id TEXT REFERENCES graph_nodes(id) ON DELETE SET NULL;
ALTER TABLE graph_nodes RENAME COLUMN kind TO node_type;
ALTER TABLE graph_nodes ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
ALTER TABLE graph_nodes ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'));

ALTER TABLE graph_edges ADD COLUMN graph_id TEXT REFERENCES graphs(id) ON DELETE CASCADE;
ALTER TABLE graph_edges ADD COLUMN relation TEXT NOT NULL DEFAULT 'related_to';

CREATE INDEX IF NOT EXISTS idx_graph_nodes_graph ON graph_nodes(graph_id);
CREATE INDEX IF NOT EXISTS idx_graph_edges_graph ON graph_edges(graph_id);
