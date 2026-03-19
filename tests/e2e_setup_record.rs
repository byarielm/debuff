mod common;

use axum::http::StatusCode;
use common::macros::dual_db_test;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

// ---------------------------------------------------------------------------
// Mock helpers
// ---------------------------------------------------------------------------

/// Mount a DID document mock that includes the #atproto_pds service
/// pointing back at the mock server itself.
async fn mock_labeler_did_doc(app: &common::app::TestApp) {
    let mock_url = app.mock_server.uri();
    let did = "did:plc:testlabeler";
    let did_doc = serde_json::json!({
        "@context": ["https://www.w3.org/ns/did/v1"],
        "id": did,
        "service": [
            {
                "id": "#atproto_pds",
                "type": "AtprotoPersonalDataServer",
                "serviceEndpoint": mock_url
            }
        ],
        "verificationMethod": []
    });

    Mock::given(method("GET"))
        .and(path(format!("/{did}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&did_doc))
        .mount(&app.mock_server)
        .await;
}

/// Mount a mock for getRecord that returns a labeler service record.
async fn mock_get_record(app: &common::app::TestApp, record_value: serde_json::Value) {
    let response = serde_json::json!({
        "uri": "at://did:plc:testlabeler/app.bsky.labeler.service/self",
        "cid": "bafyreiabc123",
        "value": record_value
    });

    Mock::given(method("GET"))
        .and(path("/xrpc/com.atproto.repo.getRecord"))
        .and(query_param("collection", "app.bsky.labeler.service"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(&app.mock_server)
        .await;
}

/// Mount a mock for getRecord that returns 400 (record not found).
async fn mock_get_record_not_found(app: &common::app::TestApp) {
    Mock::given(method("GET"))
        .and(path("/xrpc/com.atproto.repo.getRecord"))
        .and(query_param("collection", "app.bsky.labeler.service"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": "RecordNotFound",
            "message": "Record not found"
        })))
        .mount(&app.mock_server)
        .await;
}

// ---------------------------------------------------------------------------
// 1. GET /api/setup/record returns current values
// ---------------------------------------------------------------------------

dual_db_test!(get_record_returns_current_values, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Set the labeler DID in the in-memory mutex so the endpoint can find it
    *app.state.setup_labeler_did.lock().await = Some("did:plc:testlabeler".into());

    // Mock the DID document and the getRecord response
    mock_labeler_did_doc(&app).await;
    mock_get_record(
        &app,
        serde_json::json!({
            "$type": "app.bsky.labeler.service",
            "createdAt": "2024-01-01T00:00:00.000Z",
            "subjectTypes": ["account", "record"],
            "subjectCollections": ["app.bsky.feed.post"],
            "reasonTypes": ["com.atproto.moderation.defs#reasonSpam"],
            "policies": { "labelValues": [] }
        }),
    )
    .await;

    let (status, body) = app.get_authed("/api/setup/record").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["subjectTypes"],
        serde_json::json!(["account", "record"])
    );
    assert_eq!(
        body["subjectCollections"],
        serde_json::json!(["app.bsky.feed.post"])
    );
    assert_eq!(
        body["reasonTypes"],
        serde_json::json!(["com.atproto.moderation.defs#reasonSpam"])
    );
});

// ---------------------------------------------------------------------------
// 2. GET /api/setup/record returns 404 when no record exists on the PDS
// ---------------------------------------------------------------------------

dual_db_test!(
    get_record_returns_404_when_no_record,
    |backend| async move {
        let app = common::app::TestApp::new(backend).await;

        *app.state.setup_labeler_did.lock().await = Some("did:plc:testlabeler".into());

        mock_labeler_did_doc(&app).await;
        mock_get_record_not_found(&app).await;

        let (status, _body) = app.get_authed("/api/setup/record").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }
);

// ---------------------------------------------------------------------------
// 3. GET /api/setup/record requires admin auth
// ---------------------------------------------------------------------------

dual_db_test!(get_record_requires_admin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Seed a moderator (non-admin) and build their cookie
    app.seed_moderator("did:plc:modonly", "moderator").await;
    let cookie = app.cookie_for("did:plc:modonly");

    let req = axum::http::Request::builder()
        .method("GET")
        .uri("/api/setup/record")
        .header("cookie", &cookie)
        .body(axum::body::Body::empty())
        .unwrap();

    let (status, _body) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
});
