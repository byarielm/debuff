use axum::Json;
use axum::extract::State;
use axum_extra::extract::Query;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Request / response types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryLabelsParams {
    /// URI patterns to match (supports `*` wildcard suffix).
    pub uri_patterns: Vec<String>,
    /// Optional labeler DIDs to filter by.
    #[serde(default)]
    pub sources: Option<Vec<String>>,
    /// Max results (default 50, max 250).
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Pagination cursor (seq number).
    pub cursor: Option<String>,
}

fn default_limit() -> u32 {
    50
}

#[derive(Serialize)]
pub struct QueryLabelsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    pub labels: Vec<LabelEntry>,
}

#[derive(Serialize)]
pub struct LabelEntry {
    pub ver: i32,
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
// Handler
// ---------------------------------------------------------------------------

/// GET /xrpc/com.atproto.label.queryLabels
pub async fn query_labels(
    State(state): State<AppState>,
    Query(params): Query<QueryLabelsParams>,
) -> Result<Json<QueryLabelsResponse>, AppError> {
    // Validate uriPatterns
    if params.uri_patterns.is_empty() {
        return Err(AppError::BadRequest(
            "uriPatterns must contain at least one pattern".into(),
        ));
    }

    // Clamp limit
    let limit = params.limit.min(250).max(1) as i64;

    // Parse cursor
    let cursor_seq: Option<i64> = match &params.cursor {
        Some(c) => Some(
            c.parse::<i64>()
                .map_err(|_| AppError::BadRequest("invalid cursor".into()))?,
        ),
        None => None,
    };

    // Build dynamic query with typed bindings
    enum BindVal {
        Text(String),
        Int(i64),
    }

    let mut binds: Vec<BindVal> = Vec::new();

    let now_str = crate::db::now_rfc3339();

    // Rebuild SQL
    let mut sql = String::from(
        "SELECT seq, src, uri, cid, val, neg, cts, exp, sig FROM labels WHERE neg = 0 AND (exp IS NULL OR exp > ?) ",
    );
    binds.push(BindVal::Text(now_str));

    // URI patterns
    sql.push_str("AND (");
    for (i, pattern) in params.uri_patterns.iter().enumerate() {
        if i > 0 {
            sql.push_str(" OR ");
        }
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            sql.push_str("uri LIKE ?");
            binds.push(BindVal::Text(format!("{prefix}%")));
        } else {
            sql.push_str("uri = ?");
            binds.push(BindVal::Text(pattern.clone()));
        }
    }
    sql.push_str(") ");

    // Sources
    if let Some(ref sources) = params.sources {
        if !sources.is_empty() {
            sql.push_str("AND src IN (");
            for (i, src) in sources.iter().enumerate() {
                if i > 0 {
                    sql.push(',');
                }
                sql.push('?');
                binds.push(BindVal::Text(src.clone()));
            }
            sql.push_str(") ");
        }
    }

    // Cursor
    if let Some(cursor_val) = cursor_seq {
        sql.push_str("AND seq > ? ");
        binds.push(BindVal::Int(cursor_val));
    }

    // Limit
    sql.push_str("LIMIT ?");
    binds.push(BindVal::Int(limit));

    // Build the sqlx query with dynamic binds
    let mut query = sqlx::query_as::<
        _,
        (
            i64,
            String,
            String,
            Option<String>,
            String,
            i32,
            String,
            Option<String>,
            Vec<u8>,
        ),
    >(&sql);

    for bind in &binds {
        match bind {
            BindVal::Text(v) => {
                query = query.bind(v);
            }
            BindVal::Int(v) => {
                query = query.bind(v);
            }
        }
    }

    let rows = query
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to query labels: {e}")))?;

    // Build response
    let cursor = rows.last().map(|r| r.0.to_string());

    let labels = rows
        .into_iter()
        .map(
            |(_seq, src, uri, cid, val, neg, cts, exp, sig)| LabelEntry {
                ver: 1,
                src,
                uri,
                cid,
                val,
                neg: neg != 0,
                cts: crate::db::parse_dt(&cts),
                exp: exp.as_deref().map(crate::db::parse_dt),
                sig: base64_encode(&sig),
            },
        )
        .collect();

    Ok(Json(QueryLabelsResponse { cursor, labels }))
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
