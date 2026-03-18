mod common;

use common::app::TestApp;
use common::db::Backend;
use debuff::db::settings;

#[tokio::test]
async fn test_settings_crud_sqlite() {
    let app = TestApp::new(Backend::Sqlite).await;
    let backend = app.state.config.database.backend.clone();

    // Initially empty
    let all = settings::load_all(&app.pool, backend.clone())
        .await
        .unwrap();
    assert!(all.is_empty());

    // Set a value
    settings::set(&app.pool, backend.clone(), "labeler.did", "did:plc:test")
        .await
        .unwrap();

    // Read it back
    let val = settings::get(&app.pool, backend.clone(), "labeler.did")
        .await
        .unwrap();
    assert_eq!(val, Some("did:plc:test".to_string()));

    // Update it
    settings::set(&app.pool, backend.clone(), "labeler.did", "did:plc:updated")
        .await
        .unwrap();
    let val = settings::get(&app.pool, backend.clone(), "labeler.did")
        .await
        .unwrap();
    assert_eq!(val, Some("did:plc:updated".to_string()));

    // Load all
    settings::set(
        &app.pool,
        backend.clone(),
        "labeler.signing_key",
        "PEM_DATA",
    )
    .await
    .unwrap();
    let all = settings::load_all(&app.pool, backend.clone())
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all.get("labeler.did").unwrap(), "did:plc:updated");
}

#[tokio::test]
async fn test_settings_crud_postgres() {
    if std::env::var("TEST_DATABASE_URL").is_err() {
        eprintln!("Skipping Postgres test — TEST_DATABASE_URL not set");
        return;
    }
    let app = TestApp::new(Backend::Postgres).await;
    let backend = app.state.config.database.backend.clone();

    settings::set(&app.pool, backend.clone(), "labeler.did", "did:plc:test")
        .await
        .unwrap();
    let val = settings::get(&app.pool, backend.clone(), "labeler.did")
        .await
        .unwrap();
    assert_eq!(val, Some("did:plc:test".to_string()));

    settings::set(
        &app.pool,
        backend.clone(),
        "labeler.signing_key",
        "PEM_DATA",
    )
    .await
    .unwrap();
    let all = settings::load_all(&app.pool, backend.clone())
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
}
