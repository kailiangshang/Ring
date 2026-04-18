CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    ring_id TEXT,
    user_id TEXT NOT NULL REFERENCES users(token_id),
    role TEXT NOT NULL,
    sender_name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    node_refs TEXT NOT NULL DEFAULT '[]',
    tag_refs TEXT NOT NULL DEFAULT '[]',
    token_usage TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_messages_ring ON messages(ring_id);
CREATE INDEX IF NOT EXISTS idx_messages_user ON messages(user_id);
CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(ring_id, created_at);
