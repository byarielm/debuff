CREATE TABLE labels (
    id BIGSERIAL PRIMARY KEY,
    src TEXT NOT NULL,
    uri TEXT NOT NULL,
    cid TEXT,
    val TEXT NOT NULL,
    neg INTEGER NOT NULL DEFAULT 0,
    cts TEXT NOT NULL,
    exp TEXT,
    sig BYTEA NOT NULL,
    seq BIGSERIAL NOT NULL,
    UNIQUE(src, uri, val)
);

CREATE INDEX idx_labels_uri ON labels(uri);
CREATE INDEX idx_labels_seq ON labels(seq);
CREATE INDEX idx_labels_exp ON labels(exp) WHERE exp IS NOT NULL;
