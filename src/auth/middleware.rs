use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::cookie::{Key, SignedCookieJar};

use crate::AppState;
use crate::auth::COOKIE_NAME;
use crate::db::adapt_sql;
use crate::error::AppError;

/// Authenticated user identity extracted from signed session cookie.
#[derive(Debug, Clone)]
pub struct Session {
    did: String,
}

impl Session {
    pub fn did(&self) -> &str {
        &self.did
    }

    #[cfg(test)]
    pub fn new_for_test(did: String) -> Self {
        Self { did }
    }
}

impl FromRequestParts<AppState> for Session {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar: SignedCookieJar<Key> = SignedCookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let cookie = jar.get(COOKIE_NAME).ok_or(AppError::Unauthorized)?;
        let did = cookie.value().to_string();

        if did.is_empty() {
            return Err(AppError::Unauthorized);
        }

        Ok(Session { did })
    }
}

/// Authenticated moderator extracted from session cookie + moderators table.
/// Auto-bootstraps the first authenticated user as admin.
#[derive(Debug, Clone)]
pub struct ModeratorAuth {
    pub did: String,
    pub role: String,
}

impl FromRequestParts<AppState> for ModeratorAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        let did = session.did().to_string();
        let backend = state.config.database.backend.clone();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM moderators")
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("moderator count query failed: {e}")))?;

        if count.0 == 0 {
            let mut tx = state
                .db
                .begin()
                .await
                .map_err(|e| AppError::Internal(format!("transaction start failed: {e}")))?;

            let row: Option<(String, String)> = sqlx::query_as(&adapt_sql(
                "INSERT INTO moderators (did, role) VALUES ($1, 'admin')
                 ON CONFLICT (did) DO NOTHING
                 RETURNING did, role",
                backend.clone(),
            ))
            .bind(&did)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("auto-bootstrap moderator failed: {e}")))?;

            tx.commit()
                .await
                .map_err(|e| AppError::Internal(format!("transaction commit failed: {e}")))?;

            if let Some((mod_did, role)) = row {
                tracing::info!(did = %mod_did, "auto-bootstrapped first moderator as admin");
                return Ok(ModeratorAuth { did: mod_did, role });
            }
        }

        let row: Option<(String, String)> = sqlx::query_as(&adapt_sql(
            "SELECT did, role FROM moderators WHERE did = $1",
            backend.clone(),
        ))
        .bind(&did)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("moderator lookup failed: {e}")))?;

        let Some((mod_did, role)) = row else {
            return Err(AppError::Forbidden);
        };

        let db = state.db.clone();
        let update_did = mod_did.clone();
        let now_str = crate::db::now_rfc3339();
        let update_sql = adapt_sql(
            "UPDATE moderators SET last_used_at = $1 WHERE did = $2",
            backend,
        );
        tokio::spawn(async move {
            let _ = sqlx::query(&update_sql)
                .bind(&now_str)
                .bind(&update_did)
                .execute(&db)
                .await;
        });

        Ok(ModeratorAuth { did: mod_did, role })
    }
}
