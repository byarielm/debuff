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
    auto_labeled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_reports_status ON reports(status);
CREATE INDEX idx_reports_subject_uri ON reports(subject_uri);
CREATE INDEX idx_reports_subject_did ON reports(subject_did);
