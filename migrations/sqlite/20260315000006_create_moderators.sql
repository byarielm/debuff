CREATE TABLE moderators (
    did TEXT PRIMARY KEY,
    role TEXT NOT NULL DEFAULT 'moderator',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used_at TEXT
);
