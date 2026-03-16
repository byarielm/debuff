mod common;

use axum::http::StatusCode;
use common::macros::dual_db_test;
use serde_json::json;

dual_db_test!(create_definition_returns_201_with_locales, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = common::fixtures::label_definition("test-label");
    let (status, body) = app.post_authed("/api/definitions", &payload).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["identifier"], "test-label");
    assert_eq!(body["severity"], "inform");
    assert_eq!(body["builtin"], false);
    let locales = body["locales"].as_array().expect("expected locales array");
    assert_eq!(locales.len(), 1);
    assert_eq!(locales[0]["lang"], "en");
});

dual_db_test!(get_definition_by_id, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = common::fixtures::label_definition("get-me");
    let (status, created) = app.post_authed("/api/definitions", &payload).await;
    assert_eq!(status, StatusCode::CREATED);

    let id = created["id"].as_i64().unwrap();
    let (status, body) = app.get_authed(&format!("/api/definitions/{id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["identifier"], "get-me");
    assert_eq!(body["id"], id);
});

dual_db_test!(list_definitions_includes_builtins, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app.get_authed("/api/definitions").await;

    assert_eq!(status, StatusCode::OK);
    let defs = body.as_array().expect("expected array");
    assert!(
        defs.len() >= 7,
        "expected at least 7 builtin definitions, got {}",
        defs.len()
    );
});

dual_db_test!(update_definition_severity, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = common::fixtures::label_definition("updatable");
    let (status, created) = app.post_authed("/api/definitions", &payload).await;
    assert_eq!(status, StatusCode::CREATED);

    let id = created["id"].as_i64().unwrap();
    let update = json!({ "severity": "alert" });
    let (status, body) = app.patch_authed(&format!("/api/definitions/{id}"), &update).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["severity"], "alert");
    assert_eq!(body["identifier"], "updatable");
});

dual_db_test!(delete_definition_then_404, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = common::fixtures::label_definition("deletable");
    let (status, created) = app.post_authed("/api/definitions", &payload).await;
    assert_eq!(status, StatusCode::CREATED);

    let id = created["id"].as_i64().unwrap();

    let (status, _) = app.delete_authed(&format!("/api/definitions/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = app.get_authed(&format!("/api/definitions/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
});

dual_db_test!(duplicate_identifier_returns_409, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = common::fixtures::label_definition("unique-label");
    let (status, _) = app.post_authed("/api/definitions", &payload).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = app.post_authed("/api/definitions", &payload).await;
    assert_eq!(status, StatusCode::CONFLICT);
});

dual_db_test!(invalid_identifier_uppercase_returns_400, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = json!({
        "identifier": "BadLabel",
        "severity": "inform",
        "blurs": "none",
        "default_setting": "warn",
        "adult_only": false
    });
    let (status, _) = app.post_authed("/api/definitions", &payload).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

dual_db_test!(invalid_severity_returns_400, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = json!({
        "identifier": "bad-severity",
        "severity": "critical",
        "blurs": "none",
        "default_setting": "warn",
        "adult_only": false
    });
    let (status, _) = app.post_authed("/api/definitions", &payload).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

dual_db_test!(cannot_delete_builtin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Find a builtin definition ID
    let (status, body) = app.get_authed("/api/definitions").await;
    assert_eq!(status, StatusCode::OK);

    let defs = body.as_array().unwrap();
    let builtin = defs.iter().find(|d| d["builtin"] == true).expect("expected a builtin");
    let id = builtin["id"].as_i64().unwrap();

    let (status, _) = app.delete_authed(&format!("/api/definitions/{id}")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
});

dual_db_test!(cannot_update_builtin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app.get_authed("/api/definitions").await;
    assert_eq!(status, StatusCode::OK);

    let defs = body.as_array().unwrap();
    let builtin = defs.iter().find(|d| d["builtin"] == true).expect("expected a builtin");
    let id = builtin["id"].as_i64().unwrap();

    let update = json!({ "severity": "alert" });
    let (status, _) = app.patch_authed(&format!("/api/definitions/{id}"), &update).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
});

dual_db_test!(no_auth_returns_401, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, _) = app.get("/api/definitions").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
});
