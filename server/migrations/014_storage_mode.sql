ALTER TABLE rings ADD COLUMN storage_mode TEXT NOT NULL DEFAULT 'local';

CREATE TABLE IF NOT EXISTS pending_reviews (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    archive_record_id TEXT NOT NULL REFERENCES archive_records(id) ON DELETE CASCADE,
    source_branch TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'open',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_pending_reviews_ring ON pending_reviews(ring_id);
CREATE INDEX IF NOT EXISTS idx_pending_reviews_status ON pending_reviews(status);
