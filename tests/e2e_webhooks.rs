mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::macros::dual_db_test;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn sign_payload(secret: &str, body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    let result = mac.finalize().into_bytes();
    format!("sha256={}", hex::encode(result))
}

// ---------------------------------------------------------------------------
// CRUD
// ---------------------------------------------------------------------------

dual_db_test!(create_webhook_returns_201_with_secret, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app
        .post_authed("/api/webhooks", &json!({ "name": "test-hook" }))
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"], "test-hook");
    assert!(body["secret"].as_str().is_some());
    assert!(!body["secret"].as_str().unwrap().is_empty());
});

dual_db_test!(list_webhooks_omits_secret, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Create a webhook first
    app.post_authed("/api/webhooks", &json!({ "name": "listed-hook" }))
        .await;

    let (status, body) = app.get_authed("/api/webhooks").await;

    assert_eq!(status, StatusCode::OK);
    let hooks = body.as_array().unwrap();
    assert!(!hooks.is_empty());
    // The list endpoint should not include the secret field
    assert!(hooks[0].get("secret").is_none());
});

dual_db_test!(update_webhook_settings, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (_, created) = app
        .post_authed("/api/webhooks", &json!({ "name": "update-hook" }))
        .await;
    let id = created["id"].as_i64().unwrap();

    let (status, body) = app
        .patch_authed(
            &format!("/api/webhooks/{id}"),
            &json!({ "auto_label": true, "requires_review": false }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["auto_label"], true);
    assert_eq!(body["requires_review"], false);
});

dual_db_test!(delete_webhook_returns_204, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (_, created) = app
        .post_authed("/api/webhooks", &json!({ "name": "delete-hook" }))
        .await;
    let id = created["id"].as_i64().unwrap();

    let (status, _) = app.delete_authed(&format!("/api/webhooks/{id}")).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
});

dual_db_test!(non_admin_cannot_manage_webhooks, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Seed a non-admin moderator
    app.seed_moderator("did:plc:modonly", "moderator").await;
    let cookie = app.cookie_for("did:plc:modonly");

    let req = Request::builder()
        .method("GET")
        .uri("/api/webhooks")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
});

// ---------------------------------------------------------------------------
// Ingest
// ---------------------------------------------------------------------------

dual_db_test!(ingest_valid_hmac_creates_report, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Create a webhook to get a secret
    let (_, created) = app
        .post_authed("/api/webhooks", &json!({ "name": "ingest-hook" }))
        .await;
    let secret = created["secret"].as_str().unwrap().to_string();

    let payload = json!({
        "subject_uri": "at://did:plc:ingest/app.bsky.feed.post/1",
        "subject_did": "did:plc:ingest"
    });
    let body_bytes = serde_json::to_vec(&payload).unwrap();
    let signature = sign_payload(&secret, &body_bytes);

    let req = Request::builder()
        .method("POST")
        .uri("/api/ingest")
        .header("content-type", "application/json")
        .header("x-webhook-signature", &signature)
        .body(Body::from(body_bytes))
        .unwrap();

    let (status, body) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["report_id"].as_i64().is_some());
});

dual_db_test!(ingest_invalid_hmac_returns_401, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Create a webhook
    app.post_authed("/api/webhooks", &json!({ "name": "bad-hmac-hook" }))
        .await;

    let payload = json!({
        "subject_uri": "at://did:plc:bad/post/1",
        "subject_did": "did:plc:bad"
    });
    let body_bytes = serde_json::to_vec(&payload).unwrap();
    let signature = sign_payload("wrong-secret", &body_bytes);

    let req = Request::builder()
        .method("POST")
        .uri("/api/ingest")
        .header("content-type", "application/json")
        .header("x-webhook-signature", &signature)
        .body(Body::from(body_bytes))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
});

dual_db_test!(ingest_missing_signature_returns_401, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = json!({
        "subject_uri": "at://did:plc:nosig/post/1",
        "subject_did": "did:plc:nosig"
    });
    let body_bytes = serde_json::to_vec(&payload).unwrap();

    let req = Request::builder()
        .method("POST")
        .uri("/api/ingest")
        .header("content-type", "application/json")
        .body(Body::from(body_bytes))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
});

dual_db_test!(ingest_auto_label_auto_accept_no_review, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Create webhook
    let (_, created) = app
        .post_authed("/api/webhooks", &json!({ "name": "auto-hook" }))
        .await;
    let id = created["id"].as_i64().unwrap();
    let secret = created["secret"].as_str().unwrap().to_string();

    // Seed the label definition so labels can be applied
    app.seed_definition("spam").await;

    // Update webhook: auto_label=true, auto_accept=true, requires_review=false
    app.patch_authed(
        &format!("/api/webhooks/{id}"),
        &json!({ "auto_label": true, "auto_accept": true, "requires_review": false }),
    )
    .await;

    let payload = json!({
        "subject_uri": "at://did:plc:auto/app.bsky.feed.post/1",
        "subject_did": "did:plc:auto",
        "suggested_labels": ["spam"]
    });
    let body_bytes = serde_json::to_vec(&payload).unwrap();
    let signature = sign_payload(&secret, &body_bytes);

    let req = Request::builder()
        .method("POST")
        .uri("/api/ingest")
        .header("content-type", "application/json")
        .header("x-webhook-signature", &signature)
        .body(Body::from(body_bytes))
        .unwrap();

    let (status, body) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::OK);
    // In no-review mode, response is { labels_applied: N }
    assert_eq!(body["labels_applied"], 1);
});
