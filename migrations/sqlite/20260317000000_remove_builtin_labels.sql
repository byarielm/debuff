-- Remove seeded builtin labels and the builtin column.
-- Users start with no default labels.

DELETE FROM label_definition_locales WHERE definition_id IN (
    SELECT id FROM label_definitions WHERE builtin = 1
);
DELETE FROM label_definitions WHERE builtin = 1;

-- SQLite doesn't support DROP COLUMN before 3.35.0, so recreate the table.
CREATE TABLE label_definitions_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    identifier TEXT NOT NULL UNIQUE,
    severity TEXT NOT NULL DEFAULT 'none',
    blurs TEXT NOT NULL DEFAULT 'none',
    default_setting TEXT NOT NULL DEFAULT 'warn',
    adult_only INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO label_definitions_new (id, identifier, severity, blurs, default_setting, adult_only, created_at)
    SELECT id, identifier, severity, blurs, default_setting, adult_only, created_at
    FROM label_definitions;

DROP TABLE label_definitions;
ALTER TABLE label_definitions_new RENAME TO label_definitions;
