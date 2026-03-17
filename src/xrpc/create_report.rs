use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::ServiceAuth;
use crate::error::AppError;

// ---------------------------------------------------------------------------
// ATProto reasonType constants
// ---------------------------------------------------------------------------

const VALID_REASON_TYPES: &[&str] = &[
    "com.atproto.moderation.defs#reasonSpam",
    "com.atproto.moderation.defs#reasonViolation",
    "com.atproto.moderation.defs#reasonMisleading",
    "com.atproto.moderation.defs#reasonSexual",
    "com.atproto.moderation.defs#reasonRude",
    "com.atproto.moderation.defs#reasonOther",
    "com.atproto.moderation.defs#reasonAppeal",
];

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(tag = "$type")]
pub enum ReportSubject {
    #[serde(rename = "com.atproto.admin.defs#repoRef")]
    RepoRef { did: String },
    #[serde(rename = "com.atproto.repo.strongRef")]
    StrongRef { uri: String, cid: String },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReportBody {
    pub reason_type: String,
    #[serde(default)]
    pub reason: String,
    pub subject: ReportSubject,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReportResponse {
    pub id: i64,
    pub reason_type: String,
    pub reason: String,
    pub subject: serde_json::Value,
    pub reported_by: String,
    pub created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// POST /xrpc/com.atproto.moderation.createReport
pub async fn create_report(
    State(state): State<AppState>,
    auth: ServiceAuth,
    Json(body): Json<CreateReportBody>,
) -> Result<(StatusCode, Json<CreateReportResponse>), AppError> {
    // Validate reason type
    if !VALID_REASON_TYPES.contains(&body.reason_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "invalid reasonType: {}",
            body.reason_type
        )));
    }

    let reported_by = auth.did.clone();

    // Extract subject fields
    let (subject_uri, subject_cid, subject_did) = match &body.subject {
        ReportSubject::RepoRef { did } => (None, None, Some(did.clone())),
        ReportSubject::StrongRef { uri, cid } => (Some(uri.clone()), Some(cid.clone()), None),
    };

    // Build the canonical subject URI for duplicate detection.
    let canonical_uri = subject_uri
        .as_deref()
        .or(subject_did.as_deref())
        .unwrap_or("");

    // Duplicate detection: check for an open report on the same subject
    let existing: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM reports
         WHERE (subject_uri = ? OR subject_did = ?)
           AND status IN ('pending', 'in_review')
         LIMIT 1",
    )
    .bind(canonical_uri)
    .bind(canonical_uri)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("duplicate check failed: {e}")))?;

    if let Some((existing_id,)) = existing {
        // Append the reason as a note on the existing report instead
        let note_content = if body.reason.is_empty() {
            format!(
                "Duplicate report from {} — reason type: {}",
                reported_by, body.reason_type
            )
        } else {
            format!(
                "Duplicate report from {} — reason type: {}: {}",
                reported_by, body.reason_type, body.reason
            )
        };

        sqlx::query(
            "INSERT INTO report_notes (report_id, author, content)
             VALUES (?, ?, ?)",
        )
        .bind(existing_id)
        .bind(&reported_by)
        .bind(&note_content)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to add duplicate note: {e}")))?;

        // Fetch the existing report to return
        let row: (i64, String, String, Option<String>, Option<String>, Option<String>, String, String) =
            sqlx::query_as(
                "SELECT id, reason_type, reason, subject_uri, subject_cid, subject_did, reported_by, created_at
                 FROM reports WHERE id = ?",
            )
            .bind(existing_id)
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("failed to fetch existing report: {e}")))?;

        let subject = build_subject_json(row.3.as_deref(), row.4.as_deref(), row.5.as_deref());

        return Ok((
            StatusCode::OK,
            Json(CreateReportResponse {
                id: row.0,
                reason_type: row.1,
                reason: row.2,
                subject,
                reported_by: row.6,
                created_at: crate::db::parse_dt(&row.7),
            }),
        ));
    }

    // Create a new report
    let row: (i64, String) = sqlx::query_as(
        "INSERT INTO reports (subject_uri, subject_cid, subject_did, reason_type, reason, reported_by, status)
         VALUES (?, ?, ?, ?, ?, ?, 'pending')
         RETURNING id, created_at",
    )
    .bind(&subject_uri)
    .bind(&subject_cid)
    .bind(&subject_did)
    .bind(&body.reason_type)
    .bind(&body.reason)
    .bind(&reported_by)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to create report: {e}")))?;

    let subject = build_subject_json(
        subject_uri.as_deref(),
        subject_cid.as_deref(),
        subject_did.as_deref(),
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateReportResponse {
            id: row.0,
            reason_type: body.reason_type,
            reason: body.reason,
            subject,
            reported_by,
            created_at: crate::db::parse_dt(&row.1),
        }),
    ))
}

/// Build the ATProto-style subject JSON from the stored fields.
fn build_subject_json(
    uri: Option<&str>,
    cid: Option<&str>,
    did: Option<&str>,
) -> serde_json::Value {
    if let (Some(uri), Some(cid)) = (uri, cid) {
        serde_json::json!({
            "$type": "com.atproto.repo.strongRef",
            "uri": uri,
            "cid": cid,
        })
    } else if let Some(did) = did {
        serde_json::json!({
            "$type": "com.atproto.admin.defs#repoRef",
            "did": did,
        })
    } else {
        serde_json::json!({})
    }
}
