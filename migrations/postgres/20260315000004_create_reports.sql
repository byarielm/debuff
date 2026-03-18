CREATE TABLE reports (
    id BIGSERIAL PRIMARY KEY,
    subject_uri TEXT,
    subject_cid TEXT,
    subject_did TEXT,
    reason_type TEXT NOT NULL,
    reason TEXT NOT NULL DEFAULT '',
    reported_by TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    assigned_to TEXT,
    priority INT NOT NULL DEFAULT 0,
    auto_labeled INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT NOW()::TEXT,
    updated_at TEXT NOT NULL DEFAULT NOW()::TEXT
);

CREATE INDEX idx_reports_status ON reports(status);
CREATE INDEX idx_reports_subject_uri ON reports(subject_uri);
CREATE INDEX idx_reports_subject_did ON reports(subject_did);
