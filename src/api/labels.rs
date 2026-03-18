use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::ModeratorAuth;
use crate::db::adapt_sql;
use crate::error::AppError;
use crate::signing::UnsignedLabel;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ApplyLabelsBody {
    pub uri: String,
    pub cid: Option<String>,
    pub vals: Vec<String>,
}

#[derive(Deserialize)]
pub struct NegateLabelsBody {
    pub uri: String,
    pub vals: Vec<String>,
}

#[derive(Deserialize)]
pub struct LabelsQuery {
    pub uri: String,
}

#[derive(Serialize)]
pub struct LabelResponse {
    pub src: String,
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    pub val: String,
    pub neg: bool,
    pub cts: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<DateTime<Utc>>,
    pub sig: String,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/labels — apply label(s) to a subject.
pub async fn apply_labels(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Json(body): Json<ApplyLabelsBody>,
) -> Result<(StatusCode, Json<Vec<LabelResponse>>), AppError> {
    if body.vals.is_empty() {
        return Err(AppError::BadRequest("vals must not be empty".into()));
    }

    let backend = state.config.database.backend.clone();
    // Validate all vals exist in label_definitions
    for val in &body.vals {
        let exists: Option<(i32,)> = sqlx::query_as(&adapt_sql(
            "SELECT id FROM label_definitions WHERE identifier = $1",
            backend.clone(),
        ))
        .bind(val)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to check definition: {e}")))?;

        if exists.is_none() {
            return Err(AppError::BadRequest(format!(
                "unknown label value: '{val}'"
            )));
        }
    }

    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let mut results = Vec::with_capacity(body.vals.len());

    for val in &body.vals {
        let unsigned = UnsignedLabel {
            ver: 1,
            src: state.config.labeler.did.clone(),
            uri: body.uri.clone(),
            cid: body.cid.clone(),
            val: val.clone(),
            neg: false,
            cts: now.to_rfc3339(),
            exp: None,
        };

        let sig = state
            .signer
            .sign_label(&unsigned)
            .map_err(|e| AppError::Internal(format!("signing failed: {e}")))?;

        let row: (i64, String) = sqlx::query_as(&adapt_sql(
            "INSERT INTO labels (src, uri, cid, val, neg, cts, sig)
             VALUES ($1, $2, $3, $4, 0, $5, $6)
             ON CONFLICT (src, uri, val) DO UPDATE
             SET neg = 0, cid = EXCLUDED.cid, cts = EXCLUDED.cts, sig = EXCLUDED.sig
             RETURNING seq, cts",
            backend.clone(),
        ))
        .bind(&state.config.labeler.did)
        .bind(&body.uri)
        .bind(&body.cid)
        .bind(val)
        .bind(&now_str)
        .bind(&sig)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to insert label: {e}")))?;

        let seq = row.0;
        let _ = state.label_broadcast.send(seq);

        results.push(LabelResponse {
            src: state.config.labeler.did.clone(),
            uri: body.uri.clone(),
            cid: body.cid.clone(),
            val: val.clone(),
            neg: false,
            cts: crate::db::parse_dt(&row.1),
            exp: None,
            sig: base64_encode(&sig),
        });
    }

    Ok((StatusCode::CREATED, Json(results)))
}

/// DELETE /api/labels — negate label(s) on a subject.
pub async fn negate_labels(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Json(body): Json<NegateLabelsBody>,
) -> Result<Json<Vec<LabelResponse>>, AppError> {
    if body.vals.is_empty() {
        return Err(AppError::BadRequest("vals must not be empty".into()));
    }

    let backend = state.config.database.backend.clone();
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let mut results = Vec::with_capacity(body.vals.len());

    for val in &body.vals {
        let unsigned = UnsignedLabel {
            ver: 1,
            src: state.config.labeler.did.clone(),
            uri: body.uri.clone(),
            cid: None,
            val: val.clone(),
            neg: true,
            cts: now.to_rfc3339(),
            exp: None,
        };

        let sig = state
            .signer
            .sign_label(&unsigned)
            .map_err(|e| AppError::Internal(format!("signing failed: {e}")))?;

        let row: (i64, String) = sqlx::query_as(&adapt_sql(
            "INSERT INTO labels (src, uri, cid, val, neg, cts, sig)
             VALUES ($1, $2, NULL, $3, 1, $4, $5)
             ON CONFLICT (src, uri, val) DO UPDATE
             SET neg = 1, cid = NULL, cts = EXCLUDED.cts, sig = EXCLUDED.sig
             RETURNING seq, cts",
            backend.clone(),
        ))
        .bind(&state.config.labeler.did)
        .bind(&body.uri)
        .bind(val)
        .bind(&now_str)
        .bind(&sig)
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to insert negation label: {e}")))?;

        let seq = row.0;
        let _ = state.label_broadcast.send(seq);

        results.push(LabelResponse {
            src: state.config.labeler.did.clone(),
            uri: body.uri.clone(),
            cid: None,
            val: val.clone(),
            neg: true,
            cts: crate::db::parse_dt(&row.1),
            exp: None,
            sig: base64_encode(&sig),
        });
    }

    Ok(Json(results))
}

/// GET /api/labels?uri=... — query active labels for a subject.
pub async fn query_labels(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Query(params): Query<LabelsQuery>,
) -> Result<Json<Vec<LabelResponse>>, AppError> {
    let backend = state.config.database.backend.clone();
    #[allow(clippy::type_complexity)]
    let rows: Vec<(
        String,
        String,
        Option<String>,
        String,
        i32,
        String,
        Option<String>,
        Vec<u8>,
    )> = sqlx::query_as(&adapt_sql(
        "SELECT src, uri, cid, val, neg, cts, exp, sig
             FROM labels
             WHERE uri = $1 AND neg = 0
             ORDER BY cts",
        backend,
    ))
    .bind(&params.uri)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to query labels: {e}")))?;

    let labels = rows
        .into_iter()
        .map(
            |(src, uri, cid, val, neg_int, cts, exp, sig)| LabelResponse {
                src,
                uri,
                cid,
                val,
                neg: neg_int != 0,
                cts: crate::db::parse_dt(&cts),
                exp: exp.as_deref().map(crate::db::parse_dt),
                sig: base64_encode(&sig),
            },
        )
        .collect();

    Ok(Json(labels))
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
