mod common;

use axum::http::StatusCode;
use common::macros::dual_db_test;
use serde_json::json;

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

dual_db_test!(list_empty_queue, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app.get_authed("/api/queue").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert!(body["cursor"].is_null());
});

dual_db_test!(list_queue_with_seeded_reports, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    app.seed_report("at://did:plc:alice/app.bsky.feed.post/1", "did:plc:alice", "pending")
        .await;
    app.seed_report("at://did:plc:bob/app.bsky.feed.post/2", "did:plc:bob", "pending")
        .await;

    let (status, body) = app.get_authed("/api/queue").await;

    assert_eq!(status, StatusCode::OK);
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
});

dual_db_test!(list_queue_filter_by_status, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    app.seed_report("at://did:plc:a/post/1", "did:plc:a", "pending").await;
    app.seed_report("at://did:plc:b/post/2", "did:plc:b", "resolved").await;

    let (status, body) = app.get_authed("/api/queue?status=pending").await;

    assert_eq!(status, StatusCode::OK);
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["status"], "pending");
});

// ---------------------------------------------------------------------------
// Detail
// ---------------------------------------------------------------------------

dual_db_test!(get_queue_item_detail, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let id = app
        .seed_report("at://did:plc:x/post/1", "did:plc:x", "pending")
        .await;

    let (status, body) = app.get_authed(&format!("/api/queue/{id}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], id);
    assert_eq!(body["notes"].as_array().unwrap().len(), 0);
    assert_eq!(body["labels"].as_array().unwrap().len(), 0);
});

dual_db_test!(get_nonexistent_queue_item, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, _) = app.get_authed("/api/queue/999999").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
});

// ---------------------------------------------------------------------------
// Update status
// ---------------------------------------------------------------------------

dual_db_test!(update_report_status, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let id = app
        .seed_report("at://did:plc:y/post/1", "did:plc:y", "pending")
        .await;

    let (status, body) = app
        .patch_authed(&format!("/api/queue/{id}"), &json!({ "status": "resolved" }))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "resolved");
});

dual_db_test!(invalid_status_returns_400, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let id = app
        .seed_report("at://did:plc:z/post/1", "did:plc:z", "pending")
        .await;

    let (status, _) = app
        .patch_authed(&format!("/api/queue/{id}"), &json!({ "status": "bogus" }))
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

// ---------------------------------------------------------------------------
// Assign moderator
// ---------------------------------------------------------------------------

dual_db_test!(assign_moderator, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let id = app
        .seed_report("at://did:plc:w/post/1", "did:plc:w", "pending")
        .await;

    let (status, body) = app
        .patch_authed(
            &format!("/api/queue/{id}/assign"),
            &json!({ "did": common::app::ADMIN_DID }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["assigned_to"], common::app::ADMIN_DID);
});

dual_db_test!(assign_unknown_moderator_returns_400, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let id = app
        .seed_report("at://did:plc:v/post/1", "did:plc:v", "pending")
        .await;

    let (status, _) = app
        .patch_authed(
            &format!("/api/queue/{id}/assign"),
            &json!({ "did": "did:plc:unknown" }),
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

// ---------------------------------------------------------------------------
// Escalate
// ---------------------------------------------------------------------------

dual_db_test!(escalate_report_bumps_priority, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let id = app
        .seed_report("at://did:plc:e/post/1", "did:plc:e", "pending")
        .await;

    let (status, body) = app
        .post_authed(&format!("/api/queue/{id}/escalate"), &json!({}))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["priority"], 1);
});

// ---------------------------------------------------------------------------
// Notes
// ---------------------------------------------------------------------------

dual_db_test!(add_note_returns_201, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let id = app
        .seed_report("at://did:plc:n/post/1", "did:plc:n", "pending")
        .await;

    let (status, body) = app
        .post_authed(
            &format!("/api/queue/{id}/notes"),
            &json!({ "content": "looks bad" }),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["content"], "looks bad");
    assert_eq!(body["author"], common::app::ADMIN_DID);
});

dual_db_test!(empty_note_returns_400, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let id = app
        .seed_report("at://did:plc:m/post/1", "did:plc:m", "pending")
        .await;

    let (status, _) = app
        .post_authed(&format!("/api/queue/{id}/notes"), &json!({ "content": "" }))
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

dual_db_test!(note_on_nonexistent_report_returns_404, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, _) = app
        .post_authed("/api/queue/999999/notes", &json!({ "content": "hello" }))
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
});

// ---------------------------------------------------------------------------
// No auth
// ---------------------------------------------------------------------------

dual_db_test!(queue_no_auth_returns_401, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, _) = app.get("/api/queue").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
});
