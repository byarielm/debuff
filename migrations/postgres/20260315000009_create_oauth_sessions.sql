CREATE TABLE oauth_sessions (
    did TEXT PRIMARY KEY,
    session_data JSONB NOT NULL,
    created_at TEXT NOT NULL DEFAULT NOW()::TEXT,
    updated_at TEXT NOT NULL DEFAULT NOW()::TEXT
);

CREATE TABLE oauth_state (
    state_key TEXT PRIMARY KEY,
    state_data JSONB NOT NULL,
    created_at TEXT NOT NULL DEFAULT NOW()::TEXT
);
