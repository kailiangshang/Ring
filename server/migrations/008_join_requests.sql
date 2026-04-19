CREATE TABLE join_requests (
    id TEXT PRIMARY KEY,
    ring_id TEXT NOT NULL REFERENCES rings(id),
    invite_token TEXT NOT NULL REFERENCES invite_tokens(token),
    display_name TEXT NOT NULL,
    message TEXT,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'approved', 'rejected')),
    reviewer_id TEXT REFERENCES users(token_id),
    review_note TEXT,
    reviewed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
