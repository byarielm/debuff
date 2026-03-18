use atrium_api::agent::Agent;
use atrium_api::com::atproto::identity::sign_plc_operation;
use atrium_api::com::atproto::identity::submit_plc_operation;
use atrium_api::com::atproto::repo::put_record;
use atrium_api::types::TryIntoUnknown;
use atrium_api::types::string::Did;
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::cookie::{Cookie, SignedCookieJar};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::{COOKIE_NAME, ModeratorAuth};
use crate::config::update_config_file;
use crate::db::adapt_sql;
use crate::error::AppError;
use crate::signing::LabelSigner;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(status))
        .route("/labeler-did", post(set_labeler_did))
        .route("/labeler-auth", post(labeler_auth))
        .route("/labeler-auth/confirm", post(labeler_auth_confirm))
        .route("/plc/request", post(plc_request))
        .route("/plc/submit", post(plc_submit))
        .route("/record", post(create_record))
        .route("/complete", post(complete))
        .route("/resolve-nsid", get(resolve_nsid))
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

async fn get_labeler_did(state: &AppState) -> Result<String, AppError> {
    // First check the in-memory setup mutex
    let guard = state.setup_labeler_did.lock().await;
    if let Some(did) = guard.clone() {
        return Ok(did);
    }
    drop(guard);

    // Fall back to the config file value (if it's not the placeholder)
    let did = &state.config.labeler.did;
    if !did.is_empty() && did != "did:plc:placeholder" {
        return Ok(did.clone());
    }

    Err(AppError::BadRequest(
        "No labeler DID set in setup session".into(),
    ))
}

