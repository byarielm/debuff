use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::ModeratorAuth;
use crate::error::AppError;
use crate::signing::UnsignedLabel;
use crate::AppState;

const VALID_ACCOUNT_ACTIONS: &[&str] = &["!suspend", "!takedown", "!hide"];

#[derive(Deserialize)]
pub struct AccountActionBody {
    pub action: String,
    pub reason: String,
}

#[derive(Serialize, Clone)]
pub struct LabelRow {
    pub src: String,
    pub uri: String,
    pub val: String,
    pub neg: bool,
    pub cts: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<DateTime<Utc>>,
    pub sig: String,
}

#[derive(Serialize)]
pub struct AccountResponse {
    pub did: String,
    pub labels: Vec<LabelRow>,
    pub active_labels: Vec<LabelRow>,
}

/// POST /api/accounts/:did/action — Apply an account-level label.
pub async fn apply_action(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(did): Path<String>,
    Json(body): Json<AccountActionBody>,
) -> Result<(StatusCode, Json<LabelRow>), AppError> {
    if !VALID_ACCOUNT_ACTIONS.contains(&body.action.as_str()) {
        return Err(AppError::BadRequest(format!(
            "action must be one of: {}",
            VALID_ACCOUNT_ACTIONS.join(", ")
        )));
    }

    if !did.starts_with("did:") {
        return Err(AppError::BadRequest("invalid DID format".into()));
    }

    let now = Utc::now();
    let now_str = now.to_rfc3339();

    let unsigned = UnsignedLabel {
        ver: 1,
        src: state.config.labeler.did.clone(),
        uri: did.clone(),
        cid: None,
        val: body.action.clone(),
        neg: false,
        cts: now.to_rfc3339(),
        exp: None,
    };

    let sig = state
        .signer
        .sign_label(&unsigned)
        .map_err(|e| AppError::Internal(format!("signing failed: {e}")))?;

    let row: (i64, String) = sqlx::query_as(
        "INSERT INTO labels (src, uri, val, neg, cts, sig)
         VALUES (?, ?, ?, false, ?, ?)
         ON CONFLICT (src, uri, val) DO UPDATE
         SET neg = false, cts = EXCLUDED.cts, sig = EXCLUDED.sig
         RETURNING seq, cts",
    )
    .bind(&state.config.labeler.did)
    .bind(&did)
    .bind(&body.action)
    .bind(&now_str)
    .bind(&sig)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to insert label: {e}")))?;

    let _ = state.label_broadcast.send(row.0);

    Ok((
        StatusCode::CREATED,
        Json(LabelRow {
            src: state.config.labeler.did.clone(),
            uri: did,
            val: body.action,
            neg: false,
            cts: crate::db::parse_dt(&row.1),
            exp: None,
            sig: base64_encode(&sig),
        }),
    ))
}

/// GET /api/accounts/:did — Get account info and label history.
pub async fn get_account(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(did): Path<String>,
) -> Result<Json<AccountResponse>, AppError> {
    if !did.starts_with("did:") {
        return Err(AppError::BadRequest("invalid DID format".into()));
    }

    let rows: Vec<(String, String, String, i32, String, Option<String>, Vec<u8>)> =
        sqlx::query_as(
            "SELECT src, uri, val, neg, cts, exp, sig
             FROM labels
             WHERE uri = ?
             ORDER BY cts ASC",
        )
        .bind(&did)
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to query labels: {e}")))?;

    let labels: Vec<LabelRow> = rows
        .iter()
        .map(|(src, uri, val, neg, cts, exp, sig)| LabelRow {
            src: src.clone(),
            uri: uri.clone(),
            val: val.clone(),
            neg: *neg != 0,
            cts: crate::db::parse_dt(cts),
            exp: exp.as_deref().map(crate::db::parse_dt),
            sig: base64_encode(sig),
        })
        .collect();

    // Compute active labels: track which vals are currently active (not negated).
    let mut active_vals: std::collections::HashSet<String> = std::collections::HashSet::new();

    for label in &labels {
        if label.neg {
            active_vals.remove(&label.val);
        } else {
            active_vals.insert(label.val.clone());
        }
    }

    let active_labels: Vec<LabelRow> = labels
        .iter()
        .filter(|l| !l.neg && active_vals.contains(&l.val))
        .cloned()
        .collect();

    Ok(Json(AccountResponse {
        did,
        labels,
        active_labels,
    }))
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
