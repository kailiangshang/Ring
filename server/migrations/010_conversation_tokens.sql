CREATE TABLE IF NOT EXISTS conversation_tokens (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(token_id),
    ring_id TEXT REFERENCES rings(id),
    total_tokens INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(user_id, ring_id)
);

CREATE INDEX IF NOT EXISTS idx_conversation_tokens_user ON conversation_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_conversation_tokens_ring ON conversation_tokens(ring_id);

ALTER TABLE users ADD COLUMN auto_compact BOOLEAN NOT NULL DEFAULT 1;
