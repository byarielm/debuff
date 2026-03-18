use axum::extract::{Request, State};
use axum::middleware;
use axum::routing::{get, post};
use axum::{Json, Router};
use bytes::Bytes;
use http_body_util::Full;
use std::convert::Infallible;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::AppState;

pub fn router(state: AppState) -> Router {
    let static_dir = state.config.server.static_dir.clone();

    let fallback_dir = static_dir.clone();
    let spa_fallback = tower::service_fn(move |req: axum::http::Request<_>| {
        let dir = fallback_dir.clone();
        async move {
            let path = req.uri().path();
            let segments: Vec<&str> = path.trim_matches('/').split('/').collect();

            // Try _/index.html in the parent directory (matches Next.js dynamic routes)
            if segments.len() >= 2 {
                let parent = segments[..segments.len() - 1].join("/");
                let dynamic_path = format!("{}/{}/_/index.html", dir, parent);
                if let Ok(body) = tokio::fs::read(&dynamic_path).await {
                    return Ok::<_, Infallible>(
                        axum::http::Response::builder()
                            .header("content-type", "text/html; charset=utf-8")
                            .body(Full::new(Bytes::from(body)))
                            .unwrap(),
                    );
                }
            }

            // Default: serve root index.html
            let index = format!("{}/index.html", dir);
            let body = tokio::fs::read(&index).await.unwrap_or_default();
            Ok::<_, Infallible>(
                axum::http::Response::builder()
                    .header("content-type", "text/html; charset=utf-8")
                    .body(Full::new(Bytes::from(body)))
                    .unwrap(),
            )
        }
    });

    let serve_dir = ServeDir::new(&static_dir)
        .redirect_to_trailing_slash(false)
        .not_found_service(spa_fallback);

    Router::new()
        .route("/health", get(health))
        .route(
            "/xrpc/com.atproto.moderation.createReport",
            post(crate::xrpc::create_report),
        )
        .route(
            "/xrpc/com.atproto.label.queryLabels",
            get(crate::xrpc::query_labels),
        )
        .route(
            "/xrpc/com.atproto.label.subscribeLabels",
            get(crate::xrpc::subscribe_labels),
        )
        .nest("/api", crate::api::routes())
        .nest("/auth", crate::auth::routes::routes())
        .route("/oauth/client-metadata.json", get(client_metadata))
        .fallback_service(serve_dir)
        .layer(middleware::map_request(trim_trailing_slash))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn trim_trailing_slash(mut req: Request) -> Request {
    let path = req.uri().path();
    if path.len() > 1 && path.ends_with('/') {
        let new_path = path.trim_end_matches('/');
        let mut parts = req.uri().clone().into_parts();
        let new_pq = if let Some(q) = req.uri().query() {
            format!("{new_path}?{q}")
        } else {
            new_path.to_string()
        };
        parts.path_and_query = Some(new_pq.parse().unwrap());
        *req.uri_mut() = axum::http::Uri::from_parts(parts).unwrap();
    }
    req
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn client_metadata(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::to_value(&state.oauth.client_metadata).unwrap_or_default())
}
