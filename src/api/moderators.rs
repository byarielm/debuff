use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::AppState;
use crate::auth::ModeratorAuth;
use crate::db::adapt_sql;
use crate::error::AppError;

use super::types::{AddModeratorBody, ModeratorSummary};

/// POST /api/moderators — add a new moderator by DID.
pub(super) async fn add_moderator(
    State(state): State<AppState>,
    auth: ModeratorAuth,
    Json(body): Json<AddModeratorBody>,
) -> Result<(StatusCode, Json<ModeratorSummary>), AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let backend = state.config.database.backend.clone();
    let row: (String, String, String, Option<String>) = sqlx::query_as(&adapt_sql(
        "INSERT INTO moderators (did) VALUES ($1)
             RETURNING did, role, created_at, last_used_at",
        backend,
    ))
    .bind(&body.did)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
            AppError::Conflict(format!("moderator '{}' already exists", body.did))
        }
        _ => AppError::Internal(format!("failed to create moderator: {e}")),
    })?;

    Ok((
        StatusCode::CREATED,
        Json(ModeratorSummary {
            did: row.0,
            role: row.1,
            created_at: crate::db::parse_dt(&row.2),
            last_used_at: row.3.as_deref().map(crate::db::parse_dt),
        }),
    ))
}

/// GET /api/moderators — list all moderators.
pub(super) async fn list_moderators(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
) -> Result<Json<Vec<ModeratorSummary>>, AppError> {
    let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT did, role, created_at, last_used_at
         FROM moderators ORDER BY created_at",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to list moderators: {e}")))?;

    let moderators = rows
        .into_iter()
        .map(|(did, role, created_at, last_used_at)| ModeratorSummary {
            did,
            role,
            created_at: crate::db::parse_dt(&created_at),
            last_used_at: last_used_at.as_deref().map(crate::db::parse_dt),
        })
        .collect();

    Ok(Json(moderators))
}

/// DELETE /api/moderators/:did — remove a moderator.
pub(super) async fn remove_moderator(
    State(state): State<AppState>,
    auth: ModeratorAuth,
    Path(did): Path<String>,
) -> Result<StatusCode, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }

    // Cannot remove yourself.
    if auth.did == did {
        return Err(AppError::BadRequest(
            "cannot remove yourself as a moderator".into(),
        ));
    }

    // Clear assignments before deleting the moderator
    let backend = state.config.database.backend.clone();
    sqlx::query(&adapt_sql(
        "UPDATE reports SET assigned_to = NULL WHERE assigned_to = $1",
        backend.clone(),
    ))
    .bind(&did)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to clear assignments: {e}")))?;

    let result = sqlx::query(&adapt_sql("DELETE FROM moderators WHERE did = $1", backend))
        .bind(&did)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to remove moderator: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
