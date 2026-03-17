mod common;

use axum::http::StatusCode;
use common::macros::dual_db_test;
use serde_json::json;

// ---------------------------------------------------------------------------
// Get account
// ---------------------------------------------------------------------------

dual_db_test!(get_account_empty_label_history, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app.get_authed("/api/accounts/did:plc:someone").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["did"], "did:plc:someone");
    assert_eq!(body["labels"].as_array().unwrap().len(), 0);
    assert_eq!(body["active_labels"].as_array().unwrap().len(), 0);
});

// ---------------------------------------------------------------------------
// Apply actions
// ---------------------------------------------------------------------------

dual_db_test!(apply_suspend_action, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app
        .post_authed(
            "/api/accounts/did:plc:target1/action",
            &json!({ "action": "!suspend", "reason": "test" }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["val"], "!suspend");
    assert_eq!(body["uri"], "did:plc:target1");
    assert!(!body["neg"].as_bool().unwrap());
});

dual_db_test!(apply_takedown_action, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app
        .post_authed(
            "/api/accounts/did:plc:target2/action",
            &json!({ "action": "!takedown", "reason": "test" }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["val"], "!takedown");
});

dual_db_test!(apply_hide_action, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app
        .post_authed(
            "/api/accounts/did:plc:target3/action",
            &json!({ "action": "!hide", "reason": "test" }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["val"], "!hide");
});

dual_db_test!(invalid_action_returns_400, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, _) = app
        .post_authed(
            "/api/accounts/did:plc:target4/action",
            &json!({ "action": "!bogus", "reason": "test" }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

dual_db_test!(invalid_did_format_returns_400, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, _) = app
        .post_authed(
            "/api/accounts/notadid/action",
            &json!({ "action": "!suspend", "reason": "test" }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

// ---------------------------------------------------------------------------
// Label history
// ---------------------------------------------------------------------------

dual_db_test!(
    account_shows_label_history_after_action,
    |backend| async move {
        let app = common::app::TestApp::new(backend).await;

        // Apply an action first
        let (status, _) = app
            .post_authed(
                "/api/accounts/did:plc:histtest/action",
                &json!({ "action": "!suspend", "reason": "test" }),
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);

        // Now fetch the account
        let (status, body) = app.get_authed("/api/accounts/did:plc:histtest").await;

        assert_eq!(status, StatusCode::OK);
        let labels = body["labels"].as_array().unwrap();
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0]["val"], "!suspend");

        let active = body["active_labels"].as_array().unwrap();
        assert_eq!(active.len(), 1);
    }
);
