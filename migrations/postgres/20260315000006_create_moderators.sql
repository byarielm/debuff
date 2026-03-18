CREATE TABLE moderators (
    did TEXT PRIMARY KEY,
    role TEXT NOT NULL DEFAULT 'moderator',
    created_at TEXT NOT NULL DEFAULT NOW()::TEXT,
    last_used_at TEXT
);
