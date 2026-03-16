CREATE TABLE moderators (
    did TEXT PRIMARY KEY,
    role TEXT NOT NULL DEFAULT 'moderator',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);
