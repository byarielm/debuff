CREATE TABLE labeler_config (
    did TEXT PRIMARY KEY,
    signing_key BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
