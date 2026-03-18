use sqlx::AnyPool;
use std::collections::HashMap;

use super::adapt_sql;
use crate::config::DatabaseBackend;

/// Read all settings from the database into a HashMap.
pub async fn load_all(
    pool: &AnyPool,
    backend: DatabaseBackend,
) -> Result<HashMap<String, String>, sqlx::Error> {
    let rows: Vec<(String, String)> =
        sqlx::query_as(&adapt_sql("SELECT key, value FROM settings", backend))
            .fetch_all(pool)
            .await?;
    Ok(rows.into_iter().collect())
}

/// Read a single setting by key.
pub async fn get(
    pool: &AnyPool,
    backend: DatabaseBackend,
    key: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> = sqlx::query_as(&adapt_sql(
        "SELECT value FROM settings WHERE key = $1",
        backend,
    ))
    .bind(key)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(v,)| v))
}

/// Upsert a setting (insert or update).
pub async fn set(
    pool: &AnyPool,
    backend: DatabaseBackend,
    key: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(&adapt_sql(
        "INSERT INTO settings (key, value, updated_at) VALUES ($1, $2, $NOW) \
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = $NOW",
        backend,
    ))
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}
