use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::auth::ModeratorAuth;
use crate::error::AppError;
use crate::AppState;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct QueueListParams {
    pub status: Option<String>,
    pub reason_type: Option<String>,
    pub priority: Option<i32>,
    pub assigned_to: Option<String>,
    pub cursor: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueListResponse {
    pub items: Vec<QueueItemSummary>,
    pub cursor: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemSummary {
    pub id: i64,
    pub subject_uri: Option<String>,
    pub subject_cid: Option<String>,
    pub subject_did: Option<String>,
    pub reason_type: String,
    pub reason: String,
    pub reported_by: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub priority: i32,
    pub auto_labeled: bool,
    pub label_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueItemDetail {
    pub id: i64,
    pub subject_uri: Option<String>,
    pub subject_cid: Option<String>,
    pub subject_did: Option<String>,
    pub reason_type: String,
    pub reason: String,
    pub reported_by: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub priority: i32,
    pub auto_labeled: bool,
    pub notes: Vec<NoteResponse>,
    pub labels: Vec<SubjectLabel>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteResponse {
    pub id: i64,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectLabel {
    pub id: i64,
    pub val: String,
    pub neg: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct UpdateStatusBody {
    pub status: String,
}

#[derive(Deserialize)]
pub struct AssignBody {
    pub did: Option<String>,
}

#[derive(Deserialize)]
pub struct EscalateBody {
    pub assigned_to: Option<String>,
}

#[derive(Deserialize)]
pub struct AddNoteBody {
    pub content: String,
}

// ---------------------------------------------------------------------------
// Valid statuses
// ---------------------------------------------------------------------------

const VALID_STATUSES: &[&str] = &["pending", "in_review", "resolved", "dismissed"];

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/queue — list reports with optional filters and cursor-based pagination.
pub async fn list_queue(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Query(params): Query<QueueListParams>,
) -> Result<Json<QueueListResponse>, AppError> {
    let limit = params.limit.unwrap_or(50).min(200).max(1);

    // Build dynamic query
    let mut conditions: Vec<String> = Vec::new();

    if params.cursor.is_some() {
        conditions.push("r.id < ?".to_string());
    }
    if params.status.is_some() {
        conditions.push("r.status = ?".to_string());
    }
    if params.reason_type.is_some() {
        conditions.push("r.reason_type = ?".to_string());
    }
    if params.priority.is_some() {
        conditions.push("r.priority = ?".to_string());
    }
    if let Some(ref assigned_to) = params.assigned_to {
        if assigned_to == "__unassigned__" {
            conditions.push("r.assigned_to IS NULL".to_string());
        } else {
            conditions.push("r.assigned_to = ?".to_string());
        }
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // We need limit + 1 to know if there are more items
    let fetch_limit = limit + 1;

    let sql = format!(
        "SELECT r.id, r.subject_uri, r.subject_cid, r.subject_did,
                r.reason_type, r.reason, r.reported_by, r.status,
                r.assigned_to, r.priority, r.auto_labeled,
                r.created_at, r.updated_at,
                COALESCE((SELECT COUNT(*) FROM labels l WHERE l.uri = COALESCE(r.subject_uri, r.subject_did)), 0) AS label_count
         FROM reports r
         {where_clause}
         ORDER BY r.priority DESC, r.id DESC
         LIMIT ?"
    );

    // Build the query and bind values in order
    let mut query = sqlx::query_as::<_, (
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        String,
        String,
        Option<String>,
        i32,
        bool,
        String,
        String,
        i64,
    )>(&sql);

    if let Some(ref cursor) = params.cursor {
        query = query.bind(*cursor);
    }
    if let Some(ref status) = params.status {
        query = query.bind(status);
    }
    if let Some(ref reason_type) = params.reason_type {
        query = query.bind(reason_type);
    }
    if let Some(ref priority) = params.priority {
        query = query.bind(*priority);
    }
    if let Some(ref assigned_to) = params.assigned_to {
        if assigned_to != "__unassigned__" {
            query = query.bind(assigned_to);
        }
    }
    query = query.bind(fetch_limit);

    let rows = query
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to list queue: {e}")))?;

    let has_more = rows.len() as i64 > limit;
    let items: Vec<QueueItemSummary> = rows
        .into_iter()
        .take(limit as usize)
        .map(|row| QueueItemSummary {
            id: row.0,
            subject_uri: row.1,
            subject_cid: row.2,
            subject_did: row.3,
            reason_type: row.4,
            reason: row.5,
            reported_by: row.6,
            status: row.7,
            assigned_to: row.8,
            priority: row.9,
            auto_labeled: row.10,
            created_at: crate::db::parse_dt(&row.11),
            updated_at: crate::db::parse_dt(&row.12),
            label_count: row.13,
        })
        .collect();

    let cursor = if has_more {
        items.last().map(|item| item.id.to_string())
    } else {
        None
    };

    Ok(Json(QueueListResponse { items, cursor }))
}

/// GET /api/queue/:id — single report detail with notes and labels.
pub async fn get_queue_item(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(id): Path<i64>,
) -> Result<Json<QueueItemDetail>, AppError> {
    let row: Option<(
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        String,
        String,
        Option<String>,
        i32,
        bool,
        String,
        String,
    )> = sqlx::query_as(
        "SELECT id, subject_uri, subject_cid, subject_did,
                reason_type, reason, reported_by, status,
                assigned_to, priority, auto_labeled,
                created_at, updated_at
         FROM reports WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to fetch report: {e}")))?;

    let report = row.ok_or(AppError::NotFound)?;

    // Fetch notes
    let note_rows: Vec<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, author, content, created_at
         FROM report_notes WHERE report_id = ?
         ORDER BY created_at",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to fetch notes: {e}")))?;

    let notes: Vec<NoteResponse> = note_rows
        .into_iter()
        .map(|(id, author, content, created_at)| NoteResponse {
            id,
            author,
            content,
            created_at: crate::db::parse_dt(&created_at),
        })
        .collect();

    // Fetch labels on the subject
    let subject_key = report
        .1
        .as_deref()
        .or(report.3.as_deref())
        .unwrap_or("");

    let label_rows: Vec<(i64, String, bool, String)> = sqlx::query_as(
        "SELECT id, val, neg, cts FROM labels WHERE uri = ? ORDER BY cts",
    )
    .bind(subject_key)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to fetch labels: {e}")))?;

    let labels: Vec<SubjectLabel> = label_rows
        .into_iter()
        .map(|(id, val, neg, created_at)| SubjectLabel {
            id,
            val,
            neg,
            created_at: crate::db::parse_dt(&created_at),
        })
        .collect();

    Ok(Json(QueueItemDetail {
        id: report.0,
        subject_uri: report.1,
        subject_cid: report.2,
        subject_did: report.3,
        reason_type: report.4,
        reason: report.5,
        reported_by: report.6,
        status: report.7,
        assigned_to: report.8,
        priority: report.9,
        auto_labeled: report.10,
        notes,
        labels,
        created_at: crate::db::parse_dt(&report.11),
        updated_at: crate::db::parse_dt(&report.12),
    }))
}

/// PATCH /api/queue/:id — update report status.
pub async fn update_status(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(id): Path<i64>,
    Json(body): Json<UpdateStatusBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !VALID_STATUSES.contains(&body.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "status must be one of: {}",
            VALID_STATUSES.join(", ")
        )));
    }

    let now_str = crate::db::now_rfc3339();
    let result = sqlx::query(
        "UPDATE reports SET status = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&body.status)
    .bind(&now_str)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to update status: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({
        "id": id,
        "status": body.status,
    })))
}

/// PATCH /api/queue/:id/assign — assign or unassign a moderator.
pub async fn assign_moderator(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(id): Path<i64>,
    Json(body): Json<AssignBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Validate that the DID belongs to a known moderator
    if let Some(ref did) = body.did {
        let exists: Option<(String,)> =
            sqlx::query_as("SELECT did FROM moderators WHERE did = ?")
                .bind(did)
                .fetch_optional(&state.db)
                .await
                .map_err(|e| AppError::Internal(format!("failed to check moderator: {e}")))?;

        if exists.is_none() {
            return Err(AppError::BadRequest(format!(
                "DID '{}' is not a known moderator",
                did
            )));
        }
    }

    let now_str = crate::db::now_rfc3339();
    let result = sqlx::query(
        "UPDATE reports SET assigned_to = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&body.did)
    .bind(&now_str)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to assign moderator: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(serde_json::json!({
        "id": id,
        "assigned_to": body.did,
    })))
}

/// POST /api/queue/:id/escalate — bump priority by 1, optionally reassign.
pub async fn escalate(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(id): Path<i64>,
    Json(body): Json<EscalateBody>,
) -> Result<Json<serde_json::Value>, AppError> {
    let now_str = crate::db::now_rfc3339();
    let (sql, has_assign) = if body.assigned_to.is_some() {
        (
            "UPDATE reports SET priority = priority + 1, assigned_to = ?, updated_at = ? WHERE id = ? RETURNING priority, assigned_to",
            true,
        )
    } else {
        (
            "UPDATE reports SET priority = priority + 1, updated_at = ? WHERE id = ? RETURNING priority, assigned_to",
            false,
        )
    };

    let row: Option<(i32, Option<String>)> = if has_assign {
        sqlx::query_as(sql)
            .bind(&body.assigned_to)
            .bind(&now_str)
            .bind(id)
            .fetch_optional(&state.db)
            .await
    } else {
        sqlx::query_as(sql)
            .bind(&now_str)
            .bind(id)
            .fetch_optional(&state.db)
            .await
    }
    .map_err(|e| AppError::Internal(format!("failed to escalate: {e}")))?;

    let (priority, assigned_to) = row.ok_or(AppError::NotFound)?;

    Ok(Json(serde_json::json!({
        "id": id,
        "priority": priority,
        "assigned_to": assigned_to,
    })))
}

/// POST /api/queue/:id/notes — add a note to a report.
pub async fn add_note(
    State(state): State<AppState>,
    auth: ModeratorAuth,
    Path(report_id): Path<i64>,
    Json(body): Json<AddNoteBody>,
) -> Result<(StatusCode, Json<NoteResponse>), AppError> {
    if body.content.is_empty() {
        return Err(AppError::BadRequest("content must not be empty".into()));
    }

    // Verify the report exists
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT id FROM reports WHERE id = ?")
            .bind(report_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("failed to check report: {e}")))?;

    if exists.is_none() {
        return Err(AppError::NotFound);
    }

    let row: (i64, String) = sqlx::query_as(
        "INSERT INTO report_notes (report_id, author, content)
         VALUES (?, ?, ?)
         RETURNING id, created_at",
    )
    .bind(report_id)
    .bind(&auth.did)
    .bind(&body.content)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to add note: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(NoteResponse {
            id: row.0,
            author: auth.did,
            content: body.content,
            created_at: crate::db::parse_dt(&row.1),
        }),
    ))
}
