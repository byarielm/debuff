use atrium_api::agent::Agent;
use atrium_api::com::atproto::repo::{get_record, put_record};
use atrium_api::types::string::Did;
use atrium_api::types::TryIntoUnknown;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::ModeratorAuth;
use crate::error::AppError;

#[derive(Serialize)]
pub struct LocaleResponse {
    pub lang: String,
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct DefinitionResponse {
    pub id: i32,
    pub identifier: String,
    pub severity: String,
    pub blurs: String,
    pub default_setting: String,
    pub adult_only: bool,
    pub builtin: bool,
    pub locales: Vec<LocaleResponse>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct LocaleInput {
    pub lang: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Deserialize)]
pub struct CreateDefinitionBody {
    pub identifier: String,
    #[serde(default = "default_severity")]
    pub severity: String,
    #[serde(default = "default_blurs")]
    pub blurs: String,
    #[serde(default = "default_setting")]
    pub default_setting: String,
    #[serde(default)]
    pub adult_only: bool,
    #[serde(default)]
    pub locales: Vec<LocaleInput>,
}

fn default_severity() -> String {
    "none".into()
}
fn default_blurs() -> String {
    "none".into()
}
fn default_setting() -> String {
    "warn".into()
}

#[derive(Deserialize)]
pub struct UpdateDefinitionBody {
    pub identifier: Option<String>,
    pub severity: Option<String>,
    pub blurs: Option<String>,
    pub default_setting: Option<String>,
    pub adult_only: Option<bool>,
    pub locales: Option<Vec<LocaleInput>>,
}

/// Validate that an identifier is lowercase kebab-case: [a-z][a-z-]*, max 100 chars.
fn validate_identifier(id: &str) -> Result<(), AppError> {
    if id.is_empty() || id.len() > 100 {
        return Err(AppError::BadRequest(
            "identifier must be 1-100 characters".into(),
        ));
    }
    if !id.bytes().all(|b| b.is_ascii_lowercase() || b == b'-') {
        return Err(AppError::BadRequest(
            "identifier must contain only lowercase ascii letters and hyphens".into(),
        ));
    }
    Ok(())
}

const VALID_SEVERITIES: &[&str] = &["none", "inform", "alert"];
const VALID_BLURS: &[&str] = &["none", "content", "media"];
const VALID_DEFAULT_SETTINGS: &[&str] = &["ignore", "warn", "hide"];

fn validate_severity(s: &str) -> Result<(), AppError> {
    if !VALID_SEVERITIES.contains(&s) {
        return Err(AppError::BadRequest(format!(
            "severity must be one of: {}",
            VALID_SEVERITIES.join(", ")
        )));
    }
    Ok(())
}

fn validate_blurs(s: &str) -> Result<(), AppError> {
    if !VALID_BLURS.contains(&s) {
        return Err(AppError::BadRequest(format!(
            "blurs must be one of: {}",
            VALID_BLURS.join(", ")
        )));
    }
    Ok(())
}

fn validate_default_setting(s: &str) -> Result<(), AppError> {
    if !VALID_DEFAULT_SETTINGS.contains(&s) {
        return Err(AppError::BadRequest(format!(
            "default_setting must be one of: {}",
            VALID_DEFAULT_SETTINGS.join(", ")
        )));
    }
    Ok(())
}

/// GET /api/definitions/:id — get a single definition with its locales.
pub async fn get_definition(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(id): Path<i32>,
) -> Result<Json<DefinitionResponse>, AppError> {
    let row: Option<(i32, String, String, String, String, i32, i32, String)> =
        sqlx::query_as(
            "SELECT id, identifier, severity, blurs, default_setting, adult_only, builtin, created_at \
             FROM label_definitions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to fetch definition: {e}")))?;

    let (def_id, identifier, severity, blurs, default_setting, adult_only_int, builtin_int, created_at) =
        row.ok_or(AppError::NotFound)?;

    let adult_only = adult_only_int != 0;
    let builtin = builtin_int != 0;

    let locale_rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT lang, name, description FROM label_definition_locales \
         WHERE definition_id = ? ORDER BY lang",
    )
    .bind(def_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("failed to fetch locales: {e}")))?;

    let locales = locale_rows
        .into_iter()
        .map(|(lang, name, description)| LocaleResponse {
            lang,
            name,
            description,
        })
        .collect();

    Ok(Json(DefinitionResponse {
        id: def_id,
        identifier,
        severity,
        blurs,
        default_setting,
        adult_only,
        builtin,
        locales,
        created_at: crate::db::parse_dt(&created_at),
    }))
}

