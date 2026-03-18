CREATE TABLE webhook_sources (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    secret TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 1,
    auto_accept INTEGER NOT NULL DEFAULT 0,
    auto_label INTEGER NOT NULL DEFAULT 0,
    requires_review INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT NOW()::TEXT
);
