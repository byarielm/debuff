CREATE TABLE label_definitions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    identifier TEXT NOT NULL UNIQUE,
    severity TEXT NOT NULL DEFAULT 'none',
    blurs TEXT NOT NULL DEFAULT 'none',
    default_setting TEXT NOT NULL DEFAULT 'warn',
    adult_only INTEGER NOT NULL DEFAULT 0,
    builtin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
