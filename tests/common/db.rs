use debuff::config::{DatabaseBackend, DatabaseConfig};
use sqlx::AnyPool;

#[derive(Clone, Copy, Debug)]
pub enum Backend {
    Sqlite,
    Postgres,
}

pub async fn test_pool(backend: Backend, temp_dir: &std::path::Path) -> AnyPool {
    sqlx::any::install_default_drivers();

    let config = match backend {
        Backend::Sqlite => {
            let db_path = temp_dir.join("test.db");
            DatabaseConfig {
                backend: DatabaseBackend::Sqlite,
                url: format!("sqlite://{}?mode=rwc", db_path.display()),
            }
        }
        Backend::Postgres => {
            let url = std::env::var("TEST_DATABASE_URL")
                .expect("TEST_DATABASE_URL must be set for Postgres tests");
            DatabaseConfig {
                backend: DatabaseBackend::Postgres,
                url,
            }
        }
    };

    debuff::db::connect(&config).await
}

pub async fn truncate_all(pool: &AnyPool) {
    let tables = [
        "report_notes",
        "reports",
        "labels",
        "label_definition_locales",
        "label_definitions",
        "moderators",
        "webhook_sources",
        "oauth_sessions",
        "oauth_state",
        "labeler_config",
    ];

    for table in &tables {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("failed to truncate {table}: {e}"));
    }

    // Re-seed builtin label definitions
    seed_builtin_labels(pool).await;
}

async fn seed_builtin_labels(pool: &AnyPool) {
    // Use i32 for boolean columns because SQLite via the Any driver stores bools
    // as integers and cannot roundtrip them back to Rust bool.
    let builtins: &[(&str, &str, &str, &str, i32)] = &[
        ("!hide", "alert", "content", "hide", 0),
        ("!no-promote", "none", "none", "hide", 0),
        ("!warn", "alert", "content", "warn", 0),
        ("!no-unauthenticated", "none", "none", "hide", 0),
        ("dmca-violation", "alert", "content", "hide", 0),
        ("doxxing", "alert", "content", "hide", 0),
        ("porn", "alert", "media", "hide", 1),
        ("sexual", "alert", "media", "warn", 1),
        ("nudity", "alert", "media", "warn", 1),
        ("nsfl", "alert", "media", "warn", 0),
        ("gore", "alert", "media", "warn", 0),
    ];

    for &(identifier, severity, blurs, default_setting, adult_only) in builtins {
        sqlx::query(
            "INSERT INTO label_definitions (identifier, severity, blurs, default_setting, adult_only, builtin) \
             VALUES (?, ?, ?, ?, ?, 1) \
             ON CONFLICT (identifier) DO NOTHING",
        )
        .bind(identifier)
        .bind(severity)
        .bind(blurs)
        .bind(default_setting)
        .bind(adult_only)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("failed to seed builtin label {identifier}: {e}"));
    }

    // Also seed the locales to match the migration
    let locales: &[(&str, &str, &str)] = &[
        ("!hide", "Hide", "Hides the content entirely"),
        ("!no-promote", "No Promote", "Prevents content from appearing in recommendations"),
        ("!warn", "Warning", "Shows a warning before displaying content"),
        ("!no-unauthenticated", "No Unauthenticated", "Hides content from logged-out users"),
        ("dmca-violation", "DMCA Violation", "Content that violates copyright law"),
        ("doxxing", "Doxxing", "Content that reveals private personal information"),
        ("porn", "Pornography", "Explicit sexual content"),
        ("sexual", "Sexual", "Sexually suggestive content"),
        ("nudity", "Nudity", "Content containing nudity"),
        ("nsfl", "NSFL", "Content that is disturbing or not safe for life"),
        ("gore", "Gore", "Graphic violent content"),
    ];

    for &(identifier, name, description) in locales {
        sqlx::query(
            "INSERT INTO label_definition_locales (definition_id, lang, name, description) \
             VALUES ((SELECT id FROM label_definitions WHERE identifier = ?), 'en', ?, ?) \
             ON CONFLICT DO NOTHING",
        )
        .bind(identifier)
        .bind(name)
        .bind(description)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("failed to seed locale for {identifier}: {e}"));
    }
}
