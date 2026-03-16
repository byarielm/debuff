mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::macros::dual_db_test;
use serde_json::json;

dual_db_test!(no_auth_returns_401, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, _) = app.get("/api/moderators").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
});

dual_db_test!(non_moderator_returns_403, |backend| async move {
    let app = common::app::TestApp::new(backend).await;
    let cookie = app.cookie_for("did:plc:nobody");

    let req = Request::builder()
        .method("GET")
        .uri("/api/moderators")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
});

dual_db_test!(list_moderators_returns_seeded_admin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app.get_authed("/api/moderators").await;

    assert_eq!(status, StatusCode::OK);
    let mods = body.as_array().expect("expected array");
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0]["did"], common::app::ADMIN_DID);
    assert_eq!(mods[0]["role"], "admin");
});

dual_db_test!(add_moderator_returns_201, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = json!({ "did": "did:plc:newmod" });
    let (status, body) = app.post_authed("/api/moderators", &payload).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["did"], "did:plc:newmod");
    assert_eq!(body["role"], "moderator");
});

dual_db_test!(add_duplicate_moderator_returns_409, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let payload = json!({ "did": "did:plc:dupmod" });
    let (status, _) = app.post_authed("/api/moderators", &payload).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = app.post_authed("/api/moderators", &payload).await;
    assert_eq!(status, StatusCode::CONFLICT);
});

dual_db_test!(non_admin_cannot_add_moderator, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Seed a moderator-role user
    app.seed_moderator("did:plc:moduser", "moderator").await;
    let cookie = app.cookie_for("did:plc:moduser");

    let payload = json!({ "did": "did:plc:anothermod" });
    let req = Request::builder()
        .method("POST")
        .uri("/api/moderators")
        .header("cookie", &cookie)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let (status, _) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
});

dual_db_test!(remove_moderator_returns_204, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Add a moderator first
    let payload = json!({ "did": "did:plc:removeme" });
    let (status, _) = app.post_authed("/api/moderators", &payload).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = app
        .delete_authed("/api/moderators/did:plc:removeme")
        .await;

    assert_eq!(status, StatusCode::NO_CONTENT);
});

dual_db_test!(cannot_remove_self, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let uri = format!("/api/moderators/{}", common::app::ADMIN_DID);
    let (status, _) = app.delete_authed(&uri).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
});

dual_db_test!(remove_nonexistent_returns_404, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, _) = app
        .delete_authed("/api/moderators/did:plc:ghost")
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
});

dual_db_test!(first_user_auto_bootstraps_as_admin, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    // Delete all moderators to trigger bootstrap
    sqlx::query("DELETE FROM moderators")
        .execute(&app.pool)
        .await
        .expect("failed to delete moderators");

    // Authenticate as a brand-new DID
    let cookie = app.cookie_for("did:plc:bootstrap");
    let req = Request::builder()
        .method("GET")
        .uri("/api/moderators")
        .header("cookie", &cookie)
        .body(Body::empty())
        .unwrap();

    let (status, body) = common::app::send_request(&app.router, req).await;

    assert_eq!(status, StatusCode::OK);
    let mods = body.as_array().expect("expected array");
    assert!(
        mods.iter().any(|m| m["did"] == "did:plc:bootstrap" && m["role"] == "admin"),
        "bootstrap user should be auto-created as admin"
    );
});
