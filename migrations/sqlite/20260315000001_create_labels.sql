CREATE TABLE labels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    src TEXT NOT NULL,
    uri TEXT NOT NULL,
    cid TEXT,
    val TEXT NOT NULL,
    neg INTEGER NOT NULL DEFAULT 0,
    cts TEXT NOT NULL,
    exp TEXT,
    sig BLOB NOT NULL,
    seq INTEGER NOT NULL DEFAULT 0,
    UNIQUE(src, uri, val)
);

-- Auto-set seq to match id after insert (mirrors Postgres BIGSERIAL behavior)
CREATE TRIGGER labels_set_seq AFTER INSERT ON labels
BEGIN
    UPDATE labels SET seq = NEW.id WHERE id = NEW.id AND seq = 0;
END;

CREATE INDEX idx_labels_uri ON labels(uri);
CREATE INDEX idx_labels_seq ON labels(seq);
CREATE INDEX idx_labels_exp ON labels(exp) WHERE exp IS NOT NULL;
