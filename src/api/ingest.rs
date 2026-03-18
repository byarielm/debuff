use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::AppState;
use crate::db::adapt_sql;
use crate::error::AppError;
use crate::signing::UnsignedLabel;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
pub struct IngestBody {
    pub subject_uri: String,
    pub subject_did: String,
    #[serde(default = "default_reason_type")]
    pub reason_type: String,
    #[serde(default)]
    pub reason: String,
    #[serde(default)]
    pub suggested_labels: Vec<String>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn default_reason_type() -> String {
    "automated".into()
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum IngestResponse {
    Report { report_id: i64 },
    Labels { labels_applied: i64 },
}

/// Information about the matched webhook source needed for processing.
struct MatchedSource {
    id: i32,
    name: String,
    auto_accept: bool,
    auto_label: bool,
    requires_review: bool,
}

/// Verify the HMAC-SHA256 signature against all active webhook sources.
/// Returns the matched source if found.
async fn verify_signature(
    state: &AppState,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<MatchedSource, AppError> {
    let sig_header = headers
        .get("x-webhook-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    let hex_digest = sig_header
        .strip_prefix("sha256=")
        .ok_or(AppError::Unauthorized)?;

    let provided_sig = hex::decode(hex_digest).map_err(|_| AppError::Unauthorized)?;

    let rows: Vec<(i32, String, String, i32, i32, i32)> = sqlx::query_as(
        "SELECT id, name, secret, auto_accept, auto_label, requires_review \
         FROM webhook_sources WHERE active = 1",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to fetch webhook sources: {e}")))?;

    for (id, name, secret, auto_accept_int, auto_label_int, requires_review_int) in rows {
        let auto_accept = auto_accept_int != 0;
        let auto_label = auto_label_int != 0;
        let requires_review = requires_review_int != 0;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|e| AppError::Internal(format!("hmac init failed: {e}")))?;
        mac.update(body);
        let result = mac.finalize().into_bytes();

        if *result == *provided_sig {
            return Ok(MatchedSource {
                id,
                name,
                auto_accept,
                auto_label,
                requires_review,
            });
        }
    }

    Err(AppError::Unauthorized)
}

/// POST /api/ingest — receive flagged items from external systems.
pub async fn ingest(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<IngestResponse>), AppError> {
    let source = verify_signature(&state, &headers, &body).await?;

    let payload: IngestBody = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("invalid JSON body: {e}")))?;

    tracing::info!(
        source_id = source.id,
        source_name = %source.name,
        subject_uri = %payload.subject_uri,
        "ingest received"
    );

    // No-review mode: just apply labels and return
    if !source.requires_review && source.auto_accept {
        let labels_applied = if source.auto_label && !payload.suggested_labels.is_empty() {
            apply_labels(&state, &payload).await?
        } else {
            0
        };
        return Ok((
            StatusCode::OK,
            Json(IngestResponse::Labels { labels_applied }),
        ));
    }

    let backend = state.config.database.backend.clone();
    // Check for duplicate: open report for same subject_uri
    let existing: Option<(i64,)> = sqlx::query_as(&adapt_sql(
        "SELECT id FROM reports \
         WHERE subject_uri = $1 AND status IN ('pending', 'in_progress') \
         LIMIT 1",
        backend.clone(),
    ))
    .bind(&payload.subject_uri)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to check for duplicate: {e}")))?;

    let report_id = if let Some((existing_id,)) = existing {
        // Append as note to existing report
        sqlx::query(&adapt_sql(
            "INSERT INTO report_notes (report_id, author, content) \
             VALUES ($1, $2, $3)",
            backend.clone(),
        ))
        .bind(existing_id)
        .bind(format!("webhook:{}", source.name))
        .bind(format!(
            "Duplicate ingest from webhook '{}': {}",
            source.name, payload.reason
        ))
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to add note: {e}")))?;

        existing_id
    } else {
        // Create new report
        let status = if source.auto_accept {
            "resolved"
        } else {
            "pending"
        };

        let row: (i64,) = sqlx::query_as(&adapt_sql(
            "INSERT INTO reports (subject_uri, subject_did, reason_type, reason, reported_by, status, priority) \
             VALUES ($1, $2, $3, $4, $5, $6, $7) \
             RETURNING id",
            backend.clone(),
        ))
        .bind(&payload.subject_uri)
        .bind(&payload.subject_did)
        .bind(&payload.reason_type)
        .bind(&payload.reason)
        .bind(format!("webhook:{}", source.name))
        .bind(status)
        .bind(payload.priority)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to create report: {e}")))?;

        row.0
    };

    // Auto-label if configured
    if source.auto_label && !payload.suggested_labels.is_empty() {
        let labels_applied = apply_labels(&state, &payload).await?;

        if labels_applied > 0 {
            sqlx::query(&adapt_sql(
                "UPDATE reports SET auto_labeled = 1 WHERE id = $1",
                backend,
            ))
            .bind(report_id)
            .execute(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("failed to update auto_labeled: {e}")))?;
        }
    }

    Ok((StatusCode::OK, Json(IngestResponse::Report { report_id })))
}

/// Sign and insert labels, broadcasting each to the WebSocket channel.
/// Returns the number of labels applied.
async fn apply_labels(state: &AppState, payload: &IngestBody) -> Result<i64, AppError> {
    let backend = state.config.database.backend.clone();
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let mut count: i64 = 0;

    for val in &payload.suggested_labels {
        let unsigned = UnsignedLabel {
            ver: 1,
            src: state.config.labeler.did.clone(),
            uri: payload.subject_uri.clone(),
            cid: None,
            val: val.clone(),
            neg: false,
            cts: now.to_rfc3339(),
            exp: None,
        };

        let signer = state.signer.read().await;
        let sig = signer
            .sign_label(&unsigned)
            .map_err(|e| AppError::Internal(format!("failed to sign label: {e}")))?;
        drop(signer);

        let row: Option<(i64, i64)> = sqlx::query_as(&adapt_sql(
            "INSERT INTO labels (src, uri, cid, val, neg, cts, exp, sig) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (src, uri, val) DO NOTHING \
             RETURNING id, seq",
            backend.clone(),
        ))
        .bind(&unsigned.src)
        .bind(&unsigned.uri)
        .bind(&unsigned.cid)
        .bind(&unsigned.val)
        .bind(unsigned.neg as i32)
        .bind(&now_str)
        .bind::<Option<String>>(None)
        .bind(&sig)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to insert label: {e}")))?;

        if let Some((_id, seq)) = row {
            // Broadcast to WebSocket subscribers
            let _ = state.label_broadcast.send(seq);
            count += 1;
        }
    }

    Ok(count)
}
