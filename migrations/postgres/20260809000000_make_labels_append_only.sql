-- Labels are protocol events, not mutable rows. Preserve the current rows as
-- the initial available history, then let every apply or negation allocate a
-- new sequence number.
CREATE TABLE labels_new (
    seq BIGSERIAL PRIMARY KEY CHECK (seq > 0),
    src TEXT NOT NULL,
    uri TEXT NOT NULL,
    cid TEXT,
    val TEXT NOT NULL,
    neg INTEGER NOT NULL DEFAULT 0,
    cts TEXT NOT NULL,
    exp TEXT,
    sig BYTEA NOT NULL
);

INSERT INTO labels_new (seq, src, uri, cid, val, neg, cts, exp, sig)
SELECT seq, src, uri, cid, val, neg, cts, exp, sig
FROM labels;

DROP TABLE labels;
ALTER TABLE labels_new RENAME TO labels;
ALTER SEQUENCE labels_new_seq_seq RENAME TO labels_seq_seq;

SELECT setval(
    pg_get_serial_sequence('labels', 'seq'),
    COALESCE((SELECT MAX(seq) FROM labels), 1),
    (SELECT COUNT(*) > 0 FROM labels)
);

CREATE INDEX idx_labels_uri ON labels(uri);
CREATE INDEX idx_labels_seq ON labels(seq);
CREATE INDEX idx_labels_exp ON labels(exp) WHERE exp IS NOT NULL;
CREATE INDEX idx_labels_current ON labels(src, uri, val, seq DESC);