/// GET /api/definitions — list all definitions with their locales.
pub async fn list_definitions(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
) -> Result<Json<Vec<DefinitionResponse>>, AppError> {
    let rows: Vec<(i32, String, String, String, String, i32, i32, String)> =
        sqlx::query_as(
            "SELECT id, identifier, severity, blurs, default_setting, adult_only, builtin, created_at \
             FROM label_definitions ORDER BY id",
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to list definitions: {e}")))?;

    let definition_ids: Vec<i32> = rows.iter().map(|r| r.0).collect();

    let locale_rows: Vec<(i32, String, String, String)> = if definition_ids.is_empty() {
        vec![]
    } else {
        let placeholders: Vec<&str> = definition_ids.iter().map(|_| "?").collect();
        let in_clause = placeholders.join(", ");
        let sql = format!(
            "SELECT definition_id, lang, name, description \
             FROM label_definition_locales \
             WHERE definition_id IN ({in_clause}) \
             ORDER BY definition_id, lang"
        );
        let mut query = sqlx::query_as::<_, (i32, String, String, String)>(&sql);
        for id in &definition_ids {
            query = query.bind(id);
        }
        query
            .fetch_all(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("failed to list locales: {e}")))?
    };

    // Group locales by definition_id
    let mut locale_map: std::collections::HashMap<i32, Vec<LocaleResponse>> =
        std::collections::HashMap::new();
    for (def_id, lang, name, description) in locale_rows {
        locale_map
            .entry(def_id)
            .or_default()
            .push(LocaleResponse {
                lang,
                name,
                description,
            });
    }

    let definitions: Vec<DefinitionResponse> = rows
        .into_iter()
        .map(
            |(id, identifier, severity, blurs, default_setting, adult_only_int, builtin_int, created_at)| {
                DefinitionResponse {
                    id,
                    identifier,
                    severity,
                    blurs,
                    default_setting,
                    adult_only: adult_only_int != 0,
                    builtin: builtin_int != 0,
                    locales: locale_map.remove(&id).unwrap_or_default(),
                    created_at: crate::db::parse_dt(&created_at),
                }
            },
        )
        .collect();

    Ok(Json(definitions))
}

