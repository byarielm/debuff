use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::ModeratorAuth;
use crate::error::AppError;

#[derive(Serialize)]
pub struct WebhookSourceResponse {
    pub id: i32,
    pub name: String,
    pub active: bool,
    pub auto_accept: bool,
    pub auto_label: bool,
    pub requires_review: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct WebhookSourceWithSecretResponse {
    pub id: i32,
    pub name: String,
    pub secret: String,
    pub active: bool,
    pub auto_accept: bool,
    pub auto_label: bool,
    pub requires_review: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateWebhookBody {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateWebhookBody {
    pub active: Option<bool>,
    pub auto_accept: Option<bool>,
    pub auto_label: Option<bool>,
    pub requires_review: Option<bool>,
}

/// Generate a 64-character hex string (32 random bytes).
fn generate_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// GET /api/webhooks — list all webhook sources (without secrets).
pub async fn list_webhooks(
    State(state): State<AppState>,
    auth: ModeratorAuth,
) -> Result<Json<Vec<WebhookSourceResponse>>, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let rows: Vec<(i32, String, i32, i32, i32, i32, String)> = sqlx::query_as(
        "SELECT id, name, active, auto_accept, auto_label, requires_review, created_at \
         FROM webhook_sources ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to list webhook sources: {e}")))?;

    let sources = rows
        .into_iter()
        .map(
            |(id, name, active, auto_accept, auto_label, requires_review, created_at)| {
                WebhookSourceResponse {
                    id,
                    name,
                    active: active != 0,
                    auto_accept: auto_accept != 0,
                    auto_label: auto_label != 0,
                    requires_review: requires_review != 0,
                    created_at: crate::db::parse_dt(&created_at),
                }
            },
        )
        .collect();

    Ok(Json(sources))
}

/// POST /api/webhooks — create a webhook source. Returns the secret (only time it's shown).
pub async fn create_webhook(
    State(state): State<AppState>,
    auth: ModeratorAuth,
    Json(body): Json<CreateWebhookBody>,
) -> Result<(StatusCode, Json<WebhookSourceWithSecretResponse>), AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }

    if body.name.is_empty() {
        return Err(AppError::BadRequest("name is required".into()));
    }

    let secret = generate_secret();

    let row: (i32, String, String, i32, i32, i32, i32, String) = sqlx::query_as(
        "INSERT INTO webhook_sources (name, secret) VALUES (?, ?) \
         RETURNING id, name, secret, active, auto_accept, auto_label, requires_review, created_at",
    )
    .bind(&body.name)
    .bind(&secret)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to create webhook source: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(WebhookSourceWithSecretResponse {
            id: row.0,
            name: row.1,
            secret: row.2,
            active: row.3 != 0,
            auto_accept: row.4 != 0,
            auto_label: row.5 != 0,
            requires_review: row.6 != 0,
            created_at: crate::db::parse_dt(&row.7),
        }),
    ))
}

/// PATCH /api/webhooks/:id — update webhook source settings.
pub async fn update_webhook(
    State(state): State<AppState>,
    auth: ModeratorAuth,
    Path(id): Path<i32>,
    Json(body): Json<UpdateWebhookBody>,
) -> Result<Json<WebhookSourceResponse>, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let row: Option<(i32, String, i32, i32, i32, i32, String)> = sqlx::query_as(
        "SELECT id, name, active, auto_accept, auto_label, requires_review, created_at \
         FROM webhook_sources WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to fetch webhook source: {e}")))?;

    let (ws_id, name, active_int, auto_accept_int, auto_label_int, requires_review_int, created_at) =
        row.ok_or(AppError::NotFound)?;

    let mut active = active_int != 0;
    let mut auto_accept = auto_accept_int != 0;
    let mut auto_label = auto_label_int != 0;
    let mut requires_review = requires_review_int != 0;

    if let Some(v) = body.active {
        active = v;
    }
    if let Some(v) = body.auto_accept {
        auto_accept = v;
    }
    if let Some(v) = body.auto_label {
        auto_label = v;
    }
    if let Some(v) = body.requires_review {
        requires_review = v;
    }

    sqlx::query(
        "UPDATE webhook_sources \
         SET active = ?, auto_accept = ?, auto_label = ?, requires_review = ? \
         WHERE id = ?",
    )
    .bind(active)
    .bind(auto_accept)
    .bind(auto_label)
    .bind(requires_review)
    .bind(ws_id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to update webhook source: {e}")))?;

    Ok(Json(WebhookSourceResponse {
        id: ws_id,
        name,
        active,
        auto_accept,
        auto_label,
        requires_review,
        created_at: crate::db::parse_dt(&created_at),
    }))
}

/// DELETE /api/webhooks/:id — remove a webhook source.
pub async fn delete_webhook(
    State(state): State<AppState>,
    auth: ModeratorAuth,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    if auth.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let result = sqlx::query("DELETE FROM webhook_sources WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to delete webhook source: {e}")))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
