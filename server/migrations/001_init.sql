CREATE TABLE IF NOT EXISTS users (
    token_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    avatar TEXT,
    is_creator BOOLEAN NOT NULL DEFAULT 0,
    llm_provider TEXT NOT NULL DEFAULT 'openai',
    llm_api_key TEXT,
    llm_model TEXT NOT NULL DEFAULT 'gpt-4o',
    llm_base_url TEXT,
    gitlab_url TEXT,
    gitlab_token TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS rings (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    creator_id TEXT NOT NULL REFERENCES users(token_id),
    role_description TEXT,
    interaction_mode TEXT NOT NULL DEFAULT 'normal',
    skill_permission_mode TEXT NOT NULL DEFAULT 'plan',
    auto_archive BOOLEAN NOT NULL DEFAULT 0,
    blueprint_status TEXT NOT NULL DEFAULT 'pending',
    gitlab_repo_url TEXT,
    gitlab_namespace TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS members (
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(token_id) ON DELETE CASCADE,
    role TEXT NOT NULL DEFAULT 'member',
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (ring_id, user_id)
);

CREATE TABLE IF NOT EXISTS setup_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    is_setup BOOLEAN NOT NULL DEFAULT 0
);

INSERT OR IGNORE INTO setup_state (id, is_setup) VALUES (1, 0);

CREATE TABLE IF NOT EXISTS group_docs (
    ring_id TEXT NOT NULL REFERENCES rings(id) ON DELETE CASCADE,
    doc_name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (ring_id, doc_name)
);
