use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::error::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/handle", get(resolve_handle))
        .route("/profile", get(resolve_profile))
        .route("/search", get(resolve_search))
}

// ---------------------------------------------------------------------------
// GET /handle?did=...
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct HandleQuery {
    did: String,
}

#[derive(Serialize)]
struct HandleResponse {
    did: String,
    handle: Option<String>,
}

async fn resolve_handle(
    State(state): State<AppState>,
    Query(query): Query<HandleQuery>,
) -> Result<Json<HandleResponse>, AppError> {
    let did_doc = fetch_did_document(&state.http, &query.did).await?;

    let handle = did_doc["alsoKnownAs"].as_array().and_then(|arr| {
        arr.iter().find_map(|v| {
            v.as_str()
                .and_then(|s| s.strip_prefix("at://"))
                .map(String::from)
        })
    });

    Ok(Json(HandleResponse {
        did: query.did,
        handle,
    }))
}

async fn fetch_did_document(
    http: &reqwest::Client,
    did: &str,
) -> Result<serde_json::Value, AppError> {
    let url = if let Some(domain) = did.strip_prefix("did:web:") {
        format!("https://{}/.well-known/did.json", domain)
    } else if did.starts_with("did:plc:") {
        format!("https://plc.directory/{}", did)
    } else {
        return Err(AppError::BadRequest(format!(
            "Unsupported DID method: {}",
            did
        )));
    };

    let resp = http
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch DID document: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "DID document fetch returned {}",
            resp.status()
        )));
    }

    resp.json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse DID document: {e}")))
}

// ---------------------------------------------------------------------------
// GET /profile?actor=...
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ProfileQuery {
    actor: String,
}

async fn resolve_profile(
    State(state): State<AppState>,
    Query(query): Query<ProfileQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let url = format!(
        "https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile?actor={}",
        urlencoding::encode(&query.actor)
    );

    let resp = state
        .http
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch profile: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "Profile fetch returned {}",
            resp.status()
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse profile response: {e}")))?;

    Ok(Json(body))
}

// ---------------------------------------------------------------------------
// GET /search?q=...&limit=N
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<u8>,
}

async fn resolve_search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = query.limit.unwrap_or(6).min(10);

    let url = format!(
        "https://public.api.bsky.app/xrpc/app.bsky.actor.searchActorsTypeahead?q={}&limit={}",
        urlencoding::encode(&query.q),
        limit
    );

    let resp = state
        .http
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch search results: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "Search fetch returned {}",
            resp.status()
        )));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse search response: {e}")))?;

    Ok(Json(body))
}
