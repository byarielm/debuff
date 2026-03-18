CREATE TABLE labeler_config (
    did TEXT PRIMARY KEY,
    signing_key BYTEA NOT NULL,
    created_at TEXT NOT NULL DEFAULT NOW()::TEXT
);
