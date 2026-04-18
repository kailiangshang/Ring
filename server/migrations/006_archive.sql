CREATE TABLE IF NOT EXISTS archive_records (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    session_id TEXT REFERENCES sessions(id) ON DELETE SET NULL,
    node_id TEXT REFERENCES graph_nodes(id) ON DELETE SET NULL,
    file_name TEXT NOT NULL,
    commit_sha TEXT,
    branch TEXT,
    merge_request_iid INTEGER,
    status TEXT NOT NULL DEFAULT 'pending',
    archived_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_archive_records_ring ON archive_records(ring_id);
CREATE INDEX IF NOT EXISTS idx_archive_records_status ON archive_records(status);
CREATE INDEX IF NOT EXISTS idx_archive_records_archived_by ON archive_records(archived_by);

ALTER TABLE graph_nodes ADD COLUMN markdown_path TEXT;
