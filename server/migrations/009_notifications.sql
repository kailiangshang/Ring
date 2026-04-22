CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(token_id),
    ring_id TEXT REFERENCES rings(id),
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    is_read BOOLEAN NOT NULL DEFAULT 0,
    related_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_unread ON notifications(user_id, is_read);
