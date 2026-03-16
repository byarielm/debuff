CREATE TABLE label_definition_locales (
    id SERIAL PRIMARY KEY,
    definition_id INT NOT NULL REFERENCES label_definitions(id) ON DELETE CASCADE,
    lang TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    UNIQUE(definition_id, lang)
);
