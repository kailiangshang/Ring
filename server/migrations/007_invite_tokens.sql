CREATE TABLE invite_tokens (
    token TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id),
    type TEXT NOT NULL CHECK(type IN ('open', 'audit')),
    role TEXT NOT NULL DEFAULT 'member' CHECK(role IN ('member', 'readonly')),
    max_uses INTEGER NOT NULL DEFAULT 1,
    use_count INTEGER NOT NULL DEFAULT 0,
    max_members INTEGER,
    expires_at TEXT NOT NULL,
    revoked_at TEXT,
    created_by TEXT NOT NULL REFERENCES users(token_id),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
