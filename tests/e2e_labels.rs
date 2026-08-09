mod common;

use axum::http::StatusCode;
use common::macros::dual_db_test;
use serde_json::json;

dual_db_test!(
    apply_label_returns_201_with_signature,
    |backend| async move {
        let app = common::app::TestApp::new(backend).await;
        app.seed_definition("test-label").await;

        let payload = common::fixtures::apply_labels(
            "at://did:plc:test/app.bsky.feed.post/abc",
            &["test-label"],
        );
        let (status, body) = app.post_authed("/api/labels", &payload).await;

        assert_eq!(status, StatusCode::CREATED);
        let labels = body.as_array().expect("expected array");
        assert_eq!(labels.len(), 1);
        assert_eq!(labels[0]["val"], "test-label");
        assert_eq!(labels[0]["neg"], false);
        assert!(
            labels[0]["sig"].as_str().is_some_and(|s| !s.is_empty()),
            "expected non-empty signature"
        );
    }
);

dual_db_test!(query_labels_by_uri, |backend| async move {
    let app = common::app::TestApp::new(backend).await;
    app.seed_definition("query-label").await;

    let uri = "at://did:plc:test/app.bsky.feed.post/query1";
    let payload = common::fixtures::apply_labels(uri, &["query-label"]);
    let (status, _) = app.post_authed("/api/labels", &payload).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = app.get_authed(&format!("/api/labels?uri={uri}")).await;

    assert_eq!(status, StatusCode::OK);
    let labels = body.as_array().expect("expected array");
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0]["val"], "query-label");
    assert_eq!(labels[0]["uri"], uri);
});

dual_db_test!(
    negate_label_emits_later_event_and_removes_from_active_query,
    |backend| async move {
        let app = common::app::TestApp::new(backend).await;
        app.seed_definition("neg-label").await;

        let uri = "at://did:plc:test/app.bsky.feed.post/neg1";
        let apply = common::fixtures::apply_labels(uri, &["neg-label"]);
        let (status, _) = app.post_authed("/api/labels", &apply).await;
        assert_eq!(status, StatusCode::CREATED);

        // Negate the label
        let negate = common::fixtures::negate_labels(uri, &["neg-label"]);
        let (status, _) = app.delete_authed_with_body("/api/labels", &negate).await;
        assert_eq!(status, StatusCode::OK);

        // Query should return no active labels
        let (status, body) = app.get_authed(&format!("/api/labels?uri={uri}")).await;
        assert_eq!(status, StatusCode::OK);
        let labels = body.as_array().expect("expected array");
        assert_eq!(
            labels.len(),
            0,
            "negated label should not appear in active query"
        );

        let events: Vec<(i64, i32)> = sqlx::query_as(
            "SELECT seq, neg FROM labels WHERE uri = 'at://did:plc:test/app.bsky.feed.post/neg1' ORDER BY seq",
        )
        .fetch_all(&app.pool)
        .await
        .expect("failed to fetch label events");
        assert_eq!(events.len(), 2, "apply and negate must both be retained");
        assert!(events[1].0 > events[0].0, "negation needs a new sequence");
        assert_eq!(events[0].1, 0);
        assert_eq!(events[1].1, 1);
    }
);

dual_db_test!(unknown_val_returns_400, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = common::fixtures::apply_labels(
        "at://did:plc:test/app.bsky.feed.post/x",
        &["nonexistent-label"],
    );
    let (status, _) = app.post_authed("/api/labels", &payload).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

dual_db_test!(empty_vals_returns_400, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = json!({
        "uri": "at://did:plc:test/app.bsky.feed.post/empty",
        "vals": []
    });
    let (status, _) = app.post_authed("/api/labels", &payload).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

dual_db_test!(apply_multiple_labels_at_once, |backend| async move {
    let app = common::app::TestApp::new(backend).await;
    app.seed_definition("multi-a").await;
    app.seed_definition("multi-b").await;

    let uri = "at://did:plc:test/app.bsky.feed.post/multi";
    let payload = common::fixtures::apply_labels(uri, &["multi-a", "multi-b"]);
    let (status, body) = app.post_authed("/api/labels", &payload).await;

    assert_eq!(status, StatusCode::CREATED);
    let labels = body.as_array().expect("expected array");
    assert_eq!(labels.len(), 2);
});

dual_db_test!(reapplying_label_emits_new_event, |backend| async move {
    let app = common::app::TestApp::new(backend).await;
    app.seed_definition("upsert-label").await;

    let uri = "at://did:plc:test/app.bsky.feed.post/upsert";
    let payload = common::fixtures::apply_labels(uri, &["upsert-label"]);

    // Apply twice
    let (status, _) = app.post_authed("/api/labels", &payload).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = app.post_authed("/api/labels", &payload).await;
    assert_eq!(status, StatusCode::CREATED);

    // There is still only one current active label.
    let (status, body) = app.get_authed(&format!("/api/labels?uri={uri}")).await;
    assert_eq!(status, StatusCode::OK);
    let labels = body.as_array().expect("expected array");
    assert_eq!(labels.len(), 1, "only the latest event should be active");

    let events: Vec<(i64,)> = sqlx::query_as(
        "SELECT seq FROM labels WHERE uri = 'at://did:plc:test/app.bsky.feed.post/upsert' ORDER BY seq",
    )
    .fetch_all(&app.pool)
    .await
    .expect("failed to fetch label events");
    assert_eq!(events.len(), 2, "each apply must be retained as an event");
    assert!(events[1].0 > events[0].0, "reapply needs a new sequence");
});

dual_db_test!(no_auth_returns_401, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, _) = app.get("/api/labels?uri=at://test").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
});
