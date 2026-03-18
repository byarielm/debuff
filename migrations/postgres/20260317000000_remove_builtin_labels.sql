-- Remove seeded builtin labels and the builtin column.
-- Users start with no default labels.

DELETE FROM label_definition_locales WHERE definition_id IN (
    SELECT id FROM label_definitions WHERE builtin = 1
);
DELETE FROM label_definitions WHERE builtin = 1;

ALTER TABLE label_definitions DROP COLUMN builtin;