async fn resolve_did_document(
    http: &reqwest::Client,
    plc_url: &str,
    did: &str,
) -> Result<serde_json::Value, AppError> {
    let url = format!("{}/{}", plc_url.trim_end_matches('/'), did);
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

fn find_service_endpoint(did_doc: &serde_json::Value, fragment: &str) -> Option<String> {
    did_doc["service"]
        .as_array()?
        .iter()
        .find(|s| {
            s["id"]
                .as_str()
                .is_some_and(|id| id == fragment || id.ends_with(fragment))
        })
        .and_then(|s| s["serviceEndpoint"].as_str().map(String::from))
}

async fn restore_labeler_agent(
    state: &AppState,
    did_str: &str,
) -> Result<Agent<impl atrium_api::agent::SessionManager>, AppError> {
    let did = Did::new(did_str.to_string())
        .map_err(|e| AppError::Internal(format!("Invalid DID: {e}")))?;
    let session = state
        .oauth
        .restore(&did)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to restore OAuth session: {e}")))?;
    Ok(Agent::new(session))
}

fn require_admin(admin: &ModeratorAuth) -> Result<(), AppError> {
    if admin.role != "admin" {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// GET /status
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct StatusResponse {
    labeler_did: Option<String>,
    labeler_did_configured: bool,
    plc_configured: bool,
    service_record_configured: bool,
    setup_complete: bool,
}

async fn status(State(state): State<AppState>) -> Result<Json<StatusResponse>, AppError> {
    // Check in-memory mutex first (set during setup), then fall back to config
    let mutex_did = state.setup_labeler_did.lock().await.clone();
    let did_from_config = &state.config.labeler.did;
    let did = mutex_did.as_deref().unwrap_or(did_from_config.as_str());
    let labeler_did_configured = !did.is_empty() && did != "did:plc:placeholder";

    let mut plc_configured = false;
    let mut service_record_configured = false;

    if labeler_did_configured {
        // Check PLC / DID document
        if let Ok(did_doc) =
            resolve_did_document(&state.http, &state.config.labeler.plc_url, did).await
        {
            let has_labeler_service = did_doc["service"]
                .as_array()
                .map(|arr| {
                    arr.iter().any(|s| {
                        s["id"].as_str().is_some_and(|id| {
                            id == "#atproto_labeler" || id.ends_with("#atproto_labeler")
                        })
                    })
                })
                .unwrap_or(false);

            let has_label_verification = did_doc["verificationMethod"]
                .as_array()
                .map(|arr| {
                    arr.iter().any(|v| {
                        v["id"].as_str().is_some_and(|id| {
                            id == "#atproto_label" || id.ends_with("#atproto_label")
                        })
                    })
                })
                .unwrap_or(false);

            plc_configured = has_labeler_service && has_label_verification;

            // Check service record — need PDS URL from DID doc
            if let Some(pds_url) = find_service_endpoint(&did_doc, "#atproto_pds") {
                let record_url = format!(
                    "{}/xrpc/com.atproto.repo.getRecord?repo={}&collection=app.bsky.labeler.service&rkey=self",
                    pds_url.trim_end_matches('/'),
                    did
                );
                if let Ok(resp) = state.http.get(&record_url).send().await {
                    service_record_configured = resp.status().is_success();
                }
            }
        }
    }

    let setup_complete = labeler_did_configured && plc_configured && service_record_configured;

    Ok(Json(StatusResponse {
        labeler_did: if labeler_did_configured {
            Some(did.to_string())
        } else {
            None
        },
        labeler_did_configured,
        plc_configured,
        service_record_configured,
        setup_complete,
    }))
}

// ---------------------------------------------------------------------------
// POST /labeler-did
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SetLabelerDidRequest {
    identifier: String,
}

#[derive(Serialize)]
struct SetLabelerDidResponse {
    did: String,
    signing_key_generated: bool,
}

async fn set_labeler_did(
    State(state): State<AppState>,
    admin: ModeratorAuth,
    Json(body): Json<SetLabelerDidRequest>,
) -> Result<Json<SetLabelerDidResponse>, AppError> {
    require_admin(&admin)?;

    let did = if body.identifier.starts_with("did:") {
        body.identifier.clone()
    } else {
        resolve_handle_to_did(&state, &body.identifier).await?
    };

    let mut config_updates: Vec<(&str, String)> = vec![("labeler.did", did.clone())];

    let signing_key_generated = if state.config.labeler.signing_key_path.is_none() {
        let signer = LabelSigner::generate();
        let pem = signer
            .to_pem()
            .map_err(|e| AppError::Internal(format!("Failed to export signing key: {e}")))?;

        // Ensure data directory exists
        std::fs::create_dir_all("data")
            .map_err(|e| AppError::Internal(format!("Failed to create data dir: {e}")))?;

        std::fs::write("data/signing_key.pem", &pem)
            .map_err(|e| AppError::Internal(format!("Failed to write signing key: {e}")))?;

        config_updates.push(("labeler.signing_key_path", "data/signing_key.pem".into()));
        true
    } else {
        false
    };

    let updates_refs: Vec<(&str, &str)> = config_updates
        .iter()
        .map(|(k, v)| (*k, v.as_str()))
        .collect();
    update_config_file(&updates_refs)
        .map_err(|e| AppError::Internal(format!("Failed to update config: {e}")))?;

    // Store the DID in the in-memory setup mutex so other setup
    // endpoints can use it without reloading config from disk.
    {
        let mut guard = state.setup_labeler_did.lock().await;
        *guard = Some(did.clone());
    }

    Ok(Json(SetLabelerDidResponse {
        did,
        signing_key_generated,
    }))
}

async fn resolve_handle_to_did(state: &AppState, handle: &str) -> Result<String, AppError> {
    // Try DNS TXT resolution first
    if let Ok(records) = state.dns.lookup_txt(&format!("_atproto.{handle}")).await {
        for record in records {
            let cleaned = record.trim_matches('"');
            if let Some(did) = cleaned.strip_prefix("did=") {
                return Ok(did.to_string());
            }
        }
    }

    // Fallback: well-known
    let wellknown_url = format!("https://{}/.well-known/atproto-did", handle);
    let resp = state
        .http
        .get(&wellknown_url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to resolve handle: {e}")))?;

    if !resp.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "Could not resolve handle '{}' to a DID",
            handle
        )));
    }

    let did = resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read handle resolution: {e}")))?
        .trim()
        .to_string();

    if did.starts_with("did:") {
        Ok(did)
    } else {
        Err(AppError::BadRequest(format!(
            "Handle '{}' did not resolve to a valid DID",
            handle
        )))
    }
}

