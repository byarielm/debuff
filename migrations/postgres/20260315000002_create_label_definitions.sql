CREATE TABLE label_definitions (
    id SERIAL PRIMARY KEY,
    identifier TEXT NOT NULL UNIQUE,
    severity TEXT NOT NULL DEFAULT 'none',
    blurs TEXT NOT NULL DEFAULT 'none',
    default_setting TEXT NOT NULL DEFAULT 'warn',
    adult_only BOOLEAN NOT NULL DEFAULT false,
    builtin BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
