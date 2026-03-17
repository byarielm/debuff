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
}
