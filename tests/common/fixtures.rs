use serde_json::{Value, json};

/// A minimal label definition with one English locale.
pub fn label_definition(identifier: &str) -> Value {
    json!({
        "identifier": identifier,
        "severity": "inform",
        "blurs": "none",
        "default_setting": "warn",
        "adult_only": false,
        "locales": [
            {
                "lang": "en",
                "name": identifier,
                "description": format!("Test label: {identifier}")
            }
        ]
    })
}

/// A label definition with explicit fields and no locales.
pub fn label_definition_full(
    identifier: &str,
    severity: &str,
    blurs: &str,
    default_setting: &str,
    adult_only: bool,
) -> Value {
    json!({
        "identifier": identifier,
        "severity": severity,
        "blurs": blurs,
        "default_setting": default_setting,
        "adult_only": adult_only
    })
}

/// Payload for applying labels to a subject.
pub fn apply_labels(uri: &str, vals: &[&str]) -> Value {
    json!({
        "uri": uri,
        "vals": vals
    })
}

/// Payload for negating labels on a subject.
pub fn negate_labels(uri: &str, vals: &[&str]) -> Value {
    json!({
        "uri": uri,
        "vals": vals
    })
}

/// A createReport payload with a RepoRef subject (account-level report).
pub fn create_report_repo(did: &str, reason_type: &str) -> Value {
    json!({
        "reasonType": reason_type,
        "reason": "test report",
        "subject": {
            "$type": "com.atproto.admin.defs#repoRef",
            "did": did
        }
    })
}

/// A createReport payload with a StrongRef subject (record-level report).
pub fn create_report_record(uri: &str, cid: &str, reason_type: &str) -> Value {
    json!({
        "reasonType": reason_type,
        "reason": "test report",
        "subject": {
            "$type": "com.atproto.repo.strongRef",
            "uri": uri,
            "cid": cid
        }
    })
}

/// Payload for creating a webhook source.
pub fn webhook_source(name: &str) -> Value {
    json!({
        "name": name
    })
}

/// Payload for applying an account action.
pub fn account_action(action: &str) -> Value {
    json!({
        "action": action,
        "reason": "test action"
    })
}

/// Ingest payload without suggested labels.
pub fn ingest_payload(subject_uri: &str, subject_did: &str) -> Value {
    json!({
        "subject_uri": subject_uri,
        "subject_did": subject_did
    })
}

/// Ingest payload with suggested labels.
pub fn ingest_payload_with_labels(subject_uri: &str, subject_did: &str, labels: &[&str]) -> Value {
    json!({
        "subject_uri": subject_uri,
        "subject_did": subject_did,
        "suggested_labels": labels
    })
}