/// POST /api/definitions — create a custom definition.
pub async fn create_definition(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Json(body): Json<CreateDefinitionBody>,
) -> Result<(StatusCode, Json<DefinitionResponse>), AppError> {
    validate_identifier(&body.identifier)?;
    validate_severity(&body.severity)?;
    validate_blurs(&body.blurs)?;
    validate_default_setting(&body.default_setting)?;

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("transaction start failed: {e}")))?;

    let row: (i32, String) = sqlx::query_as(
        "INSERT INTO label_definitions (identifier, severity, blurs, default_setting, adult_only, builtin) \
         VALUES (?, ?, ?, ?, ?, false) \
         RETURNING id, created_at",
    )
    .bind(&body.identifier)
    .bind(&body.severity)
    .bind(&body.blurs)
    .bind(&body.default_setting)
    .bind(body.adult_only)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.is_unique_violation() {
                return AppError::Conflict(format!(
                    "definition with identifier '{}' already exists",
                    body.identifier
                ));
            }
        }
        AppError::Internal(format!("failed to create definition: {e}"))
    })?;

    let def_id = row.0;
    let created_at = crate::db::parse_dt(&row.1);

    let mut locales = Vec::new();
    for locale in &body.locales {
        sqlx::query(
            "INSERT INTO label_definition_locales (definition_id, lang, name, description) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(def_id)
        .bind(&locale.lang)
        .bind(&locale.name)
        .bind(&locale.description)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("failed to insert locale: {e}")))?;

        locales.push(LocaleResponse {
            lang: locale.lang.clone(),
            name: locale.name.clone(),
            description: locale.description.clone(),
        });
    }

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("transaction commit failed: {e}")))?;

    sync_service_record(&state).await;

    Ok((
        StatusCode::CREATED,
        Json(DefinitionResponse {
            id: def_id,
            identifier: body.identifier,
            severity: body.severity,
            blurs: body.blurs,
            default_setting: body.default_setting,
            adult_only: body.adult_only,
            builtin: false,
            locales,
            created_at,
        }),
    ))
}

/// PATCH /api/definitions/:id — update a definition. Rejects builtin definitions.
pub async fn update_definition(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(id): Path<i32>,
    Json(body): Json<UpdateDefinitionBody>,
) -> Result<Json<DefinitionResponse>, AppError> {
    // Fetch existing definition
    let row: Option<(i32, String, String, String, String, i32, i32, String)> =
        sqlx::query_as(
            "SELECT id, identifier, severity, blurs, default_setting, adult_only, builtin, created_at \
             FROM label_definitions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to fetch definition: {e}")))?;

    let (def_id, mut identifier, mut severity, mut blurs, mut default_setting, adult_only_int, builtin_int, created_at) =
        row.ok_or(AppError::NotFound)?;

    let builtin = builtin_int != 0;
    let mut adult_only = adult_only_int != 0;

    if builtin {
        return Err(AppError::Forbidden);
    }

    // Apply updates
    if let Some(ref new_id) = body.identifier {
        validate_identifier(new_id)?;
        identifier = new_id.clone();
    }
    if let Some(ref s) = body.severity {
        validate_severity(s)?;
        severity = s.clone();
    }
    if let Some(ref b) = body.blurs {
        validate_blurs(b)?;
        blurs = b.clone();
    }
    if let Some(ref ds) = body.default_setting {
        validate_default_setting(ds)?;
        default_setting = ds.clone();
    }
    if let Some(ao) = body.adult_only {
        adult_only = ao;
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("transaction start failed: {e}")))?;

    sqlx::query(
        "UPDATE label_definitions \
         SET identifier = ?, severity = ?, blurs = ?, default_setting = ?, adult_only = ? \
         WHERE id = ?",
    )
    .bind(&identifier)
    .bind(&severity)
    .bind(&blurs)
    .bind(&default_setting)
    .bind(adult_only)
    .bind(def_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.is_unique_violation() {
                return AppError::Conflict(format!(
                    "definition with identifier '{identifier}' already exists"
                ));
            }
        }
        AppError::Internal(format!("failed to update definition: {e}"))
    })?;

    // Replace locales if provided
    let locales = if let Some(new_locales) = body.locales {
        sqlx::query("DELETE FROM label_definition_locales WHERE definition_id = ?")
            .bind(def_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("failed to delete locales: {e}")))?;

        let mut result = Vec::new();
        for locale in &new_locales {
            sqlx::query(
                "INSERT INTO label_definition_locales (definition_id, lang, name, description) \
                 VALUES (?, ?, ?, ?)",
            )
            .bind(def_id)
            .bind(&locale.lang)
            .bind(&locale.name)
            .bind(&locale.description)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("failed to insert locale: {e}")))?;

            result.push(LocaleResponse {
                lang: locale.lang.clone(),
                name: locale.name.clone(),
                description: locale.description.clone(),
            });
        }
        result
    } else {
        // Fetch existing locales
        let locale_rows: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT lang, name, description FROM label_definition_locales \
             WHERE definition_id = ? ORDER BY lang",
        )
        .bind(def_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("failed to fetch locales: {e}")))?;

        locale_rows
            .into_iter()
            .map(|(lang, name, description)| LocaleResponse {
                lang,
                name,
                description,
            })
            .collect()
    };

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("transaction commit failed: {e}")))?;

    sync_service_record(&state).await;

    Ok(Json(DefinitionResponse {
        id: def_id,
        identifier,
        severity,
        blurs,
        default_setting,
        adult_only,
        builtin: false,
        locales,
        created_at: crate::db::parse_dt(&created_at),
    }))
}

