mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::macros::dual_db_test;
use serde_json::json;

// ---------------------------------------------------------------------------
// 1. GET /api/setup/status returns 200 with all false (placeholder DID)
// ---------------------------------------------------------------------------

dual_db_test!(setup_status_returns_unconfigured, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // The TestApp config uses "did:plc:testlabeler" which is not the placeholder,
    // so labeler_did_configured will be true. However, the PLC document and service
    // record checks hit the mock server (which has no mocks mounted), so those
    // remain false and setup_complete is false.
    let (status, body) = app.get("/api/setup/status").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["labeler_did_configured"], true);
    assert_eq!(body["plc_configured"], false);
    assert_eq!(body["service_record_configured"], false);
    assert_eq!(body["setup_complete"], false);
});

// ---------------------------------------------------------------------------
// 2. GET /api/setup/status works without any auth cookie
// ---------------------------------------------------------------------------

dual_db_test!(setup_status_no_auth_required, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Build a plain GET with no cookie header at all
    let req = Request::builder()
        .method("GET")
        .uri("/api/setup/status")
        .body(Body::empty())
        .unwrap();

    let (status, body) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::OK);
    // Should still return a valid JSON response with boolean fields
    assert!(body["labeler_did_configured"].is_boolean());
    assert!(body["setup_complete"].is_boolean());
});

// ---------------------------------------------------------------------------
// 3. POST /api/setup/labeler-did requires admin auth
// ---------------------------------------------------------------------------

dual_db_test!(setup_labeler_did_requires_admin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // No auth -> 401
    let payload = json!({ "identifier": "did:plc:test123" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/setup/labeler-did")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Non-admin moderator -> 403
    app.seed_moderator("did:plc:modonly", "moderator").await;
    let cookie = app.cookie_for("did:plc:modonly");

    let req = Request::builder()
        .method("POST")
        .uri("/api/setup/labeler-did")
        .header("cookie", &cookie)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
});

// ---------------------------------------------------------------------------
// 4. POST /api/setup/labeler-did accepts a DID and stores it
// ---------------------------------------------------------------------------

dual_db_test!(setup_labeler_did_accepts_did, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Point DEBUFF_CONFIG at a temp file so the config write doesn't touch
    // the project's real config.toml.
    let tmp = tempfile::NamedTempFile::new().expect("failed to create temp config");
    // SAFETY: tests run serially (serial_test) so no concurrent env access.
    unsafe { std::env::set_var("DEBUFF_CONFIG", tmp.path()) };

    let payload = json!({ "identifier": "did:plc:setuptest" });
    let (status, body) = app.post_authed("/api/setup/labeler-did", &payload).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["did"], "did:plc:setuptest");
    assert!(body["signing_key_generated"].is_boolean());

    // Verify that GET /api/setup/status now reports labeler_did_configured = true
    let (status, body) = app.get("/api/setup/status").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["labeler_did_configured"], true);

    // Clean up env var
    // SAFETY: tests run serially (serial_test) so no concurrent env access.
    unsafe { std::env::remove_var("DEBUFF_CONFIG") };
});

// ---------------------------------------------------------------------------
// 5. POST /api/setup/labeler-did rejects empty identifier
// ---------------------------------------------------------------------------

dual_db_test!(setup_labeler_did_rejects_empty, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Sending an empty object (missing required field) should fail deserialization
    let req = Request::builder()
        .method("POST")
        .uri("/api/setup/labeler-did")
        .header("cookie", &app.admin_cookie)
        .header("content-type", "application/json")
        .body(Body::from(b"{}".to_vec()))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;

    // Axum returns 422 when JSON deserialization fails (missing required field)
    assert!(
        status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::BAD_REQUEST,
        "expected 422 or 400, got {status}"
    );
});

// ---------------------------------------------------------------------------
// 6. POST /api/setup/plc/request requires admin
// ---------------------------------------------------------------------------

dual_db_test!(setup_plc_request_requires_admin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/setup/plc/request")
        .body(Body::empty())
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
});

// ---------------------------------------------------------------------------
// 7. POST /api/setup/record requires admin
// ---------------------------------------------------------------------------

dual_db_test!(setup_record_requires_admin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/setup/record")
        .header("content-type", "application/json")
        .body(Body::from(b"{}".to_vec()))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
});

// ---------------------------------------------------------------------------
// 8. POST /api/setup/complete requires admin
// ---------------------------------------------------------------------------

dual_db_test!(setup_complete_requires_admin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let req = Request::builder()
        .method("POST")
        .uri("/api/setup/complete")
        .body(Body::empty())
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
});

// ---------------------------------------------------------------------------
// 9. POST /api/setup/labeler-auth requires admin
// ---------------------------------------------------------------------------

dual_db_test!(setup_labeler_auth_requires_admin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = json!({ "handle": "test.bsky.social" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/setup/labeler-auth")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
});

// ---------------------------------------------------------------------------
// 10. POST /api/setup/labeler-auth/confirm requires admin
// ---------------------------------------------------------------------------

dual_db_test!(setup_confirm_requires_admin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = json!({
        "did": "did:plc:test",
        "restore_did": "did:plc:admin1"
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/setup/labeler-auth/confirm")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;
    // No ModeratorAuth on this endpoint — it validates via restore_did, so
    // an unknown DID returns 403 (Forbidden), not 401.
    assert_eq!(status, StatusCode::FORBIDDEN);
});
