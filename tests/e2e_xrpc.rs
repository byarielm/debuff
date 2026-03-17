mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::macros::dual_db_test;
use std::time::Duration;

// ===========================================================================
// createReport tests (with real service auth)
// ===========================================================================

dual_db_test!(create_report_with_valid_jwt, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let reporter_did = "did:plc:reporter1";
    let keypair = common::auth::TestKeypair::generate();
    common::auth::mock_did_document(&app.mock_server, reporter_did, &keypair).await;

    let aud = &app.state.config.labeler.did;
    let jwt = keypair.build_service_jwt(reporter_did, aud);

    let body = common::fixtures::create_report_repo(
        "did:plc:targetuser",
        "com.atproto.moderation.defs#reasonSpam",
    );

    let req = Request::builder()
        .method("POST")
        .uri("/xrpc/com.atproto.moderation.createReport")
        .header("authorization", format!("Bearer {jwt}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, resp) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["reportedBy"], reporter_did);
    assert_eq!(resp["reasonType"], "com.atproto.moderation.defs#reasonSpam");
});

dual_db_test!(create_report_strong_ref, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let reporter_did = "did:plc:reporter2";
    let keypair = common::auth::TestKeypair::generate();
    common::auth::mock_did_document(&app.mock_server, reporter_did, &keypair).await;

    let aud = &app.state.config.labeler.did;
    let jwt = keypair.build_service_jwt(reporter_did, aud);

    let body = common::fixtures::create_report_record(
        "at://did:plc:targetuser/app.bsky.feed.post/abc123",
        "bafyreiabc123",
        "com.atproto.moderation.defs#reasonViolation",
    );

    let req = Request::builder()
        .method("POST")
        .uri("/xrpc/com.atproto.moderation.createReport")
        .header("authorization", format!("Bearer {jwt}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, resp) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(resp["reportedBy"], reporter_did);
    assert_eq!(resp["subject"]["$type"], "com.atproto.repo.strongRef");
    assert_eq!(
        resp["subject"]["uri"],
        "at://did:plc:targetuser/app.bsky.feed.post/abc123"
    );
});

dual_db_test!(create_report_no_auth, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let body = common::fixtures::create_report_repo(
        "did:plc:targetuser",
        "com.atproto.moderation.defs#reasonSpam",
    );

    let req = Request::builder()
        .method("POST")
        .uri("/xrpc/com.atproto.moderation.createReport")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
});

dual_db_test!(create_report_invalid_reason_type, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let reporter_did = "did:plc:reporter3";
    let keypair = common::auth::TestKeypair::generate();
    common::auth::mock_did_document(&app.mock_server, reporter_did, &keypair).await;

    let aud = &app.state.config.labeler.did;
    let jwt = keypair.build_service_jwt(reporter_did, aud);

    let body = common::fixtures::create_report_repo(
        "did:plc:targetuser",
        "com.atproto.moderation.defs#reasonInvalid",
    );

    let req = Request::builder()
        .method("POST")
        .uri("/xrpc/com.atproto.moderation.createReport")
        .header("authorization", format!("Bearer {jwt}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

dual_db_test!(create_report_duplicate_appends_note, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let reporter_did = "did:plc:reporter4";
    let keypair = common::auth::TestKeypair::generate();
    common::auth::mock_did_document(&app.mock_server, reporter_did, &keypair).await;

    let aud = &app.state.config.labeler.did;
    let jwt = keypair.build_service_jwt(reporter_did, aud);

    let body = common::fixtures::create_report_repo(
        "did:plc:dupesubject",
        "com.atproto.moderation.defs#reasonSpam",
    );

    // First report
    let req1 = Request::builder()
        .method("POST")
        .uri("/xrpc/com.atproto.moderation.createReport")
        .header("authorization", format!("Bearer {jwt}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status1, resp1) = common::app::send_request(&app.router, req1).await;
    assert_eq!(status1, StatusCode::CREATED);
    let first_id = resp1["id"].as_i64().expect("expected id");

    // Second report on same subject
    let req2 = Request::builder()
        .method("POST")
        .uri("/xrpc/com.atproto.moderation.createReport")
        .header("authorization", format!("Bearer {jwt}"))
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let (status2, resp2) = common::app::send_request(&app.router, req2).await;
    assert_eq!(status2, StatusCode::OK);
    assert_eq!(
        resp2["id"].as_i64().expect("expected id"),
        first_id,
        "duplicate report should return the same report id"
    );
});

// ===========================================================================
// queryLabels tests (unauthenticated XRPC)
// ===========================================================================

dual_db_test!(query_labels_xrpc, |backend| async move {
    let app = common::app::TestApp::new(backend).await;
    app.seed_definition("xrpc-label").await;

    // Seed a label via the authed API
    let uri = "at://did:plc:test/app.bsky.feed.post/xrpc1";
    let payload = common::fixtures::apply_labels(uri, &["xrpc-label"]);
    let (status, _) = app.post_authed("/api/labels", &payload).await;
    assert_eq!(status, StatusCode::CREATED);

    // Query via unauthenticated XRPC endpoint
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/xrpc/com.atproto.label.queryLabels?uriPatterns={uri}"
        ))
        .body(Body::empty())
        .unwrap();

    let (status, body) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::OK);
    let labels = body["labels"].as_array().expect("expected labels array");
    assert!(!labels.is_empty(), "expected at least one label");
    assert_eq!(labels[0]["val"], "xrpc-label");
    assert_eq!(labels[0]["uri"], uri);
});

dual_db_test!(query_labels_wildcard_pattern, |backend| async move {
    let app = common::app::TestApp::new(backend).await;
    app.seed_definition("wild-label").await;

    // Seed two labels with different URIs under same prefix
    let uri1 = "at://did:plc:test/app.bsky.feed.post/wild1";
    let uri2 = "at://did:plc:test/app.bsky.feed.post/wild2";
    let payload1 = common::fixtures::apply_labels(uri1, &["wild-label"]);
    let payload2 = common::fixtures::apply_labels(uri2, &["wild-label"]);
    let (s1, _) = app.post_authed("/api/labels", &payload1).await;
    let (s2, _) = app.post_authed("/api/labels", &payload2).await;
    assert_eq!(s1, StatusCode::CREATED);
    assert_eq!(s2, StatusCode::CREATED);

    // Query with wildcard
    let pattern = "at://did:plc:test/app.bsky.feed.post/wild*";
    let req = Request::builder()
        .method("GET")
        .uri(format!(
            "/xrpc/com.atproto.label.queryLabels?uriPatterns={pattern}"
        ))
        .body(Body::empty())
        .unwrap();

    let (status, body) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::OK);
    let labels = body["labels"].as_array().expect("expected labels array");
    assert!(
        labels.len() >= 2,
        "expected at least 2 labels, got {}",
        labels.len()
    );
});

dual_db_test!(query_labels_empty_patterns, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // No uriPatterns param -> 400
    let req = Request::builder()
        .method("GET")
        .uri("/xrpc/com.atproto.label.queryLabels")
        .body(Body::empty())
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

// ===========================================================================
// subscribeLabels WebSocket test
// ===========================================================================

dual_db_test!(subscribe_labels_receives_live_label, |backend| async move {
    use futures_util::StreamExt;
    use tokio_tungstenite::connect_async;

    let app = common::app::TestApp::new(backend).await;
    app.seed_definition("ws-val").await;

    let router = app.router.clone();
    let admin_cookie = app.admin_cookie.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind TCP listener");
    let addr = listener.local_addr().expect("failed to get local addr");

    // Spawn the server
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    // Apply a label via the REST API first
    let http = reqwest::Client::new();
    let payload =
        common::fixtures::apply_labels("at://did:plc:test/app.bsky.feed.post/wstest", &["ws-val"]);
    let resp = http
        .post(format!("http://{addr}/api/labels"))
        .header("cookie", &admin_cookie)
        .json(&payload)
        .send()
        .await
        .expect("failed to POST label");
    let resp_status = resp.status();
    let resp_body = resp.text().await.unwrap_or_default();
    assert_eq!(
        resp_status,
        reqwest::StatusCode::CREATED,
        "label POST failed: {resp_body}"
    );

    // Connect WebSocket client with cursor=0 to replay historical labels
    let ws_url = format!("ws://{addr}/xrpc/com.atproto.label.subscribeLabels?cursor=0");
    let (mut ws_stream, _) = connect_async(&ws_url)
        .await
        .expect("failed to connect WebSocket");

    // Read the WebSocket message with a timeout
    let msg = tokio::time::timeout(Duration::from_secs(5), ws_stream.next())
        .await
        .expect("timed out waiting for WebSocket message")
        .expect("WebSocket stream ended unexpectedly")
        .expect("WebSocket read error");

    match msg {
        tokio_tungstenite::tungstenite::Message::Binary(data) => {
            assert!(!data.is_empty(), "expected non-empty binary frame");
        }
        other => panic!("expected binary WebSocket message, got: {other:?}"),
    }
});