// ---------------------------------------------------------------------------
// POST /labeler-auth
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct LabelerAuthRequest {
    handle: String,
}

#[derive(Serialize)]
struct LabelerAuthResponse {
    url: String,
}

async fn labeler_auth(
    State(state): State<AppState>,
    admin: ModeratorAuth,
    Json(body): Json<LabelerAuthRequest>,
) -> Result<Json<LabelerAuthResponse>, AppError> {
    require_admin(&admin)?;

    let url = state
        .oauth
        .authorize(
            &body.handle,
            atrium_oauth::AuthorizeOptions {
                scopes: vec![
                    atrium_oauth::Scope::Known(atrium_oauth::KnownScope::Atproto),
                    atrium_oauth::Scope::Known(atrium_oauth::KnownScope::TransitionGeneric),
                    atrium_oauth::Scope::Unknown("identity:*".into()),
                ],
                ..Default::default()
            },
        )
        .await
        .map_err(|e| AppError::Internal(format!("OAuth authorize failed: {e}")))?;

    Ok(Json(LabelerAuthResponse {
        url: url.to_string(),
    }))
}

// ---------------------------------------------------------------------------
// POST /labeler-auth/confirm
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct LabelerAuthConfirmRequest {
    did: String,
    restore_did: String,
}

#[derive(Serialize)]
struct LabelerAuthConfirmResponse {
    success: bool,
}

async fn labeler_auth_confirm(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(body): Json<LabelerAuthConfirmRequest>,
) -> Result<(SignedCookieJar, Json<LabelerAuthConfirmResponse>), AppError> {
    // No ModeratorAuth here — the cookie currently has the labeler's DID (not a moderator).
    // Instead, verify the restore_did is an admin moderator.
    let backend = state.config.database.backend.clone();
    let role: Option<(String,)> = sqlx::query_as(&adapt_sql(
        "SELECT role FROM moderators WHERE did = $1",
        backend.clone(),
    ))
    .bind(&body.restore_did)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to verify admin: {e}")))?;

    match role {
        Some((r,)) if r == "admin" => {}
        _ => return Err(AppError::Forbidden),
    }

    // Verify the DID has an OAuth session in the database
    let has_session: Option<(i32,)> = sqlx::query_as(&adapt_sql(
        "SELECT 1 FROM oauth_sessions WHERE did = $1",
        backend,
    ))
    .bind(&body.did)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to query oauth session: {e}")))?;

    if has_session.is_none() {
        return Err(AppError::BadRequest(
            "No OAuth session found for the provided DID. Complete the OAuth flow first.".into(),
        ));
    }

    // Store the DID in the setup mutex
    {
        let mut guard = state.setup_labeler_did.lock().await;
        *guard = Some(body.did);
    }

    // Restore the session cookie back to the original user's DID
    let mut cookie = Cookie::new(COOKIE_NAME, body.restore_did);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(axum_extra::extract::cookie::SameSite::Lax);
    if state.config.server.public_url.starts_with("https") {
        cookie.set_secure(true);
    }
    let jar = jar.add(cookie);

    Ok((jar, Json(LabelerAuthConfirmResponse { success: true })))
}

// ---------------------------------------------------------------------------
// POST /plc/request
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct PlcRequestResponse {
    success: bool,
}

async fn plc_request(
    State(state): State<AppState>,
    admin: ModeratorAuth,
) -> Result<Json<PlcRequestResponse>, AppError> {
    require_admin(&admin)?;

    let did = get_labeler_did(&state).await?;
    let agent = restore_labeler_agent(&state, &did).await?;

    agent
        .api
        .com
        .atproto
        .identity
        .request_plc_operation_signature()
        .await
        .map_err(|e| AppError::Internal(format!("requestPlcOperationSignature failed: {e}")))?;

    Ok(Json(PlcRequestResponse { success: true }))
}

// ---------------------------------------------------------------------------
// POST /plc/submit
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PlcSubmitRequest {
    token: String,
}

#[derive(Serialize)]
struct PlcSubmitResponse {
    success: bool,
}

