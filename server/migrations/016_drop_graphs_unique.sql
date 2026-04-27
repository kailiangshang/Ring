DROP INDEX IF EXISTS idx_graphs_ring;
CREATE INDEX IF NOT EXISTS idx_graphs_ring_id ON graphs(ring_id);