/// DELETE /api/definitions/:id — delete a definition. Rejects builtin definitions.
pub async fn delete_definition(
    State(state): State<AppState>,
    _auth: ModeratorAuth,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    // Check if it exists and whether it's builtin
    let row: Option<(i32,)> =
        sqlx::query_as("SELECT builtin FROM label_definitions WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| AppError::Internal(format!("failed to fetch definition: {e}")))?;

    let (builtin_int,) = row.ok_or(AppError::NotFound)?;
    let builtin = builtin_int != 0;

    if builtin {
        return Err(AppError::Forbidden);
    }

    // CASCADE will delete locales automatically
    sqlx::query("DELETE FROM label_definitions WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("failed to delete definition: {e}")))?;

    sync_service_record(&state).await;

    Ok(StatusCode::NO_CONTENT)
}

/// Sync all label definitions from the database to the ATProto labeler service record.
/// Runs as a background best-effort operation — failures are logged but don't block the caller.
async fn sync_service_record(state: &AppState) {
    let state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = sync_service_record_inner(&state).await {
            tracing::error!("Failed to sync labeler service record: {e}");
        }
    });
}

async fn sync_service_record_inner(state: &AppState) -> Result<(), String> {
    let labeler_did: String = get_labeler_did(state).await?;

    let did: Did = labeler_did
        .parse()
        .map_err(|e| format!("Invalid labeler DID: {e}"))?;

    let session = state
        .oauth
        .restore(&did)
        .await
        .map_err(|e| format!("Failed to restore labeler OAuth session: {e}"))?;
    let agent = Agent::new(session);

    // Fetch current service record to preserve createdAt and other fields
    let existing = agent
        .api
        .com
        .atproto
        .repo
        .get_record(
            get_record::ParametersData {
                repo: did.clone().into(),
                collection: "app.bsky.labeler.service"
                    .parse()
                    .map_err(|e| format!("Invalid NSID: {e}"))?,
                rkey: "self".parse().map_err(|e| format!("Invalid rkey: {e}"))?,
                cid: None,
            }
            .into(),
        )
        .await
        .ok();

    // Extract the existing record as JSON to preserve fields we don't manage
    let existing_json: serde_json::Value = match &existing {
        Some(resp) => serde_json::to_value(&resp.value)
            .unwrap_or_else(|_| serde_json::json!({})),
        None => serde_json::json!({}),
    };

    // Fetch all definitions with locales from the database
    let rows: Vec<(i32, String, String, String, String, i32)> = sqlx::query_as(
        "SELECT id, identifier, severity, blurs, default_setting, adult_only \
         FROM label_definitions ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("Failed to fetch definitions: {e}"))?;

    let def_ids: Vec<i32> = rows.iter().map(|r| r.0).collect();

    let locale_rows: Vec<(i32, String, String, String)> = if def_ids.is_empty() {
        vec![]
    } else {
        let placeholders: Vec<&str> = def_ids.iter().map(|_| "?").collect();
        let sql = format!(
            "SELECT definition_id, lang, name, description \
             FROM label_definition_locales \
             WHERE definition_id IN ({}) \
             ORDER BY definition_id, lang",
            placeholders.join(", ")
        );
        let mut query = sqlx::query_as::<_, (i32, String, String, String)>(&sql);
        for id in &def_ids {
            query = query.bind(id);
        }
        query
            .fetch_all(&state.db)
            .await
            .map_err(|e| format!("Failed to fetch locales: {e}"))?
    };

    // Group locales by definition_id
    let mut locale_map: std::collections::HashMap<i32, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    for (def_id, lang, name, description) in locale_rows {
        locale_map.entry(def_id).or_default().push(serde_json::json!({
            "lang": lang,
            "name": name,
            "description": description,
        }));
    }

    // Build labelValues (string array) and labelValueDefinitions (full objects)
    let mut label_values: Vec<String> = Vec::new();
    let mut label_value_definitions: Vec<serde_json::Value> = Vec::new();

    for (id, identifier, severity, blurs, default_setting, adult_only_int) in &rows {
        label_values.push(identifier.clone());

        let mut def = serde_json::json!({
            "identifier": identifier,
            "severity": severity,
            "blurs": blurs,
            "locales": locale_map.get(id).cloned().unwrap_or_default(),
        });

        if default_setting != "warn" {
            def["defaultSetting"] = serde_json::json!(default_setting);
        }
        if *adult_only_int != 0 {
            def["adultOnly"] = serde_json::json!(true);
        }

        label_value_definitions.push(def);
    }

    // Build updated record, preserving existing fields
    let now = crate::db::now_rfc3339();
    let created_at = existing_json["createdAt"]
        .as_str()
        .unwrap_or(&now);

    let mut record = serde_json::json!({
        "$type": "app.bsky.labeler.service",
        "createdAt": created_at,
        "policies": {
            "labelValues": label_values,
            "labelValueDefinitions": label_value_definitions,
        }
    });

    // Preserve optional fields from the existing record
    if let Some(subject_types) = existing_json.get("subjectTypes") {
        record["subjectTypes"] = subject_types.clone();
    }
    if let Some(subject_collections) = existing_json.get("subjectCollections") {
        record["subjectCollections"] = subject_collections.clone();
    }
    if let Some(reason_types) = existing_json.get("reasonTypes") {
        record["reasonTypes"] = reason_types.clone();
    }

    let record_unknown = record
        .try_into_unknown()
        .map_err(|e| format!("Failed to build record: {e}"))?;

    agent
        .api
        .com
        .atproto
        .repo
        .put_record(
            put_record::InputData {
                repo: did.into(),
                collection: "app.bsky.labeler.service"
                    .parse()
                    .map_err(|e| format!("Invalid NSID: {e}"))?,
                rkey: "self"
                    .parse()
                    .map_err(|e| format!("Invalid rkey: {e}"))?,
                record: record_unknown,
                swap_commit: None,
                swap_record: None,
                validate: None,
            }
            .into(),
        )
        .await
        .map_err(|e| format!("putRecord failed: {e}"))?;

    tracing::info!("Synced labeler service record with {} label definitions", rows.len());
    Ok(())
}

/// Get the labeler DID, checking setup mutex first, then in-memory config, then config file on disk.
async fn get_labeler_did(state: &AppState) -> Result<String, String> {
    // Check the in-memory setup mutex (set during setup flow)
    let guard = state.setup_labeler_did.lock().await;
    if let Some(did) = guard.clone() {
        return Ok(did);
    }
    drop(guard);

    // Check the in-memory config (set at startup)
    let did = &state.config.labeler.did;
    if !did.is_empty() && did != "did:plc:placeholder" {
        return Ok(did.clone());
    }

    // Re-read config file in case it was updated after startup (e.g. by setup)
    let fresh = crate::config::Config::load();
    let did = &fresh.labeler.did;
    if !did.is_empty() && did != "did:plc:placeholder" {
        return Ok(did.clone());
    }

    Err("Labeler DID not configured".into())
}