async fn plc_submit(
    State(state): State<AppState>,
    admin: ModeratorAuth,
    Json(body): Json<PlcSubmitRequest>,
) -> Result<Json<PlcSubmitResponse>, AppError> {
    require_admin(&admin)?;

    let did = get_labeler_did(&state).await?;
    let agent = restore_labeler_agent(&state, &did).await?;

    let public_url = state.config.server.public_url.clone();
    let multibase_key = state.signer.public_key_multibase();

    // Fetch the last PLC operation to get current rotation keys, alsoKnownAs,
    // services, and verification methods — the PLC replaces ALL fields, so we
    // must preserve everything we're not intentionally changing.
    let plc_url = state.config.labeler.plc_url.trim_end_matches('/');
    let last_op = state
        .http
        .get(format!("{}/{}/log/last", plc_url, did))
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch PLC log: {e}")))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse PLC log: {e}")))?;

    let rotation_keys: Vec<String> = last_op["rotationKeys"]
        .as_array()
        .ok_or_else(|| AppError::Internal("No rotationKeys in PLC operation".into()))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let also_known_as: Vec<String> = last_op["alsoKnownAs"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    // Merge existing services with new atproto_labeler
    let mut services_map = last_op["services"].as_object().cloned().unwrap_or_default();
    services_map.insert(
        "atproto_labeler".into(),
        serde_json::json!({
            "type": "AtprotoLabeler",
            "endpoint": public_url
        }),
    );
    let services = serde_json::Value::Object(services_map)
        .try_into_unknown()
        .map_err(|e| AppError::Internal(format!("Failed to build services: {e}")))?;

    // Merge existing verification methods with new atproto_label
    let mut vm_map = last_op["verificationMethods"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    vm_map.insert(
        "atproto_label".into(),
        serde_json::json!(format!("did:key:{multibase_key}")),
    );
    let verification_methods = serde_json::Value::Object(vm_map)
        .try_into_unknown()
        .map_err(|e| AppError::Internal(format!("Failed to build verification methods: {e}")))?;

    // Sign the PLC operation
    let sign_result = agent
        .api
        .com
        .atproto
        .identity
        .sign_plc_operation(
            sign_plc_operation::InputData {
                token: Some(body.token),
                services: Some(services),
                verification_methods: Some(verification_methods),
                also_known_as: Some(also_known_as),
                rotation_keys: Some(rotation_keys),
            }
            .into(),
        )
        .await
        .map_err(|e| AppError::Internal(format!("signPlcOperation failed: {e}")))?;

    // Submit the signed PLC operation
    agent
        .api
        .com
        .atproto
        .identity
        .submit_plc_operation(
            submit_plc_operation::InputData {
                operation: sign_result.operation.clone(),
            }
            .into(),
        )
        .await
        .map_err(|e| AppError::Internal(format!("submitPlcOperation failed: {e}")))?;

    Ok(Json(PlcSubmitResponse { success: true }))
}

// ---------------------------------------------------------------------------
// POST /record
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRecordRequest {
    #[serde(default)]
    subject_types: Vec<String>,
    #[serde(default)]
    subject_collections: Vec<String>,
    #[serde(default)]
    reason_types: Vec<String>,
}

#[derive(Serialize)]
struct CreateRecordResponse {
    success: bool,
}

async fn create_record(
    State(state): State<AppState>,
    admin: ModeratorAuth,
    Json(body): Json<CreateRecordRequest>,
) -> Result<Json<CreateRecordResponse>, AppError> {
    require_admin(&admin)?;

    let did = get_labeler_did(&state).await?;
    let agent = restore_labeler_agent(&state, &did).await?;

    let now = crate::db::now_rfc3339();

    let mut record = serde_json::json!({
        "$type": "app.bsky.labeler.service",
        "createdAt": now,
        "policies": {
            "labelValues": []
        }
    });

    if !body.subject_types.is_empty() {
        record["subjectTypes"] = serde_json::json!(body.subject_types);
    }
    if !body.subject_collections.is_empty() {
        record["subjectCollections"] = serde_json::json!(body.subject_collections);
    }
    if !body.reason_types.is_empty() {
        record["reasonTypes"] = serde_json::json!(body.reason_types);
    }

    let record_unknown = record
        .try_into_unknown()
        .map_err(|e| AppError::Internal(format!("Failed to build record: {e}")))?;

    agent
        .api
        .com
        .atproto
        .repo
        .put_record(
            put_record::InputData {
                repo: did
                    .parse()
                    .map_err(|e| AppError::Internal(format!("Invalid repo: {e}")))?,
                collection: "app.bsky.labeler.service"
                    .parse()
                    .map_err(|e| AppError::Internal(format!("Invalid NSID: {e}")))?,
                rkey: "self"
                    .parse()
                    .map_err(|e| AppError::Internal(format!("Invalid rkey: {e}")))?,
                record: record_unknown,
                swap_commit: None,
                swap_record: None,
                validate: None,
            }
            .into(),
        )
        .await
        .map_err(|e| AppError::Internal(format!("putRecord failed: {e}")))?;

    Ok(Json(CreateRecordResponse { success: true }))
}

// ---------------------------------------------------------------------------
// POST /complete
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CompleteResponse {
    success: bool,
}

async fn complete(
    State(state): State<AppState>,
    admin: ModeratorAuth,
) -> Result<Json<CompleteResponse>, AppError> {
    require_admin(&admin)?;

    // Clear the setup mutex (but keep the labeler's OAuth session for ongoing service record updates)
    let mut guard = state.setup_labeler_did.lock().await;
    guard.take();

    Ok(Json(CompleteResponse { success: true }))
}

// ---------------------------------------------------------------------------
// GET /resolve-nsid?nsid=...
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ResolveNsidQuery {
    nsid: String,
}

#[derive(Serialize)]
struct ResolveNsidResponse {
    nsid: String,
    description: Option<String>,
}

/// Resolve an NSID to its lexicon schema description.
/// Steps: NSID authority → domain → DNS TXT _lexicon.{domain} → DID → PDS → getRecord
async fn resolve_nsid(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ResolveNsidQuery>,
) -> Json<ResolveNsidResponse> {
    let description = resolve_nsid_description(&state, &query.nsid).await;
    Json(ResolveNsidResponse {
        nsid: query.nsid,
        description,
    })
}

async fn resolve_nsid_description(state: &AppState, nsid: &str) -> Option<String> {
    // Parse NSID authority: "app.bsky.feed.post" → authority "app.bsky" → domain "bsky.app"
    let parts: Vec<&str> = nsid.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let domain = format!("{}.{}", parts[1], parts[0]);
    let http = &state.http;

    // DNS TXT lookup for _lexicon.{domain}
    let records = state
        .dns
        .lookup_txt(&format!("_lexicon.{domain}"))
        .await
        .ok()?;
    let did = records.iter().find_map(|record| {
        let cleaned = record.trim_matches('"');
        cleaned.strip_prefix("did=").map(String::from)
    })?;

    // Resolve DID to find PDS (handle both did:plc and did:web)
    let did_doc = if did.starts_with("did:web:") {
        let web_domain = did.strip_prefix("did:web:")?;
        let url = format!("https://{}/.well-known/did.json", web_domain);
        http.get(&url)
            .send()
            .await
            .ok()?
            .json::<serde_json::Value>()
            .await
            .ok()?
    } else {
        resolve_did_document(http, "https://plc.directory", &did)
            .await
            .ok()?
    };
    let pds_url = find_service_endpoint(&did_doc, "#atproto_pds")?;

    // Fetch the lexicon schema record
    let record_url = format!(
        "{}/xrpc/com.atproto.repo.getRecord?repo={}&collection=com.atproto.lexicon.schema&rkey={}",
        pds_url.trim_end_matches('/'),
        did,
        nsid
    );
    let record_resp = http.get(&record_url).send().await.ok()?;
    if !record_resp.status().is_success() {
        return None;
    }
    let record: serde_json::Value = record_resp.json().await.ok()?;

    // Try defs.main.description first, then top-level description
    record["value"]["defs"]["main"]["description"]
        .as_str()
        .or_else(|| record["value"]["description"].as_str())
        .map(String::from)
}
