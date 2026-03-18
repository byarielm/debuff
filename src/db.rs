use chrono::{DateTime, Utc};
use sqlx::AnyPool;
use sqlx::migrate::Migrator;
use std::path::Path;

use crate::config::{DatabaseBackend, DatabaseConfig};

/// Convert PostgreSQL-style placeholders ($1, $2, ...) to SQLite-style (?, ?, ...).
/// Also replaces `$NOW` with the backend-native "current timestamp" expression:
/// - Postgres: `NOW()::TEXT`
/// - SQLite: `datetime('now')`
///
/// Queries should be written with $N placeholders and converted at runtime.
pub fn adapt_sql(sql: &str, backend: DatabaseBackend) -> String {
    match backend {
        DatabaseBackend::Postgres => sql.replace("$NOW", "NOW()::TEXT"),
        DatabaseBackend::Sqlite => {
            // Replace $NOW first (before numbered placeholders)
            let mut result = sql.replace("$NOW", "datetime('now')");
            // Replace $1, $2, ... with ? (reverse order to handle $10 before $1)
            for i in (1..=50).rev() {
                result = result.replace(&format!("${i}"), "?");
            }
            result
        }
    }
}

/// Helper macro to create a query with automatic placeholder conversion.
/// Usage: sql_query!(state, "SELECT * FROM foo WHERE id = $1")
#[macro_export]
macro_rules! sql_query {
    ($state:expr, $sql:expr) => {
        sqlx::query(&$crate::db::adapt_sql(
            $sql,
            $state.config.database.backend.clone(),
        ))
    };
}

/// Helper macro to create a typed query with automatic placeholder conversion.
/// Usage: sql_query_as!(state, (i64, String), "SELECT id, name FROM foo WHERE id = $1")
#[macro_export]
macro_rules! sql_query_as {
    ($state:expr, $type:ty, $sql:expr) => {
        sqlx::query_as::<_, $type>(&$crate::db::adapt_sql(
            $sql,
            $state.config.database.backend.clone(),
        ))
    };
}

/// Parse a database timestamp string to DateTime<Utc>.
/// Handles RFC 3339 (our app writes), Postgres timestamptz format, and SQLite datetime() format.
pub fn parse_dt(s: &str) -> DateTime<Utc> {
    // Try RFC 3339 first (most common — what our app writes)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return dt.with_timezone(&Utc);
    }
    // Try SQLite datetime() format: "2025-03-16 12:34:56"
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return naive.and_utc();
    }
    // Try Postgres-style with timezone offset: "2025-03-16 12:34:56.123456+00"
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f") {
        return naive.and_utc();
    }
    // Fallback
    tracing::warn!("Failed to parse datetime string: {s}");
    DateTime::UNIX_EPOCH
}

/// Get current UTC time as RFC 3339 string for database binding.
pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// Connect to the configured database and run migrations.
pub async fn connect(config: &DatabaseConfig) -> AnyPool {
    sqlx::any::install_default_drivers();

    // For SQLite, ensure the parent directory exists
    if config.backend == DatabaseBackend::Sqlite
        && let Some(path) = config.url.strip_prefix("sqlite://")
    {
        let path = path.split('?').next().unwrap_or(path);
        if let Some(parent) = std::path::Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                panic!("Failed to create data directory {}: {e}", parent.display())
            });
        }
    }

    let pool = AnyPool::connect(&config.url)
        .await
        .expect("Failed to connect to database");

    // Enable foreign keys and WAL mode for SQLite
    if config.backend == DatabaseBackend::Sqlite {
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .expect("Failed to enable foreign keys");

        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await
            .expect("Failed to enable WAL mode");

        sqlx::query("PRAGMA busy_timeout = 5000")
            .execute(&pool)
            .await
            .expect("Failed to set busy timeout");
    }

    // Run migrations from the appropriate directory
    let migration_dir = match config.backend {
        DatabaseBackend::Sqlite => "./migrations/sqlite",
        DatabaseBackend::Postgres => "./migrations/postgres",
    };

    let migrator = Migrator::new(Path::new(migration_dir))
        .await
        .unwrap_or_else(|e| panic!("Failed to load migrations from {migration_dir}: {e}"));

    migrator.run(&pool).await.expect("Failed to run migrations");

    pool
}
