CREATE TABLE IF NOT EXISTS sync_meta (
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (ring_id, key)
);
