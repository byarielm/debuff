mod common;

use axum::http::StatusCode;
use common::macros::dual_db_test;

dual_db_test!(health_returns_ok, |backend| async move {
    let app = common::app::TestApp::new(backend).await;

    let (status, body) = app.get("/health").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
});
