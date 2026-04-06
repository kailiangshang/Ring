CREATE TABLE IF NOT EXISTS users (
    id           TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    avatar_url   TEXT,
    ip_address   TEXT,
    setup_completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS rings (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    description     TEXT,
    creator_id      TEXT NOT NULL REFERENCES users(id),
    gitlab_repo     TEXT NOT NULL,
    local_path      TEXT NOT NULL,
    next_token_id   INTEGER NOT NULL DEFAULT 2,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS graphs (
    id          TEXT PRIMARY KEY,
    ring_id     TEXT NOT NULL REFERENCES rings(id),
    name        TEXT NOT NULL,
    description TEXT,
    graph_type  TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS members (
    id           TEXT PRIMARY KEY,
    ring_id      TEXT NOT NULL REFERENCES rings(id),
    user_id      TEXT NOT NULL,
    token_id     INTEGER NOT NULL,
    display_name TEXT NOT NULL,
    role         TEXT NOT NULL DEFAULT 'member',
    joined_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(ring_id, user_id),
    UNIQUE(ring_id, token_id)
);

CREATE TABLE IF NOT EXISTS invite_tokens (
    id           TEXT PRIMARY KEY,
    ring_id      TEXT NOT NULL REFERENCES rings(id),
    token        TEXT NOT NULL UNIQUE,
    token_type   TEXT NOT NULL DEFAULT 'open',
    role         TEXT NOT NULL DEFAULT 'member',
    inviter_id   TEXT NOT NULL,
    max_uses     INTEGER NOT NULL DEFAULT 1,
    use_count    INTEGER NOT NULL DEFAULT 0,
    max_members  INTEGER,
    expires_at   DATETIME NOT NULL,
    used_at      DATETIME,
    revoked_at   DATETIME,
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS conversations (
    id              TEXT PRIMARY KEY,
    ring_id         TEXT NOT NULL REFERENCES rings(id),
    title           TEXT,
    mode            TEXT NOT NULL DEFAULT 'chat',
    context_mode    TEXT NOT NULL DEFAULT 'storage',
    token_count     INTEGER NOT NULL DEFAULT 0,
    token_limit     INTEGER NOT NULL DEFAULT 100000,
    auto_compact    BOOLEAN NOT NULL DEFAULT FALSE,
    summary         TEXT,
    compacted_at    DATETIME,
    created_by      TEXT NOT NULL REFERENCES users(id),
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id),
    role            TEXT NOT NULL,
    content         TEXT NOT NULL,
    sender_id       TEXT,
    tool_calls      TEXT,
    archived        BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);

CREATE TABLE IF NOT EXISTS archive_records (
    id              TEXT PRIMARY KEY,
    ring_id         TEXT NOT NULL REFERENCES rings(id),
    node_id         TEXT,
    conversation_id TEXT REFERENCES conversations(id),
    message_ids     TEXT,
    markdown_path   TEXT NOT NULL,
    archived_by     TEXT NOT NULL,
    git_commit_sha  TEXT,
    pr_status       TEXT,
    pr_url          TEXT,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS blueprint_templates (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    description TEXT,
    graphs      TEXT NOT NULL,
    is_system   BOOLEAN NOT NULL DEFAULT FALSE,
    created_by  TEXT,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sessions (
    id              TEXT PRIMARY KEY,
    ring_id         TEXT NOT NULL REFERENCES rings(id),
    title           TEXT,
    scenario        TEXT NOT NULL,
    created_by      TEXT NOT NULL,
    archive_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    status          TEXT NOT NULL DEFAULT 'active',
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS session_members (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    user_id     TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'participant',
    status      TEXT NOT NULL DEFAULT 'active',
    joined_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    left_at     DATETIME,
    UNIQUE(session_id, user_id)
);

CREATE TABLE IF NOT EXISTS session_messages (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    sender_id   TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'user',
    content     TEXT NOT NULL,
    seq_num     INTEGER NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_session_messages_seq ON session_messages(session_id, seq_num);

CREATE TABLE IF NOT EXISTS notifications (
    id          TEXT PRIMARY KEY,
    ring_id     TEXT NOT NULL REFERENCES rings(id),
    user_id     TEXT NOT NULL REFERENCES users(id),
    type        TEXT NOT NULL,
    title       TEXT NOT NULL,
    body        TEXT,
    related_id  TEXT,
    is_read     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, is_read);

CREATE TABLE IF NOT EXISTS settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE VIRTUAL TABLE IF NOT EXISTS nodes_search USING fts5(
    node_id,
    graph_id,
    label,
    content,
    tokenize='unicode61'
);
